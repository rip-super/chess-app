import init, { ChessEngine } from "./pkg/wasm.js";

function renderBoard(board, engine) {
    board.innerHTML = "";
    for (let rank = 7; rank >= 0; rank--) {
        for (let file = 0; file < 8; file++) {
            const sq = document.createElement("div");
            sq.className = "sq " + ((rank + file) % 2 === 0 ? "dark" : "light");

            const piece = engine.piece_on(rank * 8 + file);
            if (piece) {
                const img = document.createElement("img");
                img.src = `pieces/${piece}.svg`;
                img.className = "piece";
                sq.appendChild(img);
            }

            board.appendChild(sq);
        }
    }
}

async function main() {
    await init();

    const engine = new ChessEngine();
    const board = document.getElementById("board");

    renderBoard(board, engine);
}

await main();