//! # Legal-Chain Common Types
//!
//! Shared domain identifiers, status enums, classification types, and the
//! cross-pallet `AuditHook` trait used by all legal-chain pallets.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::BoundedVec;
use frame_support::parameter_types;
use scale_info::TypeInfo;
use sp_core::H256;

// ─── Domain Identifiers ────────────────────────────────────────────

pub type MatterId = u64;
pub type EvidenceId = u64;
pub type DocumentId = u64;
pub type ApprovalId = u64;
pub type AttestationId = u64;
pub type AuditId = u64;
pub type SettlementId = u64;
pub type CredentialId = u64;
pub type ContentHash = H256;

/// Bounded byte vector used for short string fields (titles, labels).
pub type BoundedString<S> = BoundedVec<u8, S>;

// ─── Default Limits ────────────────────────────────────────────────

parameter_types! {
    pub const DefaultMaxTitleLength: u32 = 256;
    pub const DefaultMaxDescriptionLength: u32 = 1024;
    pub const DefaultMaxUriLength: u32 = 512;
    pub const DefaultMaxMetadataLength: u32 = 2048;
    pub const DefaultMaxPartiesPerMatter: u32 = 32;
}

// ─── Matter Types ──────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum MatterType {
    Litigation,
    Regulatory,
    Transactional,
    Advisory,
    Investigation,
    Compliance,
    Administrative,
}

impl Default for MatterType {
    fn default() -> Self {
        Self::Litigation
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum MatterStatus {
    Draft,
    Active,
    OnHold,
    UnderReview,
    PendingApproval,
    Settled,
    Closed,
    Archived,
}

impl Default for MatterStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl MatterStatus {
    /// Validate whether a status transition is allowed.
    pub fn can_transition_to(&self, target: &MatterStatus) -> bool {
        use MatterStatus::*;
        matches!(
            (self, target),
            (Draft, Active)
                | (Active, OnHold)
                | (Active, UnderReview)
                | (Active, PendingApproval)
                | (Active, Settled)
                | (Active, Closed)
                | (OnHold, Active)
                | (OnHold, Closed)
                | (UnderReview, Active)
                | (UnderReview, PendingApproval)
                | (PendingApproval, Active)
                | (PendingApproval, Settled)
                | (PendingApproval, Closed)
                | (Settled, Closed)
                | (Closed, Archived)
        )
    }
}

// ─── Evidence Types ────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum EvidenceStatus {
    Submitted,
    UnderReview,
    Verified,
    Challenged,
    Withdrawn,
    Admitted,
    Excluded,
}

impl Default for EvidenceStatus {
    fn default() -> Self {
        Self::Submitted
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum CustodyState {
    InPossession,
    InTransit,
    InStorage,
    InReview,
    Released,
    Sealed,
}

impl Default for CustodyState {
    fn default() -> Self {
        Self::InPossession
    }
}

// ─── Document Types ────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum DocumentStatus {
    Draft,
    UnderReview,
    Approved,
    Filed,
    Superseded,
    Withdrawn,
}

impl Default for DocumentStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum FilingReadiness {
    NotReady,
    InPreparation,
    ReadyForReview,
    ReadyForFiling,
    Filed,
    Rejected,
}

impl Default for FilingReadiness {
    fn default() -> Self {
        Self::NotReady
    }
}

// ─── Sensitivity / Classification ──────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum Sensitivity {
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl Default for Sensitivity {
    fn default() -> Self {
        Self::Confidential
    }
}

// ─── Audit Types ───────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum ActionType {
    Create,
    Update,
    Delete,
    StatusChange,
    Verify,
    Approve,
    Reject,
    Supersede,
    CustodyTransfer,
    FileDocument,
    Attest,
    Revoke,
    Register,
    Settle,
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum SubjectType {
    Matter,
    Evidence,
    Document,
    Approval,
    Attestation,
    Settlement,
    Identity,
    AgentPolicy,
}

// ─── Updated Field (for incremental updates) ──────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum UpdatedField {
    Title,
    Description,
    Jurisdiction,
    Sensitivity,
    MatterType,
    Metadata,
    Status,
    Parties,
    CustodyState,
    FilingReadiness,
    ContentHash,
    StorageUri,
}

// ─── Identity Types (Phase 2 prep) ────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum IdentityRole {
    Attorney,
    Paralegal,
    Clerk,
    Judge,
    Witness,
    Expert,
    Client,
    Operator,
    AiAgent,
    Auditor,
    Administrator,
}

// ─── Approval Types (Phase 2 prep) ────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Withdrawn,
    Expired,
}

impl Default for ApprovalStatus {
    fn default() -> Self {
        Self::Pending
    }
}

// ─── Claim Types (Phase 2 prep) ───────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Debug)]
pub enum ClaimType {
    Identity,
    Qualification,
    Jurisdiction,
    Authorization,
    Compliance,
    Certification,
}

// ─── AuditHook Trait ───────────────────────────────────────────────

/// Cross-pallet audit hook. Implemented by `pallet-audit` for production use.
/// The `()` implementation provides a no-op for isolated pallet testing.
pub trait AuditHook<AccountId> {
    fn on_state_change(
        matter_id: Option<MatterId>,
        actor: &AccountId,
        action: ActionType,
        subject: SubjectType,
        subject_id: u64,
        before_hash: Option<ContentHash>,
        after_hash: Option<ContentHash>,
    );
}

/// No-op audit hook for testing pallets in isolation.
impl<AccountId> AuditHook<AccountId> for () {
    fn on_state_change(
        _matter_id: Option<MatterId>,
        _actor: &AccountId,
        _action: ActionType,
        _subject: SubjectType,
        _subject_id: u64,
        _before_hash: Option<ContentHash>,
        _after_hash: Option<ContentHash>,
    ) {
    }
}

// ─── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matter_status_valid_transitions() {
        assert!(MatterStatus::Draft.can_transition_to(&MatterStatus::Active));
        assert!(MatterStatus::Active.can_transition_to(&MatterStatus::OnHold));
        assert!(MatterStatus::Active.can_transition_to(&MatterStatus::Settled));
        assert!(MatterStatus::Closed.can_transition_to(&MatterStatus::Archived));
    }

    #[test]
    fn matter_status_invalid_transitions() {
        assert!(!MatterStatus::Draft.can_transition_to(&MatterStatus::Closed));
        assert!(!MatterStatus::Archived.can_transition_to(&MatterStatus::Active));
        assert!(!MatterStatus::Settled.can_transition_to(&MatterStatus::Draft));
    }

    #[test]
    fn noop_audit_hook_compiles() {
        <() as AuditHook<u64>>::on_state_change(
            Some(1),
            &42u64,
            ActionType::Create,
            SubjectType::Matter,
            1,
            None,
            None,
        );
    }
}
