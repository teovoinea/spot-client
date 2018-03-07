[![Build Status](https://travis-ci.org/teovoinea/spot-client.svg?branch=master)](https://travis-ci.org/teovoinea/spot-client)
[![Build status](https://ci.appveyor.com/api/projects/status/ld3b3u2wa9rw6m7j?svg=true)](https://ci.appveyor.com/project/teovoinea/spot-client)
[![dependency status](https://deps.rs/repo/github/teovoinea/spot-client/status.svg)](https://deps.rs/repo/github/teovoinea/spot-client)

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
