import { serve } from "@hono/node-server";
import { createNodeWebSocket } from "@hono/node-ws";
import { serveStatic } from "@hono/node-server/serve-static";
import { Hono } from "hono";
import { readFile } from "fs/promises";
import init, { ChessEngine } from "./frontend/wasm/wasm.js";

const wasm = await readFile(new URL("./frontend/wasm/wasm_bg.wasm", import.meta.url));
await init({ module_or_path: wasm });

const ABANDON_TIMEOUT_MS = 60 * 1000;
const MOVE_ABANDON_MS = 30 * 1000;
const MOVE_ABANDON_WARNING_MS = 15 * 1000;
const MATCH_RESERVE_TIMEOUT_MS = 10 * 1000;

const TIME_CONTROLS = {
    "1+0": { initial: 60_000, increment: 0 },
    "1+1": { initial: 60_000, increment: 1_000 },
    "2+1": { initial: 2 * 60_000, increment: 1_000 },
    "3+0": { initial: 3 * 60_000, increment: 0 },
    "3+2": { initial: 3 * 60_000, increment: 2_000 },
    "5+0": { initial: 5 * 60_000, increment: 0 },
    "10+0": { initial: 10 * 60_000, increment: 0 },
    "15+10": { initial: 15 * 60_000, increment: 10_000 },
    "30+0": { initial: 30 * 60_000, increment: 0 },
};

const DEFAULT_TIME_CONTROL = "10+0";

const app = new Hono();
const { injectWebSocket, upgradeWebSocket } = createNodeWebSocket({ app });

const games = new Map();
const waitingPlayers = new Map();

function createNewGame(tcId = DEFAULT_TIME_CONTROL) {
    const tc = TIME_CONTROLS[tcId] ?? TIME_CONTROLS[DEFAULT_TIME_CONTROL];
    return {
        engine: new ChessEngine(),
        white: null,
        black: null,
        tokens: {},
        result: null,
        abandonTimer: null,
        timeControl: tc,
        tcId,
        clocks: { w: tc.initial, b: tc.initial },
        whiteSettings: null,
        blackSettings: null,
        clockActive: null,
        lastTickAt: null,
        flagTimer: null,
        moveAbortTimer: null,
        moveAbortWarnTimer: null,
        movesPlayed: 0,
        disconnectTimer: null,
    };
}

function clearFlagTimer(game) {
    if (game.flagTimer) {
        clearTimeout(game.flagTimer);
        game.flagTimer = null;
    }
}

function clearDisconnectTimer(game) {
    if (game.disconnectTimer) {
        clearTimeout(game.disconnectTimer);
        game.disconnectTimer = null;
    }
}

function cleanupWaitingEntry(tcId, gameId) {
    const entry = waitingPlayers.get(tcId);
    if (entry?.gameId === gameId) {
        waitingPlayers.delete(tcId);
    }
}

function maybeExpireReservation(entry) {
    if (!entry?.reserved || !entry.reservedAt) return false;
    if (Date.now() - entry.reservedAt <= MATCH_RESERVE_TIMEOUT_MS) return false;

    entry.reserved = false;
    entry.reservedAt = null;
    return true;
}

function scheduleFlagTimer(gameId, game) {
    clearFlagTimer(game);
    if (!game.clockActive) return;

    const color = game.clockActive;
    game.flagTimer = setTimeout(() => {
        const g = games.get(gameId);
        if (!g || g.clockActive !== color || g.result) return;

        const result = color === "w" ? "timeout_white" : "timeout_black";
        g.result = result;
        g.clocks[color] = 0;
        clearFlagTimer(g);
        games.delete(gameId);
        cleanupWaitingEntry(g.tcId, gameId);

        const msg = JSON.stringify({ type: "game_over", result, ...clockState(g) });
        g.white?.send(msg);
        g.black?.send(msg);
        console.log(`[${gameId}] flag - ${result}`);
    }, game.clocks[color]);
}

function startClocks(gameId, game) {
    game.clockActive = "w";
    game.lastTickAt = Date.now();
    scheduleFlagTimer(gameId, game);
}

function clockState(game) {
    return {
        clocks: { ...game.clocks },
        clockActive: game.clockActive,
        clockAt: game.lastTickAt,
    };
}

