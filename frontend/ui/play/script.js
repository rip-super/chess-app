// TODO: premoves?

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
let moveHistory = [];
let lastMove = null;
let rollbackSnapshot = null;
let arrowHighlights = new Set();
let arrows = [];
let rightDragFrom = null;
let color = "w";
let colorKnown = false;
let clocks = { w: 10 * 60 * 1000, b: 10 * 60 * 1000 };
let clockActive = null;
let clockAt = null;
let clockRafId = null;
let disconnectBanner = null;
let disconnectCountdownId = null;

const board = document.getElementById("board");
const sfx = name => Object.assign(new Audio(`assets/sounds/${name}.mp3`), { currentTime: 0 }).play();

const moveLog = document.getElementById("move-log");
const drawBtn = document.getElementById("draw-btn");
const resignBtn = document.getElementById("resign-btn");
const resignConfirm = document.getElementById("resign-confirm");
const resignYes = document.getElementById("resign-yes");
const resignNo = document.getElementById("resign-no");
const connStatus = document.getElementById("connection-status");
const clockTop = document.getElementById("clock-top");
const clockBottom = document.getElementById("clock-bottom");

resignBtn.addEventListener("pointerdown", () => {
    resignConfirm.classList.remove("hidden");
    resignBtn.classList.add("hidden");
});

resignNo.addEventListener("pointerdown", () => {
    resignConfirm.classList.add("hidden");
    resignBtn.classList.remove("hidden");
});

resignYes.addEventListener("pointerdown", () => {
    ws.send(JSON.stringify({ type: "resign" }));
    resignConfirm.classList.add("hidden");
    resignBtn.classList.remove("hidden");
});

drawBtn.addEventListener("pointerdown", () => {
    ws.send(JSON.stringify({ type: "draw_offer" }));
    drawBtn.disabled = true;
    drawBtn.textContent = "Draw offered";
});

function formatClock(ms) {
    if (ms <= 0) return "0:00";
    const totalSec = Math.ceil(ms / 1000);
    const m = Math.floor(totalSec / 60);
    const s = totalSec % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
}

function getClockMs(c) {
    if (clockActive === c && clockAt !== null) {
        return Math.max(0, clocks[c] - (Date.now() - clockAt));
    }
    return Math.max(0, clocks[c]);
}

function updateClockDisplay() {
    const opponentColor = color === "w" ? "b" : "w";

    const topMs = getClockMs(opponentColor);
    const bottomMs = getClockMs(color);

    clockTop.textContent = formatClock(topMs);
    clockBottom.textContent = formatClock(bottomMs);

    clockTop.classList.toggle("active", clockActive === opponentColor);
    clockBottom.classList.toggle("active", clockActive === color);

    clockTop.classList.toggle("low", topMs > 0 && topMs < 15_000);
    clockBottom.classList.toggle("low", bottomMs > 0 && bottomMs < 15_000);
}

function tickClocks() {
    updateClockDisplay();
    if (clockActive !== null) {
        clockRafId = requestAnimationFrame(tickClocks);
    }
}

function startClockTick() {
    if (clockRafId) cancelAnimationFrame(clockRafId);
    clockRafId = requestAnimationFrame(tickClocks);
}

function stopClockTick() {
    if (clockRafId) { cancelAnimationFrame(clockRafId); clockRafId = null; }
    updateClockDisplay();
}

function applyClockState(msg) {
    if (msg.clocks) clocks = msg.clocks;
    if (msg.clockActive !== undefined) clockActive = msg.clockActive;
    if (msg.clockAt !== undefined) clockAt = Date.now();
    if (clockActive !== null) startClockTick();
    else stopClockTick();
}

function deselect() { selectedSq = null; legalTargets = []; }
function selectSquare(sq) { selectedSq = sq; legalTargets = engine.legal_moves().filter(m => m.from_sq() === sq).map(m => m.to_sq()); }

function isPromotion(fromSq, toSq) {
    const piece = engine.piece_on(fromSq);
    if (piece !== "wP" && piece !== "bP") return false;
    return Math.floor(toSq / 8) === 7 || Math.floor(toSq / 8) === 0;
}

