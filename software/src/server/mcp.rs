// ABOUTME: MCP server implementation providing standard MCP protocol endpoints
// ABOUTME: Handles tools, resources, and prompts for external MCP clients with envelope integration

//! MCP server implementation for Qollective framework.
//!
//! This module provides a comprehensive MCP server that:
//! - Handles standard MCP protocol requests (tools, resources, prompts)
//! - Integrates with Qollective envelope system for internal communication
//! - Serves external MCP clients through standard transports
//! - Provides tool execution, resource access, and prompt handling

use crate::config::mcp::McpServerRegistryConfig;
use crate::envelope::Envelope;
use crate::error::{QollectiveError, Result};
use crate::server::common::ServerConfig;
use crate::traits::catalog::{RegisteredResource, RegisteredTool, ServerCapability, ServerCatalog};
use crate::traits::handlers::ContextDataHandler;
use crate::traits::receivers::UnifiedEnvelopeReceiver;
use crate::traits::senders::UnifiedSender;
use crate::types::mcp::{McpData, McpDiscoveryData, McpServerInfo, ServerMetadata};
use async_trait::async_trait;
use rmcp::model::{
    CallToolRequest, CallToolResult, ClientCapabilities, Content, Implementation,
    InitializeRequest, InitializeResult, RawContent, ServerCapabilities, Tool,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// MCP REGISTRY SERVER FUNCTIONALITY (originally from mcp/registry.rs)
// ============================================================================

/// Server-side MCP server registry for managing registration and health monitoring
#[derive(Debug)]
pub struct McpServerRegistry {
    /// Transport client for communication
    transport: Arc<crate::transport::HybridTransportClient>,
    /// Configuration
    config: McpServerRegistryConfig,
    /// Map of server ID to server information
    servers: Arc<tokio::sync::RwLock<HashMap<String, McpServerInfo>>>,
    /// Map of tool name to list of server IDs that provide it
    tools: Arc<tokio::sync::RwLock<HashMap<String, Vec<String>>>>,
    /// Health monitoring task handle
    health_monitor_handle: Option<tokio::task::JoinHandle<()>>,
}

impl Clone for McpServerRegistry {
    fn clone(&self) -> Self {
        Self {
            transport: Arc::clone(&self.transport),
            config: self.config.clone(),
            servers: Arc::clone(&self.servers),
            tools: Arc::clone(&self.tools),
            health_monitor_handle: None, // Can't clone JoinHandle, new instances start fresh
        }
    }
}

impl McpServerRegistry {
    /// Create a new server-side MCP server registry
    pub fn new(
        config: McpServerRegistryConfig,
        transport: Arc<crate::transport::HybridTransportClient>,
    ) -> Result<Self> {
        // Initialize storage
        let servers = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let tools = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        Ok(Self {
            transport,
            config,
            servers,
            tools,
            health_monitor_handle: None,
        })
    }

    /// Register a new MCP server with the registry
    pub async fn register_server(&mut self, server_info: McpServerInfo) -> Result<String> {
        let server_id = server_info.server_id.clone();

        // Check if we've reached the maximum number of servers
        {
            let servers_guard = self.servers.read().await;
            if servers_guard.len() >= self.config.max_servers
                && !servers_guard.contains_key(&server_id)
            {
                return Err(QollectiveError::mcp_server_registration(format!(
                    "Maximum number of servers ({}) reached",
                    self.config.max_servers
                )));
            }
        }

        // Update tool mappings
        {
            let mut tools_guard = self.tools.write().await;
            for tool in &server_info.tools {
                let tool_name = tool.name.to_string();
                tools_guard
                    .entry(tool_name)
                    .or_insert_with(Vec::new)
                    .push(server_id.clone());
            }
        }

        // Store server information
        {
            let mut servers_guard = self.servers.write().await;
            servers_guard.insert(server_id.clone(), server_info.clone());
        }

        // Announce server registration via NATS
        self.announce_server_registration(&server_info).await?;

        Ok(format!("Successfully registered MCP server: {}", server_id))
    }

    /// Deregister an MCP server from the registry
    pub async fn deregister_server(&mut self, server_id: &str) -> Result<()> {
        // Get server info before removal
        let server_info = {
            let mut servers_guard = self.servers.write().await;
            servers_guard.remove(server_id)
        };

        if let Some(server_info) = server_info {
            // Update tool mappings
            let mut tools_guard = self.tools.write().await;
            for tool in &server_info.tools {
                let tool_name = tool.name.to_string();
                if let Some(server_list) = tools_guard.get_mut(&tool_name) {
                    server_list.retain(|id| id != server_id);
                    if server_list.is_empty() {
                        tools_guard.remove(&tool_name);
                    }
                }
            }

            // Announce server deregistration via NATS
            self.announce_server_deregistration(server_id).await?;
        }

        Ok(())
    }

    /// Update health status for a server
    pub async fn update_server_health(&mut self, server_id: &str, is_healthy: bool) -> Result<()> {
        let mut servers_guard = self.servers.write().await;

        if let Some(server_info) = servers_guard.get_mut(server_id) {
            server_info.health_status.is_healthy = is_healthy;
            server_info.health_status.last_check = std::time::SystemTime::now();

            if !is_healthy {
                server_info.health_status.error_count += 1;
            } else {
                server_info.health_status.error_count = 0;
            }
        } else {
            return Err(QollectiveError::mcp_server_not_found(server_id.to_string()));
        }

        Ok(())
    }

    /// Register a tool with the registry (for standalone tool registration)
    pub async fn register_tool(&mut self, tool: Tool, server_id: &str) -> Result<()> {
        let tool_name = tool.name.to_string();

        // Update tool mappings
        {
            let mut tools_guard = self.tools.write().await;
            tools_guard
                .entry(tool_name.clone())
                .or_insert_with(Vec::new)
                .push(server_id.to_string());
        }

        // If server exists, add tool to its list
        {
            let mut servers_guard = self.servers.write().await;
            if let Some(server_info) = servers_guard.get_mut(server_id) {
                server_info.tools.push(tool);
            } else {
                tracing::warn!("Tool registered for unknown server: {}", server_id);
            }
        }

        tracing::info!("Tool '{}' registered for server '{}'", tool_name, server_id);
        Ok(())
    }

    /// Start health monitoring for registered MCP servers
    pub async fn start_health_monitoring(&mut self) -> Result<()> {
        if !self.config.enable_async_connectivity {
            return Ok(()); // Health monitoring is disabled
        }

        let servers_clone = Arc::clone(&self.servers);
        let interval = self.config.health_check_interval;

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                // Get current server list
                let server_ids: Vec<String> = {
                    let servers_guard = servers_clone.read().await;
                    servers_guard.keys().cloned().collect()
                };

                // Check health for each server
                for server_id in server_ids {
                    // In a real implementation, this would ping each MCP server
                    // For now, we'll just update the last_check timestamp
                    let mut servers_guard = servers_clone.write().await;
                    if let Some(server_info) = servers_guard.get_mut(&server_id) {
                        server_info.health_status.last_check = std::time::SystemTime::now();
                        // In a real implementation: check actual server health
                        // server_info.health_status.is_healthy = ping_server(&server_id).await;
                    }
                }
            }
        });

        self.health_monitor_handle = Some(handle);
        Ok(())
    }

    /// Stop health monitoring
    pub fn stop_health_monitoring(&mut self) {
        if let Some(handle) = self.health_monitor_handle.take() {
            handle.abort();
        }
    }

    /// Announce server registration via transport
    async fn announce_server_registration(&self, server_info: &McpServerInfo) -> Result<()> {
        let announcement = ServerRegistrationAnnouncement {
            server_id: server_info.server_id.clone(),
            server_name: server_info.server_name.clone(),
            tools: server_info.tools.clone(),
            capabilities: server_info.capabilities.clone(),
            metadata: server_info.metadata.clone(),
        };

        let envelope = Envelope::new(crate::envelope::Meta::default(), announcement);

        // Use transport abstraction instead of direct NATS
        let announcement_endpoint = "qollective://registry/mcp/server/announce/v1";
        let _response: serde_json::Value = self
            .transport
            .send(announcement_endpoint, envelope)
            .await
            .map_err(|e| {
                QollectiveError::mcp_server_registration(format!(
                    "Failed to announce server registration: {}",
                    e
                ))
            })?;

        Ok(())
    }

    /// Announce server deregistration via transport
    async fn announce_server_deregistration(&self, server_id: &str) -> Result<()> {
        let announcement = ServerDeregistrationAnnouncement {
            server_id: server_id.to_string(),
        };

        let envelope = Envelope::new(crate::envelope::Meta::default(), announcement);

        // Use transport abstraction instead of direct NATS
        let deregistration_endpoint = "qollective://registry/mcp/server/deregister/v1";
        let _response: serde_json::Value = self
            .transport
            .send(deregistration_endpoint, envelope)
            .await
            .map_err(|e| {
                QollectiveError::mcp_server_registration(format!(
                    "Failed to announce server deregistration: {}",
                    e
                ))
            })?;

        Ok(())
    }

    /// Check if async connectivity is enabled
    pub fn is_async_enabled(&self) -> bool {
        self.config.enable_async_connectivity
    }

    /// Get async timeout configuration
    pub fn get_async_timeout(&self) -> std::time::Duration {
        self.config.async_timeout
    }
}

