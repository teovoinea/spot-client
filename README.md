# spot-client

The front end for the Spot game.

## Running

```bash
git clone https://github.com/teovoinea/spot-client
cargo install -f cargo-web
cargo web start --target=wasm32-unknown-unknown
```

## Architecture

### stdweb

Compiles the Rust code into blazing fast wasm

### bincode

Encodes the pixel messages into compact binary representation