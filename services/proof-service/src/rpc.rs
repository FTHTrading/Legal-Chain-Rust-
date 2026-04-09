//! Substrate JSON-RPC client for state queries and proof retrieval.

use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use jsonrpsee::core::client::ClientT;
use serde_json::Value;
use std::sync::Arc;

/// Shared RPC client handle.
pub type RpcClient = Arc<WsClient>;

/// Connect to the Substrate node.
pub async fn connect(rpc_url: &str) -> anyhow::Result<RpcClient> {
    let client = WsClientBuilder::default()
        .build(rpc_url)
        .await?;
    Ok(Arc::new(client))
}

/// Read a storage value with optional block hash.
pub async fn get_storage(
    client: &WsClient,
    key: &str,
    at_block: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let mut params = vec![Value::String(key.to_string())];
    if let Some(hash) = at_block {
        params.push(Value::String(hash.to_string()));
    }

    let result: Option<String> = client
        .request("state_getStorage", params)
        .await?;

    Ok(result)
}

/// Retrieve a Merkle storage proof from the node.
/// Uses `state_getReadProof` RPC which returns trie proof nodes.
pub async fn get_read_proof(
    client: &WsClient,
    keys: &[String],
    at_block: Option<&str>,
) -> anyhow::Result<Value> {
    let keys_json: Vec<Value> = keys.iter().map(|k| Value::String(k.clone())).collect();
    let mut params = vec![Value::Array(keys_json)];
    if let Some(hash) = at_block {
        params.push(Value::String(hash.to_string()));
    }

    let result: Value = client
        .request("state_getReadProof", params)
        .await?;

    Ok(result)
}

/// Get block header by hash.
pub async fn get_header(
    client: &WsClient,
    block_hash: &str,
) -> anyhow::Result<Value> {
    let result: Value = client
        .request("chain_getHeader", vec![Value::String(block_hash.to_string())])
        .await?;
    Ok(result)
}

/// Get the finalized head hash.
pub async fn get_finalized_head(
    client: &WsClient,
) -> anyhow::Result<String> {
    let result: String = client
        .request("chain_getFinalizedHead", jsonrpsee::core::params::ArrayParams::new())
        .await?;
    Ok(result)
}

/// Get block hash for a given block number.
pub async fn get_block_hash(
    client: &WsClient,
    number: u64,
) -> anyhow::Result<Option<String>> {
    let result: Option<String> = client
        .request("chain_getBlockHash", vec![Value::Number(number.into())])
        .await?;
    Ok(result)
}
