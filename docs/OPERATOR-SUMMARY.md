# LEGAL-CHAIN Phase 0 + Phase 1 — Operator Summary

**Date**: 2025-07-07
**Status**: Phase 1 Complete — Ready for `cargo check` verification

---

## What's Real (Fully Implemented)

### Runtime (WASM + Native)
- **Framework**: Polkadot SDK / Substrate, pinned to `polkadot-stable2409`
- **Consensus**: Aura (sr25519 block authoring) + GRANDPA (ed25519 finality)
- **Block Time**: 6 seconds (MinimumPeriod = 3000ms)
- **Max Authorities**: 32 permissioned validators
- **Account Model**: MultiSignature → sr25519 AccountId, Balance = u128
- **Existential Deposit**: 500

### Legal Pallets (4 of 11 planned)
| Pallet | Index | Description | Status |
|--------|-------|-------------|--------|
| `pallet-matters` | 10 | Legal matter lifecycle (CRUD, status transitions, jurisdiction, sensitivity) | ✅ Complete |
| `pallet-evidence` | 11 | Evidence hash registration, verification, chain-of-custody tracking | ✅ Complete |
| `pallet-documents` | 12 | Document versioning, supersession chains, filing readiness workflow | ✅ Complete |
| `pallet-audit` | 13 | Durable audit event anchoring, cross-pallet `AuditHook` trait | ✅ Complete |

### Infrastructure Pallets
| Pallet | Index | Purpose |
|--------|-------|---------|
| `frame_system` | 0 | Core runtime types |
| `pallet_timestamp` | 1 | Block time |
| `pallet_aura` | 2 | Block authoring |
| `pallet_grandpa` | 3 | Finality |
| `pallet_balances` | 4 | Native token |
| `pallet_transaction_payment` | 5 | Fee mechanism |
| `pallet_sudo` | 6 | Superuser (dev/testnet only) |

### Common Types Crate
- Domain identifiers: MatterId, EvidenceId, DocumentId, ApprovalId, AttestationId, AuditId, SettlementId, CredentialId
- Status enums: MatterStatus (8 states, validated transitions), EvidenceStatus, CustodyState, DocumentStatus, FilingReadiness
- Classification: Sensitivity (Public → Restricted), MatterType (7 categories)
- Audit types: ActionType (12 actions), SubjectType (8 subjects), UpdatedField
- Cross-pallet trait: `AuditHook<AccountId>`

### Node Binary
- Full Substrate node: `legal-chain-node`
- CLI: key management, build-spec, check-block, export/import, purge-chain, revert
- Chain specs: `dev` (single Alice validator), `local` (3 validators: Alice/Bob/Charlie)
- RPC: system + transaction-payment JSON-RPC extensions
- Service: Aura block authoring + GRANDPA voter + offchain workers

### DevOps
- Bootstrap scripts: `scripts/bootstrap-local.ps1` (Windows), `scripts/bootstrap-local.sh` (Linux)
- Docker: multi-stage Dockerfile, docker-compose.yml (3-node local testnet)
- Makefile: build, check, test, fmt, clippy, dev, local, docker, purge, clean

---

## What's Stubbed / Partial

| Item | Status | Notes |
|------|--------|-------|
| Runtime benchmarks | Stubbed | Feature flags wired, no benchmark functions yet |
| Try-runtime | Stubbed | Feature flags wired, no migration tests |
| Equivocation reporting | Disabled | `submit_report_equivocation_unsigned_extrinsic` returns `None` |
| Weights | Placeholder | All pallets use `Weight::from_parts(N, 0)` — needs benchmarking |

---

## What's Not Built Yet (Phase 2+)

### Phase 2 Pallets (7 remaining)
- `pallet-approvals` — Multi-party approval workflows
- `pallet-attestations` — Third-party attestation anchoring
- `pallet-access-control` — Role-based access control (RBAC)
- `pallet-identities` — On-chain identity credential management
- `pallet-agent-policy` — AI agent action authorization policies
- `pallet-settlement` — Cross-chain settlement (XRPL/Stellar bridges)
- `pallet-jurisdiction-rules` — Jurisdiction-specific rule engine

### Phase 3 — Off-Chain Services
- Indexer (PostgreSQL event denormalization)
- Explorer API (GraphQL / REST)
- Proof Service (Merkle proof generation for external verification)

### Phase 4 — Integration
- TypeScript client SDK (`@legal-chain/client`)
- Web SDK integration with the Next.js Legal-Chain web app
- Agent runtime bridge (x402 / Apostle Chain connectors)

---

## Quick Start

```powershell
# Windows — build and run dev node
cd C:\Users\Kevan\legal-chain-core
cargo build --release
.\scripts\bootstrap-local.ps1

# Or directly:
.\target\release\legal-chain-node.exe --dev --tmp --rpc-cors all --rpc-port 9944
```

```bash
# Linux/macOS
cargo build --release
./scripts/bootstrap-local.sh
```

**RPC endpoint**: `ws://127.0.0.1:9944`

---

## Project Structure

```
legal-chain-core/
├── Cargo.toml              # Workspace root
├── Dockerfile              # Multi-stage build
├── docker-compose.yml      # 3-node local testnet
├── Makefile                # Common commands
├── rust-toolchain.toml     # Rust 1.81 + wasm32-unknown-unknown
├── crates/
│   └── common-types/       # Shared domain types & AuditHook trait
├── node/
│   ├── build.rs            # Substrate build script utils
│   └── src/
│       ├── main.rs         # Entry point
│       ├── cli.rs          # CLI arguments
│       ├── command.rs      # Command dispatch
│       ├── chain_spec.rs   # Genesis configuration
│       ├── rpc.rs          # JSON-RPC extensions
│       └── service.rs      # Full node service (Aura + GRANDPA)
├── pallets/
│   ├── audit/              # Index 13 — audit event anchoring
│   ├── documents/          # Index 12 — document versioning
│   ├── evidence/           # Index 11 — evidence registration
│   └── matters/            # Index 10 — matter lifecycle
├── runtime/
│   ├── build.rs            # WASM builder
│   └── src/lib.rs          # Runtime composition
├── scripts/
│   ├── bootstrap-local.ps1 # Windows bootstrap
│   └── bootstrap-local.sh  # Linux/macOS bootstrap
└── docs/                   # Architecture & design docs
```
