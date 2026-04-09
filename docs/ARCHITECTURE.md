# LEGAL-CHAIN — Architecture

## Overview

LEGAL-CHAIN is a sovereign Substrate-based blockchain purpose-built for legal operations. It serves as the integrity and state-verification layer for legal evidence, document proof, approvals, chain-of-custody tracking, audit events, identity-aware workflow enforcement, and matter-linked settlement.

The chain does NOT store raw privileged legal content. It stores hashes, references, signatures, metadata, and workflow state.

## System Layers

```
┌─────────────────────────────────────────────────────────────┐
│ PRESENTATION LAYER                                          │
│  Legal-Chain Web App (Next.js)                              │
│  Client Portal, Ops Dashboard, Media, Intake                │
└──────────┬──────────────────────────────────────────────────┘
           │
┌──────────▼──────────────────────────────────────────────────┐
│ SERVICE LAYER                                               │
│                                                              │
│  Explorer API ──── Query legal objects, blocks, events       │
│  Proof Service ─── Verify integrity, produce proof bundles   │
│  TypeScript SDK ── Typed RPC + service wrappers              │
└──────────┬──────────────────────────────────────────────────┘
           │
┌──────────▼──────────────────────────────────────────────────┐
│ DATA LAYER                                                   │
│                                                              │
│  Indexer ────────── Subscribe to blocks, persist to Postgres │
│  Off-Chain Store ── Encrypted artifact storage (FS / S3)     │
└──────────┬──────────────────────────────────────────────────┘
           │
┌──────────▼──────────────────────────────────────────────────┐
│ CONSENSUS LAYER                                              │
│                                                              │
│  LEGAL-CHAIN NODE (Substrate)                                │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ RUNTIME (WASM)                                          │ │
│  │                                                          │ │
│  │  pallet-matters          pallet-evidence                 │ │
│  │  pallet-documents        pallet-approvals                │ │
│  │  pallet-attestations     pallet-audit                    │ │
│  │  pallet-settlement       pallet-identities               │ │
│  │  pallet-access-control   pallet-agent-policy             │ │
│  │  pallet-jurisdiction-rules                               │ │
│  │                                                          │ │
│  │  Consensus: Aura (authoring) + GRANDPA (finality)        │ │
│  │  Validators: permissioned set at genesis                 │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Domain Model

The chain manages these first-class domain objects:

| Object | Pallet | On-Chain State |
|--------|--------|----------------|
| Matter | `pallet-matters` | ID, title hash, jurisdiction, type, status, parties, sensitivity |
| Evidence | `pallet-evidence` | ID, matter ref, content hash, encrypted URI, custody state |
| Document | `pallet-documents` | ID, matter ref, content hash, version, approval state, supersession |
| Approval | `pallet-approvals` | ID, matter ref, target ref, reviewer/approver sets, status |
| Attestation | `pallet-attestations` | ID, subject ref, issuer, claim type, signature, revocation |
| Audit Event | `pallet-audit` | ID, matter ref, actor, action, target, before/after hashes |
| Settlement | `pallet-settlement` | ID, matter ref, payment ref, payer/payee, amount, status |
| Identity | `pallet-identities` | ID, subject, role, org, jurisdiction scope, revocation |

## Consensus

- **Block Authoring:** Aura (authority-round) with 6-second slot duration
- **Finality:** GRANDPA deterministic finality
- **Validator Set:** Permissioned at genesis, upgradable via governance
- **Target:** 3 validators minimum for devnet, 5+ for testnet

## Security Invariants

1. No raw privileged content on-chain — hashes and references only
2. Every state mutation emits a durable audit event
3. AI agents are registered service identities with scoped permissions
4. Human approval gates for sensitive legal actions
5. Role-based access control at runtime and API layers
6. Destructive admin ops require explicit authorization
7. Off-chain storage is encrypted per-matter or per-object

## Data Flow: Evidence Registration

```
1. Web App uploads document to Off-Chain Storage (encrypted)
2. Off-Chain Storage returns content_hash + storage_uri
3. Web App calls Proof Service → register_evidence(matter_id, content_hash, uri)
4. Proof Service submits extrinsic to chain node
5. pallet-evidence validates, stores record, emits EvidenceRegistered
6. pallet-audit auto-anchors audit event
7. Indexer picks up event, persists to Postgres
8. Explorer API serves queryable evidence record
```

## Cross-Pallet Integration

Pallets communicate via trait-based loose coupling:

- `AuditHook<AccountId>` — implemented by `pallet-audit`, used by all other pallets to record state changes
- Common types shared via `legal-chain-common-types` crate
- No tight pallet-to-pallet dependencies at the frame level

## Runtime Upgrades

The runtime compiles to WASM, enabling forkless upgrades via `set_code` through the sudo or governance mechanism. All pallet storage uses explicit versioning for safe migrations.
