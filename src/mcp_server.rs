//! MCP Server for Music Chore CLI Tool
//! 
//! Provides Model Context Protocol interface for AI agents to interact
//! with the music library management functionality.

use crate::{
    build_library_hierarchy, normalize_track_titles, read_metadata, scan_dir,
    OperationResult,
};
use mcp_sdk::{
    server::Server,
    tools::{Tool, Tools},
    transport::ServerStdioTransport,
    types::{CallToolResponse, ServerCapabilities, ToolResponseContent},
};
use serde_json::{json, Value};
use std::path::PathBuf;

/// Start MCP server with stdio transport
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let transport = ServerStdioTransport::default();
    
    let mut tools = Tools::default();
    
    // Register all tools
    tools.add_tool(ScanDirectoryTool::new());
    tools.add_tool(GetLibraryTreeTool::new());
    tools.add_tool(ReadFileMetadataTool::new());
    tools.add_tool(NormalizeTitlesTool::new());
    tools.add_tool(EmitLibraryMetadataTool::new());
    
    let server = Server::builder(transport)
        .name("music-chore")
        .version(env!("CARGO_PKG_VERSION"))
        .capabilities(ServerCapabilities {
            tools: Some(json!({})),
            experimental: None,
            logging: None,
            prompts: None,
            resources: None,
        })
        .tools(tools);
    
    let server = server.build();
    server.listen().await?;
    
    Ok(())
}

// Tool implementations

struct ScanDirectoryTool;

impl ScanDirectoryTool {
    fn new() -> Self {
        Self
    }

    fn scan_directory(&self, path: &str, json_output: bool) -> Result<Value, String> {
        let path_buf = PathBuf::from(path);
        let tracks = scan_dir(&path_buf);

        if json_output {
            serde_json::to_value(tracks)
                .map_err(|e| format!("Failed to serialize to JSON: {}", e))
        } else {
            let file_paths: Vec<String> = tracks
                .into_iter()
                .map(|track| track.file_path.to_string_lossy().to_string())
                .collect();
            Ok(json!({ "files": file_paths, "count": file_paths.len() }))
        }
    }
}

impl Tool for ScanDirectoryTool {
    fn name(&self) -> String {
        "scan_directory".to_string()
    }

    fn description(&self) -> String {
        "Recursively scan a directory for music files".to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Base directory path to scan for music files"
                },
                "json_output": {
                    "type": "boolean",
                    "description": "Return results as JSON (true) or simple list (false)",
                    "default": false
                }
            },
            "required": ["path"]
        })
    }

    fn call(&self, input: Option<Value>) -> Result<CallToolResponse, anyhow::Error> {
        let input = input.ok_or_else(|| anyhow::anyhow!("Missing input"))?;
        
        let path = input.get("path")
            .and_then(|v: &Value| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        
        let json_output = input.get("json_output")
            .and_then(|v: &Value| v.as_bool())
            .unwrap_or(false);

        match self.scan_directory(path, json_output) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            Err(error) => {
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { 
                        text: format!("Error: {}", error) 
                    }],
                    is_error: Some(true),
                    meta: None,
                })
            }
        }
    }
}

struct GetLibraryTreeTool;

impl GetLibraryTreeTool {
    fn new() -> Self {
        Self
    }

    fn get_library_tree(&self, path: &str, json_output: bool) -> Result<Value, String> {
        let path_buf = PathBuf::from(path);
        let tracks = scan_dir(&path_buf);
        let library = build_library_hierarchy(tracks);

        if json_output {
            serde_json::to_value(library)
                .map_err(|e| format!("Failed to serialize library to JSON: {}", e))
        } else {
            let mut result = Vec::new();
            result.push("=== MUSIC LIBRARY TREE ===".to_string());
            result.push(format!("Total Artists: {}", library.total_artists));
            result.push(format!("Total Albums: {}", library.total_albums));
            result.push(format!("Total Tracks: {}", library.total_tracks));
            result.push(String::new());

            for artist in &library.artists {
                result.push(format!("📁 {}", artist.name));
                for album in &artist.albums {
                    let year_str = album.year.map(|y| format!(" ({})", y)).unwrap_or_default();
                    result.push(format!("├── 📂 {}{}", album.title, year_str));
                    
                    for (i, track) in album.tracks.iter().enumerate() {
                        let is_last = i == album.tracks.len() - 1;
                        let prefix = if is_last { "└─── 🎵" } else { "├─── 🎵" };
                        
                        let filename = track.file_path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        result.push(format!("{}   {}", prefix, filename));
                    }
                }
                result.push(String::new());
            }

            result.push("=== END TREE ===".to_string());
            Ok(json!({ "tree": result.join("\n") }))
        }
    }
}

