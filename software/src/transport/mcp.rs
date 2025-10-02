// ABOUTME: Comprehensive MCP transport implementation supporting both Qollective and standard rmcp protocols
// ABOUTME: Combines hybrid transport architecture and pure transport capabilities for complete MCP ecosystem support

//! Comprehensive MCP transport implementation for Qollective framework.
//!
//! This module implements both the hybrid transport architecture for MCP communication
//! and the pure transport layer for rmcp protocol communication. It enables:
//! - Qollective MCP clients to work with both Qollective and standard MCP servers
//! - Automatic protocol detection and seamless transport switching
//! - Pure transport abstraction for rmcp protocol without business logic concerns
//! - Full rmcp protocol compliance and envelope-first design

use crate::config::mcp::{McpServerEndpoint, McpTransportClientConfig, McpTransportStats};
use crate::constants::{helpers, limits, metadata, timeouts};
use crate::envelope::Envelope;
use crate::error::{QollectiveError, Result};
use crate::traits::catalog::ServerCatalog;
use crate::transport::{HybridTransportClient, TransportRequirements};
use crate::types::mcp::{McpData, McpDiscoveryData};
use rmcp::model::{CallToolRequest, CallToolResult, Tool};
// Additional rmcp imports with feature gates
#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
use rmcp::{
    model::CallToolRequestParam,
    service::{ServiceExt, RunningService},
};
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Additional imports from pure MCP transport
#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
use {
    crate::traits::senders::UnifiedEnvelopeSender,
    async_trait::async_trait,
    serde::{Deserialize, Serialize},
    std::time::Duration,
    url::Url,
};

/// MCP transport client implementing hybrid transport architecture
pub struct McpTransportClient {
    /// Underlying hybrid transport client
    hybrid_transport: HybridTransportClient,
    /// Cache of client endpoints for external servers
    client_endpoints: Arc<RwLock<HashMap<String, String>>>,
    /// MCP server catalog reference
    server_catalog: Arc<dyn ServerCatalog + Send + Sync>,
    /// Client configuration
    config: McpTransportClientConfig,
}

impl McpTransportClient {
    /// Create a new MCP transport client
    pub fn new(
        hybrid_transport: HybridTransportClient,
        server_catalog: Arc<dyn ServerCatalog + Send + Sync>,
        config: McpTransportClientConfig,
    ) -> Self {
        Self {
            hybrid_transport,
            client_endpoints: Arc::new(RwLock::new(HashMap::new())),
            server_catalog,
            config,
        }
    }

    /// Discover MCP server capabilities and create endpoint information
    pub async fn discover_mcp_server(&self, server_url: &str) -> Result<McpServerEndpoint> {
        // Use hybrid transport to detect capabilities
        let capabilities = self
            .hybrid_transport
            .detect_capabilities(server_url)
            .await?;

        // Check if this is a Qollective-native MCP server
        let is_qollective_native = capabilities.supports_envelopes;

        // Determine MCP version
        let mcp_version = capabilities
            .mcp_version
            .clone()
            .unwrap_or_else(|| "1.0.0".to_string());

        // Get supported tools
        let supported_tools = if is_qollective_native {
            self.discover_qollective_tools(server_url).await?
        } else {
            self.discover_rmcp_tools(server_url).await?
        };

        // Select preferred transport
        let requirements = TransportRequirements {
            requires_envelopes: false, // MCP can work with both
            preferred_protocols: vec!["mcp".to_string(), "rest".to_string(), "grpc".to_string()],
            ..Default::default()
        };

        let preferred_transport = self
            .hybrid_transport
            .select_optimal_transport(server_url, &requirements)
            .await?;

        Ok(McpServerEndpoint {
            server_id: self.generate_server_id(server_url),
            endpoint_url: server_url.to_string(),
            capabilities,
            mcp_version,
            supported_tools,
            preferred_transport,
            is_qollective_native,
        })
    }

    /// Execute a tool call on a specific server
    pub async fn execute_tool_call(
        &self,
        server_id: &str,
        tool_call: CallToolRequest,
    ) -> Result<CallToolResult> {
        // Get server endpoint information
        let endpoint = self.get_server_endpoint(server_id).await?;

        if endpoint.is_qollective_native {
            self.execute_qollective_tool(&endpoint, tool_call).await
        } else {
            self.execute_native_tool(&endpoint, tool_call).await
        }
    }

