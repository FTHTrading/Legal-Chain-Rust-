# LEGAL-CHAIN — Architecture Decision Records

Each decision below was made during the genesis build. Future changes should add new records, not modify old ones.

---

## ADR-001: Polkadot SDK / Substrate for Chain Framework

**Status:** Accepted
**Date:** 2026-04-09

**Context:** Need a sovereign blockchain with custom runtime logic, permissioned validators, fast finality, and Rust-native implementation. Alternatives considered: custom chain from scratch (too much infrastructure burden), Cosmos SDK (Go, not Rust), rollup on Ethereum (not sovereign enough).

**Decision:** Use Polkadot SDK (Substrate) with custom FRAME pallets. Sovereign solochain — no parachain dependency for MVP. Polkadot interoperability deferred to post-MVP.

**Consequences:**
- Full control over consensus, runtime, and governance
- WASM runtime enables forkless upgrades
- Large dependency tree (~400 crates) and long initial compile times
- Requires WASM target (wasm32-unknown-unknown) in toolchain

---

## ADR-002: Permissioned Validator Set

**Status:** Accepted
**Date:** 2026-04-09

**Context:** Legal chain handles sensitive workflow state. Open PoS is inappropriate — validators must be known and accountable entities.

**Decision:** Use a permissioned validator set defined at genesis. Aura for block authoring, GRANDPA for finality. Validator set changes require sudo or governance action.

**Consequences:**
- Fast finality (typically < 6 seconds)
- Validator identity is known and auditable
- No staking or slashing mechanics needed at MVP
- Must manage validator key rotation operationally

---

## ADR-003: Hashes On-Chain, Artifacts Off-Chain

**Status:** Accepted
**Date:** 2026-04-09

**Context:** Legal documents and evidence contain privileged content. Storing raw content on-chain would violate attorney-client privilege, create GDPR exposure, and bloat chain state.

**Decision:** Store content hashes (H256) and encrypted storage URIs on-chain. Raw binaries go to encrypted off-chain storage (local FS for dev, S3-compatible for production).

**Consequences:**
- Chain state remains small and manageable
- Integrity verification via hash comparison
- Off-chain storage must be independently reliable and backed up
- Proof bundles combine on-chain attestation with off-chain content verification

---

## ADR-004: Sequential Domain IDs (not Hashes)

**Status:** Accepted
**Date:** 2026-04-09

**Context:** Domain objects (matters, evidence records, documents) need identifiers. Could use H256 hashes or sequential u64 integers.

**Decision:** Use auto-incrementing `u64` IDs for all domain objects. Legal systems use case numbers, not hashes. Content hashes are stored as separate `H256` fields.

**Consequences:**
- Human-readable references (Matter #1, Evidence #42)
- Simple range queries and ordering
- No hash collision concerns for IDs
- Must manage NextId counters in each pallet

---

## ADR-005: Cross-Pallet Audit via Trait Hook

**Status:** Accepted
**Date:** 2026-04-09

**Context:** Every state-changing action must emit a durable audit event. Pallets could emit their own events (already do) but a centralized audit record with consistent schema adds value.

**Decision:** Define an `AuditHook<AccountId>` trait in `common-types`. The audit pallet implements it. Other pallets take it as a Config associated type and call it on every mutation.

**Consequences:**
- Single audit pallet owns the audit storage
- Pallets remain loosely coupled
- `()` no-op implementation available for isolated pallet testing
- Audit events are queryable via a single consistent schema

---

## ADR-006: No Tokenomics at Genesis

**Status:** Accepted
**Date:** 2026-04-09

**Context:** This is a legal chain, not a DeFi chain. Balances exist for transaction fees and future settlement mechanics, but there is no speculative token launch or staking economics at MVP.

**Decision:** Include `pallet-balances` for basic fee handling. Do not implement staking, inflation, or token distribution. Settlement records reference external payment systems (fiat, stablecoin) via hashes.

**Consequences:**
- Clean legal focus without tokenomics distraction
- Balances can be used for metered service fees later
- External settlement references keep the chain payment-agnostic

---

## ADR-007: Rust Toolchain Pin

**Status:** Accepted
**Date:** 2026-04-09

**Context:** Substrate requires specific Rust versions. Nightly was historically required but stable Rust support improved in 2024+.

**Decision:** Pin to Rust 1.81.0 stable via `rust-toolchain.toml`. Include `wasm32-unknown-unknown` target for runtime compilation.

**Consequences:**
- Reproducible builds across developer machines
- Must update toolchain pin when bumping Polkadot SDK version
- See VERSION_MATRIX.md for full dependency tracking
