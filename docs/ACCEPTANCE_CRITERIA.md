# LEGAL-CHAIN — Acceptance Criteria

## Phase 0 — Foundation

- [x] Workspace Cargo.toml compiles with `cargo check` (once all members exist)
- [x] rust-toolchain.toml pins Rust 1.81.0 with wasm32 target
- [x] README documents architecture, build steps, and project layout
- [x] ARCHITECTURE.md describes all system layers and data flow
- [x] DECISIONS.md has ADRs for framework, consensus, data model choices
- [x] VERSION_MATRIX.md pins all dependency versions
- [x] EVENT_MODEL.md documents all runtime events
- [x] DATA_CLASSIFICATION.md defines sensitivity levels
- [x] STORAGE_MODEL.md describes on-chain and off-chain storage
- [x] Common types crate defines shared IDs, enums, and AuditHook trait

## Phase 1 — Chain Core

### pallet-matters
- [ ] `create_matter` extrinsic stores matter with auto-increment ID
- [ ] `update_matter` modifies matter metadata with authorization check
- [ ] `change_status` enforces valid state transitions
- [ ] All mutations emit events AND call AuditHook
- [ ] Creator-indexed secondary storage works
- [ ] Errors for: not found, not authorized, invalid status transition

### pallet-evidence
- [ ] `register_evidence` stores evidence hash linked to matter
- [ ] `verify_evidence` updates verification state
- [ ] `update_custody` tracks chain-of-custody transitions
- [ ] All mutations emit events AND call AuditHook
- [ ] Evidence queryable by matter ID

### pallet-documents
- [ ] `register_document` stores document hash with version tracking
- [ ] `supersede_document` links old version to new
- [ ] `update_filing_readiness` changes filing state
- [ ] Version history maintained
- [ ] All mutations emit events AND call AuditHook

### pallet-audit
- [ ] `anchor_event` stores audit record with all fields
- [ ] Implements `AuditHook` trait for cross-pallet use
- [ ] Events queryable by matter ID and by actor
- [ ] Audit records are append-only (no update/delete)

### Runtime
- [ ] `construct_runtime!` includes all Phase 1 pallets
- [ ] Pallet indices: Matters=10, Evidence=11, Documents=12, Audit=13
- [ ] All runtime APIs implemented (Core, BlockBuilder, AuraApi, GrandpaApi, etc.)
- [ ] WASM binary compiles successfully

### Node
- [ ] Node binary starts with `--dev` flag
- [ ] Development chain spec includes 3 validators
- [ ] Pre-funded test accounts available
- [ ] RPC endpoint responds to `system_health`
- [ ] Blocks produced at 6-second intervals

### Integration
- [ ] Can submit `create_matter` via Polkadot.js Apps
- [ ] Events visible in block explorer
- [ ] Chain state queryable via RPC

## Phase 2 — Expanded Pallets (future)

- [ ] pallet-approvals: multi-party approval workflows
- [ ] pallet-attestations: verifiable claims
- [ ] pallet-access-control: RBAC enforcement
- [ ] pallet-settlement: payment proof anchoring
- [ ] pallet-identities: credential management
- [ ] pallet-agent-policy: AI agent guardrails
- [ ] pallet-jurisdiction-rules: geographic constraints

## Non-Negotiable Design Rules

Every phase must satisfy these invariants:

1. Every on-chain record has a content hash or is itself a hash-reference
2. No plaintext privileged content stored on-chain
3. Every mutation produces an audit event
4. All pallet storage uses bounded types with explicit maximums
5. Runtime compiles to both native and WASM
6. Node boots from genesis with no external dependencies
7. Pallet errors are descriptive and specific
8. Events contain enough fields to reconstruct state changes
9. Storage migrations are versioned
10. Tests use the `()` AuditHook implementation for isolation