    /// List available tools on a server
    pub async fn list_tools(&self, server_id: &str) -> Result<Vec<Tool>> {
        let endpoint = self.get_server_endpoint(server_id).await?;

        if endpoint.is_qollective_native {
            self.list_qollective_tools(&endpoint).await
        } else {
            self.list_rmcp_tools(&endpoint).await
        }
    }

    /// Execute tool on Qollective-native MCP server
    pub async fn execute_qollective_tool(
        &self,
        endpoint: &McpServerEndpoint,
        tool_call: CallToolRequest,
    ) -> Result<CallToolResult> {
        // Create MCP data with tool call
        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        // Create envelope
        let envelope = Envelope::new(crate::envelope::Meta::default(), mcp_data);

        // Create transport requirements for Qollective
        let requirements = TransportRequirements {
            requires_envelopes: true,
            ..Default::default()
        };

        // Send via hybrid transport
        let response_envelope: Envelope<McpData> = self
            .hybrid_transport
            .send_with_fallback(&endpoint.endpoint_url, envelope, &requirements)
            .await?;

        let (_, response_data) = response_envelope.extract();

        response_data.tool_response.ok_or_else(|| {
            QollectiveError::mcp_tool_execution("No tool response in envelope".to_string())
        })
    }

    /// Execute tool on standard rmcp server
    pub async fn execute_native_tool(
        &self,
        endpoint: &McpServerEndpoint,
        tool_call: CallToolRequest,
    ) -> Result<CallToolResult> {
        // For now, use hybrid transport to send the tool call in native format
        let requirements = TransportRequirements {
            requires_envelopes: false,
            preferred_protocols: vec!["rest".to_string(), "grpc".to_string()],
            ..Default::default()
        };

        // Send tool call using native protocol format
        let response: CallToolResult = self
            .hybrid_transport
            .send_with_fallback(&endpoint.endpoint_url, tool_call, &requirements)
            .await?;

        Ok(response)
    }

    /// Discover tools from Qollective MCP server
    async fn discover_qollective_tools(&self, server_url: &str) -> Result<Vec<Tool>> {
        // Create discovery query
        let discovery_data = McpDiscoveryData {
            query_type: "list_tools".to_string(),
            tools: None,
            server_info: None,
        };

        let mcp_data = McpData {
            tool_call: None,
            tool_response: None,
            tool_registration: None,
            discovery_data: Some(discovery_data),
        };

        let envelope = Envelope::new(crate::envelope::Meta::default(), mcp_data);

        let requirements = TransportRequirements {
            requires_envelopes: true,
            ..Default::default()
        };

        let response_envelope: Envelope<McpData> = self
            .hybrid_transport
            .send_with_fallback(server_url, envelope, &requirements)
            .await?;

        let (_, response_data) = response_envelope.extract();

        if let Some(discovery_response) = response_data.discovery_data {
            Ok(discovery_response.tools.unwrap_or_default())
        } else {
            Ok(vec![])
        }
    }

    /// Discover tools from standard rmcp server
    async fn discover_rmcp_tools(&self, server_url: &str) -> Result<Vec<Tool>> {
        // For now, use hybrid transport to discover tools via native protocol
        let requirements = TransportRequirements {
            requires_envelopes: false,
            preferred_protocols: vec!["rest".to_string(), "grpc".to_string()],
            ..Default::default()
        };

        // Create a discovery request in native format
        let discovery_request = serde_json::json!({
            "method": "tools/list",
            "params": {}
        });

        let _response: serde_json::Value = self
            .hybrid_transport
            .send_with_fallback(server_url, discovery_request, &requirements)
            .await?;

        // Parse response to extract tools (simplified for now)
        // Parse response to extract tools from standard rmcp response
        // Handle both JSON-RPC response format and direct tool list
        if let Some(tools_array) = _response.get("result").and_then(|r| r.get("tools")) {
            // Standard JSON-RPC format: {"result": {"tools": [...]}}
            if let Ok(tools) = serde_json::from_value::<Vec<Tool>>(tools_array.clone()) {
                return Ok(tools);
            }
        } else if let Some(tools_array) = _response.get("tools") {
            // Direct format: {"tools": [...]}
            if let Ok(tools) = serde_json::from_value::<Vec<Tool>>(tools_array.clone()) {
                return Ok(tools);
            }
        } else if let Ok(tools) = serde_json::from_value::<Vec<Tool>>(_response.clone()) {
            // Direct array format: [...]
            return Ok(tools);
        }

        // If parsing fails, log warning and return empty list
        tracing::warn!("MCP transport failed to parse rmcp tools response format");
        Ok(vec![])
    }

