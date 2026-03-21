window.addEventListener("pageshow", () => {
    const activeGame = localStorage.getItem("gameId");
    if (activeGame) window.location.href = `/play/${activeGame}`;
});

const playBtn = document.getElementById("play-btn");
const settingsBtn = document.getElementById("settings-btn");
const statusText = document.getElementById("status");
const tcOverlay = document.getElementById("tc-overlay");
const tcCancel = document.getElementById("tc-cancel");
const settingsOverlay = document.getElementById("settings-overlay");
const settingsClose = document.getElementById("settings-close");
const settingsCancel = document.getElementById("settings-cancel");
const settingsSave = document.getElementById("settings-save");
const settingsReset = document.getElementById("settings-reset");
const usernameInput = document.getElementById("username-input");
const themePicker = document.getElementById("theme-picker");
const piecePicker = document.getElementById("piece-picker");
const previewBoard = document.getElementById("settings-preview-board");

let activeInterval = null;
let isMatchmaking = false;
const previewSquares = [];
let previewBoardBuilt = false;
let previewRenderToken = 0;
let lastRenderedPieceSet = null;

const previewThemes = {
    classic: { light: "#d9e4e8", dark: "#7b9eb2", lastMoveLight: "#5ab5d0", lastMoveDark: "#4a9ab8", selected: "#6cbad2", panel: "#182229" },
    "chess.com": { light: "#ebecd0", dark: "#739552", lastMoveLight: "#f6f682", lastMoveDark: "#bacb43", selected: "#b8c740", panel: "#1f2a1a" },
    lichess: { light: "#f0d9b5", dark: "#b58863", lastMoveLight: "#cdd26a", lastMoveDark: "#aaa23a", selected: "#d4a830", panel: "#201a17" },
    arctic: { light: "#eef9ff", dark: "#4d9fd1", lastMoveLight: "#90a3f7", lastMoveDark: "#2a48cc", selected: "#a0b8ff", panel: "#0d1e28" },
    ember: { light: "#ffe1cc", dark: "#d65a31", lastMoveLight: "#f5e26b", lastMoveDark: "#c09214", selected: "#ffec80", panel: "#2b120c" },
    amethyst: { light: "#f1e4ff", dark: "#7b4bc4", lastMoveLight: "#ee82f6", lastMoveDark: "#a028b4", selected: "#f698ff", panel: "#1d1430" },
    lagoon: { light: "#dffbf7", dark: "#1f9e89", lastMoveLight: "#8ac8e5", lastMoveDark: "#1a6888", selected: "#99daf5", panel: "#0d201d" },
    rose: { light: "#ffd9e8", dark: "#d65f93", lastMoveLight: "#f5e070", lastMoveDark: "#c8b020", selected: "#f0d840", panel: "#2a1520" },
    brass: { light: "#fff0c7", dark: "#c9a84c", lastMoveLight: "#dbf56b", lastMoveDark: "#9cbe24", selected: "#eaff80", panel: "#241a0c" },
    crimson: { light: "#ffd8d8", dark: "#b23a48", lastMoveLight: "#f6b27b", lastMoveDark: "#984d21", selected: "#ffbe8c", panel: "#2a1014" },
    nebula: { light: "#e6e0ff", dark: "#5b5bd6", lastMoveLight: "#cc7ef6", lastMoveDark: "#7c28d8", selected: "#d893ff", panel: "#151533" },
    mint: { light: "#e4fff1", dark: "#4fa87d", lastMoveLight: "#f0c060", lastMoveDark: "#c88020", selected: "#e8b040", panel: "#102019" },
    plum: { light: "#f3ddf2", dark: "#944e9a", lastMoveLight: "#e8d860", lastMoveDark: "#b8a820", selected: "#d8c840", panel: "#231226" },
    obsidian: { light: "#9ea7b3", dark: "#1f2937", lastMoveLight: "#90c0e8", lastMoveDark: "#4878b8", selected: "#80b0e0", panel: "#0a0f18" },
    retro: { light: "#f7e7b7", dark: "#6f8f5f", lastMoveLight: "#d8c858", lastMoveDark: "#a09020", selected: "#d0c040", panel: "#1b2116" },
};

