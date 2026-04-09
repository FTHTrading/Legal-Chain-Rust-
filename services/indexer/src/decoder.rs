//! Event decoder — maps raw pallet events to structured data for Postgres.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A decoded chain event ready for database storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedEvent {
    pub pallet: String,
    pub variant: String,
    pub data: Value,
}

/// Pallet index → pallet name mapping (must match construct_runtime! indices).
pub fn pallet_name(index: u8) -> Option<&'static str> {
    match index {
        0 => Some("System"),
        1 => Some("Timestamp"),
        2 => Some("Aura"),
        3 => Some("Grandpa"),
        4 => Some("Balances"),
        5 => Some("TransactionPayment"),
        6 => Some("Sudo"),
        10 => Some("Matters"),
        11 => Some("Evidence"),
        12 => Some("Documents"),
        13 => Some("Audit"),
        14 => Some("Approvals"),
        15 => Some("Identities"),
        16 => Some("AccessControl"),
        17 => Some("AgentPolicy"),
        _ => None,
    }
}

/// Check if a pallet index belongs to a legal-domain pallet (indices 10–17).
pub fn is_legal_pallet(index: u8) -> bool {
    (10..=17).contains(&index)
}

/// Decode raw event bytes into a structured `DecodedEvent`.
///
/// This performs best-effort SCALE decoding. For production use, generate
/// decoding logic from the chain metadata (via `subxt` or `desub`).
///
/// The raw event record layout in Substrate:
/// - 1 byte: phase encoding
/// - 1 byte (compact): pallet index
/// - 1 byte: event variant index within the pallet
/// - N bytes: variant-specific fields (SCALE-encoded)
/// - 32 bytes: topics (Blake2-256 hashes)
pub fn decode_event_record(raw: &[u8]) -> Option<DecodedEvent> {
    if raw.len() < 3 {
        return None;
    }

    // Skip phase byte(s) — for simplicity, handle both compact and fixed
    // In Substrate, phase is a compact-encoded enum. For finalized blocks,
    // it's typically `ApplyExtrinsic(index)`.
    // We focus on extracting pallet_index and variant_index.

    // This is a simplified decoder. A production indexer should use
    // frame-metadata + scale-decode for full fidelity.
    let pallet_index = raw[0];
    let variant_index = raw[1];

    let pallet = pallet_name(pallet_index)?;

    let variant = decode_variant_name(pallet_index, variant_index)?;

    // Store remaining bytes as hex for downstream processing
    let payload_hex = hex::encode(&raw[2..]);

    Some(DecodedEvent {
        pallet: pallet.to_string(),
        variant: variant.to_string(),
        data: serde_json::json!({
            "pallet_index": pallet_index,
            "variant_index": variant_index,
            "raw_payload": payload_hex,
        }),
    })
}

/// Map (pallet_index, variant_index) to event variant name.
/// Must stay in sync with the pallet event enum declaration order.
fn decode_variant_name(pallet_index: u8, variant_index: u8) -> Option<&'static str> {
    match (pallet_index, variant_index) {
        // Matters (index 10)
        (10, 0) => Some("MatterCreated"),
        (10, 1) => Some("MatterUpdated"),
        (10, 2) => Some("MatterStatusChanged"),

        // Evidence (index 11)
        (11, 0) => Some("EvidenceRegistered"),
        (11, 1) => Some("EvidenceVerified"),
        (11, 2) => Some("CustodyStateChanged"),

        // Documents (index 12)
        (12, 0) => Some("DocumentRegistered"),
        (12, 1) => Some("DocumentSuperseded"),
        (12, 2) => Some("FilingReadinessChanged"),

        // Audit (index 13)
        (13, 0) => Some("AuditEventAnchored"),

        // Approvals (index 14)
        (14, 0) => Some("ApprovalRequested"),
        (14, 1) => Some("ApprovalDecided"),
        (14, 2) => Some("ApprovalFinalized"),
        (14, 3) => Some("ApprovalWithdrawn"),

        // Identities (index 15)
        (15, 0) => Some("IdentityRegistered"),
        (15, 1) => Some("IdentityRevoked"),
        (15, 2) => Some("RoleUpdated"),

        // AccessControl (index 16)
        (16, 0) => Some("AccessGranted"),
        (16, 1) => Some("AccessRevoked"),
        (16, 2) => Some("AdminDesignated"),
        (16, 3) => Some("AdminRemoved"),

        // AgentPolicy (index 17)
        (17, 0) => Some("AgentRegistered"),
        (17, 1) => Some("AgentRevoked"),
        (17, 2) => Some("PolicyUpdated"),
        (17, 3) => Some("UsageRecorded"),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_pallet_indices() {
        assert!(!is_legal_pallet(0));
        assert!(!is_legal_pallet(6));
        assert!(is_legal_pallet(10));
        assert!(is_legal_pallet(13));
        assert!(is_legal_pallet(17));
        assert!(!is_legal_pallet(18));
    }

    #[test]
    fn pallet_names_match_runtime() {
        assert_eq!(pallet_name(10), Some("Matters"));
        assert_eq!(pallet_name(13), Some("Audit"));
        assert_eq!(pallet_name(17), Some("AgentPolicy"));
        assert_eq!(pallet_name(99), None);
    }

    #[test]
    fn variant_names_complete() {
        // Spot-check a few
        assert_eq!(decode_variant_name(10, 0), Some("MatterCreated"));
        assert_eq!(decode_variant_name(14, 3), Some("ApprovalWithdrawn"));
        assert_eq!(decode_variant_name(17, 3), Some("UsageRecorded"));
        assert_eq!(decode_variant_name(10, 99), None);
    }
}