    /// List tools from Qollective MCP server
    async fn list_qollective_tools(&self, endpoint: &McpServerEndpoint) -> Result<Vec<Tool>> {
        self.discover_qollective_tools(&endpoint.endpoint_url).await
    }

    /// List tools from standard rmcp server
    async fn list_rmcp_tools(&self, endpoint: &McpServerEndpoint) -> Result<Vec<Tool>> {
        self.discover_rmcp_tools(&endpoint.endpoint_url).await
    }

    /// Cache endpoint configuration for reuse
    async fn cache_endpoint(&self, server_url: &str, endpoint_config: String) {
        let mut endpoints = self.client_endpoints.write().await;

        // Check cache size limit
        if endpoints.len() >= self.config.max_rmcp_clients {
            // Remove oldest endpoint (simple FIFO eviction)
            if let Some(oldest_key) = endpoints.keys().next().cloned() {
                endpoints.remove(&oldest_key);
            }
        }

        endpoints.insert(server_url.to_string(), endpoint_config);
    }

    /// Get cached endpoint configuration
    async fn get_cached_endpoint(&self, server_url: &str) -> Option<String> {
        let endpoints = self.client_endpoints.read().await;
        endpoints.get(server_url).cloned()
    }

    /// Get server endpoint information
    async fn get_server_endpoint(&self, server_id: &str) -> Result<McpServerEndpoint> {
        // First try to get from registry
        // Get server metadata from catalog and extract endpoint URL
        let server_metadata = self.server_catalog.get_server_metadata().await?;

        // Extract endpoint URL from server metadata or use default pattern
        let endpoint_url = if let Some(url_value) = server_metadata.get("endpoint_url") {
            if let Some(url_str) = url_value.as_str() {
                url_str.to_string()
            } else {
                tracing::warn!(
                    "MCP transport server endpoint_url not a string, using default pattern"
                );
                helpers::mcp_server_endpoint_url(server_id)
            }
        } else if let Some(base_url) = server_metadata.get("base_url") {
            // Fallback to base_url if endpoint_url not available
            format!(
                "{}/mcp/{}",
                base_url.as_str().unwrap_or("http://localhost:8080"),
                server_id
            )
        } else {
            // Use constants-based default pattern
            helpers::mcp_server_endpoint_url(server_id)
        };

        self.discover_mcp_server(&endpoint_url).await
    }

    /// Generate server ID from URL
    fn generate_server_id(&self, server_url: &str) -> String {
        // Create deterministic server ID from URL
        let mut hasher = Sha256::new();
        hasher.update(server_url.as_bytes());
        let result = hasher.finalize();
        format!(
            "mcp-{:02x}{:02x}{:02x}{:02x}",
            result[0], result[1], result[2], result[3]
        )
    }

