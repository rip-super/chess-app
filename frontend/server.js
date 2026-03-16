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
        console.log(`[match] paired into game ${gameId}`);
        return c.json({ gameId });
    }

    const gameId = crypto.randomUUID();
    games.set(gameId, { engine: new ChessEngine(), white: null, black: null, tokens: {}, result: null });
    waitingPlayer = { gameId };
    console.log(`[match] waiting for opponent, game ${gameId}`);
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
                games.set(gameId, { engine: new ChessEngine(), white: null, black: null, tokens: {}, result: null });
            }
            console.log(`[${gameId}] websocket opened`);
        },

        onMessage(evt, ws) {
            const { type, uci, token } = JSON.parse(evt.data);
            const game = games.get(gameId);
            if (!game) return;

            if (type === "auth") {
                if (game.tokens[token]) {
                    const restoredColor = game.tokens[token];
                    const existingWs = restoredColor === "w" ? game.white : game.black;

                    if (existingWs && existingWs !== ws) {
                        ws.send(JSON.stringify({ type: "error", msg: "already connected" }));
                        return;
                    }

                    if (restoredColor === "w") game.white = ws;
                    else game.black = ws;

                    ws.send(JSON.stringify({ type: "assign", color: restoredColor }));
                    ws.send(JSON.stringify({ type: "sync", fen: game.engine.get_fen() }));

                    if (game.result) {
                        ws.send(JSON.stringify({ type: "game_over", result: game.result }));
                    }

                    return;
                }

                if (!game.white) {
                    game.white = ws; game.tokens[token] = "w";
                    console.log(`[${gameId}] player joined as white`);
                    ws.send(JSON.stringify({ type: "assign", color: "w" }));
                } else if (!game.black) {
                    if (Math.random() < 0.5) {
                        game.tokens[token] = "w";
                        const oldToken = Object.keys(game.tokens).find(t => game.tokens[t] === "w" && t !== token);
                        if (oldToken) game.tokens[oldToken] = "b";
                        game.black = game.white;
                        game.white = ws;
                        console.log(`[${gameId}] player joined as white (colors swapped)`);
                        game.black.send(JSON.stringify({ type: "assign", color: "b" }));
                        ws.send(JSON.stringify({ type: "assign", color: "w" }));
                    } else {
                        game.black = ws; game.tokens[token] = "b";
                        console.log(`[${gameId}] player joined as black`);
                        ws.send(JSON.stringify({ type: "assign", color: "b" }));
                    }
                } else {
                    console.warn(`[${gameId}] player tried to join full game`);
                    ws.send(JSON.stringify({ type: "error", msg: "game full" }));
                    return;
                }
                ws.send(JSON.stringify({ type: "sync", fen: game.engine.get_fen() }));
                return;
            }

            if (type === "new_game") {
                game.engine = new ChessEngine();
                console.log(`[${gameId}] new game started`);
                const sync = JSON.stringify({ type: "sync", fen: game.engine.get_fen() });
                game.white?.send(sync);
                game.black?.send(sync);
                return;
            }

            if (type === "resign") {
                const resignColor = game.white === ws ? "w" : "b";
                const result = resignColor === "w" ? "resign_white" : "resign_black";
                game.result = result;
                const msg = JSON.stringify({ type: "game_over", result });
                game.white?.send(msg);
                game.black?.send(msg);
                return;
            }

            if (type === "draw_offer") {
                const opponent = game.white === ws ? game.black : game.white;
                if (!opponent) return;
                console.log(`[${gameId}] draw offered`);
                opponent.send(JSON.stringify({ type: "draw_offer" }));
                return;
            }

            if (type === "draw_accepted") {
                game.result = "draw_agreed";
                const msg = JSON.stringify({ type: "game_over", result: "draw_agreed" });
                game.white?.send(msg);
                game.black?.send(msg);
                return;
            }

            if (type === "draw_declined") {
                const opponent = game.white === ws ? game.black : game.white;
                opponent?.send(JSON.stringify({ type: "draw_declined" }));
                return;
            }

            if (type !== "move") return;

            const sideToMove = game.engine.side_to_move();
            const isWhite = game.white === ws;
            const isBlack = game.black === ws;
            if ((sideToMove === "w" && !isWhite) || (sideToMove === "b" && !isBlack)) {
                console.warn(`[${gameId}] out-of-turn move attempt: ${uci}`);
                ws.send(JSON.stringify({ type: "error", msg: "not your turn" }));
                return;
            }

            const mv = game.engine.parse_uci(uci);
            if (!mv) {
                console.warn(`[${gameId}] invalid uci: ${uci}`);
                return ws.send(JSON.stringify({ type: "error", msg: "invalid move" }));
            }

            try {
                game.engine.make_move(mv);
                const result = game.engine.game_result();
                console.log(`[${gameId}] move ${uci} - result: ${result}`);

                const msg = JSON.stringify({
                    type: "move",
                    fen: game.engine.get_fen(),
                    uci,
                    isCapture: mv.is_capture(),
                    isCastle: mv.is_castle(),
                    isPromotion: mv.is_promotion(),
                    isCheck: game.engine.is_in_check(),
                    result,
                });

                game.white?.send(msg);
                game.black?.send(msg);
            } catch (e) {
                console.warn(`[${gameId}] illegal move attempt: ${uci}`);
                ws.send(JSON.stringify({ type: "error", msg: "illegal move" }));
            }
        },

        onClose(_, ws) {
            const game = games.get(gameId);
            if (!game) return;

            if (game.white === ws) { game.white = null; console.log(`[${gameId}] white disconnected`); }
            if (game.black === ws) { game.black = null; console.log(`[${gameId}] black disconnected`); }
            if (!game.white && !game.black && game.engine.game_result() !== "ongoing") {
                games.delete(gameId);
                console.log(`[${gameId}] game cleaned up`);
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