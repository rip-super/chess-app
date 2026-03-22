this is a wip repo for my online chess app.

Currently live [here!](https://chess.sahildash.dev) (if the dev server is running lol)

# Completed Features
- Bitboard Engine in Rust
- Compiled to WASM
- GUI to play
- Matchmaking via WebSockets
- Real Time Chess Playing
- Live app at [https://chess.sahildash.dev](https://chess.sahildash.dev)
- draw/resign buttons
- cooking home page (if i do say so myself)
- Time keeping (9 different time controls!!!)
- Settings page (change sounds, board, pieces, username, profile pic)
- Username system (with username banners while playing!!)
- Profile pictures!!!!!
- absolutly fire match start animation (lowk better than chess.com idk what to say)

# TODO
- chat(?)

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