// rmcp ServerHandler trait implementation for MCP protocol compliance
impl rmcp::ServerHandler for McpServerRegistry {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            capabilities: rmcp::model::ServerCapabilities::default(),
            server_info: rmcp::model::Implementation {
                name: self.config.registry_name.clone(),
                version: "0.1.0".to_string(),
            },
            instructions: Some(
                "Qollective MCP Server Registry - Tracks MCP servers and tool capabilities"
                    .to_string(),
            ),
        }
    }
}

/// Server registration announcement message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerRegistrationAnnouncement {
    pub server_id: String,
    pub server_name: String,
    pub tools: Vec<rmcp::model::Tool>,
    pub capabilities: Vec<String>,
    pub metadata: ServerMetadata,
}

/// Server deregistration announcement message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerDeregistrationAnnouncement {
    pub server_id: String,
}

/// MCP server configuration
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Base server configuration
    pub base: ServerConfig,
    /// Registry configuration for tool/resource management
    pub registry_config: McpServerRegistryConfig,
    /// Server information for MCP protocol
    pub server_info: Implementation,
    /// Available tools
    pub tools: Vec<Tool>,
    /// Available resources
    pub resources: Vec<McpResource>,
    /// Available prompts
    pub prompts: Vec<McpPrompt>,
    /// Enable envelope integration
    pub enable_envelope_integration: bool,
}

/// MCP resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: Option<String>,
    /// MIME type
    pub mime_type: Option<String>,
}

/// MCP prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    /// Prompt name
    pub name: String,
    /// Prompt description
    pub description: Option<String>,
    /// Prompt template
    pub template: String,
    /// Required arguments
    pub arguments: Vec<McpPromptArgument>,
}

