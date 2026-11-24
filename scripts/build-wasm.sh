#!/usr/bin/env bash
set -euo pipefail

echo "Building KinVault Soroban contract WASM..."
cargo build --manifest-path contracts/kinvault/Cargo.toml --target wasm32v1-none --release
echo "WASM ready at target/wasm32v1-none/release/kinvault.wasm"