impl Tool for GetLibraryTreeTool {
    fn name(&self) -> String {
        "get_library_tree".to_string()
    }

    fn description(&self) -> String {
        "Get a hierarchical tree view of the music library".to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Base directory path to analyze"
                },
                "json_output": {
                    "type": "boolean",
                    "description": "Return results as JSON (true) or formatted tree (false)",
                    "default": false
                }
            },
            "required": ["path"]
        })
    }

    fn call(&self, input: Option<Value>) -> Result<CallToolResponse, anyhow::Error> {
        let input = input.ok_or_else(|| anyhow::anyhow!("Missing input"))?;
        
        let path = input.get("path")
            .and_then(|v: &Value| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        
        let json_output = input.get("json_output")
            .and_then(|v: &Value| v.as_bool())
            .unwrap_or(false);

        match self.get_library_tree(path, json_output) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            Err(error) => {
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { 
                        text: format!("Error: {}", error) 
                    }],
                    is_error: Some(true),
                    meta: None,
                })
            }
        }
    }
}

struct ReadFileMetadataTool;

impl ReadFileMetadataTool {
    fn new() -> Self {
        Self
    }

    fn read_file_metadata(&self, file_path: &str) -> Result<Value, String> {
        let path_buf = PathBuf::from(file_path);
        match read_metadata(&path_buf) {
            Ok(track) => serde_json::to_value(track)
                .map_err(|e| format!("Failed to serialize metadata: {}", e)),
            Err(e) => Err(format!("Failed to read metadata: {}", e)),
        }
    }
}

impl Tool for ReadFileMetadataTool {
    fn name(&self) -> String {
        "read_file_metadata".to_string()
    }

    fn description(&self) -> String {
        "Read metadata from a single music file".to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the music file"
                }
            },
            "required": ["file_path"]
        })
    }

    fn call(&self, input: Option<Value>) -> Result<CallToolResponse, anyhow::Error> {
        let input = input.ok_or_else(|| anyhow::anyhow!("Missing input"))?;
        
        let file_path = input.get("file_path")
            .and_then(|v: &Value| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_path"))?;

        match self.read_file_metadata(file_path) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            Err(error) => {
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { 
                        text: format!("Error: {}", error) 
                    }],
                    is_error: Some(true),
                    meta: None,
                })
            }
        }
    }
}

struct NormalizeTitlesTool;

impl NormalizeTitlesTool {
    fn new() -> Self {
        Self
    }

    fn normalize_titles(&self, path: &str, dry_run: bool) -> Result<Value, String> {
        let path_buf = PathBuf::from(path);
        match normalize_track_titles(&path_buf, dry_run) {
            Ok(results) => {
                let mut processed = 0;
                let mut updated = 0;
                let mut errors = 0;

                for result in &results {
                    match result {
                        OperationResult::Updated { .. } => {
                            processed += 1;
                            updated += 1;
                        }
                        OperationResult::NoChange { .. } => {
                            processed += 1;
                        }
                        OperationResult::Error { .. } => {
                            processed += 1;
                            errors += 1;
                        }
                    }
                }

                Ok(json!({
                    "processed": processed,
                    "updated": updated,
                    "errors": errors,
                    "dry_run": dry_run,
                    "details": results
                }))
            }
            Err(e) => Err(format!("Failed to normalize titles: {}", e)),
        }
    }
}

impl Tool for NormalizeTitlesTool {
    fn name(&self) -> String {
        "normalize_titles".to_string()
    }

