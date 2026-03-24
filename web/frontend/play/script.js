import init, { ChessEngine } from "../wasm/wasm.js";

await init();
let engine = new ChessEngine();

const themes = {
    classic: { light: "#d9e4e8", dark: "#7b9eb2", lastMoveLight: "#5ab5d0", lastMoveDark: "#4a9ab8", selected: "#6cbad2" },
    "chess.com": { light: "#ebecd0", dark: "#739552", lastMoveLight: "#f6f682", lastMoveDark: "#bacb43", selected: "#b8c740" },
    lichess: { light: "#f0d9b5", dark: "#b58863", lastMoveLight: "#cdd26a", lastMoveDark: "#aaa23a", selected: "#d4a830" },
    arctic: { light: "#eef9ff", dark: "#4d9fd1", lastMoveLight: "#90a3f7", lastMoveDark: "#2a48cc", selected: "#a0b8ff" },
    ember: { light: "#ffe1cc", dark: "#d65a31", lastMoveLight: "#f5e26b", lastMoveDark: "#c09214", selected: "#ffec80" },
    amethyst: { light: "#f1e4ff", dark: "#7b4bc4", lastMoveLight: "#ee82f6", lastMoveDark: "#a028b4", selected: "#f698ff" },
    lagoon: { light: "#dffbf7", dark: "#1f9e89", lastMoveLight: "#8ac8e5", lastMoveDark: "#1a6888", selected: "#99daf5" },
    rose: { light: "#ffd9e8", dark: "#d65f93", lastMoveLight: "#f5e070", lastMoveDark: "#c8b020", selected: "#f0d840" },
    brass: { light: "#fff0c7", dark: "#c9a84c", lastMoveLight: "#dbf56b", lastMoveDark: "#9cbe24", selected: "#eaff80" },
    crimson: { light: "#ffd8d8", dark: "#b23a48", lastMoveLight: "#f6b27b", lastMoveDark: "#984d21", selected: "#ffbe8c" },
    nebula: { light: "#e6e0ff", dark: "#5b5bd6", lastMoveLight: "#cc7ef6", lastMoveDark: "#7c28d8", selected: "#d893ff" },
    mint: { light: "#e4fff1", dark: "#4fa87d", lastMoveLight: "#f0c060", lastMoveDark: "#c88020", selected: "#e8b040" },
    plum: { light: "#f3ddf2", dark: "#944e9a", lastMoveLight: "#e8d860", lastMoveDark: "#b8a820", selected: "#d8c840" },
    obsidian: { light: "#9ea7b3", dark: "#1f2937", lastMoveLight: "#90c0e8", lastMoveDark: "#4878b8", selected: "#80b0e0" },
    retro: { light: "#f7e7b7", dark: "#6f8f5f", lastMoveLight: "#d8c858", lastMoveDark: "#a09020", selected: "#d0c040" },
};

const pieceExt = { standard: "png", lolz: "png", neo: "png", monarchy: "webp" };

const savedSettings = JSON.parse(localStorage.getItem("settings") ?? "{}");
const pieceSet = savedSettings.pieceSet ?? "standard";
const username = savedSettings.username?.trim() || "Guest";
const savedTheme = themes[savedSettings.theme] ?? themes.classic;

const root = document.documentElement.style;
root.setProperty("--sq-light", savedTheme.light);
root.setProperty("--sq-dark", savedTheme.dark);
root.setProperty("--sq-last-move-light", savedTheme.lastMoveLight);
root.setProperty("--sq-last-move-dark", savedTheme.lastMoveDark);
root.setProperty("--sq-selected", savedTheme.selected);

function pieceImg(pieceCode) {
    return `assets/images/${pieceSet}/${pieceCode}.${pieceExt[pieceSet] ?? "svg"}`;
}

function escapeHtml(str) {
    return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

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
let clockLowPlayed = { w: false, b: false };
let selectedIsPremove = false;
let premoveQueue = [];
let gameOver = false;
let fenHistory = [];
let viewIndex = null;
let introPlayed = false;
let introRunning = false;
let introAssetsReady = false;
let introAssetsPromise = null;
let gameStartSoundPlayed = false;
let opponentInfoReady = false;
let opponentThemeKey = "classic";
let opponentPieceSet = "standard";
let opponentUsername = "Opponent";
let myAvatar = null;
let opponentAvatar = null;

const board = document.getElementById("board");
board.style.touchAction = "none";

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
const chatLog = document.getElementById("chat-log");
const chatInput = document.getElementById("chat-input");
const chatSend = document.getElementById("chat-send");

function pieceColor(piece) {
    return piece ? piece[0] : null;
}

function setFenSideToMove(fen, side) {
    const parts = fen.split(" ");
    parts[1] = side;
    return parts.join(" ");
}

function clearPremoves(shouldRender = true) {
    premoveQueue = [];
    if (selectedIsPremove) deselect();
    if (pendingPromotion?.premove) pendingPromotion = null;
    if (shouldRender) renderBoard(color === "b");
}

function sendChat() {
    const text = chatInput.value.trim();
    if (!text || !gameStarted) return;
    ws.send(JSON.stringify({ type: "chat", message: text }));

    const div = document.createElement("div");
    div.className = "chat-msg chat-mine";
    const name = document.createElement("span");
    name.className = "chat-author";
    name.textContent = username + ": ";
    div.appendChild(name);
    div.appendChild(document.createTextNode(text));
    chatLog.appendChild(div);
    chatLog.scrollTop = chatLog.scrollHeight;

    chatInput.value = "";
}

chatSend.addEventListener("pointerdown", sendChat);
chatInput.addEventListener("keydown", e => {
    if (e.key === "Enter") { e.preventDefault(); sendChat(); }
});

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

    const bottomLow = bottomMs > 0 && bottomMs < 15_000;
    clockBottom.classList.toggle("low", bottomLow);

    if (bottomLow && !clockLowPlayed[color]) {
        clockLowPlayed[color] = true;
        sfx("time_low").catch(() => { clockLowPlayed[color] = false; });
    }
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
    if (msg.clocks) {
        if (msg.clocks.w > 15_000) clockLowPlayed.w = false;
        if (msg.clocks.b > 15_000) clockLowPlayed.b = false;
        clocks = { ...msg.clocks };

        if (msg.clockActive && msg.clockAt) {
            const elapsed = Date.now() - msg.clockAt;
            clocks[msg.clockActive] = Math.max(0, clocks[msg.clockActive] - elapsed);
        }
    }

    if (msg.clockActive !== undefined) clockActive = msg.clockActive;
    if (msg.clockAt !== undefined) clockAt = Date.now();
    if (clockActive !== null) startClockTick();
    else stopClockTick();
}

