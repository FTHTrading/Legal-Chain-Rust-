<p align="center">
  <img src="https://img.shields.io/badge/LEGAL--CHAIN-Substrate%20Blockchain-1a1a2e?style=for-the-badge&labelColor=0e1225&color=c9a84c" alt="Legal-Chain" />
</p>

<h1 align="center">⚖️ LEGAL-CHAIN — Sovereign Rust Blockchain</h1>

<p align="center">
  <strong>Integrity layer for legal evidence, document proof, approvals, chain-of-custody, audit events, identity-aware workflow enforcement, and matter-linked settlement.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.88-dea584?style=flat-square&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Substrate-polkadot--stable2409-e6007a?style=flat-square&logo=polkadot&logoColor=white" />
  <img src="https://img.shields.io/badge/Consensus-Aura%20%2B%20GRANDPA-4361ee?style=flat-square" />
  <img src="https://img.shields.io/badge/WASM-Forkless%20Upgrades-06d6a0?style=flat-square&logo=webassembly&logoColor=white" />
  <img src="https://img.shields.io/badge/License-BUSL--1.1-ef476f?style=flat-square" />
</p>

<p align="center">
  <a href="#-architecture">Architecture</a> •
  <a href="#-pallets">Pallets</a> •
  <a href="#-build-phases">Build Phases</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-repository-layout">Repo Layout</a> •
  <a href="#-documentation">Docs</a>
</p>

---

## 🏗️ Architecture

The chain does **NOT** store raw privileged legal content. It stores **hashes, references, signatures, metadata, and workflow state** — ensuring cryptographic integrity without exposing sensitive data.

