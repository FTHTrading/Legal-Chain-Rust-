#!/usr/bin/env bash
# bootstrap-local.sh — Start a local LEGAL-CHAIN devnet with a single validator (Alice).
#
# Usage:
#   ./scripts/bootstrap-local.sh          # Dev mode (single node)
#   ./scripts/bootstrap-local.sh local    # Local testnet spec
#   PURGE=1 ./scripts/bootstrap-local.sh  # Purge chain data first
#
# Prerequisites:
#   - Rust toolchain 1.81+ with wasm32-unknown-unknown target
#   - cargo build --release completed

set -euo pipefail

CHAIN="${1:-dev}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
NODE_BIN="$ROOT_DIR/target/release/legal-chain-node"

if [ ! -f "$NODE_BIN" ]; then
    echo "[*] Node binary not found. Building in release mode..."
    cd "$ROOT_DIR"
    cargo build --release
fi

if [ "${PURGE:-0}" = "1" ]; then
    echo "[*] Purging chain data for '$CHAIN'..."
    "$NODE_BIN" purge-chain --chain "$CHAIN" -y
fi

echo "[*] Starting LEGAL-CHAIN node (chain=$CHAIN)..."
EXTRA_ARGS=""
if [ "$CHAIN" = "dev" ]; then
    EXTRA_ARGS="--force-authoring"
fi

exec "$NODE_BIN" \
    --chain "$CHAIN" \
    --alice \
    --tmp \
    --rpc-cors all \
    --rpc-methods unsafe \
    --rpc-port 9944 \
    --port 30333 \
    --validator \
    $EXTRA_ARGS
