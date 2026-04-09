//! Block subscriber — connects to a Substrate node via WebSocket JSON-RPC,
//! follows finalized blocks, fetches events, and writes them to Postgres.

use crate::{db, decoder};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::ws_client::WsClientBuilder;
use serde_json::Value;
use sqlx::PgPool;

/// Main subscriber loop. Connects to the node and processes finalized blocks.
pub async fn run(rpc_url: &str, pool: &PgPool, start_block: u64) -> anyhow::Result<()> {
    let client = WsClientBuilder::default()
        .build(rpc_url)
        .await?;

    tracing::info!("Connected to node at {}", rpc_url);

    // Get the latest finalized block hash
    let finalized_hash: String = client
        .request("chain_getFinalizedHead", jsonrpsee::core::params::ArrayParams::new())
        .await?;

    // Get the finalized block header to know the current height
    let header: Value = client
        .request("chain_getHeader", vec![Value::String(finalized_hash.clone())])
        .await?;

    let finalized_number = header
        .get("number")
        .and_then(|n| n.as_str())
        .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0);

    tracing::info!(finalized = finalized_number, "Current finalized head");

    // Process historical blocks (catch-up)
    for block_num in start_block..=finalized_number {
        if let Err(e) = process_block(&client, pool, block_num).await {
            tracing::error!(block = block_num, error = %e, "Failed to process block");
            // Continue to next block rather than crashing
        }

        if block_num % 100 == 0 && block_num > start_block {
            tracing::info!(block = block_num, "Catch-up progress");
        }
    }

    tracing::info!("Historical catch-up complete, switching to live subscription");

    // Subscribe to new finalized blocks
    // Note: chain_subscribeFinalizedHeads requires a Subscription, which
    // jsonrpsee handles via subscribe(). For now, we poll every 6 seconds
    // (matching block time) as a simpler alternative.
    let mut last_processed = finalized_number;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

        let new_finalized: String = client
            .request("chain_getFinalizedHead", jsonrpsee::core::params::ArrayParams::new())
            .await?;

        let new_header: Value = client
            .request("chain_getHeader", vec![Value::String(new_finalized)])
            .await?;

        let new_number = new_header
            .get("number")
            .and_then(|n| n.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .unwrap_or(last_processed);

        for block_num in (last_processed + 1)..=new_number {
            if let Err(e) = process_block(&client, pool, block_num).await {
                tracing::error!(block = block_num, error = %e, "Failed to process block");
            }
        }

        last_processed = new_number;
    }
}

/// Process a single block: fetch header, events, and store in Postgres.
async fn process_block(
    client: &impl ClientT,
    pool: &PgPool,
    block_number: u64,
) -> anyhow::Result<()> {
    // Get block hash for this number
    let block_hash: Option<String> = client
        .request(
            "chain_getBlockHash",
            vec![Value::Number(block_number.into())],
        )
        .await?;

    let block_hash = match block_hash {
        Some(h) => h,
        None => {
            tracing::warn!(block = block_number, "Block hash not found");
            return Ok(());
        }
    };

    // Fetch block header
    let header: Value = client
        .request("chain_getHeader", vec![Value::String(block_hash.clone())])
        .await?;

    let parent_hash = header
        .get("parentHash")
        .and_then(|v| v.as_str())
        .unwrap_or("0x00")
        .to_string();

    let state_root = header
        .get("stateRoot")
        .and_then(|v| v.as_str())
        .unwrap_or("0x00")
        .to_string();

    let extrinsics_root = header
        .get("extrinsicsRoot")
        .and_then(|v| v.as_str())
        .unwrap_or("0x00")
        .to_string();

    // Fetch the full block to count extrinsics
    let block_data: Value = client
        .request("chain_getBlock", vec![Value::String(block_hash.clone())])
        .await?;

    let extrinsic_count = block_data
        .pointer("/block/extrinsics")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as i32)
        .unwrap_or(0);

    // Fetch storage events for this block
    // System::Events storage key = twox128("System") ++ twox128("Events")
    let events_key = "0x26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7";

    let events_raw: Option<String> = client
        .request(
            "state_getStorage",
            vec![
                Value::String(events_key.to_string()),
                Value::String(block_hash.clone()),
            ],
        )
        .await?;

    let mut event_count = 0i32;

    if let Some(raw_hex) = events_raw {
        let raw_bytes = hex::decode(raw_hex.trim_start_matches("0x")).unwrap_or_default();

        // The events storage is a Vec<EventRecord> encoded with SCALE.
        // First bytes are the compact-encoded length of the vector.
        // Full decoding requires runtime metadata. For now, we store
        // the raw events and decode pallet/variant indices.
        //
        // A production indexer should use `subxt` with chain metadata
        // for fully typed event decoding.

        // Store raw events blob as a single record for now
        event_count = count_events_in_blob(&raw_bytes);

        // Best-effort decode individual events
        if let Some(events) = extract_event_records(&raw_bytes) {
            for (idx, event_bytes) in events.iter().enumerate() {
                if let Some(decoded) = decoder::decode_event_record(event_bytes) {
                    if decoder::is_legal_pallet(event_bytes[0]) {
                        db::insert_event(
                            pool,
                            block_number,
                            idx as i32,
                            &decoded.pallet,
                            &decoded.variant,
                            &decoded.data,
                        )
                        .await?;
                    }
                }
            }
        }
    }

    // Record the block
    db::insert_indexed_block(
        pool,
        block_number,
        &block_hash,
        &parent_hash,
        &state_root,
        &extrinsics_root,
        event_count,
        extrinsic_count,
        None, // TODO: extract timestamp from Timestamp::set extrinsic
    )
    .await?;

    tracing::debug!(block = block_number, events = event_count, "Indexed block");
    Ok(())
}

/// Count events in a SCALE-encoded Vec<EventRecord> blob.
/// Reads the compact-encoded length prefix.
fn count_events_in_blob(data: &[u8]) -> i32 {
    if data.is_empty() {
        return 0;
    }
    // Compact encoding: first 2 bits determine mode
    let first = data[0];
    match first & 0b11 {
        0b00 => (first >> 2) as i32,
        0b01 if data.len() >= 2 => {
            let val = u16::from_le_bytes([data[0], data[1]]);
            (val >> 2) as i32
        }
        0b10 if data.len() >= 4 => {
            let val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            (val >> 2) as i32
        }
        _ => 0,
    }
}

/// Extract individual event records from the SCALE-encoded blob.
/// Returns None if the format can't be parsed.
///
/// Note: Proper extraction requires knowing the exact byte length of each
/// EventRecord, which depends on runtime metadata. This is a placeholder
/// that will be replaced with metadata-driven decoding.
fn extract_event_records(_data: &[u8]) -> Option<Vec<Vec<u8>>> {
    // TODO: Implement metadata-driven event extraction.
    // For now, return None to skip per-event decoding until
    // we integrate `subxt` or `desub` with chain metadata.
    None
}
