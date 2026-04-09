//! Proof generation and verification utilities.
//!
//! Generates cryptographic proof bundles that attest to on-chain state
//! at a specific finalized block height.

use blake2::{Blake2b, Digest};
use blake2::digest::consts::U32;

/// Blake2b-256 type alias
type Blake2b256 = Blake2b<U32>;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// A proof bundle suitable for legal discovery or compliance submission.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProofBundle {
    /// Chain identifier
    pub chain: String,
    /// Block number the proof references
    pub block_number: u64,
    /// Block hash (finalized)
    pub block_hash: String,
    /// State root from the block header
    pub state_root: String,
    /// Storage keys included in the proof
    pub storage_keys: Vec<String>,
    /// Storage values (hex-encoded SCALE)
    pub storage_values: Vec<Option<String>>,
    /// Merkle trie proof nodes (hex-encoded)
    pub proof_nodes: Vec<String>,
    /// Blake2b-256 hash of the proof payload (for integrity verification)
    pub integrity_hash: String,
    /// ISO 8601 timestamp when the proof was generated
    pub generated_at: String,
    /// Proof service version
    pub version: String,
}

/// Compute a Blake2b-256 integrity hash over the core proof data.
pub fn compute_integrity_hash(
    block_hash: &str,
    state_root: &str,
    storage_keys: &[String],
    storage_values: &[Option<String>],
    proof_nodes: &[String],
) -> String {
    let mut hasher = Blake2b256::new();

    hasher.update(block_hash.as_bytes());
    hasher.update(state_root.as_bytes());

    for key in storage_keys {
        hasher.update(key.as_bytes());
    }
    for val in storage_values {
        match val {
            Some(v) => hasher.update(v.as_bytes()),
            None => hasher.update(b"null"),
        }
    }
    for node in proof_nodes {
        hasher.update(node.as_bytes());
    }

    let result = hasher.finalize();
    hex::encode(result)
}

/// Build a complete proof bundle from on-chain data.
pub fn build_proof_bundle(
    block_number: u64,
    block_hash: String,
    state_root: String,
    storage_keys: Vec<String>,
    storage_values: Vec<Option<String>>,
    proof_nodes: Vec<String>,
) -> ProofBundle {
    let integrity_hash = compute_integrity_hash(
        &block_hash,
        &state_root,
        &storage_keys,
        &storage_values,
        &proof_nodes,
    );

    ProofBundle {
        chain: "legal-chain".to_string(),
        block_number,
        block_hash,
        state_root,
        storage_keys,
        storage_values,
        proof_nodes,
        integrity_hash,
        generated_at: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Verify a proof bundle's integrity hash.
pub fn verify_integrity(bundle: &ProofBundle) -> bool {
    let expected = compute_integrity_hash(
        &bundle.block_hash,
        &bundle.state_root,
        &bundle.storage_keys,
        &bundle.storage_values,
        &bundle.proof_nodes,
    );
    expected == bundle.integrity_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_bundle_roundtrip() {
        let bundle = build_proof_bundle(
            42,
            "0xabc123".into(),
            "0xdef456".into(),
            vec!["0xkey1".into()],
            vec![Some("0xval1".into())],
            vec!["0xnode1".into(), "0xnode2".into()],
        );

        assert!(verify_integrity(&bundle));
        assert_eq!(bundle.chain, "legal-chain");
        assert_eq!(bundle.block_number, 42);
    }

    #[test]
    fn tampered_bundle_fails_verification() {
        let mut bundle = build_proof_bundle(
            1,
            "0xaaa".into(),
            "0xbbb".into(),
            vec!["0xk".into()],
            vec![Some("0xv".into())],
            vec![],
        );

        bundle.block_hash = "0xtampered".into();
        assert!(!verify_integrity(&bundle));
    }
}