/// MCP prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    /// Argument name
    pub name: String,
    /// Argument description
    pub description: Option<String>,
    /// Whether argument is required
    pub required: bool,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            base: ServerConfig::default(),
            registry_config: McpServerRegistryConfig::default(),
            server_info: Implementation {
                name: "Qollective MCP Server".to_string(),
                version: "1.0.0".to_string(),
            },
            tools: vec![],
            resources: vec![],
            prompts: vec![],
            enable_envelope_integration: true,
        }
    }
}

/// MCP server providing standard MCP protocol endpoints
pub struct McpServer {
    /// Server configuration
    config: McpServerConfig,
    /// Transport client for communication
    transport: Arc<crate::transport::HybridTransportClient>,
    /// MCP server registry for managing tools and resources
    registry: Arc<McpServerRegistry>,
    /// Current session state
    sessions: Arc<tokio::sync::RwLock<HashMap<String, McpSession>>>,
}

/// MCP session state
#[derive(Debug, Clone)]
struct McpSession {
    /// Session ID
    session_id: String,
    /// Client capabilities
    client_capabilities: Option<ClientCapabilities>,
    /// Client info
    client_info: Option<Implementation>,
    /// Is initialized
    is_initialized: bool,
}

impl McpServer {
    /// Create a new MCP server with transport injection
    pub fn new(
        config: McpServerConfig,
        transport: Arc<crate::transport::HybridTransportClient>,
    ) -> Result<Self> {
        let registry = Arc::new(McpServerRegistry::new(
            config.registry_config.clone(),
            transport.clone(),
        )?);

        // Note: Tool registration is deferred to an async initialization method
        // since we cannot do async operations in a synchronous constructor
        tracing::info!(
            "MCP Server created with {} tools to register",
            config.tools.len()
        );

        Ok(Self {
            config,
            transport,
            registry,
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    /// Initialize the server and register tools (async initialization)
    pub async fn initialize(&mut self) -> Result<()> {
        // Use the Implementation name as the server_id
        let server_id = self.config.server_info.name.clone();

        // Register configured tools with the registry
        for tool in &self.config.tools {
            // Tools are already in the correct format (Tool)
            tracing::info!("Tool registration deferred for: {}, server id: {}", tool.name, server_id);
        }

        Ok(())
    }

    /// Get reference to transport for testing and advanced operations
    pub fn get_transport(&self) -> Option<Arc<crate::transport::HybridTransportClient>> {
        Some(self.transport.clone())
    }

    /// Handle MCP protocol initialize request
    pub async fn handle_initialize(
        &self,
        session_id: String,
        request: InitializeRequest,
    ) -> Result<InitializeResult> {
        // Create or update session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(
                session_id.clone(),
                McpSession {
                    session_id: session_id.clone(),
                    client_capabilities: Some(request.params.capabilities),
                    client_info: Some(request.params.client_info),
                    is_initialized: true,
                },
            );
        }

        // Return server capabilities
        Ok(InitializeResult {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            capabilities: ServerCapabilities {
                experimental: None,
                logging: None,
                prompts: Some(rmcp::model::PromptsCapability {
                    list_changed: Some(false),
                }),
                resources: Some(rmcp::model::ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                tools: Some(rmcp::model::ToolsCapability {
                    list_changed: Some(false),
                }),
                completions: Some(serde_json::Map::new()),
            },
            server_info: self.config.server_info.clone(),
            instructions: Some("Qollective MCP Server".to_string()),
        })
    }

    /// Handle MCP tools/list request
    pub async fn handle_tools_list(&self, _session_id: String) -> Result<Vec<Tool>> {
        Ok(self.config.tools.clone())
    }

    /// Handle MCP tools/call request
    pub async fn handle_tool_call(
        &self,
        session_id: String,
        request: CallToolRequest,
    ) -> Result<CallToolResult> {
        // Verify session is initialized
        {
            let sessions = self.sessions.read().await;
            let session = sessions.get(&session_id).ok_or_else(|| {
                QollectiveError::mcp_tool_execution("Session not found".to_string())
            })?;

            if !session.is_initialized {
                return Err(QollectiveError::mcp_tool_execution(
                    "Session not initialized".to_string(),
                ));
            }
        }

        // Find tool
        let tool = self
            .config
            .tools
            .iter()
            .find(|t| t.name == request.params.name);

        match tool {
            Some(found_tool) => {
                // Execute tool based on its name
                let tool_name = found_tool.name.as_ref();
                self.execute_tool(tool_name, request.params.arguments).await
            }
            None => {
                // Tool not found - return error response with content
                let error_message = format!("Tool not found: {}", request.params.name);
                let error_content = Content {
                    raw: RawContent::text(error_message),
                    annotations: None,
                };
                Ok(CallToolResult {
                    content: vec![error_content],
                    is_error: Some(true),
                    structured_content: None,
                })
            }
        }
    }

    /// Execute a specific tool and return the result
    async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult> {
        match tool_name {
            "echo" => {
                // Echo tool - return the input message
                let message = arguments
                    .as_ref()
                    .and_then(|args| args.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message provided");

                let response_text = format!("Echo: {}", message);
                let echo_content = Content {
                    raw: RawContent::text(response_text),
                    annotations: None,
                };

                Ok(CallToolResult {
                    content: vec![echo_content],
                    is_error: None, // Default is false
                    structured_content: None,
                })
            }
            "calculator" => {
                // Calculator tool - perform arithmetic operations
                match self.execute_calculator_tool(arguments).await {
                    Ok(result) => {
                        let calc_content = Content {
                            raw: RawContent::text(result),
                            annotations: None,
                        };
                        Ok(CallToolResult {
                            content: vec![calc_content],
                            is_error: None, // Default is false
                            structured_content: None,
                        })
                    }
                    Err(error_msg) => {
                        let error_content = Content {
                            raw: RawContent::text(error_msg),
                            annotations: None,
                        };
                        Ok(CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                            structured_content: None,
                        })
                    }
                }
            }
            _ => {
                // Unknown tool
                let error_message = format!("Unknown tool implementation: {}", tool_name);
                let error_content = Content {
                    raw: RawContent::text(error_message),
                    annotations: None,
                };
                Ok(CallToolResult {
                    content: vec![error_content],
                    is_error: Some(true),
                    structured_content: None,
                })
            }
        }
    }

    /// Execute calculator tool operations
    async fn execute_calculator_tool(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> std::result::Result<String, String> {
        let args = arguments.ok_or("No arguments provided for calculator".to_string())?;

        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'operation' parameter".to_string())?;

        let a = args
            .get("a")
            .and_then(|v| v.as_f64())
            .ok_or("Missing or invalid 'a' parameter".to_string())?;

        let b = args
            .get("b")
            .and_then(|v| v.as_f64())
            .ok_or("Missing or invalid 'b' parameter".to_string())?;

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                a / b
            }
            _ => return Err(format!("Unknown operation: {}", operation)),
        };

        Ok(format!(
            "Calculator result: {} {} {} = {}",
            a, operation, b, result
        ))
    }

    /// Handle MCP resources/list request
    pub async fn handle_resources_list(&self, _session_id: String) -> Result<Vec<McpResource>> {
        Ok(self.config.resources.clone())
    }

    /// Handle MCP resources/read request
    pub async fn handle_resource_read(&self, _session_id: String, uri: String) -> Result<Value> {
        // Find resource
        let resource = self
            .config
            .resources
            .iter()
            .find(|r| r.uri == uri)
            .ok_or_else(|| {
                QollectiveError::mcp_tool_execution(format!("Resource not found: {}", uri))
            })?;

        // Return resource content (simplified implementation)
        Ok(json!({
            "uri": resource.uri,
            "name": resource.name,
            "description": resource.description,
            "content": "Resource content placeholder",
            "mime_type": resource.mime_type
        }))
    }

    /// Handle MCP prompts/list request
    pub async fn handle_prompts_list(&self, _session_id: String) -> Result<Vec<McpPrompt>> {
        Ok(self.config.prompts.clone())
    }

    /// Handle MCP prompts/get request
    pub async fn handle_prompt_get(
        &self,
        _session_id: String,
        name: String,
        arguments: HashMap<String, Value>,
    ) -> Result<Value> {
        // Find prompt
        let prompt = self
            .config
            .prompts
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| {
                QollectiveError::mcp_tool_execution(format!("Prompt not found: {}", name))
            })?;

        // Process prompt template with arguments (simplified implementation)
        let mut processed_template = prompt.template.clone();
        for (key, value) in &arguments {
            let placeholder = format!("{{{}}}", key);
            let value_str = value.as_str().unwrap_or("");
            processed_template = processed_template.replace(&placeholder, value_str);
        }

        Ok(json!({
            "name": prompt.name,
            "description": prompt.description,
            "processed_template": processed_template,
            "arguments": arguments
        }))
    }

