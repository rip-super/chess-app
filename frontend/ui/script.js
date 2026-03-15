const playBtn = document.getElementById("play-btn");
const settingsBtn = document.getElementById("settings-btn");
const statusText = document.getElementById("status");

playBtn.addEventListener("click", async () => {
    playBtn.disabled = true;
    statusText.textContent = "Finding a game...";

    try {
        const res = await fetch("/match");
        const data = await res.json();

        if (data.gameId) {
            statusText.textContent = "Game found!";
            window.location.href = `/play/${data.gameId}`;
        } else {
            statusText.textContent = "Timed out — try again.";
            playBtn.disabled = false;
        }
    } catch {
        statusText.textContent = "Connection error — try again.";
        playBtn.disabled = false;
    }
});

settingsBtn.addEventListener("click", () => {
    statusText.textContent = "Settings coming soon.";
});