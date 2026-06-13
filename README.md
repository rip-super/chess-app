# chess

A fully featured online chess app with a custom Rust engine compiled to WebAssembly.

Currently live at https://chess.sahildash.dev

---

## The Engine

The chess engine is written from scratch in Rust and compiled to WASM via wasm-bindgen, so it runs directly in the browser with no server round-trips for move validation.

Under the hood:

* **Bitboard representation** - each piece type and color gets its own 64-bit integer, with a mailbox for fast square lookups
* **Magic bitboards** for sliding pieces (bishops, rooks, queens) - precomputed attack tables indexed by a magic number hash of the occupancy mask, giving O(1) attack generation
* **Full legal move generation** - pseudo-legal moves are generated then filtered by make/undo, keeping the king out of check
* **Draw detection** - threefold repetition, fifty-move rule, and insufficient material
* **Verified with perft** - node counts match known results up to depth 6 across standard test positions

### Search

The engine now includes a full competitive search stack:

* Negamax
* Alpha-beta pruning
* Iterative deepening
* Time management
* Principal Variation Search (PVS)
* Aspiration windows

### Move Ordering / Heuristics

* MVV-LVA capture ordering
* Killer moves
* History heuristic

### Pruning / Reductions

* Quiescence search
* Static Exchange Evaluation (SEE)
* Null Move Pruning (NMP)
* Late Move Reductions (LMR)

### Other

* Transposition table
* Piece-square tables
* Opening book

---

## Features

**Gameplay**

* Real-time multiplayer
* Play against other users in the browser
* Play against the built-in engine

If no opponent is found within ~15 seconds, you are automatically matched with the bot.

**Engine Integration**

* The same engine powers both validation (WASM) and the bot
* Runs entirely client-side

**Tournament / Testing**

There is a built-in gauntlet system to test different versions of the engine against each other.

* Compare strength across versions (v1 → v10)
* Run automated tournaments
* Output PGNs for analysis

---

## Project Structure

```
.
├── bot/        # bot wrapper + opening book
├── engine/     # core engine (move gen, search, eval)
├── wasm/       # WASM bindings
├── web/        # frontend + server
├── cli/        # command-line interface
├── gauntlet/   # engine vs engine testing
├── scripts/    # tournament + tooling
```

---

## Running Locally

make sure you have rust + cargo installed.

add the wasm target and wasm tooling:

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

build the wasm module:

```
cargo build -p wasm --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir web/frontend/wasm ./target/wasm32-unknown-unknown/release/wasm.wasm
```

then run the web app:

```
cd web
npm install
node server.js
```

open the localhost:3000 in your browser.

---

## Notes

* The engine was built incrementally (see `gauntlet/src/bin/` for progression from random -> full search engine)
* Designed to be fast enough for real-time play in the browser
* No external chess libraries are used for the engine itself (everything is built from scratch!)

---