    /// Handle Qollective envelope-wrapped MCP request
    pub async fn handle_envelope_request(
        &self,
        envelope: Envelope<McpData>,
    ) -> Result<Envelope<McpData>> {
        let (meta, data) = envelope.extract();

        // Generate session ID from envelope metadata or create new one
        let session_id = meta.request_id.clone().unwrap_or_else(|| Uuid::now_v7());

        let response_data = if let Some(tool_call) = data.tool_call {
            // Ensure session exists for envelope-based tool calls
            self.ensure_session_exists(session_id.to_string()).await?;

            // Handle tool call
            let tool_result = self
                .handle_tool_call(session_id.to_string(), tool_call)
                .await?;
            McpData {
                tool_call: None,
                tool_response: Some(tool_result),
                tool_registration: None,
                discovery_data: None,
            }
        } else if let Some(discovery_data) = data.discovery_data {
            // Handle discovery request
            let response = match discovery_data.query_type.as_str() {
                "list_tools" => {
                    let tools = self.handle_tools_list(session_id.to_string()).await?;
                    McpDiscoveryData {
                        query_type: "list_tools_response".to_string(),
                        tools: Some(tools),
                        server_info: Some(self.create_server_info_from_config()),
                    }
                }
                _ => {
                    return Err(QollectiveError::mcp_tool_execution(format!(
                        "Unknown discovery query type: {}",
                        discovery_data.query_type
                    )));
                }
            };

            McpData {
                tool_call: None,
                tool_response: None,
                tool_registration: None,
                discovery_data: Some(response),
            }
        } else {
            return Err(QollectiveError::mcp_tool_execution(
                "No valid MCP data in envelope".to_string(),
            ));
        };

        // Create response metadata using the proper preservation utility
        // This follows the same pattern as WebSocket and gRPC servers for consistency
        let response_meta = crate::envelope::Meta::preserve_for_response(Some(&meta));
        Ok(Envelope::new(response_meta, response_data))
    }

    /// Get server configuration
    pub fn get_config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Get server registry
    pub fn get_registry(&self) -> Arc<McpServerRegistry> {
        Arc::clone(&self.registry)
    }

