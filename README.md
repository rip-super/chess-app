this is a wip repo for my online chess app.

# Completed Features
- Bitboard Engine in Rust
- Compiled to WASM
- GUI to play
- Matchmaking via WebSockets
- Real Time Chess Playing

# TODO
- Live app at [https://chess.sahildash.dev](https://chess.sahildash.dev)
- Settings page (chnage sounds, board, pieces
- Username system
- chat(?)
- Time keeping
- not slopped main page (mb for that guys)

# Usage
run the following commands to install and run the app so far.

```
git clone https://github.com/rip-super/chess-app.git
cd chess-app
cargo build -p wasm --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir frontend/ui/wasm ./target/wasm32-unknown-unknown/release/wasm.wasm
```

Note: you may need to add the wasm32-unknown-unknown target using rustup:
```
rustup target add wasm32-unknown-unknown
```
and you'll need to install the wasm-bindgen-cli:
```
cargo install wasm-bindgen-cli
```

Now run the server:
```
cd frontend
npm i
node server.js
```

Go to `http://localhost:3000` and start playing!
