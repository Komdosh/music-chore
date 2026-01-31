//! MCP Server binary entry point for Music Chore

use clap::Parser;
use music_chore::mcp_server::start;

#[derive(Parser)]
#[command(name = "musicctl-mcp")]
#[command(about = "MCP server for Music Chore CLI tool")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    log::info!("Starting Music Chore MCP server v{}", env!("CARGO_PKG_VERSION"));

    // Start the MCP server
    start().await
}