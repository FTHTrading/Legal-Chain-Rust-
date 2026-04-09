# LEGAL-CHAIN — Runtime Event Model

All runtime events emitted by legal-chain pallets. These events are indexed by the indexer service and queryable via the explorer API.

## Event Naming Convention

`PalletName::EventName { field1, field2, ... }`

Events use named fields (not positional) for clarity and forward compatibility.

---

## pallet-matters

| Event | Fields | When |
|-------|--------|------|
| `MatterCreated` | `matter_id: u64, creator: AccountId, matter_type: MatterType, jurisdiction_hash: H256` | New matter registered |
| `MatterUpdated` | `matter_id: u64, updater: AccountId, field_changed: UpdatedField` | Matter metadata changed |
| `MatterStatusChanged` | `matter_id: u64, actor: AccountId, old_status: MatterStatus, new_status: MatterStatus` | Matter status transition |

## pallet-evidence

| Event | Fields | When |
|-------|--------|------|
| `EvidenceRegistered` | `evidence_id: u64, matter_id: u64, registrar: AccountId, content_hash: H256` | Evidence hash anchored |
| `EvidenceVerified` | `evidence_id: u64, verifier: AccountId` | Evidence independently verified |
| `CustodyStateChanged` | `evidence_id: u64, actor: AccountId, old_state: CustodyState, new_state: CustodyState` | Custody status updated |

## pallet-documents

| Event | Fields | When |
|-------|--------|------|
| `DocumentRegistered` | `document_id: u64, matter_id: u64, registrar: AccountId, content_hash: H256, version: u32` | Document version anchored |
| `DocumentSuperseded` | `document_id: u64, superseded_by: u64, actor: AccountId` | Old version replaced |
| `FilingReadinessChanged` | `document_id: u64, actor: AccountId, new_readiness: FilingReadiness` | Filing status updated |

## pallet-approvals (Phase 2)

| Event | Fields | When |
|-------|--------|------|
| `ApprovalOpened` | `approval_id: u64, matter_id: u64, drafter: AccountId, target_type: SubjectType, target_id: u64` | Approval workflow initiated |
| `ApprovalCompleted` | `approval_id: u64, completed_by: AccountId` | All required approvals received |
| `ApprovalRejected` | `approval_id: u64, rejected_by: AccountId, reason_hash: H256` | Approval denied |

## pallet-attestations (Phase 2)

| Event | Fields | When |
|-------|--------|------|
| `AttestationIssued` | `attestation_id: u64, issuer: AccountId, subject_type: SubjectType, subject_id: u64, claim_type: ClaimType` | Attestation created |
| `AttestationRevoked` | `attestation_id: u64, revoker: AccountId` | Attestation invalidated |

## pallet-audit

| Event | Fields | When |
|-------|--------|------|
| `AuditEventAnchored` | `audit_id: u64, matter_id: Option<u64>, actor: AccountId, action: ActionType, target_type: SubjectType, target_id: u64` | Audit record written |

## pallet-settlement (Phase 2)

| Event | Fields | When |
|-------|--------|------|
| `SettlementRecorded` | `settlement_id: u64, matter_id: u64, recorder: AccountId, settlement_hash: H256, amount: u128` | Settlement proof anchored |

## pallet-identities (Phase 2)

| Event | Fields | When |
|-------|--------|------|
| `IdentityCredentialIssued` | `credential_id: u64, subject: AccountId, role: IdentityRole, issuer: AccountId` | Identity credential created |
| `IdentityCredentialRevoked` | `credential_id: u64, revoker: AccountId` | Credential revoked |

## pallet-agent-policy (Phase 2)

| Event | Fields | When |
|-------|--------|------|
| `AgentActionAttempted` | `agent: AccountId, action_class: ActionClass, target_type: SubjectType, target_id: u64` | AI agent attempted action |
| `AgentActionApproved` | `agent: AccountId, action_class: ActionClass, approver: AccountId` | Agent action approved by human |
| `AgentActionBlocked` | `agent: AccountId, action_class: ActionClass, reason: BlockReason` | Agent action denied by policy |

---

## Indexer Mapping

The indexer decodes events from finalized blocks and persists them into Postgres tables with consistent naming: `event_{pallet}_{event_name}`. Each row includes `block_number`, `extrinsic_index`, `timestamp`, and all event fields.