async function playGameIntro() {
    if (introRunning) return;
    introRunning = true;

    document.querySelector(".game-intro")?.remove();

    const myThemeKey = savedSettings.theme ?? "classic";
    const myTheme = themes[myThemeKey] ?? themes.classic;
    const oppTheme = themes[opponentThemeKey] ?? themes.classic;

    const myColor = color;
    const oppColor = color === "w" ? "b" : "w";

    const overlay = document.createElement("div");
    overlay.className = "game-intro";

    overlay.style.setProperty("--intro-top-light", oppTheme.light);
    overlay.style.setProperty("--intro-top-dark", oppTheme.dark);
    overlay.style.setProperty("--intro-bottom-light", myTheme.light);
    overlay.style.setProperty("--intro-bottom-dark", myTheme.dark);

    overlay.innerHTML = `
        <div class="intro-camera">
            <div class="intro-half intro-half-top"></div>
            <div class="intro-half intro-half-bottom"></div>

            <div class="intro-seam"></div>
            <div class="intro-seam-glow"></div>
            <div class="intro-sheen"></div>

            <div class="intro-vignette"></div>

            <div class="intro-pieces intro-pieces-top">
                ${[
            { p: "Q", x: "20%", y: "12%", r: "-13deg", d: 0 },
            { p: "K", x: "39%", y: "7%", r: "-5deg", d: 1 },
            { p: "R", x: "61%", y: "10%", r: "6deg", d: 2 },
            { p: "N", x: "79%", y: "14%", r: "12deg", d: 3 }
        ].map(({ p, x, y, r, d }) => `
                    <img
                        class="intro-piece intro-piece-top"
                        style="left:${x}; top:${y}; --rot:${r}; --d:${d};"
                        src="assets/images/${opponentPieceSet}/${oppColor}${p}.${pieceExt[opponentPieceSet] ?? "svg"}"
                        alt=""
                    >
                `).join("")}
            </div>

            <div class="intro-pieces intro-pieces-bottom">
                ${[
            { p: "N", x: "21%", y: "72%", r: "-10deg", d: 0 },
            { p: "R", x: "40%", y: "76%", r: "-4deg", d: 1 },
            { p: "K", x: "60%", y: "73%", r: "5deg", d: 2 },
            { p: "Q", x: "78%", y: "69%", r: "12deg", d: 3 }
        ].map(({ p, x, y, r, d }) => `
                    <img
                        class="intro-piece intro-piece-bottom"
                        style="left:${x}; top:${y}; --rot:${r}; --d:${d};"
                        src="assets/images/${pieceSet}/${myColor}${p}.${pieceExt[pieceSet] ?? "svg"}"
                        alt=""
                    >
                `).join("")}
            </div>

            <div class="intro-label intro-label-top">
                <img class="intro-avatar intro-avatar-floating" src="${opponentAvatar}" alt="">
                <span class="intro-label-name">${escapeHtml(opponentUsername)}</span>
            </div>

            <div class="intro-vs">VS</div>

            <div class="intro-label intro-label-bottom">
                <img class="intro-avatar intro-avatar-floating" src="${myAvatar}" alt="">
                <span class="intro-label-name">${escapeHtml(username)}</span>
            </div>
        </div>
    `;

    document.getElementById("board-wrap").appendChild(overlay);

    overlay.querySelectorAll(".intro-label-name").forEach(el => {
        while (el.scrollWidth > 200 && parseFloat(getComputedStyle(el).fontSize) > 8) {
            el.style.fontSize = (parseFloat(getComputedStyle(el).fontSize) - 0.5) + "px";
        }
    });

    overlay.classList.add("is-playing");

    await new Promise(resolve => setTimeout(resolve, 1950));

    overlay.classList.add("is-leaving");

    await new Promise(resolve => {
        overlay.addEventListener("animationend", resolve, { once: true });
    });

    overlay.remove();
    introRunning = false;
}

function preloadIntroAssets() {
    if (introAssetsPromise) return introAssetsPromise;
    if (!colorKnown || !opponentInfoReady) return Promise.resolve(false);

    const myColor = color;
    const oppColor = color === "w" ? "b" : "w";

    const urls = [
        ...["N", "R", "K", "Q"].map(p => `assets/images/${pieceSet}/${myColor}${p}.${pieceExt[pieceSet] ?? "svg"}`),
        ...["Q", "K", "R", "N"].map(p => `assets/images/${opponentPieceSet}/${oppColor}${p}.${pieceExt[opponentPieceSet] ?? "svg"}`),
    ];

    introAssetsPromise = Promise.all(
        urls.map(src => new Promise(resolve => {
            const img = new Image();
            img.onload = () => resolve(true);
            img.onerror = () => resolve(false);
            img.src = src;

            if (img.decode) {
                img.decode().then(() => resolve(true)).catch(() => { });
            }
        }))
    ).then(() => {
        introAssetsReady = true;
        return true;
    });

    return introAssetsPromise;
}

