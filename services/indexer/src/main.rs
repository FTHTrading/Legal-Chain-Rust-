//! # Legal-Chain Indexer
//!
//! Subscribes to finalized blocks via Substrate JSON-RPC (WebSocket),
//! decodes runtime events, and writes structured records to Postgres.
//!
//! ## Usage
//! ```
//! legal-chain-indexer \
//!   --rpc-url ws://127.0.0.1:9944 \
//!   --database-url postgres://user:pass@localhost/legal_chain
//! ```

mod db;
mod decoder;
mod subscriber;

use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "legal-chain-indexer", about = "Off-chain block & event indexer for Legal-Chain")]
struct Cli {
    /// Substrate node WebSocket RPC URL
    #[arg(long, env = "RPC_URL", default_value = "ws://127.0.0.1:9944")]
    rpc_url: String,

    /// Postgres connection URL
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    /// Start indexing from this block number (0 = genesis)
    #[arg(long, default_value_t = 0)]
    start_block: u64,

    /// Maximum parallel block processing tasks
    #[arg(long, default_value_t = 4)]
    concurrency: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    let cli = Cli::parse();

    tracing::info!(rpc = %cli.rpc_url, "Starting Legal-Chain Indexer");

    // Connect to Postgres and run migrations
    let pool = db::connect(&cli.database_url).await?;
    db::migrate(&pool).await?;

    tracing::info!("Database connected and migrations applied");

    // Determine resume point
    let last_indexed = db::get_last_indexed_block(&pool).await?;
    let start = if cli.start_block > 0 {
        cli.start_block
    } else {
        last_indexed.map(|b| b + 1).unwrap_or(0)
    };

    tracing::info!(start_block = start, "Resuming from block");

    // Subscribe to finalized blocks and process events
    subscriber::run(&cli.rpc_url, &pool, start).await?;

    Ok(())
}
