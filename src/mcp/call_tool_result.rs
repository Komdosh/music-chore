use rmcp::model::{CallToolResult, Content};

/// Extension trait to reduce boilerplate around `CallToolResult` construction.
pub(crate) trait CallToolResultExt {
    fn success_text(text: impl Into<String>) -> CallToolResult;
    fn error_text(text: impl Into<String>) -> CallToolResult;
}

impl CallToolResultExt for CallToolResult {
    fn success_text(text: impl Into<String>) -> CallToolResult {
        CallToolResult::success(vec![Content::text(text)])
    }

    fn error_text(text: impl Into<String>) -> CallToolResult {
        CallToolResult::error(vec![Content::text(text)])
    }
}
