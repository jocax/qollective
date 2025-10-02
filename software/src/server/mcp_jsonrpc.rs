// ABOUTME: MCP jsonrpsee WebSocket server implementation with envelope integration
// ABOUTME: Provides MCP protocol server using jsonrpsee WebSocket with qollective envelope pattern

//! MCP jsonrpsee WebSocket server implementation.
//!
//! This module provides a WebSocket-based MCP server using jsonrpsee for JSON-RPC 2.0
//! communication while maintaining the qollective envelope pattern for metadata and
//! context propagation.

use crate::envelope::{Context, Meta};
use crate::error::{QollectiveError, Result};
use crate::traits::handlers::ContextDataHandler;
use crate::traits::receivers::UnifiedEnvelopeReceiver;
use crate::transport::jsonrpc::{JsonRpcRequest};
use async_trait::async_trait;
use jsonrpsee::server::{ServerBuilder, ServerHandle, ServerConfig};
use jsonrpsee::types::{Params, ErrorObjectOwned};
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP jsonrpsee WebSocket server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpJsonRpcServerConfig {
    /// Server binding address
    pub bind_address: String,
    /// Server binding port
    pub bind_port: u16,
    /// Maximum number of concurrent connections
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Enable CORS support
    pub enable_cors: bool,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
    /// Server info metadata
    pub server_info: HashMap<String, String>,
}

impl Default for McpJsonRpcServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            bind_port: 8080,
            max_connections: 1000,
            connection_timeout: 30,
            enable_cors: true,
            cors_origins: vec!["*".to_string()],
            server_info: HashMap::new(),
        }
    }
}

/// MCP jsonrpsee WebSocket server
pub struct McpJsonRpcServer<H> {
    /// Server configuration
    config: McpJsonRpcServerConfig,
    /// Context data handler
    handler: Arc<H>,
    /// Server handle (set after starting)
    server_handle: Arc<RwLock<Option<ServerHandle>>>,
}

