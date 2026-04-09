# LEGAL-CHAIN — Local Development Runbook

## Prerequisites

- Rust 1.81.0 (installed via `rustup`, pinned by `rust-toolchain.toml`)
- WASM target: `rustup target add wasm32-unknown-unknown`
- (Optional) Docker for containerized node + Postgres
- (Optional) protobuf compiler for networking: `choco install protoc` (Windows) / `apt install protobuf-compiler` (Linux)

## Quick Start

### 1. Clone and Build

```bash
git clone https://github.com/FTHTrading/legal-chain-core.git
cd legal-chain-core
cargo build --release
```

First build will take 15-30 minutes (downloads and compiles ~400 Polkadot SDK crates). Subsequent builds are incremental and much faster.

### 2. Run Development Node

```bash
# Single-node dev chain with temporary storage (purged on restart)
./target/release/legal-chain-node --dev --tmp

# Dev chain with persistent storage
./target/release/legal-chain-node --dev --base-path ./data/dev

# Purge existing chain data and restart fresh
./target/release/legal-chain-node purge-chain --dev --base-path ./data/dev -y
./target/release/legal-chain-node --dev --base-path ./data/dev
```

### 3. Verify Node is Running

```bash
# Check RPC endpoint
curl -s http://127.0.0.1:9944 -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' | jq .

# Expected: {"result":{"peers":0,"isSyncing":false,"shouldHavePeers":false}}
```

### 4. Connect Polkadot.js Apps

Open https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944 in a browser. This provides a GUI for:
- Submitting extrinsics to legal pallets
- Browsing chain state and storage
- Viewing events and blocks

## Windows-Specific Notes

### PowerShell

```powershell
# Build
cargo build --release 2>&1 | Tee-Object -FilePath build.log

# Run dev node
.\target\release\legal-chain-node.exe --dev --tmp

# Run with WSL2 (if native build has issues)
wsl -- bash -c "cd /mnt/c/Users/Kevan/legal-chain-core && cargo build --release"
```

### Environment Variables

Copy `.env.example` to `.env` and configure:
```powershell
Copy-Item .env.example .env
# Edit .env with your local settings
```

## Common Operations

### Submit a Test Matter

Using curl against the RPC endpoint:
```bash
# Via Polkadot.js Apps: Developer → Extrinsics → matters → createMatter
# Or via TypeScript SDK (Phase 4)
```

### Check Pallet Storage

```bash
# Via RPC: state_getStorage with pallet prefix
# Via Polkadot.js Apps: Developer → Chain State → matters → matters(u64)
```

### View Events

```bash
# Via Polkadot.js Apps: Network → Explorer → click a block → Events tab
```

## Troubleshooting

| Problem | Cause | Fix |
|---------|-------|-----|
| `wasm32-unknown-unknown` errors | Missing WASM target | `rustup target add wasm32-unknown-unknown` |
| Slow first build | Normal for Substrate | Wait, or use `sccache` for caching |
| `protoc` not found | protobuf compiler missing | Install via package manager |
| Port 9944 in use | Another node running | Kill existing process or use `--rpc-port` |
| Storage corruption | Unclean shutdown | Purge chain data and restart |
