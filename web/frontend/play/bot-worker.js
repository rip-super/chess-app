import init, { ChessBot } from "../wasm/wasm.js";
await init();
const bot = new ChessBot();

self.onmessage = ({ data }) => {
    const { fen, searchMs } = data;
    bot.set_move_time(searchMs);

    const t0 = Date.now();
    const uci = bot.best_move(fen);
    const elapsed = Date.now() - t0;

    const minElapsed = Math.min(searchMs * 0.4, 500);
    const hold = Math.max(0, minElapsed - elapsed);

    if (hold > 0) {
        setTimeout(() => self.postMessage({ uci }), hold);
    } else {
        self.postMessage({ uci });
    }
};