function resetMoveAbortTimer(gameId, game) {
    if (game.moveAbortTimer) {
        clearTimeout(game.moveAbortTimer);
        game.moveAbortTimer = null;
    }
    if (game.moveAbortWarnTimer) {
        clearTimeout(game.moveAbortWarnTimer);
        game.moveAbortWarnTimer = null;
    }
    if (game.result) return;

    const waitingOn = game.engine.side_to_move();

    game.moveAbortWarnTimer = setTimeout(() => {
        const g = games.get(gameId);
        if (!g || g.result || g.engine.side_to_move() !== waitingOn) return;

        const msg = JSON.stringify({
            type: "move_abort_warning",
            color: waitingOn,
            remainingMs: MOVE_ABANDON_WARNING_MS,
        });

        g.white?.send(msg);
        g.black?.send(msg);
    }, MOVE_ABANDON_MS - MOVE_ABANDON_WARNING_MS);

    game.moveAbortTimer = setTimeout(() => {
        const g = games.get(gameId);
        if (!g || g.result || g.engine.side_to_move() !== waitingOn) return;

        g.result = waitingOn === "w" ? "abandon_white" : "abandon_black";
        g.clockActive = null;

        clearFlagTimer(g);

        if (g.moveAbortWarnTimer) {
            clearTimeout(g.moveAbortWarnTimer);
            g.moveAbortWarnTimer = null;
        }
        g.moveAbortTimer = null;

        const msg = JSON.stringify({
            type: "game_over",
            result: g.result,
            ...clockState(g),
        });

        g.white?.send(msg);
        g.black?.send(msg);
    }, MOVE_ABANDON_MS);
}

function sanitizeSettings(settings) {
    if (!settings) return {};
    return {
        ...settings,
        username: String(settings.username ?? "Guest").trim().slice(0, 30) || "Guest",
    };
}

app.get("/match", (c) => {
    const tcId = c.req.query("tc") ?? DEFAULT_TIME_CONTROL;
    if (!TIME_CONTROLS[tcId]) return c.json({ error: "invalid time control" }, 400);

    const entry = waitingPlayers.get(tcId);

    if (entry) {
        const waitingGame = games.get(entry.gameId);

        if (!waitingGame) {
            waitingPlayers.delete(tcId);
        } else {
            maybeExpireReservation(entry);

            if (!entry.reserved) {
                entry.reserved = true;
                entry.reservedAt = Date.now();
                console.log(`[match] reserved game ${entry.gameId} (${tcId})`);
                return c.json({ gameId: entry.gameId });
            }

            return c.json({ waiting: true });
        }
    }

    const gameId = crypto.randomUUID();
    games.set(gameId, createNewGame(tcId));
    waitingPlayers.set(tcId, {
        gameId,
        reserved: false,
        reservedAt: null,
    });

    console.log(`[match] waiting for opponent, game ${gameId} (${tcId})`);
    return c.json({ waiting: true, gameId });
});

app.get("/match/:gameId", (c) => {
    const gameId = c.req.param("gameId");
    const tcId = c.req.query("tc") ?? DEFAULT_TIME_CONTROL;
    const game = games.get(gameId);

    if (!game) return c.json({ error: "expired" }, 404);

    const entry = waitingPlayers.get(tcId);
    if (entry?.gameId === gameId) {
        maybeExpireReservation(entry);

        if (entry.reserved) {
            return c.json({ gameId });
        }

        return c.json({ waiting: true });
    }

    return c.json({ gameId });
});

app.delete("/match/:gameId", (c) => {
    const gameId = c.req.param("gameId");
    const tcId = c.req.query("tc") ?? DEFAULT_TIME_CONTROL;
    const entry = waitingPlayers.get(tcId);

    if (entry?.gameId === gameId) {
        waitingPlayers.delete(tcId);
        games.delete(gameId);
        console.log(`[match] cancelled waiting game ${gameId} (${tcId})`);
    }

    return c.json({ ok: true });
});

