import init, { ChessEngine } from "../wasm/wasm.js";

await init();
let engine = new ChessEngine();

let selectedSq = null;
let legalTargets = [];
let dragState = null;
let animating = false;
let pendingPointer = null;
let animatingToSq = null;
let pendingPromotion = null;
let gameStarted = false;
let color = "w";

const board = document.getElementById("board");
const sfx = name => Object.assign(new Audio(`assets/sounds/${name}.mp3`), { currentTime: 0 }).play();

function deselect() { selectedSq = null; legalTargets = []; }
function selectSquare(sq) { selectedSq = sq; legalTargets = engine.legal_moves().filter(m => m.from_sq() === sq).map(m => m.to_sq()); }

function isPromotion(fromSq, toSq) {
    const piece = engine.piece_on(fromSq);
    if (piece !== "wP" && piece !== "bP") return false;
    return Math.floor(toSq / 8) === 7 || Math.floor(toSq / 8) === 0;
}

let token = localStorage.getItem("token");
if (!token) { token = crypto.randomUUID(); localStorage.setItem("token", token); }

const gameId = window.location.pathname.split("/").pop();
const ws = new WebSocket(`${location.protocol === "https:" ? "wss:" : "ws:"}//${location.host}/ws/${gameId}`);

function sendMove(uci) {
    ws.send(JSON.stringify({ type: "move", uci }));
}

ws.addEventListener("open", () => {
    ws.send(JSON.stringify({ type: "auth", token }));
});

ws.addEventListener("message", e => {
    const msg = JSON.parse(e.data);

    if (msg.type === "error") {
        console.warn("Move rejected:", msg.msg);
        deselect();
        renderBoard(color === "b");
        return;
    }

    if (msg.type === "assign") {
        color = msg.color;
        renderBoard(color === "b");
        return;
    }

    if (msg.type === "sync") {
        engine = ChessEngine.from_fen(msg.fen);
        deselect();

        if (!gameStarted) {
            gameStarted = true;
            sfx("game_start").catch(() => { });
        }

        renderBoard(color === "b");
        return;
    }

    if (msg.type === "move") {
        const mv = engine.parse_uci(msg.uci);
        if (mv) engine.make_move(mv);

        if (msg.result !== "ongoing") sfx("game_end");
        else if (msg.isCheck) sfx("check");
        else if (msg.isPromotion) sfx("promote");
        else if (msg.isCastle) sfx("castle");
        else if (msg.isCapture) sfx("capture");
        else sfx("move");

        deselect();

        const fromSq = uciToSq(msg.uci.slice(0, 2));
        const toSq = uciToSq(msg.uci.slice(2, 4));
        const pieceCode = engine.piece_on(toSq);
        const fromRect = board.querySelector(`[data-sq="${fromSq}"]`)?.getBoundingClientRect();
        const toRect = board.querySelector(`[data-sq="${toSq}"]`)?.getBoundingClientRect();

        animatingToSq = toSq;
        animating = true;
        renderBoard(color === "b");

        const size = fromRect.width * 0.85;
        const anim = Object.assign(document.createElement("img"), { src: `assets/images/${pieceCode}.svg`, className: "piece-anim" });

        Object.assign(anim.style, { width: size + "px", height: size + "px", left: (fromRect.left + (fromRect.width - size) / 2) + "px", top: (fromRect.top + (fromRect.height - size) / 2) + "px" });
        document.body.appendChild(anim);

        anim.getBoundingClientRect();

        Object.assign(anim.style, { left: (toRect.left + (toRect.width - size) / 2) + "px", top: (toRect.top + (toRect.height - size) / 2) + "px" });
        anim.addEventListener("transitionend", () => {
            anim.remove();
            animating = false; animatingToSq = null;
            renderBoard(color === "b");
            checkGameOver(msg.result);
        }, { once: true });
    }
});

function uciToSq(coord) {
    return (parseInt(coord[1]) - 1) * 8 + (coord.charCodeAt(0) - 97);
}

function checkGameOver(result) {
    if (result === "ongoing") return;

    const msgs = {
        checkmate_white: { top: "Checkmate", bottom: "White wins!" },
        checkmate_black: { top: "Checkmate", bottom: "Black wins!" },
        stalemate: { top: "Stalemate", bottom: "Draw!" },
        draw_repetition: { top: "Draw", bottom: "By Repetition" },
        draw_fifty_move: { top: "Draw", bottom: "By Fifty Move Rule" },
        draw_insufficient: { top: "Draw", bottom: "By Insufficient Material" },
    };

    const { top, bottom } = msgs[result];
    const panel = document.createElement("div");

    panel.className = "gameover-panel";
    panel.innerHTML = `
        <div class="gameover-text">
            <span class="gameover-top">${top}</span>
            <span class="gameover-bottom">${bottom}</span>
        </div>
        <div class="gameover-actions">
            <button class="gameover-btn" id="new-game-btn">New game</button>
        </div>
    `;

    localStorage.removeItem("gameId");

    board.appendChild(panel);

    document.getElementById("new-game-btn").addEventListener("pointerdown", e => {
        e.stopPropagation();
        panel.remove();
        sessionStorage.setItem("autoplay", "1");
        window.location.href = "/";
    });
}

