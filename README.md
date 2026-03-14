this is a wip repo for my online chess app.

as of right now, i have finished writing the chess engine itself in rust, and i have successfully converted it to wasm and you can use it in the broswer

# Usage
run the following commands to install and run the app so far.

```
git clone https://github.com/rip-super/chess-app.git
cd chess-app
cargo build -p wasm --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir frontend/pkg ./target/wasm32-unknown-unknown/release/wasm.wasm
```

Note: you may need to add the wasm32-unknown-unknown target using rustup:
```
rustup target add wasm32-unknown-unknown
```
and you'll need to install the wasm-bindgen-cli:
```
cargo install wasm-bindgen-cli
```

Now just serve the frontend folder locally, you need to serve it, since just opening the file in the broswer wont allow for the wasm to load.

I personally use `npx serve frontend` but you can use any other static server like http.server in python or whatever.
