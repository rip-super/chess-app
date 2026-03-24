# chess

A fully featured online chess app with a custom Rust engine compiled to WebAssembly.

Currently live at [chess.sahildash.dev](https://chess.sahildash.dev)

---

## The Engine

The chess engine is written from scratch in Rust and compiled to WASM via wasm-bindgen, so it runs directly in the browser with no server round-trips for move validation.

Under the hood:
- **Bitboard representation** - each piece type and color gets its own 64-bit integer, with a mailbox for fast square lookups
- **Magic bitboards** for sliding pieces (bishops, rooks, queens) - precomputed attack tables indexed by a magic number hash of the occupancy mask, giving O(1) attack generation
- **Full legal move generation** - pseudo-legal moves are generated then filtered by make/undo, keeping the king out of check
- **Draw detection** - threefold repetition, fifty-move rule, and insufficient material (including same-color bishop endings)
- **Verified with perft** - node counts match known results up to depth 6 across all standard test positions

---

## Features

**Gameplay**
- Real-time multiplayer over WebSockets
- 9 time controls (1+0 up to 30+0)
- Premoves
- Draw offers and resignation
- Abandon detection - if neither player moves in the first 30 seconds, the game is abandoned
- In-game chat

**Presentation**
- Match start animation (lowk better than chess.com, idk what to say)
- Move history (with arrow keys)
- Arrows and square highlights (right-click drag)
- Piece animation

**Customization**
- 35 piece sets
- 15 board themes
- 500+ unique combinations!
- Custom profile pictures
- Username system with player banners

---

## Setup

You'll need Rust, Node.js, and the following tools:
```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

Clone and build:
```
git clone https://github.com/rip-super/chess-app.git
cd chess-app
cargo build -p wasm --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir web/frontend/wasm ./target/wasm32-unknown-unknown/release/wasm.wasm
```

Run the server:
```
cd web
npm i
node server.js
```

Then open `http://localhost:3000`.
