# LEGAL-CHAIN — Threat Model

## Scope

This document covers the sovereign legal chain node, runtime pallets, and associated services. It identifies threats relevant to a permissioned legal blockchain.

## Threat Categories

### T1 — Unauthorized State Mutation

**Threat:** An unauthorized party submits extrinsics that modify legal records.

**Mitigations:**
- Runtime `ensure_signed(origin)` verifies transaction signer identity
- Pallet-level authorization checks (creator-only updates in Phase 1, RBAC in Phase 2)
- Permissioned validator set prevents block injection
- All mutations produce audit events for forensic review

### T2 — Content Leakage via Chain State

**Threat:** Privileged legal content (documents, evidence) is stored in readable chain state.

**Mitigations:**
- Only content hashes (H256) stored on-chain — see DATA_CLASSIFICATION.md
- Actual content encrypted in off-chain storage
- Storage URIs point to encrypted blobs, not readable content
- Code review enforced: no string fields that could contain case content

### T3 — Validator Compromise

**Threat:** A validator key is compromised, allowing adversarial block production or finality attacks.

**Mitigations:**
- Permissioned validators with known operators
- Key rotation procedures documented in operational runbook
- GRANDPA finality requires 2/3+ honest validators
- Sudo key can force validator set updates in emergency
- Production: HSM-backed validator keys (Phase 5)

### T4 — Storage Key Collision

**Threat:** Crafted input causes storage key collisions in pallet storage maps.

**Mitigations:**
- All storage maps use `Blake2_128Concat` key hashing
- Domain object IDs are auto-incremented (not user-supplied)
- Content hashes computed server-side from canonical inputs

### T5 — Denial of Service via Transaction Spam

**Threat:** Adversary floods the chain with transactions to exhaust block capacity.

**Mitigations:**
- Transaction fees via `pallet-transaction-payment`
- Block weight limits enforced by Substrate runtime
- Permissioned network: only known validators produce blocks
- Rate limiting at RPC/API layer (Phase 3)

### T6 — Off-Chain Storage Tampering

**Threat:** Adversary modifies off-chain document/evidence files.

**Mitigations:**
- Content hash verified against on-chain H256 record
- Any modification detected via hash mismatch
- Proof service provides cryptographic verification bundles
- Off-chain storage encrypted at rest (AES-256-GCM)

### T7 — Unauthorized API Access

**Threat:** Unauthenticated users access indexed data or proof services.

**Mitigations (Phase 3+):**
- Explorer API requires authentication
- Role-based access to matter-specific data
- API rate limiting and request logging
- mTLS for service-to-service communication

### T8 — Runtime Upgrade Attack

**Threat:** Malicious runtime code deployed via `set_code`.

**Mitigations:**
- `set_code` restricted to sudo account (Phase 1)
- Multi-sig governance for runtime upgrades (Phase 5)
- Runtime code reviewed and tested before deployment
- WASM binary hash verified before execution

## Risk Matrix

| Threat | Likelihood | Impact | Phase Addressed |
|--------|-----------|--------|-----------------|
| T1 — Unauthorized mutation | Medium | High | Phase 1 (basic), Phase 2 (RBAC) |
| T2 — Content leakage | Low | Critical | Phase 1 (design rule) |
| T3 — Validator compromise | Low | High | Phase 1 (permissioned), Phase 5 (HSM) |
| T4 — Storage collision | Very Low | Medium | Phase 1 (Blake2_128Concat) |
| T5 — DoS | Medium | Medium | Phase 1 (fees), Phase 3 (rate limits) |
| T6 — Off-chain tampering | Medium | High | Phase 1 (hashes), Phase 3 (verification) |
| T7 — API access | Medium | Medium | Phase 3 (auth) |
| T8 — Runtime attack | Low | Critical | Phase 1 (sudo), Phase 5 (governance) |