impl<H> McpJsonRpcServer<H>
where
    H: ContextDataHandler<serde_json::Value, serde_json::Value> + Send + Sync + 'static,
{
    /// Create new MCP jsonrpsee WebSocket server
    pub fn new(config: McpJsonRpcServerConfig, handler: H) -> Self {
        Self {
            config,
            handler: Arc::new(handler),
            server_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the MCP WebSocket server
    pub async fn start(&self) -> Result<()> {
        let bind_addr = format!("{}:{}", self.config.bind_address, self.config.bind_port);
        let socket_addr: SocketAddr = bind_addr.parse().map_err(|e| {
            QollectiveError::config(format!("Invalid bind address {}: {}", bind_addr, e))
        })?;

        // Create RPC module with MCP methods
        let mut module = RpcModule::new(());
        let handler = Arc::clone(&self.handler);

        // Register MCP initialize method
        {
            let handler = Arc::clone(&handler);
            let _callback = module.register_async_method("initialize", move |params: Params<'_>, _ctx, _extensions| {
                let handler = Arc::clone(&handler);
                async move {
                    let request = parse_jsonrpc_request("initialize", params)
                        .map_err(|e| ErrorObjectOwned::owned(1, format!("Failed to parse request: {}", e), None::<()>))?;
                    let response = handle_mcp_request(handler, request).await
                        .map_err(|e| ErrorObjectOwned::owned(2, format!("Handler error: {}", e), None::<()>))?;
                    Ok::<serde_json::Value, ErrorObjectOwned>(response)
                }
            }).map_err(|e| QollectiveError::transport(format!("Failed to register initialize method: {}", e)))?;
        }

        // Register MCP tools/list method
        {
            let handler = Arc::clone(&handler);
            let _callback = module.register_async_method("tools/list", move |params: Params<'_>, _ctx, _extensions| {
                let handler = Arc::clone(&handler);
                async move {
                    let request = parse_jsonrpc_request("tools/list", params)
                        .map_err(|e| ErrorObjectOwned::owned(1, format!("Failed to parse request: {}", e), None::<()>))?;
                    let response = handle_mcp_request(handler, request).await
                        .map_err(|e| ErrorObjectOwned::owned(2, format!("Handler error: {}", e), None::<()>))?;
                    Ok::<serde_json::Value, ErrorObjectOwned>(response)
                }
            }).map_err(|e| QollectiveError::transport(format!("Failed to register tools/list method: {}", e)))?;
        }

        // Register MCP tools/call method
        {
            let handler = Arc::clone(&handler);
            let _callback = module.register_async_method("tools/call", move |params: Params<'_>, _ctx, _extensions| {
                let handler = Arc::clone(&handler);
                async move {
                    let request = parse_jsonrpc_request("tools/call", params)
                        .map_err(|e| ErrorObjectOwned::owned(1, format!("Failed to parse request: {}", e), None::<()>))?;
                    let response = handle_mcp_request(handler, request).await
                        .map_err(|e| ErrorObjectOwned::owned(2, format!("Handler error: {}", e), None::<()>))?;
                    Ok::<serde_json::Value, ErrorObjectOwned>(response)
                }
            }).map_err(|e| QollectiveError::transport(format!("Failed to register tools/call method: {}", e)))?;
        }

        // Register MCP resources/list method
        {
            let handler = Arc::clone(&handler);
            let _callback = module.register_async_method("resources/list", move |params: Params<'_>, _ctx, _extensions| {
                let handler = Arc::clone(&handler);
                async move {
                    let request = parse_jsonrpc_request("resources/list", params)
                        .map_err(|e| ErrorObjectOwned::owned(1, format!("Failed to parse request: {}", e), None::<()>))?;
                    let response = handle_mcp_request(handler, request).await
                        .map_err(|e| ErrorObjectOwned::owned(2, format!("Handler error: {}", e), None::<()>))?;
                    Ok::<serde_json::Value, ErrorObjectOwned>(response)
                }
            }).map_err(|e| QollectiveError::transport(format!("Failed to register resources/list method: {}", e)))?;
        }

        // Register MCP resources/read method
        {
            let handler = Arc::clone(&handler);
            let _callback = module.register_async_method("resources/read", move |params: Params<'_>, _ctx, _extensions| {
                let handler = Arc::clone(&handler);
                async move {
                    let request = parse_jsonrpc_request("resources/read", params)
                        .map_err(|e| ErrorObjectOwned::owned(1, format!("Failed to parse request: {}", e), None::<()>))?;
                    let response = handle_mcp_request(handler, request).await
                        .map_err(|e| ErrorObjectOwned::owned(2, format!("Handler error: {}", e), None::<()>))?;
                    Ok::<serde_json::Value, ErrorObjectOwned>(response)
                }
            }).map_err(|e| QollectiveError::transport(format!("Failed to register resources/read method: {}", e)))?;
        }

        // Register MCP prompts/list method
        {
            let handler = Arc::clone(&handler);
            let _callback = module.register_async_method("prompts/list", move |params: Params<'_>, _ctx, _extensions| {
                let handler = Arc::clone(&handler);
                async move {
                    let request = parse_jsonrpc_request("prompts/list", params)
                        .map_err(|e| ErrorObjectOwned::owned(1, format!("Failed to parse request: {}", e), None::<()>))?;
                    let response = handle_mcp_request(handler, request).await
                        .map_err(|e| ErrorObjectOwned::owned(2, format!("Handler error: {}", e), None::<()>))?;
                    Ok::<serde_json::Value, ErrorObjectOwned>(response)
                }
            }).map_err(|e| QollectiveError::transport(format!("Failed to register prompts/list method: {}", e)))?;
        }

        // Register MCP prompts/get method
        {
            let handler = Arc::clone(&handler);
            let _callback = module.register_async_method("prompts/get", move |params: Params<'_>, _ctx, _extensions| {
                let handler = Arc::clone(&handler);
                async move {
                    let request = parse_jsonrpc_request("prompts/get", params)
                        .map_err(|e| ErrorObjectOwned::owned(1, format!("Failed to parse request: {}", e), None::<()>))?;
                    let response = handle_mcp_request(handler, request).await
                        .map_err(|e| ErrorObjectOwned::owned(2, format!("Handler error: {}", e), None::<()>))?;
                    Ok::<serde_json::Value, ErrorObjectOwned>(response)
                }
            }).map_err(|e| QollectiveError::transport(format!("Failed to register prompts/get method: {}", e)))?;
        }

        // Build server with configuration
        let server_config = ServerConfig::builder()
            .max_connections(self.config.max_connections)
            .build();

        let server = ServerBuilder::default()
            .set_config(server_config)
            .build(socket_addr)
            .await
            .map_err(|e| QollectiveError::transport(format!("Failed to build jsonrpsee server: {}", e)))?;

        // Start server
        let server_handle = server.start(module);

        // Store server handle
        {
            let mut handle_guard = self.server_handle.write().await;
            *handle_guard = Some(server_handle);
        }

        tracing::info!("MCP jsonrpsee WebSocket server started on {}", socket_addr);
        Ok(())
    }

    /// Stop the MCP WebSocket server
    pub async fn stop(&self) -> Result<()> {
        let mut handle_guard = self.server_handle.write().await;
        if let Some(handle) = handle_guard.take() {
            handle.stop().map_err(|e| {
                QollectiveError::transport(format!("Failed to stop MCP server: {}", e))
            })?;
            tracing::info!("MCP jsonrpsee WebSocket server stopped");
        }
        Ok(())
    }

    /// Check if server is running
    pub async fn is_running(&self) -> bool {
        let handle_guard = self.server_handle.read().await;
        handle_guard.is_some()
    }

    /// Get server configuration
    pub fn config(&self) -> &McpJsonRpcServerConfig {
        &self.config
    }
}

