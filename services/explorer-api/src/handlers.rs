//! Request handlers for the Explorer API.
//!
//! Each handler queries Postgres (via SQLx) and returns JSON responses.
//! Pagination uses `?limit=N&offset=M` query parameters.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::routes::AppState;

// ─── Query Parameters ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct EventFilter {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub pallet: Option<String>,
    pub variant: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MatterFilter {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub status: Option<String>,
    pub creator: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuditFilter {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub matter_id: Option<i64>,
    pub actor: Option<String>,
    pub action: Option<String>,
}

// ─── Response Types ────────────────────────────────────────────────

#[derive(Debug, Serialize, FromRow)]
pub struct BlockRow {
    pub block_number: i64,
    pub block_hash: String,
    pub parent_hash: String,
    pub state_root: String,
    pub extrinsics_root: String,
    pub event_count: i32,
    pub extrinsic_count: i32,
    pub block_timestamp: Option<chrono::NaiveDateTime>,
    pub indexed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EventRow {
    pub id: i64,
    pub block_number: i64,
    pub event_index: i32,
    pub pallet: String,
    pub variant: String,
    pub data: serde_json::Value,
    pub indexed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MatterRow {
    pub matter_id: i64,
    pub creator: String,
    pub matter_type: String,
    pub status: String,
    pub jurisdiction_hash: String,
    pub created_block: i64,
    pub updated_block: i64,
    pub indexed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EvidenceRow {
    pub evidence_id: i64,
    pub matter_id: i64,
    pub registrar: String,
    pub content_hash: String,
    pub status: String,
    pub custody_state: String,
    pub created_block: i64,
    pub updated_block: i64,
    pub indexed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DocumentRow {
    pub document_id: i64,
    pub matter_id: i64,
    pub registrar: String,
    pub content_hash: String,
    pub version: i32,
    pub status: String,
    pub filing_readiness: String,
    pub created_block: i64,
    pub updated_block: i64,
    pub indexed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ApprovalRow {
    pub approval_id: i64,
    pub matter_id: i64,
    pub subject_type: String,
    pub subject_id: i64,
    pub requester: String,
    pub status: String,
    pub created_block: i64,
    pub updated_block: i64,
    pub indexed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct IdentityRow {
    pub credential_id: i64,
    pub subject: String,
    pub role: String,
    pub registered_by: String,
    pub is_active: bool,
    pub created_block: i64,
    pub updated_block: i64,
    pub indexed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AuditRow {
    pub audit_id: i64,
    pub matter_id: Option<i64>,
    pub actor: String,
    pub action: String,
    pub target_type: String,
    pub target_id: i64,
    pub block_number: i64,
    pub indexed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct ChainStats {
    pub total_blocks: i64,
    pub total_events: i64,
    pub total_matters: i64,
    pub total_evidence: i64,
    pub total_documents: i64,
    pub total_approvals: i64,
    pub total_identities: i64,
    pub total_audit_entries: i64,
    pub latest_block: Option<i64>,
}

// ─── Handlers ──────────────────────────────────────────────────────

pub async fn health() -> &'static str {
    "ok"
}

pub async fn list_blocks(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> Result<Json<Vec<BlockRow>>, StatusCode> {
    let rows = sqlx::query_as::<_, BlockRow>(
        "SELECT * FROM indexed_blocks ORDER BY block_number DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit.min(1000))
    .bind(p.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn get_block(
    State(state): State<AppState>,
    Path(number): Path<i64>,
) -> Result<Json<BlockRow>, StatusCode> {
    let row = sqlx::query_as::<_, BlockRow>(
        "SELECT * FROM indexed_blocks WHERE block_number = $1",
    )
    .bind(number)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    row.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn list_events(
    State(state): State<AppState>,
    Query(f): Query<EventFilter>,
) -> Result<Json<Vec<EventRow>>, StatusCode> {
    let rows = sqlx::query_as::<_, EventRow>(
        r#"
        SELECT * FROM chain_events
        WHERE ($1::TEXT IS NULL OR pallet = $1)
          AND ($2::TEXT IS NULL OR variant = $2)
        ORDER BY block_number DESC, event_index ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&f.pallet)
    .bind(&f.variant)
    .bind(f.limit.min(1000))
    .bind(f.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn list_matters(
    State(state): State<AppState>,
    Query(f): Query<MatterFilter>,
) -> Result<Json<Vec<MatterRow>>, StatusCode> {
    let rows = sqlx::query_as::<_, MatterRow>(
        r#"
        SELECT * FROM matters
        WHERE ($1::TEXT IS NULL OR status = $1)
          AND ($2::TEXT IS NULL OR creator = $2)
        ORDER BY matter_id DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&f.status)
    .bind(&f.creator)
    .bind(f.limit.min(1000))
    .bind(f.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn get_matter(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<MatterRow>, StatusCode> {
    let row = sqlx::query_as::<_, MatterRow>(
        "SELECT * FROM matters WHERE matter_id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    row.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn list_evidence(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> Result<Json<Vec<EvidenceRow>>, StatusCode> {
    let rows = sqlx::query_as::<_, EvidenceRow>(
        "SELECT * FROM evidence ORDER BY evidence_id DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit.min(1000))
    .bind(p.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn get_evidence_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<EvidenceRow>, StatusCode> {
    let row = sqlx::query_as::<_, EvidenceRow>(
        "SELECT * FROM evidence WHERE evidence_id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    row.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn list_documents(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> Result<Json<Vec<DocumentRow>>, StatusCode> {
    let rows = sqlx::query_as::<_, DocumentRow>(
        "SELECT * FROM documents ORDER BY document_id DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit.min(1000))
    .bind(p.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<DocumentRow>, StatusCode> {
    let row = sqlx::query_as::<_, DocumentRow>(
        "SELECT * FROM documents WHERE document_id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    row.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn list_approvals(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> Result<Json<Vec<ApprovalRow>>, StatusCode> {
    let rows = sqlx::query_as::<_, ApprovalRow>(
        "SELECT * FROM approvals ORDER BY approval_id DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit.min(1000))
    .bind(p.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn get_approval(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApprovalRow>, StatusCode> {
    let row = sqlx::query_as::<_, ApprovalRow>(
        "SELECT * FROM approvals WHERE approval_id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    row.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn list_identities(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> Result<Json<Vec<IdentityRow>>, StatusCode> {
    let rows = sqlx::query_as::<_, IdentityRow>(
        "SELECT * FROM identities ORDER BY credential_id DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit.min(1000))
    .bind(p.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn get_identity(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<IdentityRow>, StatusCode> {
    let row = sqlx::query_as::<_, IdentityRow>(
        "SELECT * FROM identities WHERE credential_id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    row.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn list_audit(
    State(state): State<AppState>,
    Query(f): Query<AuditFilter>,
) -> Result<Json<Vec<AuditRow>>, StatusCode> {
    let rows = sqlx::query_as::<_, AuditRow>(
        r#"
        SELECT * FROM audit_trail
        WHERE ($1::BIGINT IS NULL OR matter_id = $1)
          AND ($2::TEXT IS NULL OR actor = $2)
          AND ($3::TEXT IS NULL OR action = $3)
        ORDER BY audit_id DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(&f.matter_id)
    .bind(&f.actor)
    .bind(&f.action)
    .bind(f.limit.min(1000))
    .bind(f.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn chain_stats(
    State(state): State<AppState>,
) -> Result<Json<ChainStats>, StatusCode> {
    let counts = sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64, i64, i64, Option<i64>)>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM indexed_blocks),
            (SELECT COUNT(*) FROM chain_events),
            (SELECT COUNT(*) FROM matters),
            (SELECT COUNT(*) FROM evidence),
            (SELECT COUNT(*) FROM documents),
            (SELECT COUNT(*) FROM approvals),
            (SELECT COUNT(*) FROM identities),
            (SELECT COUNT(*) FROM audit_trail),
            (SELECT MAX(block_number) FROM indexed_blocks)
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ChainStats {
        total_blocks: counts.0,
        total_events: counts.1,
        total_matters: counts.2,
        total_evidence: counts.3,
        total_documents: counts.4,
        total_approvals: counts.5,
        total_identities: counts.6,
        total_audit_entries: counts.7,
        latest_block: counts.8,
    }))
}
