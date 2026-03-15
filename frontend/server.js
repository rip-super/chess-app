import { serve } from "@hono/node-server";
import { createNodeWebSocket } from "@hono/node-ws";
import { serveStatic } from "@hono/node-server/serve-static";
import { Hono } from "hono";
import { readFile } from "fs/promises";
import init, { ChessEngine } from "./ui/wasm/wasm.js";

const wasm = await readFile(new URL("./ui/wasm/wasm_bg.wasm", import.meta.url));
await init({ module_or_path: wasm });

const app = new Hono();
const { injectWebSocket, upgradeWebSocket } = createNodeWebSocket({ app });

const games = new Map();
let waitingPlayer = null;

app.get("/match", (c) => {
    if (waitingPlayer) {
        const gameId = waitingPlayer.gameId;
        waitingPlayer = null;
        return c.json({ gameId });
    }

    const gameId = crypto.randomUUID();
    games.set(gameId, { engine: new ChessEngine(), white: null, black: null, tokens: {} });
    waitingPlayer = { gameId };
    return c.json({ waiting: true, gameId });
});

app.get("/match/:gameId", (c) => {
    const gameId = c.req.param("gameId");

    if (!waitingPlayer || waitingPlayer.gameId !== gameId) {
        if (games.has(gameId)) return c.json({ gameId });
        return c.json({ error: "expired" }, 404);
    }

    return c.json({ waiting: true });
});

app.get("/ws/:gameId", upgradeWebSocket(c => {
    const gameId = c.req.param("gameId");

    return {
        onOpen() {
            if (!games.has(gameId)) {
                games.set(gameId, { engine: new ChessEngine(), white: null, black: null, tokens: {} });
            }
        },

        onMessage(evt, ws) {
            const { type, uci, token } = JSON.parse(evt.data);
            const game = games.get(gameId);
            if (!game) return;

            if (type === "auth") {
                if (game.tokens[token]) {
                    const restoredColor = game.tokens[token];
                    if (restoredColor === "w") game.white = ws;
                    else game.black = ws;
                    ws.send(JSON.stringify({ type: "assign", color: restoredColor }));
                    ws.send(JSON.stringify({ type: "sync", fen: game.engine.get_fen() }));
                    return;
                }

                if (!game.white) {
                    game.white = ws; game.tokens[token] = "w";
                    ws.send(JSON.stringify({ type: "assign", color: "w" }));
                } else if (!game.black) {
                    game.black = ws; game.tokens[token] = "b";
                    ws.send(JSON.stringify({ type: "assign", color: "b" }));
                } else {
                    ws.send(JSON.stringify({ type: "error", msg: "game full" }));
                    return;
                }
                ws.send(JSON.stringify({ type: "sync", fen: game.engine.get_fen() }));
                return;
            }

            if (type === "new_game") {
                game.engine = new ChessEngine();
                const sync = JSON.stringify({ type: "sync", fen: game.engine.get_fen() });
                game.white?.send(sync);
                game.black?.send(sync);
                return;
            }

            if (type !== "move") return;

            const sideToMove = game.engine.side_to_move();
            const isWhite = game.white === ws;
            const isBlack = game.black === ws;
            if ((sideToMove === "w" && !isWhite) || (sideToMove === "b" && !isBlack)) {
                ws.send(JSON.stringify({ type: "error", msg: "not your turn" }));
                return;
            }

            const mv = game.engine.parse_uci(uci);
            if (!mv) return ws.send(JSON.stringify({ type: "error", msg: "invalid move" }));

            try {
                game.engine.make_move(mv);
                const msg = JSON.stringify({
                    type: "move",
                    fen: game.engine.get_fen(),
                    uci,
                    isCapture: mv.is_capture(),
                    isCastle: mv.is_castle(),
                    isPromotion: mv.is_promotion(),
                    isCheck: game.engine.is_in_check(),
                    result: game.engine.game_result(),
                });
                game.white?.send(msg);
                game.black?.send(msg);
            } catch {
                ws.send(JSON.stringify({ type: "error", msg: "illegal move" }));
            }
        },

        onClose(_, ws) {
            const game = games.get(gameId);

            if (!game) return;
            if (game.white === ws) game.white = null;
            if (game.black === ws) game.black = null;
            if (!game.white && !game.black && game.engine.game_result() !== "ongoing") {
                games.delete(gameId);
            }
        }
    };
}));

app.get("/", async (c) => c.html(await readFile("./ui/index.html", "utf8")));
app.use("/*", serveStatic({ root: "./ui" }));
app.get("/play/:gameId", async (c) => c.html(await readFile("./ui/play/index.html", "utf8")));

const server = serve({ fetch: app.fetch, port: 3000 }, info => {
    console.log(`Server running on http://localhost:${info.port}`);
});

injectWebSocket(server);