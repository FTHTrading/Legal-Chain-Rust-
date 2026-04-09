//! # Legal-Chain Proof Service
//!
//! Verifies on-chain state integrity and produces cryptographic proof bundles
//! for legal discovery, compliance audits, and court submissions.
//!
//! ## Usage
//! ```
//! legal-chain-proof-service \
//!   --rpc-url ws://127.0.0.1:9944 \
//!   --port 8400
//! ```

mod proofs;
mod rpc;
mod handlers;

use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "legal-chain-proof-service", about = "Merkle proof & integrity verification for Legal-Chain")]
struct Cli {
    /// Substrate node WebSocket RPC URL
    #[arg(long, env = "RPC_URL", default_value = "ws://127.0.0.1:9944")]
    rpc_url: String,

    /// HTTP listen port
    #[arg(long, env = "PORT", default_value_t = 8400)]
    port: u16,

    /// Bind address
    #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0")]
    bind: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    let cli = Cli::parse();

    tracing::info!(port = cli.port, rpc = %cli.rpc_url, "Starting Legal-Chain Proof Service");

    let rpc_client = rpc::connect(&cli.rpc_url).await?;

    let state = handlers::AppState {
        rpc: rpc_client,
    };

    let app = handlers::build_router(state);

    let addr = format!("{}:{}", cli.bind, cli.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "Listening");

    axum::serve(listener, app).await?;

    Ok(())
}
