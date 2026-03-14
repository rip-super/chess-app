// todo: promotion picker

import init, { ChessEngine } from "./pkg/wasm.js";

let engine = null;
let selectedSq = null;
let legalTargets = [];
let dragState = null;
let animating = false;
let pendingPointer = null;
let animatingToSq = null;

const board = document.getElementById("board");

function renderBoard(invert = false) {
    board.innerHTML = "";

    for (let row = 0; row < 8; row++) {
        for (let col = 0; col < 8; col++) {
            const rank = invert ? row : 7 - row;
            const file = invert ? 7 - col : col;
            const sqIndex = rank * 8 + file;
            const piece = engine.piece_on(sqIndex);

            const div = document.createElement("div");
            div.className = "sq " + ((rank + file) % 2 === 0 ? "dark" : "light");
            div.dataset.sq = sqIndex;

            if (sqIndex === selectedSq) div.classList.add("selected");
            if (legalTargets.includes(sqIndex)) div.classList.add(piece ? "legal-capture" : "legal");

            if (piece) {
                const img = document.createElement("img");
                img.src = `pieces/${piece}.svg`;
                img.className = "piece";

                if (dragState?.fromSq === sqIndex) img.style.opacity = "0.2";
                if (animatingToSq === sqIndex) img.style.opacity = "0";

                div.appendChild(img);
            }

            div.addEventListener("pointerdown", e => {
                if (animating) return;
                e.preventDefault();

                if (selectedSq !== null && legalTargets.includes(sqIndex)) {
                    const fromSq = selectedSq;
                    const pieceCode = engine.piece_on(fromSq);
                    const mv = engine.legal_moves().find(m => m.from_sq() === fromSq && m.to_sq() === sqIndex);
                    if (!mv) return;

                    const fromRect = board.querySelector(`[data-sq="${fromSq}"]`).getBoundingClientRect();
                    const toRect = div.getBoundingClientRect();

                    engine.make_move(mv);
                    selectedSq = null;
                    legalTargets = [];
                    animatingToSq = sqIndex;
                    animating = true;
                    renderBoard(invert);

                    const size = fromRect.width * 0.85;
                    const anim = Object.assign(document.createElement("img"), { src: `pieces/${pieceCode}.svg`, className: "piece-anim" });

                    Object.assign(anim.style, { width: size + "px", height: size + "px", left: (fromRect.left + (fromRect.width - size) / 2) + "px", top: (fromRect.top + (fromRect.height - size) / 2) + "px" });
                    document.body.appendChild(anim);

                    anim.getBoundingClientRect();

                    Object.assign(anim.style, { left: (toRect.left + (toRect.width - size) / 2) + "px", top: (toRect.top + (toRect.height - size) / 2) + "px" });
                    anim.addEventListener("transitionend", () => {
                        anim.remove();
                        animating = false;
                        animatingToSq = null;
                        renderBoard(invert);
                    }, { once: true });

                    return;
                }

                if (piece) {
                    pendingPointer = { sqIndex, piece, startX: e.clientX, startY: e.clientY };
                    return;
                }

                selectedSq = null; legalTargets = [];
                renderBoard(invert);
            });

            if (engine.is_in_check()) {
                const kingSq = engine.king_square(engine.side_to_move());
                if (kingSq === sqIndex) div.classList.add("in-check");
            }

            board.appendChild(div);
        }
    }
}

document.addEventListener("pointermove", e => {
    if (pendingPointer && !dragState) {
        if (Math.hypot(e.clientX - pendingPointer.startX, e.clientY - pendingPointer.startY) < 5) return;
        const { sqIndex, piece } = pendingPointer;
        const ghost = Object.assign(document.createElement("img"), { src: `pieces/${piece}.svg`, className: "piece-ghost" });

        Object.assign(ghost.style, { left: e.clientX + "px", top: e.clientY + "px" });
        document.body.appendChild(ghost);

        dragState = { fromSq: sqIndex, ghostEl: ghost };
        selectedSq = sqIndex;
        legalTargets = engine.legal_moves().filter(m => m.from_sq() === sqIndex).map(m => m.to_sq());

        renderBoard();
    }

    if (dragState) Object.assign(dragState.ghostEl.style, { left: e.clientX + "px", top: e.clientY + "px" });
});

document.addEventListener("pointerup", e => {
    if (pendingPointer && !dragState) {
        const { sqIndex } = pendingPointer;
        pendingPointer = null;

        if (selectedSq === sqIndex) {
            selectedSq = null; legalTargets = [];
        } else {
            selectedSq = sqIndex;
            legalTargets = engine.legal_moves().filter(m => m.from_sq() === sqIndex).map(m => m.to_sq());
        }

        renderBoard();
        return;
    }

    pendingPointer = null;
    if (!dragState) return;

    dragState.ghostEl.remove();
    const { fromSq } = dragState;
    dragState = null;

    const toSq = parseInt(document.elementFromPoint(e.clientX, e.clientY)?.closest(".sq")?.dataset.sq);
    if (!isNaN(toSq) && toSq !== fromSq && legalTargets.includes(toSq)) {
        const mv = engine.legal_moves().find(m => m.from_sq() === fromSq && m.to_sq() === toSq);
        if (mv) { engine.make_move(mv); selectedSq = null; legalTargets = []; }
    } else {
        selectedSq = null; legalTargets = [];
    }

    renderBoard();
});

await init();
engine = new ChessEngine();
renderBoard();