    /// Add tool to server
    pub async fn add_tool(&mut self, tool: Tool) -> Result<()> {
        // Add tool directly to config since tools is Vec<Tool>
        self.config.tools.push(tool.clone());

        // Update registry with the new tool
        // Note: registry is behind Arc, so we can't mutate it directly here
        // This would require the registry to be behind Arc<Mutex<>> or similar
        // For now, log that the tool was added to config
        tracing::info!("Tool '{}' added to server configuration", tool.name);

        Ok(())
    }

    /// Create McpServerInfo from the server configuration
    fn create_server_info_from_config(&self) -> McpServerInfo {
        use crate::types::mcp::{HealthStatus, ServerMetadata};

        // Create default metadata from Implementation
        let metadata = ServerMetadata {
            description: None,
            version: self.config.server_info.version.clone(),
            contact: None,
            documentation_url: None,
            tags: Vec::new(),
        };

        // Create default health status
        let health_status = HealthStatus {
            is_healthy: true,
            last_check: std::time::SystemTime::now(),
            response_time: std::time::Duration::from_millis(0),
            error_count: 0,
            uptime: std::time::Duration::from_secs(0),
        };

        McpServerInfo {
            server_id: self.config.server_info.name.clone(),
            server_name: self.config.server_info.name.clone(),
            tools: self.config.tools.clone(),
            capabilities: Vec::new(), // Default empty capabilities
            metadata,
            async_config: None, // Default no async config
            health_status,
        }
    }

    /// Add resource to server
    pub async fn add_resource(&mut self, resource: McpResource) -> Result<()> {
        self.config.resources.push(resource);
        Ok(())
    }

    /// Add prompt to server
    pub async fn add_prompt(&mut self, prompt: McpPrompt) -> Result<()> {
        self.config.prompts.push(prompt);
        Ok(())
    }

    /// Get active sessions count
    pub async fn get_active_sessions_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Remove session
    pub async fn remove_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    /// Ensure a session exists for envelope-based requests
    async fn ensure_session_exists(&self, session_id: String) -> Result<()> {
        let sessions = self.sessions.read().await;
        if sessions.contains_key(&session_id) {
            return Ok(());
        }
        drop(sessions); // Release read lock

        // Create new session
        let mut sessions = self.sessions.write().await;
        // Double-check pattern to avoid race condition
        if !sessions.contains_key(&session_id) {
            sessions.insert(
                session_id.clone(),
                McpSession {
                    session_id: session_id.clone(),
                    client_capabilities: None, // Will be set when client initializes properly
                    client_info: None,         // Will be set when client initializes properly
                    is_initialized: true,      // Auto-initialize for envelope-based requests
                },
            );
        }
        Ok(())
    }
}

impl Clone for McpServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            transport: Arc::clone(&self.transport),
            registry: Arc::clone(&self.registry),
            sessions: Arc::clone(&self.sessions),
        }
    }
}

// ============================================================================
// UNIFIED ENVELOPE RECEIVER IMPLEMENTATION (Step 24 Phase 3)
// ============================================================================

/// Adapter to bridge ContextDataHandler to MCP envelope handling
#[allow(dead_code)] // Infrastructure for future MCP envelope integration
struct McpEnvelopeAdapter<T, R, H>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + Send + 'static,
    H: ContextDataHandler<T, R>,
{
    handler: H,
    _phantom_t: std::marker::PhantomData<T>,
    _phantom_r: std::marker::PhantomData<R>,
}

impl<T, R, H> McpEnvelopeAdapter<T, R, H>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + Send + 'static,
    H: ContextDataHandler<T, R>,
{
    #[allow(dead_code)] // Used by McpEnvelopeAdapter infrastructure
    fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom_t: std::marker::PhantomData,
            _phantom_r: std::marker::PhantomData,
        }
    }
}

// rmcp ServerHandler trait implementation for MCP protocol compliance
#[cfg(feature = "mcp-server")]
impl rmcp::ServerHandler for McpServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability { list_changed: Some(true) }),
                resources: Some(rmcp::model::ResourcesCapability { list_changed: Some(true), subscribe: Some(false) }),
                prompts: Some(rmcp::model::PromptsCapability { list_changed: Some(true) }),
                logging: None,
                completions: None,
                experimental: None,
            },
            server_info: self.config.server_info.clone(),
            instructions: Some(format!(
                "Qollective MCP Server - {} tools, {} resources, {} prompts available",
                self.config.tools.len(),
                self.config.resources.len(),
                self.config.prompts.len()
            )),
        }
    }
}

#[async_trait]
impl UnifiedEnvelopeReceiver for McpServer {
    /// Receive and process envelopes for MCP protocol.
    ///
    /// This implementation handles generic envelope processing for MCP servers,
    /// extracting context and routing to appropriate handlers.
    async fn receive_envelope<T, R, H>(&mut self, _handler: H) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
        H: ContextDataHandler<T, R> + Send + Sync + 'static,
    {
        // For MCP servers, we need to implement a basic envelope receiver
        // This is a simplified implementation - in a real scenario, this would
        // integrate with an actual transport layer (NATS, HTTP, etc.)

        // For now, return an error indicating this needs transport integration
        Err(QollectiveError::mcp_tool_execution(
            "MCP server envelope receiving requires transport layer integration".to_string(),
        ))
    }

