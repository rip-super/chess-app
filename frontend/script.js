// todo: move anims, dragging, promotion picker

import init, { ChessEngine } from "./pkg/wasm.js";

let selectedSq = null;
let legalTargets = [];

function onSquareClick(sq, engine) {
    if (selectedSq !== null && legalTargets.includes(sq)) {
        const mv = engine.legal_moves().find(mv => mv.from_sq() === selectedSq && mv.to_sq() === sq);
        if (mv) {
            engine.make_move(mv);
            selectedSq = null;
            legalTargets = [];
            renderBoard(board, engine);
        }

        return;
    }

    if (engine.piece_on(sq)) {
        selectedSq = sq;
        legalTargets = engine.legal_moves().filter(mv => mv.from_sq() === sq).map(mv => mv.to_sq());
        renderBoard(board, engine);
        return;
    }

    selectedSq = null;
    legalTargets = [];
    renderBoard(board, engine);
}

function renderBoard(board, engine, invert = false) {
    board.innerHTML = "";
    for (let row = 0; row < 8; row++) {
        for (let col = 0; col < 8; col++) {
            const rank = invert ? row : 7 - row;
            const file = invert ? 7 - col : col;
            const sqIndex = rank * 8 + file;

            const sq = document.createElement("div");
            sq.className = "sq " + ((rank + file) % 2 === 0 ? "dark" : "light");

            if (sqIndex === selectedSq) sq.classList.add("selected");
            if (legalTargets.includes(sqIndex)) sq.classList.add("legal");

            sq.addEventListener("click", () => onSquareClick(sqIndex, engine));

            const piece = engine.piece_on(sqIndex);
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

    const engine = ChessEngine.from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ");
    const board = document.getElementById("board");

    renderBoard(board, engine);
}

await main();