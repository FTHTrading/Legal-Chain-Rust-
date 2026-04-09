//! HTTP handlers for the Proof Service.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::{proofs, rpc};

/// Application state.
#[derive(Clone)]
pub struct AppState {
    pub rpc: rpc::RpcClient,
}

/// Build the Axum router.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/proof", get(generate_proof))
        .route("/v1/verify", axum::routing::post(verify_proof))
        .route("/v1/state", get(read_state))
        .route("/v1/finalized", get(finalized_head))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn health() -> &'static str {
    "ok"
}

// ─── Request / Response Types ──────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ProofRequest {
    /// Comma-separated storage keys (hex-encoded)
    pub keys: String,
    /// Block number (uses finalized head if omitted)
    pub block: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct StateRequest {
    /// Storage key (hex-encoded)
    pub key: String,
    /// Block hash (uses finalized head if omitted)
    pub at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StateResponse {
    pub key: String,
    pub value: Option<String>,
    pub at_block: String,
}

#[derive(Debug, Serialize)]
pub struct FinalizedResponse {
    pub hash: String,
    pub number: Option<u64>,
}

// ─── Handlers ──────────────────────────────────────────────────────

/// Generate a Merkle proof bundle for the given storage keys.
pub async fn generate_proof(
    State(state): State<AppState>,
    Query(req): Query<ProofRequest>,
) -> Result<Json<proofs::ProofBundle>, StatusCode> {
    let keys: Vec<String> = req.keys.split(',').map(|s| s.trim().to_string()).collect();

    if keys.is_empty() || keys.iter().any(|k| k.is_empty()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Determine block
    let (block_hash, block_number) = match req.block {
        Some(num) => {
            let hash = rpc::get_block_hash(&state.rpc, num)
                .await
                .map_err(|_| StatusCode::BAD_GATEWAY)?
                .ok_or(StatusCode::NOT_FOUND)?;
            (hash, num)
        }
        None => {
            let hash = rpc::get_finalized_head(&state.rpc)
                .await
                .map_err(|_| StatusCode::BAD_GATEWAY)?;
            let header = rpc::get_header(&state.rpc, &hash)
                .await
                .map_err(|_| StatusCode::BAD_GATEWAY)?;
            let num = header
                .get("number")
                .and_then(|n| n.as_str())
                .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                .unwrap_or(0);
            (hash, num)
        }
    };

    // Get block header for state root
    let header = rpc::get_header(&state.rpc, &block_hash)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let state_root = header
        .get("stateRoot")
        .and_then(|v| v.as_str())
        .unwrap_or("0x00")
        .to_string();

    // Fetch storage values
    let mut storage_values = Vec::new();
    for key in &keys {
        let val = rpc::get_storage(&state.rpc, key, Some(&block_hash))
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;
        storage_values.push(val);
    }

    // Get Merkle proof
    let read_proof = rpc::get_read_proof(&state.rpc, &keys, Some(&block_hash))
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let proof_nodes: Vec<String> = read_proof
        .get("proof")
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let bundle = proofs::build_proof_bundle(
        block_number,
        block_hash,
        state_root,
        keys,
        storage_values,
        proof_nodes,
    );

    Ok(Json(bundle))
}

/// Verify a previously generated proof bundle.
pub async fn verify_proof(
    Json(bundle): Json<proofs::ProofBundle>,
) -> Result<Json<VerifyResponse>, StatusCode> {
    let valid = proofs::verify_integrity(&bundle);
    Ok(Json(VerifyResponse {
        valid,
        integrity_hash: bundle.integrity_hash,
    }))
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub integrity_hash: String,
}

/// Read a single storage value.
pub async fn read_state(
    State(state): State<AppState>,
    Query(req): Query<StateRequest>,
) -> Result<Json<StateResponse>, StatusCode> {
    let at_block = match &req.at {
        Some(hash) => hash.clone(),
        None => rpc::get_finalized_head(&state.rpc)
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?,
    };

    let value = rpc::get_storage(&state.rpc, &req.key, Some(&at_block))
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(StateResponse {
        key: req.key,
        value,
        at_block,
    }))
}

/// Get the current finalized head.
pub async fn finalized_head(
    State(state): State<AppState>,
) -> Result<Json<FinalizedResponse>, StatusCode> {
    let hash = rpc::get_finalized_head(&state.rpc)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let header = rpc::get_header(&state.rpc, &hash)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let number = header
        .get("number")
        .and_then(|n| n.as_str())
        .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok());

    Ok(Json(FinalizedResponse { hash, number }))
}