    /// Receive and process envelopes at a specific MCP route.
    ///
    /// This implementation handles MCP-specific route patterns like:
    /// - "tools/list" - List available tools
    /// - "tools/call" - Execute a tool
    /// - "resources/list" - List available resources
    /// - "resources/read" - Read a resource
    /// - "prompts/list" - List available prompts
    /// - "prompts/get" - Get a prompt
    async fn receive_envelope_at<T, R, H>(&mut self, route: &str, _handler: H) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
        H: ContextDataHandler<T, R> + Send + Sync + 'static,
    {
        // Validate that the route is an MCP route
        let valid_mcp_routes = [
            "tools/list",
            "tools/call",
            "resources/list",
            "resources/read",
            "prompts/list",
            "prompts/get",
            "initialize",
            "ping",
        ];

        if !valid_mcp_routes.contains(&route) {
            return Err(QollectiveError::mcp_tool_execution(format!(
                "Invalid MCP route: {}. Valid routes: {:?}",
                route, valid_mcp_routes
            )));
        }

        // For now, return an error indicating this needs transport integration
        // In a real implementation, this would:
        // 1. Set up route-specific handlers
        // 2. Extract envelopes from transport messages
        // 3. Create context from envelope metadata
        // 4. Route to appropriate MCP method based on route
        Err(QollectiveError::mcp_tool_execution(format!(
            "MCP server route '{}' requires transport layer integration",
            route
        )))
    }
}

// ============================================================================
// SERVER CATALOG IMPLEMENTATION (Step 24 Phase 4)
// ============================================================================

#[async_trait]
impl ServerCatalog for McpServer {
    /// List all server capabilities for MCP
    async fn list_capabilities(&self) -> Result<Vec<ServerCapability>> {
        let mut capabilities = Vec::new();

        // MCP Tools capability
        let tools_capability = ServerCapability {
            name: "tools".to_string(),
            version: "1.0".to_string(),
            enabled: true,
            metadata: std::collections::HashMap::from([
                (
                    "count".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.config.tools.len())),
                ),
                ("supports_call".to_string(), serde_json::Value::Bool(true)),
                ("supports_list".to_string(), serde_json::Value::Bool(true)),
            ]),
        };
        capabilities.push(tools_capability);

        // MCP Resources capability
        let resources_capability = ServerCapability {
            name: "resources".to_string(),
            version: "1.0".to_string(),
            enabled: true,
            metadata: std::collections::HashMap::from([
                (
                    "count".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        self.config.resources.len(),
                    )),
                ),
                ("supports_read".to_string(), serde_json::Value::Bool(true)),
                ("supports_list".to_string(), serde_json::Value::Bool(true)),
            ]),
        };
        capabilities.push(resources_capability);

        // MCP Prompts capability
        let prompts_capability = ServerCapability {
            name: "prompts".to_string(),
            version: "1.0".to_string(),
            enabled: true,
            metadata: std::collections::HashMap::from([
                (
                    "count".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.config.prompts.len())),
                ),
                ("supports_get".to_string(), serde_json::Value::Bool(true)),
                ("supports_list".to_string(), serde_json::Value::Bool(true)),
            ]),
        };
        capabilities.push(prompts_capability);

        // MCP Sampling capability (optional)
        let sampling_capability = ServerCapability {
            name: "sampling".to_string(),
            version: "1.0".to_string(),
            enabled: false, // Not implemented yet
            metadata: std::collections::HashMap::from([(
                "supported".to_string(),
                serde_json::Value::Bool(false),
            )]),
        };
        capabilities.push(sampling_capability);

        Ok(capabilities)
    }

    /// List all registered tools
    async fn list_registered_tools(&self) -> Result<Vec<RegisteredTool>> {
        let mut registered_tools = Vec::new();

        for tool in &self.config.tools {
            let registered_tool = RegisteredTool {
                name: tool.name.to_string(),
                description: Some(tool.description.as_ref().map(|d| d.to_string()).unwrap_or_else(|| "No description".to_string())),
                input_schema: Some(serde_json::to_value(&*tool.input_schema)?),
                metadata: std::collections::HashMap::from([
                    (
                        "source".to_string(),
                        serde_json::Value::String("mcp_server".to_string()),
                    ),
                    (
                        "protocol".to_string(),
                        serde_json::Value::String("mcp".to_string()),
                    ),
                ]),
            };
            registered_tools.push(registered_tool);
        }

        Ok(registered_tools)
    }

    /// List all registered resources
    async fn list_registered_resources(&self) -> Result<Vec<RegisteredResource>> {
        let mut registered_resources = Vec::new();

        for resource in &self.config.resources {
            let registered_resource = RegisteredResource {
                uri: resource.uri.clone(),
                name: resource.name.clone(),
                mime_type: resource.mime_type.clone(),
                metadata: std::collections::HashMap::from([
                    (
                        "description".to_string(),
                        serde_json::Value::String(
                            resource
                                .description
                                .clone()
                                .unwrap_or_else(|| "No description".to_string()),
                        ),
                    ),
                    (
                        "source".to_string(),
                        serde_json::Value::String("mcp_server".to_string()),
                    ),
                ]),
            };
            registered_resources.push(registered_resource);
        }

        Ok(registered_resources)
    }

    /// Get server metadata
    async fn get_server_metadata(
        &self,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        let mut metadata = std::collections::HashMap::new();

        metadata.insert(
            "name".to_string(),
            serde_json::Value::String(self.config.server_info.name.clone()),
        );
        metadata.insert(
            "version".to_string(),
            serde_json::Value::String(self.config.server_info.version.clone()),
        );
        metadata.insert(
            "protocol".to_string(),
            serde_json::Value::String("mcp".to_string()),
        );
        metadata.insert(
            "envelope_integration".to_string(),
            serde_json::Value::Bool(self.config.enable_envelope_integration),
        );

        // Add capability counts
        metadata.insert(
            "tools_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.config.tools.len())),
        );
        metadata.insert(
            "resources_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.config.resources.len())),
        );
        metadata.insert(
            "prompts_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.config.prompts.len())),
        );

        // Add session info
        let sessions_count = self.sessions.read().await.len();
        metadata.insert(
            "active_sessions".to_string(),
            serde_json::Value::Number(serde_json::Number::from(sessions_count)),
        );

        // Add registry information
        metadata.insert(
            "registry_name".to_string(),
            serde_json::Value::String(self.registry.config.registry_name.clone()),
        );
        metadata.insert(
            "max_servers".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.registry.config.max_servers)),
        );

        Ok(metadata)
    }

    /// Check if a specific capability is supported
    async fn supports_capability(&self, capability_name: &str) -> Result<bool> {
        let capabilities = self.list_capabilities().await?;
        Ok(capabilities
            .iter()
            .any(|cap| cap.name == capability_name && cap.enabled))
    }

    /// Get capability details
    async fn get_capability_details(
        &self,
        capability_name: &str,
    ) -> Result<Option<ServerCapability>> {
        let capabilities = self.list_capabilities().await?;
        Ok(capabilities
            .into_iter()
            .find(|cap| cap.name == capability_name))
    }
}

