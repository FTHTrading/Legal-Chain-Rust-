//! Axum route definitions for the Explorer API.

use axum::{routing::get, Router};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

/// Build the full Axum router with all endpoints.
pub fn build(pool: PgPool) -> Router {
    let state = AppState { pool };

    Router::new()
        // Health
        .route("/health", get(handlers::health))
        // Blocks
        .route("/v1/blocks", get(handlers::list_blocks))
        .route("/v1/blocks/:number", get(handlers::get_block))
        // Events
        .route("/v1/events", get(handlers::list_events))
        // Matters
        .route("/v1/matters", get(handlers::list_matters))
        .route("/v1/matters/:id", get(handlers::get_matter))
        // Evidence
        .route("/v1/evidence", get(handlers::list_evidence))
        .route("/v1/evidence/:id", get(handlers::get_evidence_by_id))
        // Documents
        .route("/v1/documents", get(handlers::list_documents))
        .route("/v1/documents/:id", get(handlers::get_document))
        // Approvals
        .route("/v1/approvals", get(handlers::list_approvals))
        .route("/v1/approvals/:id", get(handlers::get_approval))
        // Identities
        .route("/v1/identities", get(handlers::list_identities))
        .route("/v1/identities/:id", get(handlers::get_identity))
        // Audit trail
        .route("/v1/audit", get(handlers::list_audit))
        // Stats
        .route("/v1/stats", get(handlers::chain_stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