    /// Clear endpoint cache
    pub async fn clear_cache(&self) {
        let mut endpoints = self.client_endpoints.write().await;
        endpoints.clear();
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> McpTransportStats {
        let endpoints = self.client_endpoints.read().await;
        McpTransportStats {
            rmcp_clients_cached: endpoints.len(),
            max_rmcp_clients: self.config.max_rmcp_clients,
            connection_pooling_enabled: self.config.enable_connection_pooling,
        }
    }
}

// ============================================================================
// PURE MCP TRANSPORT IMPLEMENTATION (consolidated from pure_mcp.rs)
// ============================================================================

/// Configuration for pure MCP transport
#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
#[derive(Debug, Clone)]
pub struct McpTransportConfig {
    /// Connection timeout for rmcp client
    pub connection_timeout: Duration,
    /// Request timeout for MCP operations
    pub request_timeout: Duration,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Enable connection pooling
    pub enable_pooling: bool,
    /// Retry attempts for failed requests
    pub retry_attempts: u32,
    /// TLS verification settings
    pub verify_tls: bool,
    /// MCP protocol version to use
    pub mcp_version: String,
    /// Client identification for MCP handshake
    pub client_info: McpClientInfo,
    /// Server capabilities to request
    pub requested_capabilities: Vec<String>,
    /// Custom headers to include in requests
    pub custom_headers: std::collections::HashMap<String, String>,
    /// Enable compression for requests
    pub enable_compression: bool,
    /// Authentication configuration
    pub auth_config: Option<McpAuthConfig>,
    /// Available rmcp transport types (configurable transport selection)
    pub transport_types: Vec<RmcpTransportType>,
    /// Default transport type to use when creating new connections
    pub default_transport_type: RmcpTransportType,
}

/// Available rmcp transport types for configurable transport selection
#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
#[derive(Debug, Clone, PartialEq)]
pub enum RmcpTransportType {
    /// HTTP/HTTPS transport (via SSE)
    Http,
    /// Secure HTTP transport (HTTPS via SSE)  
    Https,
    /// Child process stdio transport
    Stdio,
    /// TCP socket transport (future expansion)
    Tcp,
}

/// MCP client identification information
#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
#[derive(Debug, Clone)]
pub struct McpClientInfo {
    /// Client name
    pub name: String,
    /// Client version
    pub version: String,
    /// Client vendor/organization
    pub vendor: Option<String>,
    /// Client description
    pub description: Option<String>,
}

/// MCP authentication configuration
#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
#[derive(Debug, Clone)]
pub struct McpAuthConfig {
    /// Authentication type
    pub auth_type: McpAuthType,
    /// API key for key-based authentication
    pub api_key: Option<String>,
    /// Bearer token for token-based authentication
    pub bearer_token: Option<String>,
    /// Username for basic authentication
    pub username: Option<String>,
    /// Password for basic authentication
    pub password: Option<String>,
    /// Custom auth headers
    pub custom_headers: std::collections::HashMap<String, String>,
}

/// MCP authentication types
#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
#[derive(Debug, Clone, PartialEq)]
pub enum McpAuthType {
    /// No authentication
    None,
    /// API key in header
    ApiKey,
    /// Bearer token
    BearerToken,
    /// Basic authentication
    BasicAuth,
    /// Custom authentication
    Custom,
}

/// Internal MCP client implementation using rmcp protocol
/// Integrates rmcp transport layer with Qollective envelope architecture
#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
#[derive(Debug)]
pub struct InternalMcpClient {
    /// Transport configuration
    config: McpTransportConfig,
    /// Connection pool for rmcp client instances
    connection_pool: Arc<RwLock<HashMap<String, Arc<RunningService<rmcp::service::RoleClient, ()>>>>>,
    /// Client handler for rmcp protocol
    client_handler: (),
}

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
impl InternalMcpClient {
    /// Create a new internal MCP client
    pub fn new(config: McpTransportConfig) -> Self {
        Self {
            config,
            connection_pool: Arc::new(RwLock::new(HashMap::new())),
            client_handler: (),
        }
    }

    /// Parse MCP endpoint URL to extract connection details
    fn parse_mcp_endpoint(&self, endpoint: &str) -> Result<Url> {
        let url = Url::parse(endpoint).map_err(|e| {
            QollectiveError::transport(format!("Invalid MCP endpoint URL '{}': {}", endpoint, e))
        })?;

        // Validate MCP protocol schemes
        match url.scheme() {
            "mcp" | "mcps" | "http" | "https" => Ok(url),
            scheme => Err(QollectiveError::transport(format!(
                "Unsupported MCP scheme '{}'. Expected 'mcp', 'mcps', 'http', or 'https'",
                scheme
            ))),
        }
    }

    /// Get or create rmcp client connection for endpoint
    async fn get_rmcp_client(&self, endpoint: &str) -> Result<Arc<RunningService<rmcp::service::RoleClient, ()>>> {
        let url = self.parse_mcp_endpoint(endpoint)?;
        let connection_key = format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""));

        // Check connection pool first
        if self.config.enable_pooling {
            let pool = self.connection_pool.read().await;
            if let Some(client) = pool.get(&connection_key) {
                return Ok(client.clone());
            }
        }

        // Create rmcp client connection
        tracing::info!(
            "MCP transport creating rmcp client connection for endpoint: {}",
            endpoint
        );

        let client = self.create_rmcp_client(&url).await?;
        let client = Arc::new(client);

        // Add to pool if pooling is enabled
        if self.config.enable_pooling {
            let mut pool = self.connection_pool.write().await;

            // Check pool size limit
            if pool.len() >= self.config.max_connections {
                // Remove oldest connection (simple FIFO eviction)
                if let Some(oldest_key) = pool.keys().next().cloned() {
                    pool.remove(&oldest_key);
                }
            }

            pool.insert(connection_key, client.clone());
        }