/// Parse jsonrpsee params into JsonRpcRequest
fn parse_jsonrpc_request(method: &str, params: Params<'_>) -> Result<JsonRpcRequest<serde_json::Value>> {
    let params_value = match params.one::<serde_json::Value>() {
        Ok(value) => value,
        Err(_) => serde_json::Value::Null,
    };

    Ok(JsonRpcRequest::new(
        method.to_string(),
        params_value,
        Some(0), // ID will be set by jsonrpsee
        Meta::default(),
    ))
}

/// Handle MCP request using context data handler
async fn handle_mcp_request<H>(
    handler: Arc<H>,
    request: JsonRpcRequest<serde_json::Value>,
) -> Result<serde_json::Value>
where
    H: ContextDataHandler<serde_json::Value, serde_json::Value> + Send + Sync + 'static,
{
    // Extract context from request metadata
    let context = if request.meta.tenant.is_some() {
        Some(Context::new(request.meta.clone()))
    } else {
        None
    };

    // Process through context data handler
    let response_data = handler.handle(context, request.params).await?;

    Ok(response_data)
}

#[async_trait]
impl<H> UnifiedEnvelopeReceiver for McpJsonRpcServer<H>
where
    H: ContextDataHandler<serde_json::Value, serde_json::Value> + Send + Sync + 'static,
{
    async fn receive_envelope<T, R, Handler>(&mut self, handler: Handler) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
        Handler: ContextDataHandler<T, R> + Send + Sync + 'static,
    {
        // The server is already running with the configured handler
        // This method is for compatibility with the UnifiedEnvelopeReceiver trait
        let _ = handler; // Suppress unused warning
        Ok(())
    }

    async fn receive_envelope_at<T, R, Handler>(&mut self, route: &str, handler: Handler) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
        Handler: ContextDataHandler<T, R> + Send + Sync + 'static,
    {
        // The server is already running with the configured handler
        // This method is for compatibility with the UnifiedEnvelopeReceiver trait
        let _ = route; // Suppress unused warning
        let _ = handler; // Suppress unused warning
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::handlers::ContextDataHandler;
    use async_trait::async_trait;

    #[derive(Debug, Clone)]
    struct TestHandler;

    #[async_trait]
    impl ContextDataHandler<serde_json::Value, serde_json::Value> for TestHandler {
        async fn handle(
            &self,
            _context: Option<Context>,
            _data: serde_json::Value,
        ) -> Result<serde_json::Value> {
            // Simple echo handler for testing
            Ok(serde_json::json!({"status": "ok", "method": "test"}))
        }
    }

    #[test]
    fn test_mcp_jsonrpc_server_config_default() {
        let config = McpJsonRpcServerConfig::default();
        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.bind_port, 8080);
        assert_eq!(config.max_connections, 1000);
        assert_eq!(config.connection_timeout, 30);
        assert!(config.enable_cors);
        assert_eq!(config.cors_origins, vec!["*".to_string()]);
    }

    #[test]
    fn test_mcp_jsonrpc_server_creation() {
        let config = McpJsonRpcServerConfig::default();
        let handler = TestHandler;
        let server = McpJsonRpcServer::new(config.clone(), handler);
        assert_eq!(server.config().bind_address, config.bind_address);
        assert_eq!(server.config().bind_port, config.bind_port);
    }

    #[tokio::test]
    async fn test_mcp_jsonrpc_server_not_running_initially() {
        let config = McpJsonRpcServerConfig::default();
        let handler = TestHandler;
        let server = McpJsonRpcServer::new(config, handler);
        assert!(!server.is_running().await);
    }

    #[test]
    fn test_parse_jsonrpc_request() {
        let params = Params::new(None);
        let request = parse_jsonrpc_request("test_method", params).unwrap();
        assert_eq!(request.method, "test_method");
        assert_eq!(request.params, serde_json::Value::Null);
    }
}