```
┌─────────────────────────────────────────────────────────────────┐
│  PRESENTATION                                                   │
│  Legal-Chain Web App (Next.js) — UI, Client Portal, Ops Dash   │
└──────────┬──────────────────────────────────────────────────────┘
           │
┌──────────▼──────────────────────────────────────────────────────┐
│  SERVICES                                                       │
│  Explorer API ·· Proof Service ·· TypeScript SDK                │
└──────────┬──────────────────────────────────────────────────────┘
           │
┌──────────▼──────────────────────────────────────────────────────┐
│  DATA                                                           │
│  Indexer (Rust → Postgres) ·· Encrypted Off-Chain Store         │
└──────────┬──────────────────────────────────────────────────────┘
           │  WebSocket RPC
┌──────────▼──────────────────────────────────────────────────────┐
│  ⛓️  LEGAL-CHAIN NODE  (Substrate)                              │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  RUNTIME (compiles to native + WASM)                       │ │
│  │                                                            │ │
│  │  Phase 1 ─ Core          Phase 2 ─ Workflow                │ │
│  │  ├─ pallet-matters       ├─ pallet-approvals               │ │
│  │  ├─ pallet-evidence      ├─ pallet-identities              │ │
│  │  ├─ pallet-documents     ├─ pallet-access-control          │ │
│  │  └─ pallet-audit         └─ pallet-agent-policy            │ │
│  │                                                            │ │
│  │  Consensus: Aura (sr25519) + GRANDPA (ed25519)             │ │
│  │  Block Time: 6s · MaxAuthorities: 32 · Permissioned Set    │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Tech Stack

| Layer | Technology | Details |
|:------|:-----------|:--------|
| 🦀 **Language** | Rust | Edition 2021, toolchain 1.88.0 |
| ⛓️ **Framework** | Polkadot SDK / Substrate | Tag `polkadot-stable2409` |
| 🔐 **Consensus** | Aura + GRANDPA | Authority round + deterministic finality |
| 📦 **Encoding** | SCALE codec | `parity-scale-codec` 3.6 |
| 🧬 **Hashing** | Blake2-256 | For block, state, and content hashes |
| 🧾 **Account Model** | MultiSignature | sr25519 accounts, `Balance = u128` |
| 🌐 **WASM** | `wasm32-unknown-unknown` | Forkless runtime upgrades via `set_code` |

---

## 🧱 Pallets

### Phase 1 — Chain Core (indices 10–13)

| # | Pallet | Purpose | Key Extrinsics |
|:-:|:-------|:--------|:---------------|
| 10 | `pallet-matters` | Legal matter lifecycle | `create`, `update`, `transition_status` |
| 11 | `pallet-evidence` | Evidence vault + chain of custody | `register`, `update`, `transfer_custody` |
| 12 | `pallet-documents` | Document proof + version control | `register`, `update`, `approve`, `supersede` |
| 13 | `pallet-audit` | Immutable audit log (hash-chained) | Auto-anchored via `AuditHook` trait |

### Phase 2 — Workflow (indices 14–17)

| # | Pallet | Purpose | Key Extrinsics |
|:-:|:-------|:--------|:---------------|
| 14 | `pallet-approvals` | Quorum-based multi-reviewer approvals | `request_approval`, `decide`, `withdraw` |
| 15 | `pallet-identities` | Identity registration with role/org/jurisdiction | `register`, `revoke`, `update_role` |
| 16 | `pallet-access-control` | Matter-scoped RBAC with admin bootstrapping | `grant_access`, `revoke_access`, `designate_admin` |
| 17 | `pallet-agent-policy` | AI agent registration + rate-limited capabilities | `register_agent`, `update_policy`, `revoke_agent` |

### Shared Types (`legal-chain-common-types`)

| Type | Variants |
|:-----|:---------|
| `IdentityRole` | Attorney, Paralegal, Clerk, Judge, Witness, Expert, Client, Operator, AiAgent, Auditor, Administrator |
| `ApprovalStatus` | Pending, Approved, Rejected, Withdrawn, Expired |
| `MatterStatus` | Draft → Active → OnHold / UnderReview / PendingApproval → Settled → Closed → Archived |
| `SubjectType` | Matter, Evidence, Document, Approval, Attestation, Settlement, Identity, AgentPolicy |
| `ActionType` | Create, Update, Delete, StatusChange, Verify, Approve, Reject, Supersede, CustodyTransfer, Attest, Revoke, Register, Settle |

### Phase 3 — Data Services

| Service | Port | Purpose |
|:--------|:----:|:--------|
| `legal-chain-indexer` | — | Subscribes to finalized blocks via WebSocket RPC, decodes events, writes to Postgres |
| `legal-chain-explorer-api` | 8300 | REST API (Axum) — 17 endpoints for blocks, events, matters, evidence, documents, approvals, identities, audit, stats |
| `legal-chain-proof-service` | 8400 | Fetches Merkle read-proofs from the node, produces signed `ProofBundle` for legal discovery |

### Cross-Pallet Integration

```
pallet-matters ──┐
pallet-evidence ─┤                  ┌──────────────┐
pallet-documents ├── AuditHook ────▶│ pallet-audit │  (immutable log)
pallet-approvals ─┤                  └──────────────┘
pallet-identities ┤
pallet-access-control ┤
pallet-agent-policy ───┘
```

All pallets call `AuditHook::on_state_change()` on every mutation — ensuring a complete, tamper-evident audit trail.

---

## 🔒 Security Invariants

| # | Invariant |
|:-:|:----------|
| 1 | **No raw privileged content on-chain** — hashes and references only |
| 2 | **Every state mutation** emits a durable audit event |
| 3 | **AI agents** are registered service identities with scoped, rate-limited permissions |
| 4 | **Human approval gates** for sensitive legal actions (quorum-based) |
| 5 | **Role-based access control** at the runtime level |
| 6 | **Forkless upgrades** via WASM — no hard forks needed for pallet changes |

---

## 📊 Build Phases

| Phase | Scope | Status |
|:-----:|:------|:------:|
| **0** | Foundation — scaffold, docs, shared types | ✅ Complete |
| **1** | Chain Core — node, runtime, matters / evidence / documents / audit | ✅ Complete |
| **2** | Workflow — approvals, identities, access-control, agent-policy | ✅ Complete |
| **3** | Data Services — indexer, explorer API, proof service | ✅ Complete |
| **4** | Integration — TypeScript client, web SDK, proof bundles | 🔲 Planned |
| **5** | Hardening — threat model, negative tests, runbooks, observability | 🔲 Planned |

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.88.0 (with `wasm32-unknown-unknown` target)
- **Perl** (for native C dependency builds on Windows)
- **protoc** (Protocol Buffers compiler)

### Build

```powershell
# Native check (fast — skips WASM compilation)
$env:SKIP_WASM_BUILD = "1"
cargo check