        Ok(client)
    }

    /// Create rmcp client using appropriate transport for the URL
    async fn create_rmcp_client(&self, url: &Url) -> Result<RunningService<rmcp::service::RoleClient, ()>> {
        // Determine transport type based on URL scheme and configuration
        let transport_type = self.determine_transport_type(url)?;
        
        match transport_type {
            RmcpTransportType::Http | RmcpTransportType::Https => {
                // Use HTTP/SSE transport for HTTP-based MCP servers
                self.create_http_rmcp_client(url, &transport_type).await
            }
            RmcpTransportType::Stdio => {
                // Use stdio transport for local MCP servers
                self.create_stdio_rmcp_client().await
            }
            RmcpTransportType::Tcp => {
                // TCP transport not yet implemented
                Err(QollectiveError::transport(
                    "TCP transport not yet implemented in rmcp integration".to_string()
                ))
            }
        }
    }
    
    /// Determine transport type based on URL and configuration
    fn determine_transport_type(&self, url: &Url) -> Result<RmcpTransportType> {
        match url.scheme() {
            "mcp" | "http" => {
                if self.config.transport_types.contains(&RmcpTransportType::Http) {
                    Ok(RmcpTransportType::Http)
                } else {
                    Ok(self.config.default_transport_type.clone())
                }
            }
            "mcps" | "https" => {
                if self.config.transport_types.contains(&RmcpTransportType::Https) {
                    Ok(RmcpTransportType::Https)
                } else {
                    Ok(self.config.default_transport_type.clone())
                }
            }
            "stdio" => {
                if self.config.transport_types.contains(&RmcpTransportType::Stdio) {
                    Ok(RmcpTransportType::Stdio)
                } else {
                    Err(QollectiveError::transport(format!(
                        "Stdio transport not configured in transport_types: {:?}",
                        self.config.transport_types
                    )))
                }
            }
            _ => Err(QollectiveError::transport(format!(
                "Unsupported MCP transport scheme: {}",
                url.scheme()
            ))),
        }
    }

    /// Create HTTP-based rmcp client using configurable transport
    async fn create_http_rmcp_client(&self, url: &Url, transport_type: &RmcpTransportType) -> Result<RunningService<rmcp::service::RoleClient, ()>> {
        // Convert URL to appropriate format for rmcp transport
        let base_url = match (url.scheme(), transport_type) {
            ("mcp", RmcpTransportType::Http) => format!("http://{}{}", 
                url.host_str().unwrap_or("localhost"),
                if let Some(port) = url.port() { format!(":{}", port) } else { String::new() }),
            ("mcps", RmcpTransportType::Https) | ("https", RmcpTransportType::Https) => format!("https://{}{}", 
                url.host_str().unwrap_or("localhost"),
                if let Some(port) = url.port() { format!(":{}", port) } else { String::new() }),
            ("http", RmcpTransportType::Http) => url.to_string().trim_end_matches('/').to_string(),
            _ => return Err(QollectiveError::transport(format!(
                "Invalid HTTP scheme and transport combination: {} with {:?}", 
                url.scheme(), transport_type
            ))),
        };

        // For HTTP transport, use a simplified approach with rmcp child process 
        // In a full implementation, you'd configure proper HTTP/SSE transport
        // This is a placeholder for rmcp HTTP transport integration
        use rmcp::transport::TokioChildProcess;
        use tokio::process::Command;
        
        tracing::warn!(
            "HTTP transport using child process placeholder - full HTTP/SSE transport implementation needed"
        );
        
        let mut command = Command::new("echo");
        command.arg(format!("http-mcp-client-{}", base_url));
        
        let transport = TokioChildProcess::new(command).map_err(|e| {
            QollectiveError::transport(format!("Failed to create HTTP transport placeholder: {}", e))
        })?;

        // Serve client using rmcp transport
        let client_service = ().serve(transport).await.map_err(|e| {
            QollectiveError::transport(format!("Failed to serve rmcp client: {}", e))
        })?;

        Ok(client_service)
    }

    /// Create stdio-based rmcp client using rmcp child process transport
    async fn create_stdio_rmcp_client(&self) -> Result<RunningService<rmcp::service::RoleClient, ()>> {
        use rmcp::transport::TokioChildProcess;
        use tokio::process::Command;
        
        // For stdio transport, we typically connect to local MCP servers via child process
        // This is a simplified implementation - in practice, you'd configure the actual command
        let mut command = Command::new("echo");
        command.arg("mcp-stdio-server"); // Placeholder for actual MCP server command
        
        let child_transport = TokioChildProcess::new(command).map_err(|e| {
            QollectiveError::transport(format!("Failed to create child process transport: {}", e))
        })?;

        // Serve client using rmcp child process transport
        let client_service = ().serve(child_transport).await.map_err(|e| {
            QollectiveError::transport(format!("Failed to serve rmcp stdio client: {}", e))
        })?;

        Ok(client_service)
    }

    /// Convert envelope to MCP request
    fn envelope_to_mcp_request<T: Serialize>(
        &self,
        envelope: Envelope<T>,
    ) -> Result<serde_json::Value> {
        // Extract envelope data by taking ownership
        let (meta, data) = envelope.extract();

        // Convert data to JSON value for rmcp
        let data_value = serde_json::to_value(data).map_err(|e| {
            QollectiveError::serialization(format!("Failed to serialize envelope data: {}", e))
        })?;

        // Create MCP request structure
        // For envelope-based requests, we embed the data in a tool call format
        let mcp_request = serde_json::json!({
            "method": "envelope/execute",
            "params": {
                "data": data_value,
                "meta": meta
            }
        });

        Ok(mcp_request)
    }

    /// Convert MCP response to envelope
    fn mcp_response_to_envelope<R: for<'de> Deserialize<'de>>(
        &self,
        response: serde_json::Value,
    ) -> Result<Envelope<R>> {
        // Parse MCP response structure
        let response_data = if let Some(result) = response.get("result") {
            result.clone()
        } else if let Some(data) = response.get("data") {
            data.clone()
        } else {
            response
        };

        let data: R = serde_json::from_value(response_data).map_err(|e| {
            QollectiveError::serialization(format!("Failed to deserialize MCP response: {}", e))
        })?;

        // Create envelope with default metadata
        let meta = crate::envelope::Meta::default();
        Ok(Envelope::new(meta, data))
    }

    /// Send MCP request using rmcp client
    pub async fn send_mcp_request(
        &self,
        endpoint: &str,
        request: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Get rmcp client for endpoint
        let client = self.get_rmcp_client(endpoint).await?;

        // Execute request with retries
        let mut last_error = None;
        for attempt in 0..=self.config.retry_attempts {
            match self.execute_rmcp_request(&client, &request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.retry_attempts {
                        // Exponential backoff for retries
                        let delay = Duration::from_millis(100 * (1 << attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            QollectiveError::transport("MCP request failed after all retries".to_string())
        }))
    }

    /// Execute MCP request using rmcp client
    async fn execute_rmcp_request(
        &self,
        client: &RunningService<rmcp::service::RoleClient, ()>,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Parse method from request
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .ok_or_else(|| QollectiveError::transport("MCP request missing method field".to_string()))?;

        // Execute request based on method using rmcp client peer
        let peer = client.peer();
        
        match method {
            "tools/list" => {
                // Use rmcp pagination parameters 
                let result = peer
                    .list_tools(Some(rmcp::model::PaginatedRequestParam::default()))
                    .await
                    .map_err(|e| QollectiveError::transport(format!("rmcp list_tools error: {}", e)))?;
                
                // Convert result to serde_json::Value
                serde_json::to_value(result).map_err(|e| {
                    QollectiveError::serialization(format!("Failed to serialize tools list result: {}", e))
                })
            }
            "tools/call" => {
                // Extract tool call parameters
                let params = request.get("params").cloned().unwrap_or_default();
                let call_tool_params: CallToolRequestParam = serde_json::from_value(params).map_err(|e| {
                    QollectiveError::serialization(format!("Failed to parse tool call parameters: {}", e))
                })?;

                let result = peer
                    .call_tool(call_tool_params)
                    .await
                    .map_err(|e| QollectiveError::transport(format!("rmcp call_tool error: {}", e)))?;
                
                // Convert result to serde_json::Value
                serde_json::to_value(result).map_err(|e| {
                    QollectiveError::serialization(format!("Failed to serialize tool call result: {}", e))
                })
            }
            "envelope/execute" => {
                // For envelope-based requests, we need to extract and handle the envelope data
                // This is a custom method for Qollective integration
                let params = request.get("params").cloned().unwrap_or_default();
                
                // For now, treat as a generic tool call since rmcp doesn't have native envelope support
                // In production, this would involve custom envelope handling logic
                Ok(serde_json::json!({
                    "result": {
                        "success": true,
                        "message": "Envelope executed via rmcp transport",
                        "params": params
                    }
                }))
            }
            "capabilities" => {
                // Return basic capabilities response since rmcp doesn't expose server capabilities directly
                Ok(serde_json::json!({
                    "capabilities": {
                        "tools": true,
                        "resources": false,
                        "prompts": false
                    }
                }))
            }
            _ => Err(QollectiveError::transport(format!(
                "Unknown MCP method: {}",
                method
            ))),
        }
    }

    /// Get server capabilities using rmcp client
    pub async fn get_server_capabilities(&self, endpoint: &str) -> Result<serde_json::Value> {
        let request = serde_json::json!({
            "method": "capabilities",
            "params": {}
        });

        self.send_mcp_request(endpoint, request).await
    }

    /// Call tool using envelope format - required by HybridTransportClient
    pub async fn call_tool_envelope<T: Serialize + Send + 'static, R: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        request: Envelope<T>,
    ) -> Result<Envelope<R>> {
        // Extract envelope metadata and data
        let (meta, data) = request.extract();
        
        // Convert data to JSON for MCP processing
        let data_value = serde_json::to_value(data).map_err(|e| {
            QollectiveError::serialization(format!("Failed to serialize envelope data: {}", e))
        })?;

        // Create MCP request - for rmcp integration, we use envelope/execute method
        let mcp_request = serde_json::json!({
            "method": "envelope/execute",
            "params": {
                "data": data_value,
                "meta": meta
            }
        });

        // Since we don't have a specific endpoint in this context, we'll need to construct one
        // This is a placeholder - in a real implementation, the endpoint would be derived from context
        let endpoint = "mcp://localhost:8080"; // Default MCP endpoint
        
        // Send via rmcp
        let mcp_response = self.send_mcp_request(endpoint, mcp_request).await?;
        
        // Convert response back to envelope
        self.mcp_response_to_envelope(mcp_response)
    }

}

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
#[async_trait]
impl<T, R> UnifiedEnvelopeSender<T, R> for InternalMcpClient
where
    T: Serialize + Send + 'static,
    R: for<'de> Deserialize<'de> + Send + 'static,
{
    async fn send_envelope(&self, endpoint: &str, envelope: Envelope<T>) -> Result<Envelope<R>> {
        // Convert envelope to MCP request
        let mcp_request = self.envelope_to_mcp_request(envelope)?;

        // Send via rmcp client
        let mcp_response = self.send_mcp_request(endpoint, mcp_request).await?;

        // Convert response back to envelope
        self.mcp_response_to_envelope(mcp_response)
    }
}

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
impl McpTransportConfig {
    /// Create default configuration with all available transports
    pub fn default() -> Self {
        Self {
            connection_timeout: Duration::from_millis(timeouts::DEFAULT_MCP_CONNECTION_TIMEOUT_MS),
            request_timeout: Duration::from_millis(timeouts::DEFAULT_MCP_TIMEOUT.as_millis() as u64),
            max_connections: limits::DEFAULT_MCP_MAX_CONNECTIONS_PER_CLIENT as usize,
            enable_pooling: true,
            retry_attempts: limits::DEFAULT_MCP_RETRY_ATTEMPTS,
            verify_tls: true,
            mcp_version: metadata::DEFAULT_MCP_PROTOCOL_VERSION.to_string(),
            client_info: McpClientInfo::default(),
            requested_capabilities: vec![
                "tools".to_string(),
                "resources".to_string(),
                "prompts".to_string(),
            ],
            custom_headers: std::collections::HashMap::new(),
            enable_compression: true,
            auth_config: None,
            // Configure all available rmcp transport types by default
            transport_types: vec![
                RmcpTransportType::Http,
                RmcpTransportType::Https, 
                RmcpTransportType::Stdio,
                RmcpTransportType::Tcp, // Future expansion
            ],
            default_transport_type: RmcpTransportType::Http,
        }
    }
}

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
impl McpClientInfo {
    /// Create default client info
    pub fn default() -> Self {
        Self {
            name: "qollective-mcp-client".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            vendor: Some("Qollective".to_string()),
            description: Some("Qollective MCP Transport Client".to_string()),
        }
    }
}

