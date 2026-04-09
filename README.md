# LEGAL-CHAIN

**Sovereign Rust blockchain for legal evidence, document proof, approvals, chain-of-custody, audit events, identity-aware workflow enforcement, and matter-linked settlement.**

Built on Polkadot SDK / Substrate with custom FRAME pallets purpose-built for legal operations.

## Architecture

```
┌──────────────────────────────────────────────────┐
│              Legal-Chain Web App                  │
│          (Next.js — existing UX layer)            │
└────────────┬──────────────────┬───────────────────┘
             │ TypeScript SDK   │ Proof Bundles
┌────────────▼────────┐ ┌──────▼───────────────────┐
│   Explorer API      │ │   Proof Service          │
│   (Axum, SQLx)      │ │   (Axum, chain RPC)      │
└────────────┬────────┘ └──────┬───────────────────┘
             │                 │
┌────────────▼─────────────────▼───────────────────┐
│              Indexer (Rust, Postgres)             │
│         subscribes to chain blocks/events         │
└────────────┬─────────────────────────────────────┘
             │ WebSocket RPC
┌────────────▼─────────────────────────────────────┐
│            LEGAL-CHAIN NODE (Substrate)           │
│                                                   │
│  Runtime Pallets:                                 │
│   • matters    • evidence   • documents           │
│   • approvals  • audit      • attestations        │
│   • settlement • identities • access-control      │
│   • agent-policy • jurisdiction-rules             │
│                                                   │
│  Consensus: Aura (block) + GRANDPA (finality)     │
│  Validators: permissioned set                     │
└───────────────────────────────────────────────────┘
```

## Quick Start (Local Dev)

```powershell
# 1. Build
cargo build --release

# 2. Run dev node (single validator, instant seal)
./target/release/legal-chain-node --dev --tmp

# 3. Node is available at ws://127.0.0.1:9944
```

See [docs/RUNBOOK_LOCAL.md](docs/RUNBOOK_LOCAL.md) for full setup including indexer and services.

## Repository Layout

| Path | Description |
|------|-------------|
| `node/` | Substrate node binary |
| `runtime/` | WASM runtime with all pallets composed |
| `pallets/` | Custom FRAME pallets for legal domain |
| `crates/` | Shared libraries (types, crypto, codecs) |
| `indexer/` | Event indexer → Postgres |
| `explorer-api/` | Query API for legal objects |
| `proof-service/` | Proof verification + bundle generation |
| `integrations/` | TypeScript client + web SDK |
| `docs/` | Architecture, runbooks, threat model |
| `scripts/` | Bootstrap, seed, e2e test scripts |

## Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) — system design
- [DECISIONS.md](docs/DECISIONS.md) — architecture decision records
- [VERSION_MATRIX.md](docs/VERSION_MATRIX.md) — pinned dependency versions
- [EVENT_MODEL.md](docs/EVENT_MODEL.md) — runtime event catalog
- [THREAT_MODEL.md](docs/THREAT_MODEL.md) — security analysis
- [RUNBOOK_LOCAL.md](docs/RUNBOOK_LOCAL.md) — local development guide

## Build Phases

| Phase | Scope | Status |
|-------|-------|--------|
| 0 | Foundation — scaffold, docs, shared types | ✅ |
| 1 | Chain Core — node, runtime, matters/evidence/documents/audit | ✅ |
| 2 | Workflow — approvals, identities, access-control, agent-policy | ✅ |
| 3 | Data Services — indexer, explorer API, proof service | ⬜ |
| 4 | Integration — TypeScript client, web SDK, proof bundles | ⬜ |
| 5 | Hardening — threat model, negative tests, runbooks, observability | ⬜ |

## License

BUSL-1.1 — Business Source License