const pieceSets = [
    "standard", "alpha", "anarchy", "caliente", "california",
    "cardinal", "cburnett", "celtic", "chess7", "chessnut",
    "companion", "cooke", "fantasy", "fresca", "gioco",
    "governor", "horsey", "icpieces", "kiwen-suwi", "kosal",
    "leipzig", "letter", "lolz", "maestro", "merida",
    "monarchy", "mpchess", "neo", "pixel", "riohacha",
    "shapes", "spatial", "staunty", "tatiana", "xkcd",
];

const previewPosition = {
    0: "bR",
    1: "bN",
    2: "bB",
    3: "bK",
    4: "bP",
    6: "bP",
    9: "wP",
    11: "bP",
    12: "wQ",
    13: "wP",
    14: "wN",
    15: "wK",
};

const pieceExt = {
    standard: "png",
    lolz: "png",
    neo: "png",
    monarchy: "webp",
};

const DEFAULT_SETTINGS = {
    username: generateUsername(),
    theme: "classic",
    pieceSet: "standard",
};

const PREVIEW_LAST_MOVE = new Set([8, 12]);

let selectedTheme = DEFAULT_SETTINGS.theme;
let activeTheme = DEFAULT_SETTINGS.theme;
let selectedPieceSet = DEFAULT_SETTINGS.pieceSet;
let previewSelectedIdx = null;

function generateUsername() {
    const adjectives = [
        "Quiet", "Swift", "Bold", "Silver", "Golden",
        "Rapid", "Calm", "Shadow", "Royal", "Clever"
    ];

    const chessWords = [
        "Pawn", "Knight", "Bishop", "Rook", "Queen",
        "King", "Castle", "Fork", "Pin", "Gambit"
    ];

    const adjective = adjectives[Math.floor(Math.random() * adjectives.length)];
    const chessWord = chessWords[Math.floor(Math.random() * chessWords.length)];
    const number = Math.floor(Math.random() * 90) + 10;

    return `${adjective}${chessWord}${number}`;
}

function preloadImage(src) {
    return new Promise(resolve => {
        const img = new Image();
        img.onload = () => resolve({ ok: true, src });
        img.onerror = () => resolve({ ok: false, src });
        img.src = src;
    });
}

function getPieceAssetPath(pieceSet, pieceCode) {
    return `play/assets/images/${pieceSet}/${pieceCode}.${pieceExt[pieceSet] || "svg"}`;
}

function renderSettingsUI() {
    themePicker.innerHTML = "";
    piecePicker.innerHTML = "";

    for (const [themeName, palette] of Object.entries(previewThemes)) {
        const button = document.createElement("button");
        button.type = "button";
        button.className = "option-tile";
        if (themeName === selectedTheme) button.classList.add("active");
        button.dataset.theme = themeName;

        const swatch = document.createElement("div");
        swatch.className = "theme-swatch";

        for (let i = 0; i < 4; i++) {
            const sq = document.createElement("div");
            sq.className = "theme-swatch-square";
            sq.style.background = i % 2 === 0 ? palette.light : palette.dark;
            swatch.appendChild(sq);
        }

        const label = document.createElement("span");
        label.className = "option-tile-name";
        label.textContent = themeName[0].toUpperCase() + themeName.slice(1);

        button.appendChild(swatch);
        button.appendChild(label);
        themePicker.appendChild(button);
    }

    for (const setName of pieceSets) {
        const button = document.createElement("button");
        button.type = "button";
        button.className = "piece-option";
        if (setName === selectedPieceSet) button.classList.add("active");
        button.dataset.pieceSet = setName;

        const img = document.createElement("img");
        img.className = "piece-option-thumb";
        img.src = getPieceAssetPath(setName, "wN");
        img.alt = `${setName} knight`;

        const label = document.createElement("span");
        label.className = "piece-option-name";
        label.textContent = setName
            .split("-")
            .map(part => part[0].toUpperCase() + part.slice(1))
            .join(" ");

        button.appendChild(img);
        button.appendChild(label);
        piecePicker.appendChild(button);
    }
}