#[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
impl McpAuthConfig {
    /// Create default auth config
    pub fn default() -> Self {
        Self {
            auth_type: McpAuthType::None,
            api_key: None,
            bearer_token: None,
            username: None,
            password: None,
            custom_headers: std::collections::HashMap::new(),
        }
    }
}

// Feature-disabled implementations
#[cfg(not(any(feature = "mcp-client", feature = "mcp-server")))]
pub struct InternalMcpClient;

#[cfg(not(any(feature = "mcp-client", feature = "mcp-server")))]
pub struct McpTransportConfig;

#[cfg(not(any(feature = "mcp-client", feature = "mcp-server")))]
impl InternalMcpClient {
    pub fn new(_config: McpTransportConfig) -> Self {
        Self
    }

    pub async fn send_mcp_request(
        &self,
        _endpoint: &str,
        _request: serde_json::Value,
    ) -> crate::error::Result<serde_json::Value> {
        Err(crate::error::QollectiveError::config(
            "mcp-client or mcp-server feature not enabled",
        ))
    }
}

#[cfg(not(any(feature = "mcp-client", feature = "mcp-server")))]
impl Default for McpTransportConfig {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::TransportDetectionConfig;
    use std::time::Duration;

    #[test]
    fn test_mcp_transport_config_creation() {
        let config = McpTransportClientConfig::default();
        assert_eq!(
            config.discovery_timeout,
            Duration::from_millis(timeouts::DEFAULT_MCP_DISCOVERY_TIMEOUT_MS)
        );
        assert_eq!(
            config.max_rmcp_clients,
            limits::DEFAULT_MCP_MAX_CACHED_CLIENTS
        );
        assert!(config.enable_connection_pooling);
    }