function pushMove(uci) {
    moveHistory.push(uci);
    const i = moveHistory.length - 1;
    if (i % 2 === 0) {
        const pair = document.createElement("div");
        pair.className = "move-pair";
        pair.dataset.pair = Math.floor(i / 2);
        pair.innerHTML = `
            <span class="move-num">${Math.floor(i / 2) + 1}.</span>
            <span class="move-entry" data-idx="${i}">${uci}</span>
            <span class="move-entry" data-idx=""></span>
        `;
        moveLog.appendChild(pair);
    } else {
        const pair = moveLog.querySelector(`[data-pair="${Math.floor(i / 2)}"]`);
        if (pair) {
            const blank = pair.querySelector("[data-idx='']");
            if (blank) { blank.textContent = uci; blank.dataset.idx = i; }
        }
    }
    moveLog.scrollTop = moveLog.scrollHeight;
}

function uciToSan(uci) {
    const files = "abcdefgh";
    const fromSq = uciToSq(uci.slice(0, 2));
    const toSq = uciToSq(uci.slice(2, 4));
    const promo = uci[4];

    const piece = engine.piece_on(fromSq);
    const pieceType = piece[1];
    const fromFile = fromSq % 8;
    const fromRank = Math.floor(fromSq / 8);
    const toFile = toSq % 8;
    const toRank = Math.floor(toSq / 8);
    const toCoord = files[toFile] + (toRank + 1);

    if (pieceType === "K") {
        if (fromFile === 4 && toFile === 6) return "O-O";
        if (fromFile === 4 && toFile === 2) return "O-O-O";
    }

    const isCapture = !!engine.piece_on(toSq);
    const isEnPassant = pieceType === "P" && fromFile !== toFile && !engine.piece_on(toSq);
    const capture = isCapture || isEnPassant;

    if (pieceType === "P") {
        let san = "";
        if (capture) san += files[fromFile] + "x";
        san += toCoord;
        if (promo) san += "=" + promo.toUpperCase();
        return san;
    }

    const ambiguous = engine.legal_moves().filter(m =>
        m.to_sq() === toSq &&
        m.from_sq() !== fromSq &&
        engine.piece_on(m.from_sq()) === piece
    );

    let disambig = "";
    if (ambiguous.length > 0) {
        const sameFile = ambiguous.some(m => m.from_sq() % 8 === fromFile);
        const sameRank = ambiguous.some(m => Math.floor(m.from_sq() / 8) === fromRank);
        if (!sameFile) disambig = files[fromFile];
        else if (!sameRank) disambig = (fromRank + 1).toString();
        else disambig = files[fromFile] + (fromRank + 1);
    }

    let san = pieceType + disambig;
    if (capture) san += "x";
    san += toCoord;
    return san;
}

let token = localStorage.getItem("token");
if (!token) { token = crypto.randomUUID(); localStorage.setItem("token", token); }

const gameId = window.location.pathname.split("/").pop();

function sendMove(uci) {
    const mv = engine.parse_uci(uci);
    if (!mv) return;

    const fromSq = uciToSq(uci.slice(0, 2));
    const toSq = uciToSq(uci.slice(2, 4));

    const fromRect = board.querySelector(`[data-sq="${fromSq}"]`)?.getBoundingClientRect();
    const toRect = board.querySelector(`[data-sq="${toSq}"]`)?.getBoundingClientRect();
    const pieceCode = engine.piece_on(fromSq);

    rollbackSnapshot = { fen: engine.get_fen(), lastMove, moveHistory: [...moveHistory] };

    let san = uciToSan(uci);
    engine.make_move(mv);
    san += engine.is_in_check() ? "+" : "";

    lastMove = { from: fromSq, to: toSq };
    pushMove(san);
    deselect();
    arrowHighlights.clear();
    arrows = [];

    if (engine.is_in_check()) sfx("check");
    else if (mv.is_promotion()) sfx("promote");
    else if (mv.is_castle()) sfx("castle");
    else if (mv.is_capture()) sfx("capture");
    else sfx("move");

    animatingToSq = toSq;
    animating = true;
    renderBoard(color === "b");

    const size = fromRect.width * 0.85;
    const anim = Object.assign(document.createElement("img"), { src: `assets/images/${pieceCode}.png`, className: "piece-anim" });

    Object.assign(anim.style, { width: size + "px", height: size + "px", left: (fromRect.left + (fromRect.width - size) / 2) + "px", top: (fromRect.top + (fromRect.height - size) / 2) + "px" });
    document.body.appendChild(anim);

    anim.getBoundingClientRect();

    Object.assign(anim.style, { left: (toRect.left + (toRect.width - size) / 2) + "px", top: (toRect.top + (toRect.height - size) / 2) + "px" });
    anim.addEventListener("transitionend", () => {
        anim.remove();
        animating = false; animatingToSq = null;
        renderBoard(color === "b");
    }, { once: true });

    ws.send(JSON.stringify({ type: "move", uci }));
}