function openSettings() {
    settingsOverlay.classList.remove("hidden");
    renderSettingsUI();
    renderSettingsPreview();
}

function closeSettings() {
    selectedTheme = activeTheme;
    settingsOverlay.classList.add("hidden");
}

function loadSettings() {
    const saved = JSON.parse(localStorage.getItem("settings") ?? "{}");

    const settings = {
        username: saved.username?.trim() || generateUsername(),
        theme: saved.theme ?? DEFAULT_SETTINGS.theme,
        pieceSet: saved.pieceSet ?? DEFAULT_SETTINGS.pieceSet,
    };

    localStorage.setItem("settings", JSON.stringify(settings));

    usernameInput.value = settings.username;
    selectedTheme = settings.theme;
    activeTheme = settings.theme;
    selectedPieceSet = settings.pieceSet;
}

function resetSettingsForm() {
    const saved = JSON.parse(localStorage.getItem("settings") ?? "{}");
    usernameInput.value = saved.username ?? generateUsername();
    selectedTheme = DEFAULT_SETTINGS.theme;
    selectedPieceSet = DEFAULT_SETTINGS.pieceSet;

    renderSettingsUI();
    renderSettingsPreview();
}

function applyPreviewSquareColors() {
    const palette = previewThemes[selectedTheme] || previewThemes.classic;
    for (const item of previewSquares) {
        const row = Math.floor(item.index / 4);
        const col = item.index % 4;
        const isLight = (row + col) % 2 === 0;

        if (item.index === previewSelectedIdx) {
            item.square.style.background = palette.selected;
        } else if (PREVIEW_LAST_MOVE.has(item.index)) {
            item.square.style.background = isLight ? palette.lastMoveLight : palette.lastMoveDark;
        } else {
            item.square.style.background = isLight ? palette.light : palette.dark;
        }
    }
}

function buildSettingsPreviewBoard() {
    if (previewBoardBuilt) return;

    previewBoard.innerHTML = "";
    previewSquares.length = 0;

    for (let row = 0; row < 4; row++) {
        for (let col = 0; col < 4; col++) {
            const sq = row * 4 + col;

            const square = document.createElement("div");
            square.className = "preview-square";
            square.style.cursor = "pointer";

            const img = document.createElement("img");
            img.className = "preview-piece";
            img.alt = "";
            img.draggable = false;
            img.style.display = "block";

            square.appendChild(img);
            previewBoard.appendChild(square);

            const item = { square, img, index: sq };
            previewSquares.push(item);

            square.addEventListener("click", () => {
                if (!previewPosition[sq]) return;
                previewSelectedIdx = previewSelectedIdx === sq ? null : sq;
                applyPreviewSquareColors();
            });
        }
    }

    previewBoardBuilt = true;
}

async function renderSettingsPreview() {
    const renderToken = ++previewRenderToken;
    const palette = previewThemes[selectedTheme] || previewThemes.classic;
    const pieceSetChanged = selectedPieceSet !== lastRenderedPieceSet;

    buildSettingsPreviewBoard();
    previewBoard.style.background = palette.panel;
    applyPreviewSquareColors();

    const imageSources = [];

    for (const item of previewSquares) {
        const pieceCode = previewPosition[item.index];
        if (pieceCode) imageSources.push(getPieceAssetPath(selectedPieceSet, pieceCode));
    }

    if (!pieceSetChanged) return;

    for (const item of previewSquares) {
        item.img.style.opacity = "0";
    }

    await Promise.all([
        ...[...new Set(imageSources)].map(preloadImage),
        new Promise(r => setTimeout(r, 180)),
    ]);

    if (renderToken !== previewRenderToken) return;

    for (const item of previewSquares) {
        const pieceCode = previewPosition[item.index];
        if (pieceCode) {
            item.img.src = getPieceAssetPath(selectedPieceSet, pieceCode);
            item.img.alt = pieceCode;
        }
    }

    await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    if (renderToken !== previewRenderToken) return;

    lastRenderedPieceSet = selectedPieceSet;

    for (const item of previewSquares) {
        const pieceCode = previewPosition[item.index];
        if (pieceCode) item.img.style.opacity = "1";
    }
}

