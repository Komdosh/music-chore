use crate::mcp::config::Config;

use rmcp::handler::server::router::prompt::PromptRouter;
use rmcp::{handler::server::{tool::ToolRouter, ServerHandler}, model::{
    GetPromptRequestParams, GetPromptResult,
    Implementation, ListPromptsResult, PaginatedRequestParams
    , ProtocolVersion, ServerCapabilities, ServerInfo,
}, prompt_handler, service::RequestContext, tool_handler, RoleServer};

#[derive(Clone)]
pub struct MusicChoreServer {
    pub(crate) tool_router: ToolRouter<Self>,
    pub(crate) prompt_router: PromptRouter<Self>,
    pub(crate) config: Config,
}

impl Default for MusicChoreServer {
    fn default() -> Self {
        Self::new_with_config(Config::default())
    }
}

#[tool_handler]
#[prompt_handler]
impl ServerHandler for MusicChoreServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "music-chore".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            instructions: Some("Music Chore CLI - Music library metadata management tool".into()),
        }
    }
}
