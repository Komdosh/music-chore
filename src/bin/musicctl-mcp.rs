//! MCP Server binary entry point for Music Chore

use clap::Parser;
use music_chore::mcp::music_chore_server::MusicChoreServer;

use rmcp::{ServiceExt, transport::stdio};

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

    // log::info!("Starting Music Chore MCP server v{}", env!("CARGO_PKG_VERSION"));

    // Start the MCP server
    start().await
}

/// Start MCP server with stdio transport
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let server = MusicChoreServer::new();

    // Run the server with stdio transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        eprintln!("Error starting server: {}", e);
    })?;
    service.waiting().await?;

    Ok(())
}
