//! MCP Server binary entry point for Music Chore

use clap::Parser;
use music_chore::mcp::{config::Config, music_chore_server::MusicChoreServer};

use rmcp::{ServiceExt, transport::stdio};

#[derive(Parser)]
#[command(name = "musicctl-mcp")]
#[command(about = "MCP server for Music Chore CLI tool")]
#[command(long_about = "MCP server for Music Chore CLI tool

Environment Variables:
  RUST_LOG                Logging level (error, warn, info, debug, trace) [default: info]
  MUSIC_LIBRARY_PATH       Default music library path
  MUSIC_SCAN_TIMEOUT       Scan timeout in seconds [default: 300]
  MUSIC_ALLOWED_PATHS      Comma-separated list of allowed paths for security

Examples:
  export RUST_LOG=debug
  export MUSIC_LIBRARY_PATH=/Users/username/Music
  export MUSIC_ALLOWED_PATHS=/Users/username/Music,/Volumes/Music
  musicctl-mcp")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Enable verbose logging (overrides RUST_LOG environment variable)
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Load configuration from environment variables
    let mut config = Config::from_env();
    
    // Override log level if verbose flag is set
    if cli.verbose {
        config.log_level = "debug".to_string();
    }

    // Initialize logging
    config.init_logging();

    // Start the MCP server with configuration
    start_with_config(config).await
}

/// Start MCP server with stdio transport and configuration
pub async fn start_with_config(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let server = MusicChoreServer::new_with_config(config);

    // Run the server with stdio transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        eprintln!("Error starting server: {}", e);
    })?;
    service.waiting().await?;

    Ok(())
}
