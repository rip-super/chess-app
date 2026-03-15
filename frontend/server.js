import { serve } from "@hono/node-server";
import { createNodeWebSocket } from "@hono/node-ws";
import { serveStatic } from "@hono/node-server/serve-static";
import { Hono } from "hono";
import { readFile } from "fs/promises";
import init, { ChessEngine } from "./wasm-server/wasm.js";

const wasm = await readFile(new URL("./wasm-server/wasm_bg.wasm", import.meta.url));
await init({ module_or_path: wasm });

const app = new Hono();
const { injectWebSocket, upgradeWebSocket } = createNodeWebSocket({ app });

const games = new Map();

app.get("/ws/:gameId", upgradeWebSocket(c => {
    const gameId = c.req.param("gameId");

    return {
        onOpen(_, ws) {
            if (!games.has(gameId)) games.set(gameId, new ChessEngine());
            const engine = games.get(gameId);
            ws.send(JSON.stringify({ type: "sync", fen: engine.get_fen() }));
        },

        onMessage(evt, ws) {
            const { type, uci } = JSON.parse(evt.data);

            if (type === "new_game") {
                games.set(gameId, new ChessEngine());
                const engine = games.get(gameId);
                ws.send(JSON.stringify({ type: "sync", fen: engine.get_fen() }));
                return;
            }

            if (type !== "move") return;

            const engine = games.get(gameId);
            if (!engine) return ws.send(JSON.stringify({ type: "error", msg: "game not found" }));

            const mv = engine.parse_uci(uci);
            if (!mv) return ws.send(JSON.stringify({ type: "error", msg: "invalid move" }));

            try {
                engine.make_move(mv);
                ws.send(JSON.stringify({
                    type: "move",
                    fen: engine.get_fen(),
                    uci,
                    isCapture: mv.is_capture(),
                    isCastle: mv.is_castle(),
                    isPromotion: mv.is_promotion(),
                    isCheck: engine.is_in_check(),
                    result: engine.game_result(),
                }));
            } catch {
                ws.send(JSON.stringify({ type: "error", msg: "illegal move" }));
            }
        },

        onClose() {
            const engine = games.get(gameId);
            if (engine?.game_result() !== "ongoing") games.delete(gameId);
        }
    };
}));

app.use("/*", serveStatic({ root: "./ui" }));

const server = serve({ fetch: app.fetch, port: 3000 }, info => {
    console.log(`Server running on http://localhost:${info.port}`);
});

injectWebSocket(server);