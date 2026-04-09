//! LEGAL-CHAIN Node
//!
//! Substrate node binary for the legal-chain sovereign blockchain.
//! Runs Aura block authoring with GRANDPA finality.

#![warn(missing_docs)]

mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

fn main() -> sc_cli::Result<()> {
    command::run()
}
