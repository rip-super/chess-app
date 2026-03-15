window.addEventListener("pageshow", () => {
    const activeGame = localStorage.getItem("gameId");
    if (activeGame) window.location.href = `/play/${activeGame}`;
});

const playBtn = document.getElementById("play-btn");
const settingsBtn = document.getElementById("settings-btn");
const statusText = document.getElementById("status");

if (sessionStorage.getItem("autoplay")) {
    sessionStorage.removeItem("autoplay");
    playBtn.click();
}

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