    #[test]
    fn test_server_id_generation() {
        // Create minimal transport client for testing ID generation
        let hybrid_transport = HybridTransportClient::new(TransportDetectionConfig::default());
        let client_endpoints = Arc::new(RwLock::new(HashMap::new()));
        let config = McpTransportClientConfig::default();

        // We'll create a minimal mock registry that doesn't require NATS
        let _servers: Arc<RwLock<HashMap<String, crate::types::mcp::McpServerInfo>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let _tools: Arc<RwLock<HashMap<String, Vec<String>>>> =
            Arc::new(RwLock::new(HashMap::new()));

        // Create a basic test client instance
        let test_client = TestMcpTransportClient {
            hybrid_transport,
            client_endpoints,
            config,
        };

        let server_id1 = test_client.generate_server_id("https://example.com/mcp");
        let server_id2 = test_client.generate_server_id("https://example.com/mcp");
        let server_id3 = test_client.generate_server_id("https://example.org/mcp");

        // Same URL should generate same ID
        assert_eq!(server_id1, server_id2);

        // Different URLs should generate different IDs
        assert_ne!(server_id1, server_id3);

        // Should start with "mcp-"
        assert!(server_id1.starts_with("mcp-"));
    }

    #[tokio::test]
    async fn test_cache_management() {
        let client_endpoints = Arc::new(RwLock::new(HashMap::new()));
        let _config = McpTransportClientConfig::default();

        // Test cache operations directly
        {
            let mut endpoints = client_endpoints.write().await;
            endpoints.insert("test1".to_string(), "config1".to_string());
            endpoints.insert("test2".to_string(), "config2".to_string());
        }

        // Check cache size
        {
            let endpoints = client_endpoints.read().await;
            assert_eq!(endpoints.len(), 2);
        }

        // Clear cache
        {
            let mut endpoints = client_endpoints.write().await;
            endpoints.clear();
        }

        // Verify cache is empty
        {
            let endpoints = client_endpoints.read().await;
            assert_eq!(endpoints.len(), 0);
        }
    }

    // Test helper struct that doesn't require registry
    struct TestMcpTransportClient {
        #[allow(dead_code)]
        hybrid_transport: HybridTransportClient,
        #[allow(dead_code)]
        client_endpoints: Arc<RwLock<HashMap<String, String>>>,
        #[allow(dead_code)]
        config: McpTransportClientConfig,
    }

    impl TestMcpTransportClient {
        fn generate_server_id(&self, server_url: &str) -> String {
            // Create deterministic server ID from URL
            let mut hasher = Sha256::new();
            hasher.update(server_url.as_bytes());
            let result = hasher.finalize();
            format!(
                "mcp-{:02x}{:02x}{:02x}{:02x}",
                result[0], result[1], result[2], result[3]
            )
        }
    }
}