    fn description(&self) -> String {
        "Normalize track titles to title case".to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path containing music files to normalize"
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "Preview changes without applying them",
                    "default": true
                }
            },
            "required": ["path"]
        })
    }

    fn call(&self, input: Option<Value>) -> Result<CallToolResponse, anyhow::Error> {
        let input = input.ok_or_else(|| anyhow::anyhow!("Missing input"))?;
        
        let path = input.get("path")
            .and_then(|v: &Value| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        
        let dry_run = input.get("dry_run")
            .and_then(|v: &Value| v.as_bool())
            .unwrap_or(true);

        match self.normalize_titles(path, dry_run) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            Err(error) => {
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { 
                        text: format!("Error: {}", error) 
                    }],
                    is_error: Some(true),
                    meta: None,
                })
            }
        }
    }
}

struct EmitLibraryMetadataTool;

impl EmitLibraryMetadataTool {
    fn new() -> Self {
        Self
    }

    fn emit_library_metadata(&self, path: &str, format: &str) -> Result<Value, String> {
        let path_buf = PathBuf::from(path);
        let tracks = scan_dir(&path_buf);
        let library = build_library_hierarchy(tracks);

        match format {
            "json" => serde_json::to_value(library)
                .map_err(|e| format!("Failed to serialize to JSON: {}", e)),
            "text" => {
                let mut output = Vec::new();
                output.push("=== MUSIC LIBRARY METADATA ===".to_string());
                output.push(format!("Total Artists: {}", library.total_artists));
                output.push(format!("Total Albums: {}", library.total_albums));
                output.push(format!("Total Tracks: {}", library.total_tracks));
                output.push(String::new());

                for artist in &library.artists {
                    output.push(format!("ARTIST: {}", artist.name));
                    for album in &artist.albums {
                        let year_str = album.year.map(|y| format!(" ({})", y)).unwrap_or_default();
                        output.push(format!("  ALBUM: {}{}", album.title, year_str));
                        
                        for track in &album.tracks {
                            let title = track
                                .metadata
                                .title
                                .as_ref()
                                .map(|t| t.value.as_str())
                                .unwrap_or("[Unknown Title]");
                            let duration = track
                                .metadata
                                .duration
                                .as_ref()
                                .map(|d| {
                                    let total_seconds = d.value as u64;
                                    let minutes = total_seconds / 60;
                                    let seconds = total_seconds % 60;
                                    format!("{}:{:02}", minutes, seconds)
                                })
                                .unwrap_or_else(|| "0:00".to_string());
                            let file_path = track.file_path.to_string_lossy();

                            output.push(format!(
                                "    TRACK: \"{}\" | Duration: {} | File: {}",
                                title, duration, file_path
                            ));
                        }
                    }
                    output.push(String::new());
                }

                output.push("=== END METADATA ===".to_string());
                Ok(json!({ "metadata": output.join("\n") }))
            }
            _ => Err(format!("Unsupported format: {}", format)),
        }
    }
}

impl Tool for EmitLibraryMetadataTool {
    fn name(&self) -> String {
        "emit_library_metadata".to_string()
    }

    fn description(&self) -> String {
        "Emit complete library metadata in structured format".to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Base directory path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["text", "json"],
                    "description": "Output format: 'text' for AI-friendly structured text, 'json' for JSON",
                    "default": "text"
                }
            },
            "required": ["path"]
        })
    }

    fn call(&self, input: Option<Value>) -> Result<CallToolResponse, anyhow::Error> {
        let input = input.ok_or_else(|| anyhow::anyhow!("Missing input"))?;
        
        let path = input.get("path")
            .and_then(|v: &Value| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        
        let format = input.get("format")
            .and_then(|v: &Value| v.as_str())
            .unwrap_or("text");

        match self.emit_library_metadata(path, format) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { text }],
                    is_error: Some(false),
                    meta: None,
                })
            }
            Err(error) => {
                Ok(CallToolResponse {
                    content: vec![ToolResponseContent::Text { 
                        text: format!("Error: {}", error) 
                    }],
                    is_error: Some(true),
                    meta: None,
                })
            }
        }
    }
}