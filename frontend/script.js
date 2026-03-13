import init, { ChessEngine } from "./pkg/wasm.js";

function renderBoard(engine) {
    let out = "  +------------------------+\n";
    for (let rank = 7; rank >= 0; rank--) {
        out += `${rank + 1} |`;
        for (let file = 0; file < 8; file++) {
            const sq = rank * 8 + file;
            const piece = engine.piece_on(sq);
            if (piece) {
                const ch = piece[0] === "w"
                    ? piece[1].toUpperCase()
                    : piece[1].toLowerCase();
                out += ` ${ch} `;
            } else {
                out += " . ";
            }
        }
        out += "|\n";
    }
    out += "  +------------------------+\n";
    out += "    a  b  c  d  e  f  g  h";
    return out;
}

async function main() {
    await init();

    const engine = new ChessEngine();
    const status = document.getElementById("status");
    const board = document.getElementById("board");
    const result = document.getElementById("result");

    function update() {
        board.textContent = renderBoard(engine);
        result.textContent = engine.is_in_check() ? "Check!" : "";
        const gr = engine.game_result();
        if (gr !== "ongoing") {
            result.textContent = `Game over: ${gr}`;
            document.getElementById("move-btn").disabled = true;
        }
    }

    status.textContent = "WASM loaded!";
    update();

    document.getElementById("move-btn").addEventListener("click", () => {
        const input = document.getElementById("move-input").value.trim();
        const mv = engine.parse_uci(input);
        if (!mv) {
            status.textContent = `Invalid move: ${input}`;
            return;
        }
        try {
            engine.make_move(mv);
            status.textContent = `Played: ${input}`;
            document.getElementById("move-input").value = "";
            update();
        } catch (e) {
            status.textContent = `Error: ${e}`;
        }
    });

    document.getElementById("undo-btn").addEventListener("click", () => {
        engine.undo_move();
        document.getElementById("move-btn").disabled = false;
        status.textContent = "Undone";
        update();
    });

    document.getElementById("move-input").addEventListener("keydown", (e) => {
        if (e.key === "Enter") document.getElementById("move-btn").click();
    });
}

await main();