function connect() {
    const socket = new WebSocket(`${location.protocol === "https:" ? "wss:" : "ws:"}//${location.host}/ws/${gameId}`);

    socket.addEventListener("open", () => {
        connStatus.classList.add("hidden");
        socket.send(JSON.stringify({ type: "auth", token }));
    });

    socket.addEventListener("close", () => {
        connStatus.classList.remove("hidden");
        setTimeout(() => { ws = connect(); }, 2000);
    });

    socket.addEventListener("message", e => {
        const msg = JSON.parse(e.data);

        if (msg.type === "error") {
            console.warn("Move rejected:", msg.msg);

            if (rollbackSnapshot) {
                engine = ChessEngine.from_fen(rollbackSnapshot.fen);
                lastMove = rollbackSnapshot.lastMove;
                moveHistory = rollbackSnapshot.moveHistory;

                const lastPair = moveLog.lastElementChild;
                if (lastPair) {
                    const entries = lastPair.querySelectorAll(".move-entry");
                    const lastFilled = [...entries].reverse().find(e => e.textContent);
                    if (lastFilled) {
                        if (lastFilled.dataset.idx % 2 === 0) lastPair.remove();
                        else lastFilled.textContent = "";
                    }
                }

                rollbackSnapshot = null;
            }

            deselect();
            renderBoard(color === "b");
            return;
        }

        if (msg.type === "not_found") {
            localStorage.removeItem("gameId");
            window.location.href = "/";
            return;
        }

        if (msg.type === "assign") {
            color = msg.color;
            colorKnown = true;
            renderBoard(color === "b");
            return;
        }

        if (msg.type === "sync") {
            engine = ChessEngine.from_fen(msg.fen);
            applyClockState(msg);
            deselect();

            if (!gameStarted) {
                gameStarted = true;
                sfx("game_start").catch(() => { });
            }

            renderBoard(color === "b");
            return;
        }

        if (msg.type === "move") {
            const isOwnMove = rollbackSnapshot !== null;
            rollbackSnapshot = null;

            applyClockState(msg);

            if (!isOwnMove) {
                const mv = engine.parse_uci(msg.uci);
                let san = mv ? uciToSan(msg.uci) : msg.uci;

                if (mv) engine.make_move(mv);
                if (msg.result === "checkmate_white" || msg.result === "checkmate_black") san += "#";
                else if (msg.isCheck) san += "+";

                pushMove(san);
                lastMove = { from: uciToSq(msg.uci.slice(0, 2)), to: uciToSq(msg.uci.slice(2, 4)) };

                if (msg.result !== "ongoing") sfx("game_end");
                else if (msg.isCheck) sfx("check");
                else if (msg.isPromotion) sfx("promote");
                else if (msg.isCastle) sfx("castle");
                else if (msg.isCapture) sfx("capture");
                else sfx("move");

                deselect();

                const fromSq = uciToSq(msg.uci.slice(0, 2));
                const toSq = uciToSq(msg.uci.slice(2, 4));
                const fromRect = board.querySelector(`[data-sq="${fromSq}"]`)?.getBoundingClientRect();
                const toRect = board.querySelector(`[data-sq="${toSq}"]`)?.getBoundingClientRect();
                const pieceCode = engine.piece_on(toSq);

                animatingToSq = toSq;
                animating = true;
                renderBoard(color === "b");

                const size = fromRect.width * 0.85;
                const anim = Object.assign(document.createElement("img"), { src: `assets/images/${pieceCode}.png`, className: "piece-anim" });

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
            } else {
                if (moveHistory.length > 0) {
                    const lastIdx = moveHistory.length - 1;
                    const correctSuffix = (msg.result === "checkmate_white" || msg.result === "checkmate_black") ? "#" : msg.isCheck ? "+" : "";
                    moveHistory[lastIdx] = moveHistory[lastIdx].replace(/[+#]?$/, correctSuffix);
                    const entry = moveLog.querySelector(`[data-idx="${lastIdx}"]`);
                    if (entry) entry.textContent = moveHistory[lastIdx];
                }

                if (msg.result !== "ongoing") sfx("game_end");
                checkGameOver(msg.result);
            }
        }

        if (msg.type === "draw_offer") {
            const banner = document.createElement("div");
            banner.className = "draw-offer-banner";
            banner.innerHTML = `
            <span>Draw offered</span>
            <div class="offer-btns">
                <button class="action-btn" id="draw-accept">Accept</button>
                <button class="action-btn danger" id="draw-decline">Decline</button>
            </div>
        `;
            document.getElementById("sidebar-actions").prepend(banner);

            document.getElementById("draw-accept").addEventListener("pointerdown", () => {
                ws.send(JSON.stringify({ type: "draw_accepted" }));
                banner.remove();
            });
            document.getElementById("draw-decline").addEventListener("pointerdown", () => {
                ws.send(JSON.stringify({ type: "draw_declined" }));
                banner.remove();
            });
            return;
        }

        if (msg.type === "draw_declined") {
            drawBtn.disabled = false;
            drawBtn.textContent = "Draw";
            return;
        }

        if (msg.type === "opponent_disconnected") {
            disconnectBanner = document.createElement("div");
            disconnectBanner.className = "draw-offer-banner";
            document.getElementById("sidebar-actions").prepend(disconnectBanner);

            const endsAt = Date.now() + msg.claimInMs;

            function updateDisconnectBanner() {
                const remaining = Math.max(0, endsAt - Date.now());
                const secs = Math.ceil(remaining / 1000);
                const m = Math.floor(secs / 60);
                const s = secs % 60;
                const timeStr = `${m}:${s.toString().padStart(2, "0")}`;
                disconnectBanner.innerHTML = `
            <span>Opponent disconnected</span>
            <span style="font-size:0.65rem;color:var(--text-muted)">Claim victory in ${timeStr}</span>`;
                if (remaining > 0) disconnectCountdownId = setTimeout(updateDisconnectBanner, 500);
            }

            updateDisconnectBanner();
            return;
        }

        if (msg.type === "opponent_reconnected") {
            if (disconnectCountdownId) { clearTimeout(disconnectCountdownId); disconnectCountdownId = null; }
            disconnectBanner?.remove();
            disconnectBanner = null;
            return;
        }

        if (msg.type === "can_claim_victory") {
            if (disconnectCountdownId) { clearTimeout(disconnectCountdownId); disconnectCountdownId = null; }
            if (disconnectBanner) {
                disconnectBanner.innerHTML = `
            <span>Opponent still gone</span>
            <div class="offer-btns">
                <button class="action-btn" id="claim-victory-btn">Claim victory</button>
            </div>`;
                document.getElementById("claim-victory-btn").addEventListener("pointerdown", () => {
                    ws.send(JSON.stringify({ type: "claim_victory" }));
                });
            }
            return;
        }

        if (msg.type === "game_over") {
            sfx("game_end");
            applyClockState({ clockActive: null });
            checkGameOver(msg.result);
            return;
        }
    });

    return socket;
}

let ws = connect();

function uciToSq(coord) {
    return (parseInt(coord[1]) - 1) * 8 + (coord.charCodeAt(0) - 97);
}

function checkGameOver(result) {
    if (result === "ongoing") return;
    stopClockTick();

    const msgs = {
        checkmate_white: { top: "Checkmate", bottom: "White wins!" },
        checkmate_black: { top: "Checkmate", bottom: "Black wins!" },
        stalemate: { top: "Stalemate", bottom: "Draw!" },
        draw_repetition: { top: "Draw", bottom: "By Repetition" },
        draw_fifty_move: { top: "Draw", bottom: "By Fifty Move Rule" },
        draw_insufficient: { top: "Draw", bottom: "By Insufficient Material" },
        resign_white: { top: "Resignation", bottom: "Black wins!" },
        resign_black: { top: "Resignation", bottom: "White wins!" },
        draw_agreed: { top: "Draw", bottom: "By Mutual Agreement" },
        timeout_white: { top: "Time Out", bottom: "Black wins!" },
        timeout_black: { top: "Time Out", bottom: "White wins!" },
        abandon_white: { top: "Game Abandoned", bottom: "Black wins!" },
        abandon_black: { top: "Game Abandoned", bottom: "White wins!" },
    };

    const { top, bottom } = msgs[result] ?? { top: "Game Over", bottom: "" };
    const panel = document.createElement("div");

    panel.className = "gameover-panel";
    panel.innerHTML = `
        <div class="gameover-text">
            <span class="gameover-top">${top}</span>
            <span class="gameover-bottom">${bottom}</span>
        </div>
        <div class="gameover-actions">
            <button class="gameover-btn" id="new-game-btn">New game</button>
            <button class="gameover-btn" id="home-btn">Home</button>
        </div>
    `;

    localStorage.removeItem("gameId");

    drawBtn.disabled = true;
    resignBtn.disabled = true;

    if (disconnectCountdownId) { clearTimeout(disconnectCountdownId); disconnectCountdownId = null; }
    disconnectBanner?.remove();
    disconnectBanner = null;

    board.appendChild(panel);

    document.getElementById("new-game-btn").addEventListener("pointerdown", e => {
        e.stopPropagation();
        sessionStorage.setItem("autoplay", "1");
        window.location.href = "/";
    });

    document.getElementById("home-btn").addEventListener("pointerdown", e => {
        e.stopPropagation();
        window.location.href = "/";
    });
}

function renderArrows(invert) {
    board.querySelectorAll("svg.arrows").forEach(el => el.remove());
    if (arrows.length === 0) return;

    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.classList.add("arrows");
    svg.setAttribute("viewBox", "0 0 8 8");
    svg.style.cssText = "position:absolute;inset:0;width:100%;height:100%;pointer-events:none;z-index:10;overflow:visible;";

    for (const { from, to } of arrows) {
        const fx = (invert ? 7 - (from % 8) : from % 8) + 0.5;
        const fy = (invert ? Math.floor(from / 8) : 7 - Math.floor(from / 8)) + 0.5;
        const tx = (invert ? 7 - (to % 8) : to % 8) + 0.5;
        const ty = (invert ? Math.floor(to / 8) : 7 - Math.floor(to / 8)) + 0.5;

        const fileDiff = Math.abs((to % 8) - (from % 8));
        const rankDiff = Math.abs(Math.floor(to / 8) - Math.floor(from / 8));
        const isKnight = (fileDiff === 2 && rankDiff === 1) || (fileDiff === 1 && rankDiff === 2);

        const poly = document.createElementNS("http://www.w3.org/2000/svg", "polygon");
        poly.setAttribute("fill", "#E5A824");
        poly.setAttribute("fill-opacity", "0.85");

        if (isKnight) {
            const ex = fx, ey = ty;

            const dx1 = ex - fx, dy1 = ey - fy;
            const dx2 = tx - ex, dy2 = ty - ey;

            const len2 = Math.sqrt(dx2 * dx2 + dy2 * dy2);
            const ux2 = dx2 / len2, uy2 = dy2 / len2;
            const px2 = -uy2, py2 = ux2;

            const shaftW = 0.09, headW = 0.22, headLen = 0.3;
            const shaftEndX = tx - ux2 * headLen, shaftEndY = ty - uy2 * headLen;

            const hlen = Math.sqrt(dx1 * dx1 + dy1 * dy1);
            const uhx = dx1 / hlen, uhy = dy1 / hlen;
            const phx = -uhy, phy = uhx;

            const points = [
                [fx + phx * shaftW, fy + phy * shaftW],
                [ex + phx * shaftW, ey + py2 * shaftW],
                [shaftEndX + px2 * shaftW, shaftEndY + py2 * shaftW],
                [shaftEndX + px2 * headW, shaftEndY + py2 * headW],
                [tx, ty],
                [shaftEndX - px2 * headW, shaftEndY - py2 * headW],
                [shaftEndX - px2 * shaftW, shaftEndY - py2 * shaftW],
                [ex - phx * shaftW, ey - py2 * shaftW],
                [fx - phx * shaftW, fy - phy * shaftW],
            ].map(([x, y]) => `${x},${y}`).join(" ");

            poly.setAttribute("points", points);
        } else {
            const dx = tx - fx, dy = ty - fy;
            const len = Math.sqrt(dx * dx + dy * dy);
            const ux = dx / len, uy = dy / len;
            const px = -uy, py = ux;

            const shaftW = 0.09, headW = 0.22, headLen = 0.3;
            const shaftEnd = 1 - headLen / len;

            const sx = fx + ux * 0.3, sy = fy + uy * 0.3;
            const ex = fx + ux * len * shaftEnd, ey = fy + uy * len * shaftEnd;

            const points = [
                [sx + px * shaftW, sy + py * shaftW],
                [ex + px * shaftW, ey + py * shaftW],
                [ex + px * headW, ey + py * headW],
                [tx, ty],
                [ex - px * headW, ey - py * headW],
                [ex - px * shaftW, ey - py * shaftW],
                [sx - px * shaftW, sy - py * shaftW],
            ].map(([x, y]) => `${x},${y}`).join(" ");

            poly.setAttribute("points", points);
        }

        svg.appendChild(poly);
    }

    board.appendChild(svg);
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
            if (lastMove && (sqIndex === lastMove.from || sqIndex === lastMove.to)) div.classList.add("last-move");
            if (arrowHighlights.has(sqIndex)) div.classList.add("arrow-highlight");

            let img = div.querySelector("img.piece");
            if (colorKnown) {
                if (piece) {
                    if (!img) {
                        img = document.createElement("img");
                        img.className = "piece";
                        div.appendChild(img);
                    }
                    const newSrc = `assets/images/${piece}.png`;
                    if (img.src !== newSrc) img.src = newSrc;
                    img.style.opacity = (dragState?.fromSq === sqIndex || animatingToSq === sqIndex) ? "0" : "";
                } else if (img) {
                    img.remove();
                }
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

        const atTop = (promoRank === 7) !== invert;
        card.style.top = atTop ? "0px" : "auto";
        card.style.bottom = atTop ? "auto" : "0px";

        ["Q", "R", "B", "N"].forEach(p => {
            const btn = document.createElement("div");
            btn.className = "promo-btn";
            btn.appendChild(Object.assign(document.createElement("img"), { src: `assets/images/${side}${p}.png`, className: "piece" }));
            btn.addEventListener("pointerdown", e => {
                e.stopPropagation();
                const uci = `${"abcdefgh"[fromSq % 8]}${Math.floor(fromSq / 8) + 1}${"abcdefgh"[toSq % 8]}${Math.floor(toSq / 8) + 1}${p.toLowerCase()}`;
                sendMove(uci); pendingPromotion = null; deselect(); renderBoard(invert);
            });

            card.appendChild(btn);
        });

        board.appendChild(card);
    }

    renderArrows(invert);
}

board.addEventListener("contextmenu", e => e.preventDefault());

board.addEventListener("pointerdown", e => {
    if (e.button === 2) {
        const div = e.target.closest(".sq");
        if (!div) return;
        rightDragFrom = parseInt(div.dataset.sq);
        return;
    }

    if (arrowHighlights.size > 0 || arrows.length > 0) {
        arrowHighlights.clear();
        arrows = [];
        renderBoard(color === "b");
    }

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

board.addEventListener("pointerup", e => {
    if (e.button === 2) {
        const div = e.target.closest(".sq");
        if (!div || rightDragFrom === null) { rightDragFrom = null; return; }
        const toSq = parseInt(div.dataset.sq);

        if (toSq === rightDragFrom) {
            if (arrowHighlights.has(toSq)) arrowHighlights.delete(toSq);
            else arrowHighlights.add(toSq);
        } else {
            const idx = arrows.findIndex(a => a.from === rightDragFrom && a.to === toSq);
            if (idx >= 0) arrows.splice(idx, 1);
            else arrows.push({ from: rightDragFrom, to: toSq });
        }

        rightDragFrom = null;
        renderBoard(color === "b");
        return;
    }
});

document.addEventListener("pointermove", e => {
    if (pendingPointer && !dragState) {
        if (Math.hypot(e.clientX - pendingPointer.startX, e.clientY - pendingPointer.startY) < 5) return;
        const { sqIndex, piece } = pendingPointer;
        const ghost = Object.assign(document.createElement("img"), { src: `assets/images/${piece}.png`, className: "piece-ghost" });

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

renderBoard();
updateClockDisplay();