function cancelMatchmaking() {
    isMatchmaking = false;
    if (activeInterval) { clearInterval(activeInterval); activeInterval = null; }
    playBtn.disabled = false;
    playBtn.querySelector("span").textContent = "Play";
    statusText.innerHTML = "";
}

async function startMatchmaking(tc) {
    isMatchmaking = true;
    playBtn.disabled = false;
    playBtn.querySelector("span").textContent = "Cancel";
    statusText.innerHTML = "Finding a game<span class='dots'></span>";

    try {
        const res = await fetch(`/match?tc=${encodeURIComponent(tc)}`);
        const data = await res.json();

        if (data.gameId && !data.waiting) {
            isMatchmaking = false;
            localStorage.setItem("gameId", data.gameId);
            window.location.href = `/play/${data.gameId}`;
            return;
        }

        if (data.waiting) {
            const gameId = data.gameId;
            activeInterval = setInterval(async () => {
                try {
                    const r = await fetch(`/match/${gameId}?tc=${encodeURIComponent(tc)}`);
                    const d = await r.json();
                    if (d.gameId) {
                        clearInterval(activeInterval);
                        activeInterval = null;
                        isMatchmaking = false;
                        localStorage.setItem("gameId", d.gameId);
                        window.location.href = `/play/${d.gameId}`;
                    } else if (d.error) {
                        cancelMatchmaking();
                        statusText.textContent = "Something went wrong - try again.";
                    }
                } catch {
                    cancelMatchmaking();
                }
            }, 500);
        }
    } catch {
        cancelMatchmaking();
        statusText.textContent = "Connection error - try again.";
    }
}

playBtn.addEventListener("click", () => {
    if (isMatchmaking) {
        cancelMatchmaking();
        return;
    }

    tcOverlay.classList.remove("hidden");
});

tcCancel.addEventListener("click", () => {
    tcOverlay.classList.add("hidden");
    cancelMatchmaking();
});

tcOverlay.addEventListener("mousedown", e => {
    if (e.target === tcOverlay) {
        tcOverlay.classList.add("hidden");
        cancelMatchmaking();
    }
});

document.querySelectorAll(".tc-btn").forEach(btn => {
    btn.addEventListener("click", () => {
        tcOverlay.classList.add("hidden");
        startMatchmaking(btn.dataset.tc);
    });
});

settingsBtn.addEventListener("click", () => {
    openSettings();
});

settingsClose.addEventListener("click", closeSettings);
settingsCancel.addEventListener("click", closeSettings);

settingsSave.addEventListener("click", () => {
    if (!usernameInput.value.trim()) usernameInput.value = generateUsername();

    startBgMorph(activeTheme, selectedTheme);
    activeTheme = selectedTheme;

    localStorage.setItem("settings", JSON.stringify({
        username: usernameInput.value.trim(),
        theme: activeTheme,
        pieceSet: selectedPieceSet,
    }));

    closeSettings();
    statusText.textContent = "";
});

settingsReset.addEventListener("click", () => {
    resetSettingsForm();
});

settingsOverlay.addEventListener("mousedown", e => {
    if (e.target === settingsOverlay) closeSettings();
});

usernameInput.addEventListener("input", () => { });

document.addEventListener("keydown", e => {
    if (e.key !== "Escape") return;

    if (!settingsOverlay.classList.contains("hidden")) {
        closeSettings();
        return;
    }

    if (!tcOverlay.classList.contains("hidden")) {
        tcOverlay.classList.add("hidden");
    }
});

themePicker.addEventListener("click", e => {
    const btn = e.target.closest(".option-tile");
    if (!btn) return;

    selectedTheme = btn.dataset.theme;
    renderSettingsUI();
    renderSettingsPreview();
});

piecePicker.addEventListener("click", e => {
    const btn = e.target.closest(".piece-option");
    if (!btn) return;

    selectedPieceSet = btn.dataset.pieceSet;
    renderSettingsUI();
    renderSettingsPreview();
});

