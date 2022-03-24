# https://github.com/casey/just

build-web-release:
  #!/usr/bin/env bash
  set -euo pipefail
  if ! rustup target list --installed | grep '^wasm32-unknown-unknown$'; then
    echo You are going to need to install the wasm32-unknown-unknown target.
    echo "this command should do the trick: 'rustup target add wasm32-unknown-unknown'"
    exit 1
  fi
  mkdir -p dist
  cargo build --target wasm32-unknown-unknown --release
  cp target/wasm32-unknown-unknown/release/mandelbddap.wasm dist/game.wasm
  cp index.html dist/index.html

serve-web:
  #!/usr/bin/env bash
  set -euo pipefail
  # cargo install basic-http-server
  if ! which basic-http-server; then
    echo basic-http-server is required.
    echo "you can get it with: 'cargo install basic-http-server'"
    exit 1
  fi
  basic-http-server dist