# Full build (native + WASM runtime)
$env:WASM_BUILD_TOOLCHAIN = "1.85.0"
cargo build --release

# Run development node (single validator, ephemeral state)
./target/release/legal-chain-node --dev --tmp
# → ws://127.0.0.1:9944
```

### Toolchain Notes

| Component | Version | Purpose |
|:----------|:--------|:--------|
| Native build | Rust 1.88.0 | Node binary + runtime (native) |
| WASM build | Rust 1.85.0 | Runtime WASM blob (avoids sp-io lint) |
| `rust-toolchain.toml` | 1.88.0 | Default channel for the workspace |

---

## 📂 Repository Layout

```
legal-chain-core/
├── node/                    # 🖥️  Substrate node binary
│   └── src/
│       ├── main.rs          #     Entry point
│       ├── cli.rs           #     CLI definition
│       ├── command.rs       #     Subcommand handlers
│       ├── chain_spec.rs    #     Genesis configuration
│       ├── service.rs       #     Full node service wiring
│       └── rpc.rs           #     JSON-RPC extensions
│
├── runtime/                 # 🧬  WASM + native runtime
│   └── src/lib.rs           #     All pallet Config + construct_runtime!
│
├── pallets/                 # ⚖️  Custom FRAME pallets
│   ├── matters/             #     Legal matter lifecycle
│   ├── evidence/            #     Evidence vault + custody
│   ├── documents/           #     Document proof + versioning
│   ├── audit/               #     Immutable audit log
│   ├── approvals/           #     Quorum-based approvals
│   ├── identities/          #     Identity + role management
│   ├── access-control/      #     Matter-scoped RBAC
│   └── agent-policy/        #     AI agent policies + rate limits
│
├── services/                # 🔌  Off-chain data services
│   ├── indexer/             #     Block subscriber → Postgres ETL
│   ├── explorer-api/        #     REST API (Axum) for indexed data
│   └── proof-service/       #     Merkle proof & integrity verification
│
├── crates/                  # 📦  Shared libraries
│   └── common-types/        #     Domain types, AuditHook trait
│
├── docs/                    # 📚  Architecture & operations docs
│   ├── ARCHITECTURE.md
│   ├── STORAGE_MODEL.md
│   ├── DECISIONS.md
│   └── OPERATOR-SUMMARY.md
│
├── scripts/                 # 🔧  Bootstrap & automation
├── Cargo.toml               #     Workspace root (14 members)
├── Cargo.lock               #     Curated lockfile with pins
├── rust-toolchain.toml      #     Rust 1.88.0 + wasm32 target
├── Dockerfile               #     Multi-stage build (rust:1.88)
├── docker-compose.yml       #     Local dev orchestration
└── Makefile                 #     Common build targets
```

---

## 📚 Documentation

| Document | Description |
|:---------|:------------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design, domain model, data flows |
| [STORAGE_MODEL.md](docs/STORAGE_MODEL.md) | On-chain storage patterns and conventions |
| [DECISIONS.md](docs/DECISIONS.md) | Architecture decision records |
| [OPERATOR-SUMMARY.md](docs/OPERATOR-SUMMARY.md) | Operator quickstart & node management |

---

## 🔗 Related Repository

| Repo | Branch | Contents |
|:-----|:-------|:---------|
| [Legal-Chain](https://github.com/FTHTrading/Legal-Chain) | `main` | Next.js web app — UI, API routes, Zod schemas, agent network |
| **Legal-Chain-Rust-** | `main` | ← **You are here** — Substrate node, runtime, FRAME pallets |

---

<p align="center">
  <strong>UNYKORN // LAW</strong><br/>
  <sub>Sovereign legal intelligence infrastructure · Built by <a href="https://github.com/FTHTrading">FTH Trading</a></sub><br/>
  <sub>Human Supervised · Apostle Chain 7332 · ATP Settlement</sub>
</p>

---

<p align="center">
  <img src="https://img.shields.io/badge/BUSL--1.1-Business%20Source%20License-ef476f?style=flat-square" />
</p>
