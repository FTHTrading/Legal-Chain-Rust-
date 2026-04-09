# LEGAL-CHAIN — Version Matrix

All dependency versions are pinned and must be updated together.

## Core Toolchain

| Component | Version | Notes |
|-----------|---------|-------|
| Rust | 1.81.0 stable | Pinned in `rust-toolchain.toml` |
| WASM target | wasm32-unknown-unknown | Required for runtime compilation |
| Polkadot SDK | `polkadot-stable2409` (git tag) | Substrate + FRAME + SP + SC crates |

## Polkadot SDK Crates (all from same git tag)

| Crate | Usage |
|-------|-------|
| `frame-support` | Pallet framework, storage, events, errors |
| `frame-system` | System pallet, account management |
| `frame-executive` | Runtime execution orchestration |
| `sp-core` | Crypto primitives, H256, AccountId |
| `sp-runtime` | Runtime types, DispatchResult, MultiSignature |
| `sp-io` | Host functions, storage I/O |
| `sp-api` | Runtime API trait definitions |
| `sp-version` | Runtime version metadata |
| `sp-consensus-aura` | Aura consensus primitives |
| `sp-consensus-grandpa` | GRANDPA finality primitives |
| `sp-genesis-builder` | Genesis config building |
| `pallet-aura` | Authority round block authoring |
| `pallet-grandpa` | GRANDPA finality gadget |
| `pallet-balances` | Account balances and transfers |
| `pallet-timestamp` | Block timestamp oracle |
| `pallet-transaction-payment` | Fee calculation |
| `pallet-sudo` | Superuser operations (dev/early stage) |
| `sc-cli` | CLI framework for node binary |
| `sc-service` | Node service orchestration |
| `sc-consensus-aura` | Aura client-side consensus |
| `sc-consensus-grandpa` | GRANDPA client-side finality |

## Encoding

| Crate | Version |
|-------|---------|
| `parity-scale-codec` | 3.6.x |
| `scale-info` | 2.11.x |

## Data Services (Phase 3+)

| Component | Planned Version |
|-----------|----------------|
| PostgreSQL | 16.x |
| SQLx | 0.8.x |
| Axum | 0.7.x |
| tokio | 1.x |

## TypeScript Integration (Phase 4+)

| Component | Planned Version |
|-----------|----------------|
| `@polkadot/api` | latest stable |
| TypeScript | 5.x |

## Update Procedure

1. Choose new Polkadot SDK stable tag
2. Update all git tag references in workspace `Cargo.toml`
3. Update `rust-toolchain.toml` if the new SDK requires a different Rust version
4. Run `cargo check` across all workspace members
5. Run full test suite
6. Update this file
7. Commit and tag the update
