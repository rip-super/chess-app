import init, { ChessEngine } from "./assets/wasm/wasm.js";

let engine = null;
let selectedSq = null;
let legalTargets = [];
let dragState = null;
let animating = false;
let pendingPointer = null;
let animatingToSq = null;
let pendingPromotion = null;

const board = document.getElementById("board");

const sfx = name => Object.assign(new Audio(`assets/sounds/${name}.mp3`), { currentTime: 0 }).play();

function isPromotion(fromSq, toSq) {
    const piece = engine.piece_on(fromSq);
    if (piece !== "wP" && piece !== "bP") return false;
    const toRank = Math.floor(toSq / 8);
    return toRank === 7 || toRank === 0;
}

function checkGameOver() {
    const result = engine.game_result();
    if (result === "ongoing") return;

    const messages = {
        checkmate_white: { top: "Checkmate", bottom: "White wins!" },
        checkmate_black: { top: "Checkmate", bottom: "Black wins!" },
        stalemate: { top: "Stalemate", bottom: "Draw!" },
        draw_repetition: { top: "Draw!", bottom: "By Repetition" },
        draw_fifty_move: { top: "Draw!", bottom: "By Fifty Move Rule" },
        draw_insufficient: { top: "Draw!", bottom: "Due To Insufficient Material" },
    };

    const { top, bottom } = messages[result];

    const panel = document.createElement("div");
    panel.className = `gameover-panel`;
    panel.innerHTML = `
        <div class="gameover-text">
            <span class="gameover-top">${top}</span>
            <span class="gameover-bottom">${bottom}</span>
        </div>
        <div class="gameover-actions">
            <button class="gameover-btn" id="new-game-btn">New game</button>
        </div>
    `;
    board.appendChild(panel);

    document.getElementById("new-game-btn").addEventListener("pointerdown", e => {
        e.stopPropagation();
        engine = new ChessEngine();
        selectedSq = null; legalTargets = [];
        panel.remove();
        sfx("game_start");
        renderBoard();
    });
}

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
                img.src = `assets/images/${piece}.svg`;
                img.className = "piece";

                if (dragState?.fromSq === sqIndex) img.style.opacity = "0.2";
                if (animatingToSq === sqIndex) img.style.opacity = "0";

                div.appendChild(img);
            }

            if (engine.is_in_check()) {
                const kingSq = engine.king_square(engine.side_to_move());
                if (kingSq === sqIndex) div.classList.add("in-check");
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

                    if (isPromotion(fromSq, sqIndex)) {
                        pendingPromotion = { fromSq, toSq: sqIndex };
                        selectedSq = null; legalTargets = [];
                        renderBoard(invert);
                        return;
                    }

                    engine.make_move(mv);
                    sfx(engine.game_result() !== "ongoing" ? "game_end" : engine.is_in_check() ? "check" : mv.is_promotion() ? "promote" : mv.is_castle() ? "castle" : mv.is_capture() ? "capture" : "move");

                    selectedSq = null;
                    legalTargets = [];
                    animatingToSq = sqIndex;
                    animating = true;
                    renderBoard(invert);

                    const size = fromRect.width * 0.85;
                    const anim = Object.assign(document.createElement("img"), { src: `assets/images/${pieceCode}.svg`, className: "piece-anim" });

                    Object.assign(anim.style, { width: size + "px", height: size + "px", left: (fromRect.left + (fromRect.width - size) / 2) + "px", top: (fromRect.top + (fromRect.height - size) / 2) + "px" });
                    document.body.appendChild(anim);

                    anim.getBoundingClientRect();

                    Object.assign(anim.style, { left: (toRect.left + (toRect.width - size) / 2) + "px", top: (toRect.top + (toRect.height - size) / 2) + "px" });
                    anim.addEventListener("transitionend", () => {
                        anim.remove();
                        animating = false;
                        animatingToSq = null;
                        renderBoard(invert);
                        checkGameOver();
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

            board.appendChild(div);
        }
    }

    if (pendingPromotion) {
        const { fromSq, toSq } = pendingPromotion;
        const promoRank = Math.floor(toSq / 8);
        const promoFile = toSq % 8;
        const side = promoRank === 7 ? "w" : "b";
        const pieces = ["Q", "R", "B", "N"];

        const backdrop = document.createElement("div");
        backdrop.className = "promo-backdrop";
        backdrop.addEventListener("pointerdown", e => {
            e.stopPropagation();
            pendingPromotion = null;
            selectedSq = null; legalTargets = [];
            renderBoard(invert);
        });

        board.appendChild(backdrop);

        const card = document.createElement("div");
        card.className = "promo-card";

        const visualCol = invert ? (7 - promoFile) : promoFile;
        const sqSizePx = board.clientWidth / 8;

        card.style.left = (visualCol * sqSizePx) + "px";
        card.style.top = promoRank === 7 ? "0px" : "auto";
        card.style.bottom = promoRank === 0 ? "0px" : "auto";

        pieces.forEach((p, _) => {
            const btn = document.createElement("div");
            btn.className = "promo-btn";

            const img = document.createElement("img");
            img.src = `pieces/${side}${p}.svg`;
            img.className = "piece";

            btn.appendChild(img);
            btn.addEventListener("pointerdown", e => {
                e.stopPropagation();
                const uci = `${"abcdefgh"[fromSq % 8]}${Math.floor(fromSq / 8) + 1}${"abcdefgh"[toSq % 8]}${Math.floor(toSq / 8) + 1}${p.toLowerCase()}`;

                const mv = engine.parse_uci(uci);
                if (mv) { engine.make_move(mv); sfx(engine.game_result() !== "ongoing" ? "game_end" : engine.is_in_check() ? "check" : mv.is_promotion() ? "promote" : mv.is_castle() ? "castle" : mv.is_capture() ? "capture" : "move"); }

                pendingPromotion = null;
                renderBoard(invert);
                checkGameOver();
            });

            card.appendChild(btn);
        });

        board.appendChild(card);
    }
}

document.addEventListener("pointermove", e => {
    if (pendingPointer && !dragState) {
        if (Math.hypot(e.clientX - pendingPointer.startX, e.clientY - pendingPointer.startY) < 5) return;
        const { sqIndex, piece } = pendingPointer;
        const ghost = Object.assign(document.createElement("img"), { src: `assets/images/${piece}.svg`, className: "piece-ghost" });

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
        if (mv) {
            if (isPromotion(fromSq, toSq)) {
                pendingPromotion = { fromSq, toSq };
                selectedSq = null; legalTargets = [];
                renderBoard();
                return;
            }

            engine.make_move(mv);
            sfx(engine.game_result() !== "ongoing" ? "game_end" : engine.is_in_check() ? "check" : mv.is_promotion() ? "promote" : mv.is_castle() ? "castle" : mv.is_capture() ? "capture" : "move");

            selectedSq = null;
            legalTargets = [];
        }
    } else {
        selectedSq = null; legalTargets = [];
    }

    renderBoard();
    checkGameOver();
});

await init();
engine = new ChessEngine();
sfx("game_start");
renderBoard();