window.addEventListener("pageshow", () => {
    const activeGame = localStorage.getItem("gameId");
    if (activeGame) window.location.href = `/play/${activeGame}`;
});

const playBtn = document.getElementById("play-btn");
const settingsBtn = document.getElementById("settings-btn");
const statusText = document.getElementById("status");

playBtn.addEventListener("click", async () => {
    playBtn.disabled = true;
    statusText.innerHTML = "Finding a game<span class='dots'></span>";

    try {
        const res = await fetch("/match");
        const data = await res.json();

        if (data.gameId && !data.waiting) {
            localStorage.setItem("gameId", data.gameId);
            window.location.href = `/play/${data.gameId}`;
            return;
        }

        if (data.waiting) {
            const gameId = data.gameId;
            const interval = setInterval(async () => {
                try {
                    const r = await fetch(`/match/${gameId}`);
                    const d = await r.json();
                    if (d.gameId) {
                        clearInterval(interval);
                        localStorage.setItem("gameId", d.gameId);
                        window.location.href = `/play/${d.gameId}`;
                    } else if (d.error) {
                        clearInterval(interval);
                        statusText.textContent = "Timed out - try again.";
                        playBtn.disabled = false;
                    }
                } catch { clearInterval(interval); playBtn.disabled = false; }
            }, 500);

            setTimeout(() => {
                clearInterval(interval);
                if (playBtn.disabled) {
                    statusText.textContent = "Timed out - try again.";
                    playBtn.disabled = false;
                }
            }, 30000);
        }
    } catch {
        statusText.textContent = "Connection error - try again.";
        playBtn.disabled = false;
    }
});

settingsBtn.addEventListener("click", () => {
    statusText.textContent = "TODO: Settings";
});

if (sessionStorage.getItem("autoplay")) {
    sessionStorage.removeItem("autoplay");
    playBtn.click();
}

const bgCanvas = document.getElementById("bg-canvas");
const bgCtx = bgCanvas.getContext("2d");
const DPR = window.devicePixelRatio || 1;
const TILE = 64;

let bgW, bgH, bgOffsetX = 0, bgOffsetY = 0, bgPrevTs = null;
let bgAngle = Math.random() * Math.PI * 2;

function bgResize() {
    bgW = window.innerWidth;
    bgH = window.innerHeight;
    bgCanvas.width = Math.round(bgW * DPR);
    bgCanvas.height = Math.round(bgH * DPR);
    bgCtx.setTransform(DPR, 0, 0, DPR, 0, 0);
}

function bgDraw(ts) {
    if (bgPrevTs === null) bgPrevTs = ts;
    const dt = Math.min((ts - bgPrevTs) / 1000, 0.1);
    bgPrevTs = ts;

    bgAngle += 0.0005 * 60 * dt;

    bgOffsetX += Math.cos(bgAngle) * 0.5 * 60 * dt;
    bgOffsetY += Math.sin(bgAngle) * 0.5 * 60 * dt;

    const startCol = Math.floor(-bgOffsetX / TILE) - 1;
    const startRow = Math.floor(-bgOffsetY / TILE) - 1;
    const endCol = startCol + Math.ceil(bgW / TILE) + 3;
    const endRow = startRow + Math.ceil(bgH / TILE) + 3;

    for (let col = startCol; col <= endCol; col++) {
        for (let row = startRow; row <= endRow; row++) {
            const x = col * TILE + bgOffsetX;
            const y = row * TILE + bgOffsetY;
            const isLight = (((col % 2) + (row % 2)) % 2 + 2) % 2 === 0;
            bgCtx.fillStyle = isLight ? "#d9e4e8" : "#7b9eb2";
            bgCtx.fillRect(x, y, TILE + 1, TILE + 1);
        }
    }

    requestAnimationFrame(bgDraw);
}

window.addEventListener("resize", bgResize);
bgResize();
requestAnimationFrame(bgDraw);