// Test file to verify rmcp macro functionality
use rmcp::{ErrorData as McpError, model::*, tool, tool_router, handler::server::tool::ToolRouter};

#[derive(Clone)]
pub struct TestServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router(router = tool_router)]
impl TestServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Simple test tool")]
    async fn test_tool(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("test".to_string())]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rmcp_macro_generation() {
        // This test verifies that the rmcp macros generate the expected functions
        let _attr = TestServer::test_tool_tool_attr();
        let _server = TestServer::new();
    }
}