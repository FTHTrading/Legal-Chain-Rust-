//! Database connection, migrations, and write operations for the indexer.

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;

/// Connect to Postgres with a connection pool.
pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Run embedded SQL migrations.
pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

/// Get the last indexed block number (for resume logic).
pub async fn get_last_indexed_block(pool: &PgPool) -> anyhow::Result<Option<u64>> {
    let row = sqlx::query("SELECT MAX(block_number) as max_block FROM indexed_blocks")
        .fetch_optional(pool)
        .await?;

    match row {
        Some(r) => {
            let val: Option<i64> = r.try_get("max_block").unwrap_or(None);
            Ok(val.map(|v| v as u64))
        }
        None => Ok(None),
    }
}

/// Record a block as indexed.
pub async fn insert_indexed_block(
    pool: &PgPool,
    block_number: u64,
    block_hash: &str,
    parent_hash: &str,
    state_root: &str,
    extrinsics_root: &str,
    event_count: i32,
    extrinsic_count: i32,
    timestamp: Option<chrono::NaiveDateTime>,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO indexed_blocks
            (block_number, block_hash, parent_hash, state_root, extrinsics_root,
             event_count, extrinsic_count, block_timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (block_number) DO NOTHING
        "#,
    )
    .bind(block_number as i64)
    .bind(block_hash)
    .bind(parent_hash)
    .bind(state_root)
    .bind(extrinsics_root)
    .bind(event_count)
    .bind(extrinsic_count)
    .bind(timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

/// Insert a decoded event into the events table.
pub async fn insert_event(
    pool: &PgPool,
    block_number: u64,
    event_index: i32,
    pallet: &str,
    variant: &str,
    data_json: &serde_json::Value,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO chain_events
            (block_number, event_index, pallet, variant, data)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (block_number, event_index) DO NOTHING
        "#,
    )
    .bind(block_number as i64)
    .bind(event_index)
    .bind(pallet)
    .bind(variant)
    .bind(data_json)
    .execute(pool)
    .await?;
    Ok(())
}

/// Upsert a matter record from a decoded event.
pub async fn upsert_matter(
    pool: &PgPool,
    matter_id: u64,
    creator: &str,
    matter_type: &str,
    status: &str,
    jurisdiction_hash: &str,
    block_number: u64,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO matters
            (matter_id, creator, matter_type, status, jurisdiction_hash, created_block, updated_block)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        ON CONFLICT (matter_id) DO UPDATE SET
            status = EXCLUDED.status,
            updated_block = EXCLUDED.updated_block
        "#,
    )
    .bind(matter_id as i64)
    .bind(creator)
    .bind(matter_type)
    .bind(status)
    .bind(jurisdiction_hash)
    .bind(block_number as i64)
    .execute(pool)
    .await?;
    Ok(())
}

/// Upsert an evidence record.
pub async fn upsert_evidence(
    pool: &PgPool,
    evidence_id: u64,
    matter_id: u64,
    registrar: &str,
    content_hash: &str,
    status: &str,
    custody_state: &str,
    block_number: u64,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO evidence
            (evidence_id, matter_id, registrar, content_hash, status, custody_state,
             created_block, updated_block)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
        ON CONFLICT (evidence_id) DO UPDATE SET
            status = EXCLUDED.status,
            custody_state = EXCLUDED.custody_state,
            updated_block = EXCLUDED.updated_block
        "#,
    )
    .bind(evidence_id as i64)
    .bind(matter_id as i64)
    .bind(registrar)
    .bind(content_hash)
    .bind(status)
    .bind(custody_state)
    .bind(block_number as i64)
    .execute(pool)
    .await?;
    Ok(())
}

/// Upsert a document record.
pub async fn upsert_document(
    pool: &PgPool,
    document_id: u64,
    matter_id: u64,
    registrar: &str,
    content_hash: &str,
    version: i32,
    status: &str,
    filing_readiness: &str,
    block_number: u64,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO documents
            (document_id, matter_id, registrar, content_hash, version,
             status, filing_readiness, created_block, updated_block)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
        ON CONFLICT (document_id) DO UPDATE SET
            content_hash = EXCLUDED.content_hash,
            version = EXCLUDED.version,
            status = EXCLUDED.status,
            filing_readiness = EXCLUDED.filing_readiness,
            updated_block = EXCLUDED.updated_block
        "#,
    )
    .bind(document_id as i64)
    .bind(matter_id as i64)
    .bind(registrar)
    .bind(content_hash)
    .bind(version)
    .bind(status)
    .bind(filing_readiness)
    .bind(block_number as i64)
    .execute(pool)
    .await?;
    Ok(())
}

/// Insert an audit trail entry.
pub async fn insert_audit_entry(
    pool: &PgPool,
    audit_id: u64,
    matter_id: Option<u64>,
    actor: &str,
    action: &str,
    target_type: &str,
    target_id: u64,
    block_number: u64,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO audit_trail
            (audit_id, matter_id, actor, action, target_type, target_id, block_number)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (audit_id) DO NOTHING
        "#,
    )
    .bind(audit_id as i64)
    .bind(matter_id.map(|m| m as i64))
    .bind(actor)
    .bind(action)
    .bind(target_type)
    .bind(target_id as i64)
    .bind(block_number as i64)
    .execute(pool)
    .await?;
    Ok(())
}

/// Upsert an approval record.
pub async fn upsert_approval(
    pool: &PgPool,
    approval_id: u64,
    matter_id: u64,
    subject_type: &str,
    subject_id: u64,
    requester: &str,
    status: &str,
    block_number: u64,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO approvals
            (approval_id, matter_id, subject_type, subject_id, requester,
             status, created_block, updated_block)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
        ON CONFLICT (approval_id) DO UPDATE SET
            status = EXCLUDED.status,
            updated_block = EXCLUDED.updated_block
        "#,
    )
    .bind(approval_id as i64)
    .bind(matter_id as i64)
    .bind(subject_type)
    .bind(subject_id as i64)
    .bind(requester)
    .bind(status)
    .bind(block_number as i64)
    .execute(pool)
    .await?;
    Ok(())
}

/// Upsert an identity record.
pub async fn upsert_identity(
    pool: &PgPool,
    credential_id: u64,
    subject: &str,
    role: &str,
    registered_by: &str,
    is_active: bool,
    block_number: u64,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO identities
            (credential_id, subject, role, registered_by, is_active,
             created_block, updated_block)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        ON CONFLICT (credential_id) DO UPDATE SET
            role = EXCLUDED.role,
            is_active = EXCLUDED.is_active,
            updated_block = EXCLUDED.updated_block
        "#,
    )
    .bind(credential_id as i64)
    .bind(subject)
    .bind(role)
    .bind(registered_by)
    .bind(is_active)
    .bind(block_number as i64)
    .execute(pool)
    .await?;
    Ok(())
}