app.get("/ws/:gameId", upgradeWebSocket((c) => {
    const gameId = c.req.param("gameId");

    return {
        onOpen() {
            const game = games.get(gameId);
            if (game?.abandonTimer) {
                clearTimeout(game.abandonTimer);
                game.abandonTimer = null;
            }

            console.log(`[${gameId}] websocket opened`);
        },

        onMessage(evt, ws) {
            const { type, uci, token, settings, message } = JSON.parse(evt.data);

            if (type === "auth" && !games.has(gameId)) {
                ws.send(JSON.stringify({ type: "not_found" }));
                return;
            }

            const game = games.get(gameId);
            if (!game) return;

            if (type === "auth") {
                if (game.tokens[token]) {
                    const restoredColor = game.tokens[token];
                    const existingWs = restoredColor === "w" ? game.white : game.black;

                    if (existingWs && existingWs !== ws) {
                        ws.send(JSON.stringify({ type: "error", msg: "already connected" }));
                        return;
                    }

                    if (restoredColor === "w") game.white = ws;
                    else game.black = ws;

                    if (game.disconnectTimer) {
                        clearTimeout(game.disconnectTimer);
                        game.disconnectTimer = null;
                        const opponent = restoredColor === "w" ? game.black : game.white;
                        opponent?.send(JSON.stringify({ type: "opponent_reconnected" }));
                    }

                    if (settings) {
                        if (restoredColor === "w") game.whiteSettings = sanitizeSettings(settings);
                        else game.blackSettings = sanitizeSettings(settings);
                    }

                    if (game.white && game.black) {
                        cleanupWaitingEntry(game.tcId, gameId);
                    }

                    ws.send(JSON.stringify({ type: "assign", color: restoredColor }));
                    ws.send(JSON.stringify({ type: "sync", fen: game.engine.get_fen(), ...clockState(game) }));

                    const oppSettings = restoredColor === "w" ? game.blackSettings : game.whiteSettings;
                    if (oppSettings) ws.send(JSON.stringify({ type: "opponent_info", ...oppSettings }));

                    if (game.result) ws.send(JSON.stringify({ type: "game_over", result: game.result }));
                    return;
                }

                if (!game.white) {
                    game.white = ws;
                    game.tokens[token] = "w";
                    game.whiteSettings = sanitizeSettings(settings);

                    console.log(`[${gameId}] player joined as white`);
                    ws.send(JSON.stringify({ type: "assign", color: "w" }));
                } else if (!game.black) {
                    if (Math.random() < 0.5) {
                        game.tokens[token] = "w";

                        const oldToken = Object.keys(game.tokens).find(
                            (t) => game.tokens[t] === "w" && t !== token
                        );

                        if (oldToken) game.tokens[oldToken] = "b";

                        game.black = game.white;
                        game.white = ws;
                        game.blackSettings = game.whiteSettings;
                        game.whiteSettings = sanitizeSettings(settings);

                        console.log(`[${gameId}] player joined as white (colors swapped)`);
                        game.black?.send(JSON.stringify({ type: "assign", color: "b" }));
                        ws.send(JSON.stringify({ type: "assign", color: "w" }));
                    } else {
                        game.black = ws;
                        game.tokens[token] = "b";
                        game.blackSettings = sanitizeSettings(settings);

                        console.log(`[${gameId}] player joined as black`);
                        ws.send(JSON.stringify({ type: "assign", color: "b" }));
                    }

                    cleanupWaitingEntry(game.tcId, gameId);

                    game.white?.send(JSON.stringify({ type: "opponent_info", ...(game.blackSettings ?? {}) }));
                    game.black?.send(JSON.stringify({ type: "opponent_info", ...(game.whiteSettings ?? {}) }));

                    startClocks(gameId, game);
                    resetMoveAbortTimer(gameId, game);
                } else {
                    console.warn(`[${gameId}] player tried to join full game`);
                    ws.send(JSON.stringify({ type: "error", msg: "game full" }));
                    return;
                }

                const syncMsg = JSON.stringify({
                    type: "sync",
                    fen: game.engine.get_fen(),
                    ...clockState(game),
                });

                game.white?.send(syncMsg);
                game.black?.send(syncMsg);
                return;
            }

            if (type === "new_game") {
                clearFlagTimer(game);
                game.engine = new ChessEngine();
                game.clocks = { w: game.timeControl.initial, b: game.timeControl.initial };
                startClocks(gameId, game);
                resetMoveAbortTimer(gameId, game);

                console.log(`[${gameId}] new game started`);

                const sync = JSON.stringify({
                    type: "sync",
                    fen: game.engine.get_fen(),
                    ...clockState(game),
                });

                game.white?.send(sync);
                game.black?.send(sync);
                return;
            }

            if (type === "resign") {
                clearFlagTimer(game);
                clearDisconnectTimer(game);

                const resignColor = game.white === ws ? "w" : "b";
                const result = resignColor === "w" ? "resign_white" : "resign_black";

                if (game.clockActive && game.lastTickAt) {
                    const elapsed = Date.now() - game.lastTickAt;
                    game.clocks[game.clockActive] = Math.max(0, game.clocks[game.clockActive] - elapsed);
                }

                game.clockActive = null;
                game.result = result;

                if (game.moveAbortTimer) {
                    clearTimeout(game.moveAbortTimer);
                    game.moveAbortTimer = null;
                }
                if (game.moveAbortWarnTimer) {
                    clearTimeout(game.moveAbortWarnTimer);
                    game.moveAbortWarnTimer = null;
                }

                const msg = JSON.stringify({ type: "game_over", result, ...clockState(game) });
                game.white?.send(msg);
                game.black?.send(msg);
                return;
            }

            if (type === "draw_offer") {
                const opponent = game.white === ws ? game.black : game.white;
                if (!opponent) return;

                opponent.send(JSON.stringify({ type: "draw_offer" }));
                return;
            }

            if (type === "draw_accepted") {
                clearFlagTimer(game);
                clearDisconnectTimer(game);

                if (game.clockActive && game.lastTickAt) {
                    const elapsed = Date.now() - game.lastTickAt;
                    game.clocks[game.clockActive] = Math.max(0, game.clocks[game.clockActive] - elapsed);
                }

                game.clockActive = null;
                game.result = "draw_agreed";

                if (game.moveAbortTimer) {
                    clearTimeout(game.moveAbortTimer);
                    game.moveAbortTimer = null;
                }
                if (game.moveAbortWarnTimer) {
                    clearTimeout(game.moveAbortWarnTimer);
                    game.moveAbortWarnTimer = null;
                }

                const msg = JSON.stringify({
                    type: "game_over",
                    result: "draw_agreed",
                    ...clockState(game),
                });

                game.white?.send(msg);
                game.black?.send(msg);
                return;
            }

            if (type === "draw_declined") {
                const opponent = game.white === ws ? game.black : game.white;
                opponent?.send(JSON.stringify({ type: "draw_declined" }));
                return;
            }

            if (type === "claim_victory") {
                if (game.result) return;

                const claimColor = game.white === ws ? "w" : "b";
                const opponent = claimColor === "w" ? game.black : game.white;
                if (opponent) return;

                const result = claimColor === "w" ? "abandon_black" : "abandon_white";

                if (game.clockActive && game.lastTickAt) {
                    const elapsed = Date.now() - game.lastTickAt;
                    game.clocks[game.clockActive] = Math.max(0, game.clocks[game.clockActive] - elapsed);
                }

                if (game.moveAbortTimer) {
                    clearTimeout(game.moveAbortTimer);
                    game.moveAbortTimer = null;
                }
                if (game.moveAbortWarnTimer) {
                    clearTimeout(game.moveAbortWarnTimer);
                    game.moveAbortWarnTimer = null;
                }

                game.clockActive = null;
                game.result = result;
                clearFlagTimer(game);
                games.delete(gameId);
                cleanupWaitingEntry(game.tcId, gameId);

                ws.send(JSON.stringify({ type: "game_over", result, ...clockState(game) }));
                return;
            }

            if (type === "chat") {
                if (game.result) return;

                const senderColor = game.white === ws ? "w" : "b";
                const senderSettings = senderColor === "w" ? game.whiteSettings : game.blackSettings;
                const senderUsername = senderSettings?.username ?? "Guest";
                const text = String(message ?? "").trim().slice(0, 200);
                if (!text) return;

                const opponent = senderColor === "w" ? game.black : game.white;
                opponent?.send(JSON.stringify({ type: "chat", username: senderUsername, message: text }));
                return;
            }

            if (type !== "move") return;

            const sideToMove = game.engine.side_to_move();
            const isWhite = game.white === ws;
            const isBlack = game.black === ws;

            if ((sideToMove === "w" && !isWhite) || (sideToMove === "b" && !isBlack)) {
                ws.send(JSON.stringify({ type: "error", msg: "not your turn" }));
                return;
            }

            const mv = game.engine.parse_uci(uci);
            if (!mv) {
                ws.send(JSON.stringify({ type: "error", msg: "invalid move" }));
                return;
            }

            try {
                if (game.clockActive === sideToMove && game.lastTickAt) {
                    const elapsed = Date.now() - game.lastTickAt;
                    game.clocks[sideToMove] = Math.max(0, game.clocks[sideToMove] - elapsed);

                    if (game.clocks[sideToMove] <= 0) {
                        clearFlagTimer(game);
                        game.clocks[sideToMove] = 0;
                        game.clockActive = null;
                        game.result = sideToMove === "w" ? "timeout_white" : "timeout_black";

                        const msg = JSON.stringify({
                            type: "game_over",
                            result: game.result,
                            ...clockState(game),
                        });

                        game.white?.send(msg);
                        game.black?.send(msg);
                        return;
                    }

                    game.clocks[sideToMove] += game.timeControl.increment;
                }

                game.engine.make_move(mv);
                game.movesPlayed++;
                const result = game.engine.game_result();

                console.log(`[${gameId}] move ${uci} - result: ${result}`);

                if (result !== "ongoing") {
                    clearFlagTimer(game);
                    game.result = result;
                    game.clockActive = null;
                } else {
                    game.clockActive = sideToMove === "w" ? "b" : "w";
                    game.lastTickAt = Date.now();
                    scheduleFlagTimer(gameId, game);

                    if (game.movesPlayed < 2) {
                        resetMoveAbortTimer(gameId, game);
                    } else {
                        if (game.moveAbortTimer) { clearTimeout(game.moveAbortTimer); game.moveAbortTimer = null; }
                        if (game.moveAbortWarnTimer) { clearTimeout(game.moveAbortWarnTimer); game.moveAbortWarnTimer = null; }
                    }
                }

                const msg = JSON.stringify({
                    type: "move",
                    fen: game.engine.get_fen(),
                    uci,
                    isCapture: mv.is_capture(),
                    isCastle: mv.is_castle(),
                    isPromotion: mv.is_promotion(),
                    isCheck: game.engine.is_in_check(),
                    result,
                    ...clockState(game),
                });

                game.white?.send(msg);
                game.black?.send(msg);
            } catch (e) {
                console.warn(`[${gameId}] illegal move attempt: ${uci}`);
                ws.send(JSON.stringify({ type: "error", msg: "illegal move" }));
            }
        },

        onClose(_, ws) {
            const game = games.get(gameId);
            if (!game) return;

            const disconnectedColor =
                game.white === ws ? "w" :
                    game.black === ws ? "b" :
                        null;

            if (!disconnectedColor) return;

            if (disconnectedColor === "w") {
                game.white = null;
                console.log(`[${gameId}] white disconnected`);
            } else {
                game.black = null;
                console.log(`[${gameId}] black disconnected`);
            }

            const remaining = game.white ?? game.black;
            const isFinished = !!(game.result || game.engine.game_result() !== "ongoing");

            if (!game.white && !game.black) {
                clearDisconnectTimer(game);
                clearFlagTimer(game);

                game.abandonTimer = setTimeout(() => {
                    games.delete(gameId);
                    cleanupWaitingEntry(game.tcId, gameId);
                    console.log(`[${gameId}] game cleaned up (both disconnected)`);
                }, isFinished ? 30_000 : ABANDON_TIMEOUT_MS);
            } else if (remaining && !isFinished) {
                const CLAIM_TIMEOUT_MS = 60 * 1000;

                remaining.send(JSON.stringify({
                    type: "opponent_disconnected",
                    claimInMs: CLAIM_TIMEOUT_MS,
                }));

                game.disconnectTimer = setTimeout(() => {
                    game.disconnectTimer = null;
                    const still = game.white ?? game.black;
                    still?.send(JSON.stringify({ type: "can_claim_victory" }));
                }, CLAIM_TIMEOUT_MS);
            }
        },
    };
}));

app.get("/", async (c) => c.html(await readFile("./frontend/index.html", "utf8")));
app.use("/*", serveStatic({ root: "./frontend" }));
app.get("/play/:gameId", async (c) => c.html(await readFile("./frontend/play/index.html", "utf8")));

const server = serve({ fetch: app.fetch, port: 3000 }, info => {
    console.log(`Server running on http://localhost:${info.port}`);
});

injectWebSocket(server);