if (sessionStorage.getItem("autoplay")) {
    sessionStorage.removeItem("autoplay");
    tcOverlay.classList.remove("hidden");
}

const bgCanvas = document.getElementById("bg-canvas");
const bgCtx = bgCanvas.getContext("2d");
const DPR = window.devicePixelRatio || 1;
const TILE = 64;

let bgW, bgH, bgOffsetX = 0, bgOffsetY = 0, bgPrevTs = null;
let bgAngle = Math.random() * Math.PI * 2;
let bgFromPalette = null;
let bgToPalette = null;
let bgMorphProgress = 1;
const BG_MORPH_DURATION = 600;
let bgMorphStart = null;

const saved = JSON.parse(localStorage.getItem("settings") ?? "{}");
const initTheme = previewThemes[saved.theme] ?? previewThemes.classic;
let bgCurrentLight = initTheme.light;
let bgCurrentDark = initTheme.dark;

function bgResize() {
    bgW = window.innerWidth;
    bgH = window.innerHeight;
    bgCanvas.width = Math.round(bgW * DPR);
    bgCanvas.height = Math.round(bgH * DPR);
    bgCtx.setTransform(DPR, 0, 0, DPR, 0, 0);
}

function lerpHex(a, b, t) {
    const ah = parseInt(a.slice(1), 16);
    const bh = parseInt(b.slice(1), 16);
    const ar = (ah >> 16) & 0xff, ag = (ah >> 8) & 0xff, ab = ah & 0xff;
    const br = (bh >> 16) & 0xff, bg = (bh >> 8) & 0xff, bb = bh & 0xff;
    const r = Math.round(ar + (br - ar) * t);
    const g = Math.round(ag + (bg - ag) * t);
    const b2 = Math.round(ab + (bb - ab) * t);
    return `rgb(${r},${g},${b2})`;
}

function startBgMorph(fromTheme, toTheme) {
    bgFromPalette = previewThemes[fromTheme] ?? previewThemes.classic;
    bgToPalette = previewThemes[toTheme] ?? previewThemes.classic;
    bgMorphProgress = 0;
    bgMorphStart = null;
}

function bgDraw(ts) {
    if (bgPrevTs === null) bgPrevTs = ts;
    const dt = Math.min((ts - bgPrevTs) / 1000, 0.1);
    bgPrevTs = ts;

    bgAngle += 0.0005 * 60 * dt;
    bgOffsetX += Math.cos(bgAngle) * 0.5 * 60 * dt;
    bgOffsetY += Math.sin(bgAngle) * 0.5 * 60 * dt;

    if (bgMorphProgress < 1) {
        if (!bgMorphStart) bgMorphStart = ts;
        bgMorphProgress = Math.min((ts - bgMorphStart) / BG_MORPH_DURATION, 1);
        const t = bgMorphProgress < 0.5 ? 2 * bgMorphProgress * bgMorphProgress : 1 - Math.pow(-2 * bgMorphProgress + 2, 2) / 2;
        bgCurrentLight = lerpHex(bgFromPalette.light, bgToPalette.light, t);
        bgCurrentDark = lerpHex(bgFromPalette.dark, bgToPalette.dark, t);
    }

    const startCol = Math.floor(-bgOffsetX / TILE) - 1;
    const startRow = Math.floor(-bgOffsetY / TILE) - 1;
    const endCol = startCol + Math.ceil(bgW / TILE) + 3;
    const endRow = startRow + Math.ceil(bgH / TILE) + 3;

    for (let col = startCol; col <= endCol; col++) {
        for (let row = startRow; row <= endRow; row++) {
            const x = col * TILE + bgOffsetX;
            const y = row * TILE + bgOffsetY;
            const isLight = (((col % 2) + (row % 2)) % 2 + 2) % 2 === 0;

            bgCtx.fillStyle = isLight ? bgCurrentLight : bgCurrentDark;
            bgCtx.fillRect(x, y, TILE + 1, TILE + 1);
        }
    }

    requestAnimationFrame(bgDraw);
}

loadSettings();
window.addEventListener("resize", bgResize);
bgResize();
requestAnimationFrame(bgDraw);