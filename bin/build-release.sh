#!/bin/bash

selfdir="$(dirname "$(readlink -f "$0")")"

set -o errexit
set -o pipefail

podman run --rm -it \
  --mount "type=bind,src=$(pwd),dst=/pwd,ro" \
  --mount "type=bind,src=$(pwd)/dist,dst=/dist" \
  -w /build \
  rust-builder:bookworm-1.82 \
  -l -x -c "rsync --verbose --archive --exclude='.*' /pwd/{src,Cargo.*,rust-toolchain.toml} /build/ && cargo build --release && cp target/release/caplike /dist/"
tree dist