#[cfg(test)]
#[cfg(any(feature = "mcp-server"))]
mod tests {
    use super::*;
    use crate::envelope::{Context, Envelope, Meta};
    use crate::traits::catalog::{
        RegisteredResource, RegisteredTool, ServerCapability, ServerCatalog,
    };
    use crate::traits::handlers::ContextDataHandler;
    use crate::traits::receivers::UnifiedEnvelopeReceiver;
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Test data structures
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRequest {
        message: String,
        id: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestResponse {
        result: String,
        status: u32,
    }

    // Mock handler for testing
    #[derive(Debug, Clone)]
    struct MockHandler {
        call_count: Arc<Mutex<u32>>,
        response: TestResponse,
    }

    impl MockHandler {
        fn new(response: TestResponse) -> Self {
            Self {
                call_count: Arc::new(Mutex::new(0)),
                response,
            }
        }

        async fn get_call_count(&self) -> u32 {
            *self.call_count.lock().await
        }
    }

    #[async_trait]
    impl ContextDataHandler<TestRequest, TestResponse> for MockHandler {
        async fn handle(
            &self,
            _context: Option<Context>,
            _data: TestRequest,
        ) -> crate::error::Result<TestResponse> {
            let mut count = self.call_count.lock().await;
            *count += 1;
            Ok(self.response.clone())
        }
    }

    // ============================================================================
    // TEST HELPERS
    // ============================================================================

    fn create_mock_transport() -> Arc<crate::transport::HybridTransportClient> {
        Arc::new(crate::transport::HybridTransportClient::new(
            crate::transport::TransportDetectionConfig::default(),
        ))
    }

    fn create_test_server() -> McpServer {
        let transport = create_mock_transport();
        let config = McpServerConfig::default();
        McpServer::new(config, transport).unwrap()
    }

    fn create_server_with_tools() -> McpServer {
        let transport = create_mock_transport();
        let config = McpServerConfig::default();

        // For now, just create a basic server without complex tools configuration
        // to avoid rmcp API type mismatches
        McpServer::new(config, transport).unwrap()
    }

    #[tokio::test]
    async fn test_mcp_server_implements_unified_envelope_receiver() {
        // Create mock transport for testing
        let transport = Arc::new(crate::transport::HybridTransportClient::new(
            crate::transport::TransportDetectionConfig::default(),
        ));
        let config = McpServerConfig::default();
        let mut server = McpServer::new(config, transport).unwrap();

        // This should compile once UnifiedEnvelopeReceiver is implemented
        let handler = MockHandler::new(TestResponse {
            result: "test result".to_string(),
            status: 200,
        });

        // This should fail until UnifiedEnvelopeReceiver is implemented
        let result = server.receive_envelope(handler).await;
        assert!(
            result.is_err(),
            "Should fail without UnifiedEnvelopeReceiver implementation"
        );
    }

    #[tokio::test]
    async fn test_mcp_server_receive_envelope_at_route() {
        // Create mock transport for testing
        let transport = Arc::new(crate::transport::HybridTransportClient::new(
            crate::transport::TransportDetectionConfig::default(),
        ));
        let config = McpServerConfig::default();
        let mut server = McpServer::new(config, transport).unwrap();

        let handler = MockHandler::new(TestResponse {
            result: "tool result".to_string(),
            status: 200,
        });

        // This should fail until receive_envelope_at is implemented
        let result = server.receive_envelope_at("tools/list", handler).await;
        assert!(
            result.is_err(),
            "Should fail without receive_envelope_at implementation"
        );
    }

    #[tokio::test]
    async fn test_mcp_server_handles_tool_routes() {
        // Create mock transport for testing
        let transport = Arc::new(crate::transport::HybridTransportClient::new(
            crate::transport::TransportDetectionConfig::default(),
        ));
        let config = McpServerConfig::default();
        let mut server = McpServer::new(config, transport).unwrap();

        let handler = MockHandler::new(TestResponse {
            result: "tool execution result".to_string(),
            status: 200,
        });

        // Test MCP-specific routes
        let routes = vec![
            "tools/list",
            "tools/call",
            "resources/list",
            "resources/read",
            "prompts/list",
            "prompts/get",
        ];

        for route in routes {
            let result = server.receive_envelope_at(route, handler.clone()).await;
            assert!(
                result.is_err(),
                "Route {} should fail without implementation",
                route
            );
        }
    }

    #[tokio::test]
    async fn test_mcp_server_envelope_context_integration() {
        // Create mock transport for testing
        let transport = Arc::new(crate::transport::HybridTransportClient::new(
            crate::transport::TransportDetectionConfig::default(),
        ));
        let config = McpServerConfig::default();
        let mut server = McpServer::new(config, transport).unwrap();

        let handler = MockHandler::new(TestResponse {
            result: "context handled".to_string(),
            status: 200,
        });

        // Should handle context extraction from MCP envelopes
        let result = server.receive_envelope(handler).await;
        assert!(result.is_err(), "Should fail without context integration");
    }

    #[tokio::test]
    async fn test_envelope_request_handling() {
        // Create mock transport for testing
        let transport = Arc::new(crate::transport::HybridTransportClient::new(
            crate::transport::TransportDetectionConfig::default(),
        ));
        let config = McpServerConfig::default();
        let server = McpServer::new(config, transport).unwrap();

        // Test tool call through envelope - using correct rmcp API structure
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: rmcp::model::CallToolRequestParam {
                name: "test_tool".to_string().into(),
                arguments: Some({
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "param".to_string(),
                        serde_json::Value::String("value".to_string()),
                    );
                    map
                }),
            },
            extensions: rmcp::model::Extensions::default(),
        };

        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let envelope = Envelope::new(Meta::default(), mcp_data);
        let result = server.handle_envelope_request(envelope).await;