function renderBoard(invert = false) {
    if (board.querySelectorAll(".sq").length === 0) {
        for (let i = 0; i < 64; i++) {
            const div = document.createElement("div");
            div.className = "sq";
            board.appendChild(div);
        }
    }

    const inCheck = engine.is_in_check();
    const kingSq = inCheck ? engine.king_square(engine.side_to_move()) : null;
    const sqs = board.querySelectorAll(".sq");

    for (let row = 0; row < 8; row++) {
        for (let col = 0; col < 8; col++) {
            const rank = invert ? row : 7 - row;
            const file = invert ? 7 - col : col;
            const sqIndex = rank * 8 + file;
            const piece = engine.piece_on(sqIndex);
            const div = sqs[row * 8 + col];

            div.className = "sq " + ((rank + file) % 2 === 0 ? "dark" : "light");
            div.dataset.sq = sqIndex;

            if (sqIndex === selectedSq) div.classList.add("selected");
            if (legalTargets.includes(sqIndex)) div.classList.add(piece ? "legal-capture" : "legal");
            if (sqIndex === kingSq) div.classList.add("in-check");

            let img = div.querySelector("img.piece");
            if (piece) {
                if (!img) {
                    img = document.createElement("img");
                    img.className = "piece";
                    div.appendChild(img);
                }
                const newSrc = `assets/images/${piece}.svg`;
                if (img.src !== newSrc) img.src = newSrc;
                img.style.opacity = (dragState?.fromSq === sqIndex || animatingToSq === sqIndex) ? "0" : "";
            } else if (img) {
                img.remove();
            }
        }
    }

    board.querySelectorAll(".promo-backdrop, .promo-card, .gameover-panel").forEach(el => el.remove());

    if (pendingPromotion) {
        const { fromSq, toSq } = pendingPromotion;
        const promoRank = Math.floor(toSq / 8);
        const promoFile = toSq % 8;
        const side = promoRank === 7 ? "w" : "b";

        const backdrop = document.createElement("div");
        backdrop.className = "promo-backdrop";
        backdrop.addEventListener("pointerdown", e => {
            e.stopPropagation(); pendingPromotion = null; deselect(); renderBoard(invert);
        });

        board.appendChild(backdrop);

        const card = document.createElement("div");
        card.className = "promo-card";
        card.style.left = ((invert ? 7 - promoFile : promoFile) * (board.clientWidth / 8)) + "px";
        card.style.top = promoRank === 7 ? "0px" : "auto";
        card.style.bottom = promoRank === 0 ? "0px" : "auto";

        ["Q", "R", "B", "N"].forEach(p => {
            const btn = document.createElement("div");
            btn.className = "promo-btn";
            btn.appendChild(Object.assign(document.createElement("img"), { src: `assets/images/${side}${p}.svg`, className: "piece" }));
            btn.addEventListener("pointerdown", e => {
                e.stopPropagation();
                const uci = `${"abcdefgh"[fromSq % 8]}${Math.floor(fromSq / 8) + 1}${"abcdefgh"[toSq % 8]}${Math.floor(toSq / 8) + 1}${p.toLowerCase()}`;
                sendMove(uci); pendingPromotion = null; deselect(); renderBoard(invert);
            });

            card.appendChild(btn);
        });

        board.appendChild(card);
    }
}

board.addEventListener("pointerdown", e => {
    const div = e.target.closest(".sq");
    if (!div) return;
    const sqIndex = parseInt(div.dataset.sq);

    if (!gameStarted) return;
    if (animating) return;
    if (engine.side_to_move() !== color) return;
    e.preventDefault();

    const piece = engine.piece_on(sqIndex);

    if (selectedSq !== null && legalTargets.includes(sqIndex)) {
        const fromSq = selectedSq;
        const mv = engine.legal_moves().find(m => m.from_sq() === fromSq && m.to_sq() === sqIndex);
        if (!mv) return;
        if (isPromotion(fromSq, sqIndex)) {
            pendingPromotion = { fromSq, toSq: sqIndex };
            deselect(); renderBoard(color === "b"); return;
        }
        sendMove(mv.to_uci()); deselect(); return;
    }

    if (piece) { pendingPointer = { sqIndex, piece, startX: e.clientX, startY: e.clientY }; return; }

    deselect(); renderBoard(color === "b");
});

document.addEventListener("pointermove", e => {
    if (pendingPointer && !dragState) {
        if (Math.hypot(e.clientX - pendingPointer.startX, e.clientY - pendingPointer.startY) < 5) return;
        const { sqIndex, piece } = pendingPointer;
        const ghost = Object.assign(document.createElement("img"), { src: `assets/images/${piece}.svg`, className: "piece-ghost" });

        Object.assign(ghost.style, { left: e.clientX + "px", top: e.clientY + "px" });
        document.body.appendChild(ghost);

        dragState = { fromSq: sqIndex, ghostEl: ghost };
        selectSquare(sqIndex);
        renderBoard(color === "b");
    }

    if (dragState) Object.assign(dragState.ghostEl.style, { left: e.clientX + "px", top: e.clientY + "px" });
});

document.addEventListener("pointerup", e => {
    if (pendingPointer && !dragState) {
        const { sqIndex } = pendingPointer;
        pendingPointer = null;
        selectedSq === sqIndex ? deselect() : selectSquare(sqIndex);
        renderBoard(color === "b");
        return;
    }

    pendingPointer = null;
    if (!dragState) return;

    dragState.ghostEl.remove();
    const { fromSq } = dragState;
    dragState = null;

    const toSq = parseInt(document.elementFromPoint(e.clientX, e.clientY)?.closest(".sq")?.dataset.sq);
    if (!isNaN(toSq) && toSq !== fromSq && legalTargets.includes(toSq)) {
        if (isPromotion(fromSq, toSq)) {
            pendingPromotion = { fromSq, toSq };
            deselect();
            renderBoard(color === "b");
            return;
        }

        const mv = engine.legal_moves().find(m => m.from_sq() === fromSq && m.to_sq() === toSq);
        if (mv) { sendMove(mv.to_uci()); deselect(); }
    } else {
        deselect();
    }

    renderBoard(color === "b");
});