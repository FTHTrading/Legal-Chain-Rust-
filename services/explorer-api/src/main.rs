//! # Legal-Chain Explorer API
//!
//! REST API for querying indexed legal-chain data from Postgres.
//! Provides endpoints for matters, evidence, documents, approvals,
//! identities, audit trail, and block/event queries.
//!
//! ## Usage
//! ```
//! legal-chain-explorer-api \
//!   --database-url postgres://user:pass@localhost/legal_chain \
//!   --port 8300
//! ```

mod db;
mod handlers;
mod routes;

use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "legal-chain-explorer-api", about = "REST API for Legal-Chain indexed data")]
struct Cli {
    /// Postgres connection URL
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    /// HTTP listen port
    #[arg(long, env = "PORT", default_value_t = 8300)]
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

    tracing::info!(port = cli.port, "Starting Legal-Chain Explorer API");

    let pool = db::connect(&cli.database_url).await?;

    let app = routes::build(pool);

    let addr = format!("{}:{}", cli.bind, cli.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "Listening");

    axum::serve(listener, app).await?;

    Ok(())
}
