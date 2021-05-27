#!/usr/bin/bash

# fail fast
#
set -e

# print each command before it's executed
#
set -x

wasm-pack test --headless --firefox --release -- --no-default-features
wasm-pack test --headless --firefox           -- --no-default-features