        // Should return success with an error flag for tool not found
        assert!(result.is_ok());
        let response = result.unwrap();
        let (_, response_data) = response.extract();

        // Verify we got a tool response with error flag
        assert!(response_data.tool_response.is_some());
        let tool_result = response_data.tool_response.unwrap();
        assert_eq!(tool_result.is_error, Some(true));
        assert!(!tool_result.content.is_empty());

        // Verify error message mentions the tool name
        if let Some(content) = tool_result.content.first() {
            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("test_tool"));
                assert!(text_content.text.contains("not found"));
            }
        }
    }

    #[tokio::test]
    async fn test_server_configuration() {
        let mut config = McpServerConfig::default();

        // Create tool using correct rmcp API structure
        let tool = Tool {
            name: "test_tool".into(),
            description: Some("A test tool".into()),
            input_schema: std::sync::Arc::new({
                let mut schema = serde_json::Map::new();
                schema.insert(
                    "type".to_string(),
                    serde_json::Value::String("object".to_string()),
                );
                schema
            }),
            output_schema: None,
            annotations: None,
        };

        config.tools.push(tool);

        // Create mock transport for testing
        let transport = Arc::new(crate::transport::HybridTransportClient::new(
            crate::transport::TransportDetectionConfig::default(),
        ));
        let server = McpServer::new(config, transport).unwrap();
        let server_config = server.get_config();

        assert_eq!(server_config.tools.len(), 1);
        assert_eq!(server_config.tools[0].name, "test_tool");
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let server = create_test_server();

        // Create session through initialization
        let init_request = InitializeRequest {
            method: rmcp::model::InitializeResultMethod::default(),
            params: rmcp::model::InitializeRequestParam {
                protocol_version: rmcp::model::ProtocolVersion::default(),
                capabilities: ClientCapabilities::default(),
                client_info: Implementation {
                    name: "test_client".to_string(),
                    version: "1.0.0".to_string(),
                },
            },
            extensions: rmcp::model::Extensions::default(),
        };

        let result = server
            .handle_initialize("session_test".to_string(), init_request)
            .await;
        assert!(result.is_ok());

        // Session should now exist and be usable for tool calls
        let tools = server.handle_tools_list("session_test".to_string()).await;
        assert!(tools.is_ok());

        // Remove session
        let remove_result = server.remove_session("session_test").await;
        assert!(remove_result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_sessions() {
        let server = create_test_server();

        let init_request = InitializeRequest {
            method: rmcp::model::InitializeResultMethod::default(),
            params: rmcp::model::InitializeRequestParam {
                protocol_version: rmcp::model::ProtocolVersion::default(),
                capabilities: ClientCapabilities::default(),
                client_info: Implementation {
                    name: "test_client".to_string(),
                    version: "1.0.0".to_string(),
                },
            },
            extensions: rmcp::model::Extensions::default(),
        };

        // Create multiple sessions
        let result1 = server
            .handle_initialize("session_1".to_string(), init_request.clone())
            .await;
        let result2 = server
            .handle_initialize("session_2".to_string(), init_request.clone())
            .await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Both sessions should be independent
        let tools1 = server.handle_tools_list("session_1".to_string()).await;
        let tools2 = server.handle_tools_list("session_2".to_string()).await;

        assert!(tools1.is_ok());
        assert!(tools2.is_ok());
    }

    #[tokio::test]
    async fn test_prompt_get_nonexistent() {
        let server = create_server_with_tools();

        let arguments = HashMap::new();
        let result = server
            .handle_prompt_get(
                "session_1".to_string(),
                "nonexistent_prompt".to_string(),
                arguments,
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Prompt not found"));
    }

    #[tokio::test]
    async fn test_invalid_envelope_data() {
        let server = create_test_server();

        // Empty MCP data should be handled gracefully
        let mcp_data = McpData {
            tool_call: None,
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let envelope = Envelope::new(Meta::default(), mcp_data);
        let result = server.handle_envelope_request(envelope).await;

        // Should return an error for invalid data
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_server_cloning() {
        let server = create_server_with_tools();
        let cloned_server = server.clone();

        // Both servers should have the same configuration
        assert_eq!(
            server.get_config().tools.len(),
            cloned_server.get_config().tools.len()
        );
        assert_eq!(
            server.get_config().server_info.name,
            cloned_server.get_config().server_info.name
        );
    }
}