function applyMobileBarPieceLayout() {
    const isMobile = window.matchMedia("(max-width: 900px)").matches;

    const layouts = isMobile
        ? {
            top: [60, 88, 116, 144],
            bottom: [58, 86, 114, 142],
            height: "58%"
        }
        : {
            top: [96, 160, 224, 288],
            bottom: [96, 160, 224, 288],
            height: "102%"
        };

    const topBar = document.getElementById("bar-pieces-top");
    const bottomBar = document.getElementById("bar-pieces-bottom");

    if (topBar) {
        [...topBar.querySelectorAll("img")].forEach((img, i) => {
            img.style.height = layouts.height;
            img.style.width = "auto";
            img.style.left = `${layouts.top[i] ?? layouts.top[layouts.top.length - 1]}px`;
            img.style.right = "auto";
            img.style.opacity = "1";
        });
    }

    if (bottomBar) {
        [...bottomBar.querySelectorAll("img")].forEach((img, i) => {
            img.style.height = layouts.height;
            img.style.width = "auto";
            img.style.right = `${layouts.bottom[i] ?? layouts.bottom[layouts.bottom.length - 1]}px`;
            img.style.left = "auto";
            img.style.opacity = "1";
        });
    }
}

function deselect() {
    selectedSq = null;
    legalTargets = [];
    selectedIsPremove = false;
}

function selectSquare(sq, premove = false) {
    selectedSq = sq;
    selectedIsPremove = premove;

    let srcEngine = engine;

    if (premove) {
        srcEngine = ChessEngine.from_fen(engine.get_fen());

        for (const uci of premoveQueue) {
            if (srcEngine.side_to_move() !== color) {
                srcEngine = ChessEngine.from_fen(setFenSideToMove(srcEngine.get_fen(), color));
            }

            const mv = srcEngine.parse_uci(uci);
            if (!mv) break;

            try {
                srcEngine.make_move(mv);
            } catch {
                break;
            }
        }

        if (srcEngine.side_to_move() !== color) {
            srcEngine = ChessEngine.from_fen(setFenSideToMove(srcEngine.get_fen(), color));
        }
    }

    legalTargets = srcEngine.legal_moves()
        .filter(m => m.from_sq() === sq)
        .map(m => m.to_sq());

    if (premove) {
        const piece = srcEngine.piece_on(sq);

        if (piece === "wP") {
            const file = sq % 8;
            const rank = Math.floor(sq / 8);

            if (file > 0 && rank < 7) legalTargets.push(sq + 7);
            if (file < 7 && rank < 7) legalTargets.push(sq + 9);
        } else if (piece === "bP") {
            const file = sq % 8;
            const rank = Math.floor(sq / 8);

            if (file > 0 && rank > 0) legalTargets.push(sq - 9);
            if (file < 7 && rank > 0) legalTargets.push(sq - 7);
        }

        legalTargets = [...new Set(legalTargets)];
    }
}

