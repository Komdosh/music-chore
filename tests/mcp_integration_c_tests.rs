use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParams,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_musicchore_mcp_server() -> Result<()> {
    // Enable logging in tests (RUST_LOG=debug cargo test)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Spawn your MCP server via cargo
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--bin", "musicctl-mcp", "--"]);

    // Important: configure for MCP stdio mode
    let child = TokioChildProcess::new(cmd)?;

    let client = ().serve(child).await?;

    println!("{:?}", client.list_all_tools().await?);

    // Check server info
    let info = client.peer_info();
    tracing::info!("Server info: {info:#?}");

    // List tools
    let tools = client.list_all_tools().await?;
    tracing::info!("Tools: {tools:#?}");

    assert!(
        tools.iter().any(|t| t.name == "scan_directory"),
        "scan_directory tool missing"
    );

    // Call scan_directory on a test dir
    let result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "scan_directory".into(),
            arguments: Some(object!({
                "path": "./tests/fixtures/music",
                "json_output": false
            })),
            task: None,
        })
        .await?;

    tracing::info!("scan_directory result: {result:#?}");

    assert_eq!(result.is_error.unwrap_or(false), false, "scan_directory failed");

    // Call get_library_tree
    let tree = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "get_library_tree".into(),
            arguments: Some(object!({
                "path": "./tests/fixtures/flac"
            })),
            task: None,
        })
        .await?;

    tracing::info!("library tree: {tree:#?}");
    assert_eq!(tree.is_error.unwrap_or(false), false, "get_library_tree failed");

    // Gracefully shut down
    client.cancel().await?;

    Ok(())
}