function pushMove(san) {
    moveHistory.push(san);
    fenHistory.push({ fen: engine.get_fen(), from: lastMove?.from ?? null, to: lastMove?.to ?? null });
    sessionStorage.setItem(`history_${gameId}`, JSON.stringify({ moveHistory, fenHistory, lastMove }));
    const i = moveHistory.length - 1;
    if (i % 2 === 0) {
        const pair = document.createElement("div");
        pair.className = "move-pair";
        pair.dataset.pair = Math.floor(i / 2);
        pair.innerHTML = `
            <span class="move-num">${Math.floor(i / 2) + 1}.</span>
            <span class="move-entry" data-idx="${i}">${san}</span>
            <span class="move-entry" data-idx=""></span>
        `;
        moveLog.appendChild(pair);
    } else {
        const pair = moveLog.querySelector(`[data-pair="${Math.floor(i / 2)}"]`);
        if (pair) {
            const blank = pair.querySelector("[data-idx='']");
            if (blank) { blank.textContent = san; blank.dataset.idx = i; }
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
    if (!piece) return uci;

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

function sendMove(uci, opts = {}) {
    const { preservePremoves = false } = opts;

    const mv = engine.parse_uci(uci);
    if (!mv) return;

    if (!preservePremoves) premoveQueue = [];

    const fromSq = uciToSq(uci.slice(0, 2));
    const toSq = uciToSq(uci.slice(2, 4));

    const fromRect = board.querySelector(`[data-sq="${fromSq}"]`)?.getBoundingClientRect();
    const toRect = board.querySelector(`[data-sq="${toSq}"]`)?.getBoundingClientRect();
    const pieceCode = engine.piece_on(fromSq);

    rollbackSnapshot = {
        fen: engine.get_fen(),
        lastMove,
        moveHistory: [...moveHistory],
        fenHistory: [...fenHistory],
        premoveQueue: [...premoveQueue],
    };

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
    const anim = Object.assign(document.createElement("img"), { src: pieceImg(pieceCode), className: "piece-anim" });

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

        socket.send(JSON.stringify({
            type: "auth",
            token,
            settings: {
                username: username,
                pieceSet: pieceSet,
                theme: savedSettings.theme ?? "classic",
                avatar: savedSettings.avatar ?? null,
            }
        }));
    });

    socket.addEventListener("close", () => {
        connStatus.classList.remove("hidden");
        setTimeout(() => { ws = connect(); }, 2000);
    });

    socket.addEventListener("message", async e => {
        const msg = JSON.parse(e.data);

        if (msg.type === "error") {
            console.warn("Move rejected:", msg.msg);

            if (rollbackSnapshot) {
                engine = ChessEngine.from_fen(rollbackSnapshot.fen);
                lastMove = rollbackSnapshot.lastMove;
                moveHistory = rollbackSnapshot.moveHistory;
                fenHistory = rollbackSnapshot.fenHistory;
                premoveQueue = rollbackSnapshot.premoveQueue ?? [];

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

            myAvatar = savedSettings.avatar ?? pieceImg(`${color}K`);
            const avatarBottom = document.getElementById("avatar-bottom");
            avatarBottom.src = savedSettings.avatar ?? pieceImg(`${color}K`);
            avatarBottom.style.display = "block";

            preloadIntroAssets();

            const barEl = document.getElementById("bar-pieces-bottom");
            barEl.innerHTML = "";
            ["K", "Q", "R", "N"].forEach((p, i) => {
                const img = document.createElement("img");
                img.src = pieceImg(`${color}${p}`);
                const rights = [96, 160, 224, 288];
                const rots = [-7, 4, -4, 7];
                img.style.cssText = `position:absolute;height:102%;width:auto;right:${rights[i]}px;top:50%;transform:translateY(-50%) rotate(${rots[i]}deg);opacity:1;pointer-events:none;filter:drop-shadow(0 2px 8px rgba(0,0,0,0.4));`;
                barEl.appendChild(img);
            });

            document.getElementById("player-name").textContent = username;

            applyMobileBarPieceLayout();

            renderBoard(color === "b");
            return;
        }

        if (msg.type === "opponent_info") {
            const opp = color === "w" ? "b" : "w";
            const oppSet = msg.pieceSet ?? "standard";
            const oppTheme = themes[msg.theme] ?? themes.classic;

            opponentThemeKey = msg.theme ?? "classic";
            opponentPieceSet = oppSet;
            opponentUsername = msg.username ?? "Opponent";
            opponentInfoReady = true;

            opponentAvatar = msg.avatar ?? `assets/images/${oppSet}/${opp}K.${pieceExt[oppSet] ?? "svg"}`;
            const avatarTop = document.getElementById("avatar-top");
            avatarTop.src = msg.avatar ?? `assets/images/${oppSet}/${opp}K.${pieceExt[oppSet] ?? "svg"}`;
            avatarTop.style.display = "block";

            preloadIntroAssets();

            const barTop = document.getElementById("player-bar-top");
            barTop.style.setProperty("--sq-dark", oppTheme.dark);
            barTop.style.setProperty("--sq-light", oppTheme.light);

            const barEl = document.getElementById("bar-pieces-top");
            barEl.innerHTML = "";
            ["K", "Q", "R", "N"].forEach((p, i) => {
                const img = document.createElement("img");
                img.src = `assets/images/${oppSet}/${opp}${p}.${pieceExt[oppSet] ?? "svg"}`;
                const lefts = [96, 160, 224, 288];
                const rots = [7, -4, 4, -7];
                img.style.cssText = `position:absolute;height:102%;width:auto;left:${lefts[i]}px;top:50%;transform:translateY(-50%) rotate(${rots[i]}deg);opacity:1;pointer-events:none;filter:drop-shadow(0 2px 8px rgba(0,0,0,0.4));`;
                barEl.appendChild(img);
            });

            applyMobileBarPieceLayout();

            document.getElementById("opponent-name").textContent = opponentUsername;
            return;
        }

        if (msg.type === "move_abort_warning") {
            const banner = document.createElement("div");
            banner.className = "draw-offer-banner";

            const isUs = msg.color === color;
            const endsAt = Date.now() + (msg.remainingMs ?? 10000);

            function tick() {
                const remaining = Math.max(0, endsAt - Date.now());
                const secs = Math.ceil(remaining / 1000);

                banner.innerHTML = `
                    <span>${isUs ? "Make a move or the game will be abandoned" : "Waiting for opponent's move - game may be abandoned soon"}</span>
                    <span style="font-size:0.65rem;color:var(--text-muted)">${secs}s remaining</span>
                `;

                if (remaining > 0) setTimeout(tick, 250);
            }

            document.querySelectorAll(".move-abort-banner").forEach(el => el.remove());
            banner.classList.add("move-abort-banner");
            document.getElementById("sidebar-actions").prepend(banner);
            tick();
            return;
        }

        if (msg.type === "sync") {
            engine = ChessEngine.from_fen(msg.fen);
            viewIndex = null;
            clearPremoves(false);
            applyClockState(msg);
            deselect();

            const stored = JSON.parse(sessionStorage.getItem(`history_${gameId}`) ?? "null");
            if (stored?.fenHistory?.length && stored.fenHistory[stored.fenHistory.length - 1]?.fen === msg.fen) {
                moveHistory = stored.moveHistory;
                fenHistory = stored.fenHistory;
                lastMove = stored.lastMove ?? null;
                moveLog.innerHTML = "";
                moveHistory.forEach((san, i) => {
                    if (i % 2 === 0) {
                        const pair = document.createElement("div");
                        pair.className = "move-pair";
                        pair.dataset.pair = Math.floor(i / 2);
                        pair.innerHTML = `
                            <span class="move-num">${Math.floor(i / 2) + 1}.</span>
                            <span class="move-entry" data-idx="${i}">${san}</span>
                            <span class="move-entry" data-idx=""></span>
                        `;
                        moveLog.appendChild(pair);
                    } else {
                        const pair = moveLog.querySelector(`[data-pair="${Math.floor(i / 2)}"]`);
                        if (pair) {
                            const blank = pair.querySelector("[data-idx='']");
                            if (blank) { blank.textContent = san; blank.dataset.idx = i; }
                        }
                    }
                });
                moveLog.scrollTop = moveLog.scrollHeight;
            } else {
                moveHistory = [];
                fenHistory = [{ fen: msg.fen, from: null, to: null }];
                lastMove = null;
                moveLog.innerHTML = "";
            }

            const newGame =
                !stored?.moveHistory?.length &&
                moveHistory.length === 0 &&
                msg.fen.startsWith("rnbqkbnr/pppppppp/");

            if (!gameStarted && !introPlayed && newGame && colorKnown && opponentInfoReady) {
                await preloadIntroAssets();

                if (!introAssetsReady) {
                    renderBoard(color === "b");
                    return;
                }

                renderBoard(color === "b");
                await playGameIntro();
                introPlayed = true;
                gameStarted = true;

                if (!gameStartSoundPlayed) {
                    gameStartSoundPlayed = true;
                    sfx("game_start").catch(() => { });
                }

                renderBoard(color === "b");
                return;
            }

            if (!gameStarted && (!newGame || introPlayed || (colorKnown && opponentInfoReady && introAssetsReady))) {
                gameStarted = true;

                if (!gameStartSoundPlayed) {
                    gameStartSoundPlayed = true;
                    sfx("game_start").catch(() => { });
                }
            }

            renderBoard(color === "b");
            return;
        }

        if (msg.type === "move") {
            document.querySelectorAll(".move-abort-banner").forEach(el => el.remove());

            const isOwnMove = rollbackSnapshot !== null;
            rollbackSnapshot = null;

            if (!gameStarted) return;

            applyClockState(msg);

            if (!isOwnMove) {
                const mv = engine.parse_uci(msg.uci);
                let san = mv ? uciToSan(msg.uci) : msg.uci;

                if (mv) engine.make_move(mv);
                if (msg.result === "checkmate_white" || msg.result === "checkmate_black") san += "#";
                else if (msg.isCheck) san += "+";

                lastMove = { from: uciToSq(msg.uci.slice(0, 2)), to: uciToSq(msg.uci.slice(2, 4)) };
                pushMove(san);

                if (msg.result !== "ongoing") sfx("game_end");
                else if (msg.isCheck) sfx("check");
                else if (msg.isPromotion) sfx("promote");
                else if (msg.isCastle) sfx("castle");
                else if (msg.isCapture) sfx("capture");
                else sfx("move");

                deselect();

                const fromSq = uciToSq(msg.uci.slice(0, 2));
                const toSq = uciToSq(msg.uci.slice(2, 4));

                if (viewIndex !== null) {
                    renderBoard(color === "b");

                    if (msg.result === "ongoing" && premoveQueue.length > 0 && engine.side_to_move() === color && !pendingPromotion) {
                        const nextUci = premoveQueue[0];
                        const legal = engine.legal_moves().some(m => m.to_uci() === nextUci);
                        if (!legal) clearPremoves();
                        else { premoveQueue.shift(); sendMove(nextUci, { preservePremoves: true }); }
                    }

                    checkGameOver(msg.result);
                } else {
                    const fromRect = board.querySelector(`[data-sq="${fromSq}"]`)?.getBoundingClientRect();
                    const toRect = board.querySelector(`[data-sq="${toSq}"]`)?.getBoundingClientRect();

                    const pieceCode = engine.piece_on(toSq) ?? engine.piece_on(fromSq);
                    if (!pieceCode) { checkGameOver(msg.result); return; }

                    animatingToSq = toSq;
                    animating = true;
                    renderBoard(color === "b");

                    const size = fromRect.width * 0.85;
                    const anim = Object.assign(document.createElement("img"), { src: pieceImg(pieceCode), className: "piece-anim" });

                    Object.assign(anim.style, { width: size + "px", height: size + "px", left: (fromRect.left + (fromRect.width - size) / 2) + "px", top: (fromRect.top + (fromRect.height - size) / 2) + "px" });
                    document.body.appendChild(anim);
                    anim.getBoundingClientRect();
                    Object.assign(anim.style, { left: (toRect.left + (toRect.width - size) / 2) + "px", top: (toRect.top + (toRect.height - size) / 2) + "px" });

                    anim.addEventListener("transitionend", () => {
                        anim.remove();
                        animating = false;
                        animatingToSq = null;
                        renderBoard(color === "b");

                        if (msg.result === "ongoing" && premoveQueue.length > 0 && engine.side_to_move() === color && !pendingPromotion) {
                            const nextUci = premoveQueue[0];
                            const legal = engine.legal_moves().some(m => m.to_uci() === nextUci);
                            if (!legal) clearPremoves();
                            else { premoveQueue.shift(); sendMove(nextUci, { preservePremoves: true }); }
                        }

                        checkGameOver(msg.result);
                    }, { once: true });
                }
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

        if (msg.type === "chat") {
            const div = document.createElement("div");
            div.className = "chat-msg";
            const name = document.createElement("span");
            name.className = "chat-author";
            name.textContent = msg.username + ": ";
            div.appendChild(name);
            div.appendChild(document.createTextNode(msg.message));
            chatLog.appendChild(div);
            chatLog.scrollTop = chatLog.scrollHeight;
            return;
        }

        if (msg.type === "game_over") {
            document.querySelectorAll(".move-abort-banner").forEach(el => el.remove());
            sfx("game_end");
            applyClockState({ ...msg, clockActive: null });
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
    if (result === "ongoing" || gameOver) return;
    gameOver = true;
    sessionStorage.removeItem(`history_${gameId}`);

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

    localStorage.removeItem("gameId");
    drawBtn.disabled = true;
    resignBtn.disabled = true;
    resignConfirm.classList.add("hidden");

    if (disconnectCountdownId) { clearTimeout(disconnectCountdownId); disconnectCountdownId = null; }
    disconnectBanner?.remove();
    disconnectBanner = null;

    const panel = document.createElement("div");
    panel.className = "gameover-panel";
    panel.innerHTML = `
        <button class="gameover-close" id="gameover-close">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
                width="14" height="14" fill="none" stroke="#c8b89a" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
        </button>
        <div class="gameover-text">
            <span class="gameover-top">${top}</span>
            <span class="gameover-bottom">${bottom}</span>
        </div>
        <div class="gameover-actions">
            <button class="gameover-btn" id="new-game-btn">New game</button>
            <button class="gameover-btn" id="home-btn">Home</button>
        </div>
    `;

    document.body.appendChild(panel);

    document.getElementById("new-game-btn").addEventListener("pointerdown", e => {
        e.stopPropagation();
        sessionStorage.setItem("autoplay", "1");
        window.location.href = "/";
    });

    document.getElementById("home-btn").addEventListener("pointerdown", e => {
        e.stopPropagation();
        window.location.href = "/";
    });

    document.getElementById("gameover-close").addEventListener("pointerdown", e => {
        e.stopPropagation();
        panel.classList.add("closing");
        panel.addEventListener("animationend", () => {
            panel.remove();

            drawBtn.classList.add("hidden");
            resignBtn.classList.add("hidden");

            const sidebarActions = document.getElementById("sidebar-actions");

            const newGameBtn = document.createElement("button");
            newGameBtn.className = "action-btn";
            newGameBtn.textContent = "New Game";
            newGameBtn.addEventListener("pointerdown", () => {
                sessionStorage.setItem("autoplay", "1");
                window.location.href = "/";
            });

            const homeBtn = document.createElement("button");
            homeBtn.className = "action-btn";
            homeBtn.textContent = "Home";
            homeBtn.addEventListener("pointerdown", () => window.location.href = "/");

            sidebarActions.appendChild(newGameBtn);
            sidebarActions.appendChild(homeBtn);
        }, { once: true });
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

function updateMoveLogHighlight() {
    moveLog.querySelectorAll(".move-entry").forEach(el => el.classList.remove("active"));
    const activeIdx = viewIndex !== null ? viewIndex - 1 : moveHistory.length - 1;
    if (activeIdx >= 0) {
        const entry = moveLog.querySelector(`.move-entry[data-idx="${activeIdx}"]`);
        if (entry) {
            entry.classList.add("active");
            entry.scrollIntoView({ block: "nearest" });
        }
    }
}

function animateHistoryMove(animFromSq, animToSq, pieceCode) {
    const fromEl = board.querySelector(`[data-sq="${animFromSq}"]`);
    const toEl = board.querySelector(`[data-sq="${animToSq}"]`);
    const fromRect = fromEl?.getBoundingClientRect();
    const toRect = toEl?.getBoundingClientRect();
    if (!fromRect || !toRect) { renderBoard(color === "b"); return; }

    renderBoard(color === "b");

    const toImg = board.querySelector(`[data-sq="${animToSq}"] img.piece`);
    if (toImg) toImg.style.opacity = "0";

    const size = fromRect.width * 0.85;
    const anim = Object.assign(document.createElement("img"), { src: pieceImg(pieceCode), className: "piece-anim" });

    Object.assign(anim.style, {
        width: size + "px", height: size + "px",
        left: (fromRect.left + (fromRect.width - size) / 2) + "px",
        top: (fromRect.top + (fromRect.height - size) / 2) + "px",
    });

    document.body.appendChild(anim);
    anim.getBoundingClientRect();
    Object.assign(anim.style, {
        left: (toRect.left + (toRect.width - size) / 2) + "px",
        top: (toRect.top + (toRect.height - size) / 2) + "px",
    });

    anim.addEventListener("transitionend", () => {
        anim.remove();
        const img = board.querySelector(`[data-sq="${animToSq}"] img.piece`);
        if (img) img.style.opacity = "";
    }, { once: true });
}

function renderBoard(invert = false) {
    if (board.querySelectorAll(".sq").length === 0) {
        for (let i = 0; i < 64; i++) {
            const div = document.createElement("div");
            div.className = "sq";
            board.appendChild(div);
        }
    }

    const inHistory = viewIndex !== null;
    const histEntry = inHistory ? fenHistory[viewIndex] : null;

    let displayEngine = engine;
    if (inHistory) {
        displayEngine = ChessEngine.from_fen(histEntry.fen);
    } else if (premoveQueue.length > 0) {
        displayEngine = ChessEngine.from_fen(engine.get_fen());

        for (const uci of premoveQueue) {
            if (displayEngine.side_to_move() !== color) displayEngine = ChessEngine.from_fen(setFenSideToMove(displayEngine.get_fen(), color));
            const mv = displayEngine.parse_uci(uci);
            if (!mv) break;
            try { displayEngine.make_move(mv); } catch { break; }
        }
    }

    const effectiveLastMove = inHistory
        ? (histEntry.from != null ? { from: histEntry.from, to: histEntry.to } : null)
        : lastMove;

    const premoveFrom = new Set();
    const premoveTo = new Set();
    if (!inHistory) {
        for (const uci of premoveQueue) {
            premoveFrom.add(uciToSq(uci.slice(0, 2)));
            premoveTo.add(uciToSq(uci.slice(2, 4)));
        }
    }

    const inCheck = displayEngine.is_in_check();
    const kingSq = inCheck ? displayEngine.king_square(displayEngine.side_to_move()) : null;
    const sqs = board.querySelectorAll(".sq");

    for (let row = 0; row < 8; row++) {
        for (let col = 0; col < 8; col++) {
            const rank = invert ? row : 7 - row;
            const file = invert ? 7 - col : col;
            const sqIndex = rank * 8 + file;
            const piece = displayEngine.piece_on(sqIndex);
            const div = sqs[row * 8 + col];

            div.className = "sq " + ((rank + file) % 2 === 0 ? "dark" : "light");
            div.dataset.sq = sqIndex;

            if (!inHistory && sqIndex === selectedSq) div.classList.add("selected");
            if (!inHistory && legalTargets.includes(sqIndex)) div.classList.add(piece ? "legal-capture" : "legal");
            if (sqIndex === kingSq) div.classList.add("in-check");
            if (effectiveLastMove && (sqIndex === effectiveLastMove.from || sqIndex === effectiveLastMove.to)) div.classList.add("last-move");
            if (!inHistory && arrowHighlights.has(sqIndex)) div.classList.add("arrow-highlight");
            if (!inHistory && premoveFrom.has(sqIndex)) div.classList.add("premove-from");
            if (!inHistory && premoveTo.has(sqIndex)) div.classList.add("premove-to");

            let img = div.querySelector("img.piece");
            if (colorKnown) {
                if (piece) {
                    if (!img) {
                        img = document.createElement("img");
                        img.className = "piece";
                        div.appendChild(img);
                    }
                    const newSrc = pieceImg(piece);
                    if (img.src !== newSrc) img.src = newSrc;
                    img.style.opacity = (!inHistory && (dragState?.fromSq === sqIndex || animatingToSq === sqIndex)) ? "0" : "";
                } else if (img) {
                    img.remove();
                }
            }
        }
    }

    board.querySelectorAll(".promo-backdrop, .promo-card" + (gameOver ? "" : ", .gameover-panel")).forEach(el => el.remove());

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
            btn.appendChild(Object.assign(document.createElement("img"), { src: pieceImg(`${side}${p}`), className: "piece" }));
            btn.addEventListener("pointerdown", e => {
                e.stopPropagation();
                const uci = `${"abcdefgh"[fromSq % 8]}${Math.floor(fromSq / 8) + 1}${"abcdefgh"[toSq % 8]}${Math.floor(toSq / 8) + 1}${p.toLowerCase()}`;

                if (pendingPromotion.premove) {
                    premoveQueue.push(uci);
                    sfx("premove");
                    renderBoard(invert);
                } else {
                    sendMove(uci);
                }

                pendingPromotion = null;
                deselect();
                renderBoard(invert);
            });

            card.appendChild(btn);
        });

        board.appendChild(card);
    }

    renderArrows(invert);
    updateMoveLogHighlight();
}

board.addEventListener("contextmenu", e => e.preventDefault());

board.addEventListener("pointerdown", e => {
    if (viewIndex !== null) return;
    if (gameOver) return;

    if (e.button === 2) {
        clearPremoves(false);
        const div = e.target.closest(".sq");
        if (!div) return;
        rightDragFrom = parseInt(div.dataset.sq);
        renderBoard(color === "b");
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
    e.preventDefault();

    const ourTurn = engine.side_to_move() === color;

    let interactionEngine = engine;
    if (!ourTurn) {
        interactionEngine = ChessEngine.from_fen(engine.get_fen());

        for (const uci of premoveQueue) {
            if (interactionEngine.side_to_move() !== color) {
                interactionEngine = ChessEngine.from_fen(setFenSideToMove(interactionEngine.get_fen(), color));
            }

            const mv = interactionEngine.parse_uci(uci);
            if (!mv) break;

            try {
                interactionEngine.make_move(mv);
            } catch {
                break;
            }
        }

        if (interactionEngine.side_to_move() !== color) {
            interactionEngine = ChessEngine.from_fen(setFenSideToMove(interactionEngine.get_fen(), color));
        }
    }

    const piece = interactionEngine.piece_on(sqIndex);

    if (selectedSq !== null && legalTargets.includes(sqIndex)) {
        const fromSq = selectedSq;

        const movingPiece = interactionEngine.piece_on(fromSq);
        const isPromo = movingPiece &&
            movingPiece[1] === "P" &&
            (Math.floor(sqIndex / 8) === 7 || Math.floor(sqIndex / 8) === 0);

        if (isPromo) {
            pendingPromotion = { fromSq, toSq: sqIndex, premove: !ourTurn };
            deselect();
            renderBoard(color === "b");
            return;
        }

        const mv = interactionEngine.legal_moves().find(m => m.from_sq() === fromSq && m.to_sq() === sqIndex);

        if (ourTurn) {
            if (!mv) return;
            sendMove(mv.to_uci());
        } else {
            let uci = mv?.to_uci();

            if (!uci) {
                const movingPiece = interactionEngine.piece_on(fromSq);
                const fromFile = fromSq % 8;
                const toFile = sqIndex % 8;

                const isPawnDiagonal =
                    movingPiece &&
                    movingPiece[1] === "P" &&
                    Math.abs(toFile - fromFile) === 1;

                if (!isPawnDiagonal) return;

                uci =
                    `${"abcdefgh"[fromSq % 8]}${Math.floor(fromSq / 8) + 1}` +
                    `${"abcdefgh"[sqIndex % 8]}${Math.floor(sqIndex / 8) + 1}`;
            }

            premoveQueue.push(uci);
            sfx("premove");
            deselect();
            arrowHighlights.clear();
            arrows = [];
            renderBoard(color === "b");
        }

        deselect();
        return;
    }

    if (piece && pieceColor(piece) === color) {
        pendingPointer = {
            sqIndex,
            piece,
            startX: e.clientX,
            startY: e.clientY,
            premove: !ourTurn,
        };
        return;
    }

    deselect();
    renderBoard(color === "b");
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

document.addEventListener("pointerdown", e => {
    if (e.button === 2 && premoveQueue.length > 0) {
        clearPremoves();
    }
}, true);

document.addEventListener("pointermove", e => {
    if (pendingPointer && !dragState) {
        if (Math.hypot(e.clientX - pendingPointer.startX, e.clientY - pendingPointer.startY) < 5) return;
        const { sqIndex, piece } = pendingPointer;
        const ghost = Object.assign(document.createElement("img"), { src: pieceImg(piece), className: "piece-ghost" });

        Object.assign(ghost.style, { left: e.clientX + "px", top: e.clientY + "px" });
        document.body.appendChild(ghost);

        dragState = { fromSq: sqIndex, ghostEl: ghost, premove: pendingPointer.premove };
        selectSquare(sqIndex, pendingPointer.premove);
        renderBoard(color === "b");
    }

    if (dragState) Object.assign(dragState.ghostEl.style, { left: e.clientX + "px", top: e.clientY + "px" });
});

document.addEventListener("pointerup", e => {
    if (pendingPointer && !dragState) {
        const { sqIndex, premove } = pendingPointer;
        pendingPointer = null;
        selectedSq === sqIndex ? deselect() : selectSquare(sqIndex, premove);
        renderBoard(color === "b");
        return;
    }

    pendingPointer = null;
    if (!dragState) return;

    dragState.ghostEl.remove();
    const { fromSq, premove } = dragState;
    dragState = null;

    const toSq = parseInt(document.elementFromPoint(e.clientX, e.clientY)?.closest(".sq")?.dataset.sq);
    if (!isNaN(toSq) && toSq !== fromSq && legalTargets.includes(toSq)) {
        let interactionEngine = engine;

        if (premove) {
            interactionEngine = ChessEngine.from_fen(engine.get_fen());

            for (const uci of premoveQueue) {
                if (interactionEngine.side_to_move() !== color) {
                    interactionEngine = ChessEngine.from_fen(setFenSideToMove(interactionEngine.get_fen(), color));
                }

                const mv = interactionEngine.parse_uci(uci);
                if (!mv) break;

                try {
                    interactionEngine.make_move(mv);
                } catch {
                    break;
                }
            }

            if (interactionEngine.side_to_move() !== color) {
                interactionEngine = ChessEngine.from_fen(setFenSideToMove(interactionEngine.get_fen(), color));
            }
        }

        const movingPiece = interactionEngine.piece_on(fromSq);
        const isPromo = movingPiece && movingPiece[1] === "P" && (Math.floor(toSq / 8) === 7 || Math.floor(toSq / 8) === 0);

        if (isPromo) {
            pendingPromotion = { fromSq, toSq, premove };
            deselect();
            renderBoard(color === "b");
            return;
        }

        const mv = interactionEngine.legal_moves().find(m => m.from_sq() === fromSq && m.to_sq() === toSq);

        if (premove) {
            let uci = mv?.to_uci();

            if (!uci) {
                const movingPiece = interactionEngine.piece_on(fromSq);
                const fromFile = fromSq % 8;
                const toFile = toSq % 8;

                const isPawnDiagonal =
                    movingPiece &&
                    movingPiece[1] === "P" &&
                    Math.abs(toFile - fromFile) === 1;

                if (!isPawnDiagonal) {
                    deselect();
                    renderBoard(color === "b");
                    return;
                }

                uci = `${"abcdefgh"[fromSq % 8]}${Math.floor(fromSq / 8) + 1}` + `${"abcdefgh"[toSq % 8]}${Math.floor(toSq / 8) + 1}`;
            }

            premoveQueue.push(uci);
            sfx("premove");
            renderBoard(color === "b");
            deselect();
        } else if (mv) {
            sendMove(mv.to_uci());
            deselect();
        }
    } else {
        deselect();
    }

    renderBoard(color === "b");
});

document.addEventListener("keydown", e => {
    if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
    if (!fenHistory.length) return;
    e.preventDefault();

    const prevViewIndex = viewIndex;

    if (e.key === "ArrowLeft") {
        const current = viewIndex ?? fenHistory.length - 1;
        if (current <= 0) return;
        viewIndex = current - 1;
    } else {
        if (viewIndex === null) return;
        const next = viewIndex + 1;
        viewIndex = next >= fenHistory.length - 1 ? null : next;
    }

    if (viewIndex === prevViewIndex) return;

    const effectiveNewIdx = viewIndex ?? fenHistory.length - 1;
    const effectivePrevIdx = prevViewIndex ?? fenHistory.length - 1;

    if (e.key === "ArrowRight") {
        const entry = fenHistory[effectiveNewIdx];
        if (entry?.from !== null) {
            const pc = ChessEngine.from_fen(entry.fen).piece_on(entry.to);
            if (pc) { animateHistoryMove(entry.from, entry.to, pc); return; }
        }
    } else {
        const entry = fenHistory[effectivePrevIdx];
        if (entry?.from !== null) {
            const pc = ChessEngine.from_fen(entry.fen).piece_on(entry.to);
            if (pc) { animateHistoryMove(entry.to, entry.from, pc); return; }
        }
    }

    renderBoard(color === "b");
});

document.addEventListener("pointercancel", () => {
    pendingPointer = null;

    if (!dragState) return;

    dragState.ghostEl?.remove();
    dragState = null;
    deselect();
    renderBoard(color === "b");
});

window.addEventListener("resize", applyMobileBarPieceLayout);

renderBoard();
updateClockDisplay();