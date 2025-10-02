// ABOUTME: Binary entry point for holodeck-storybook MCP server using qollective framework
// ABOUTME: Starts qollective MCP server with real LLM-powered HolodeckStorybookServer integration

use shared_types::constants::network::{HOLODECK_STORYBOOK_PORT, DEFAULT_HOST};
use qollective::server::websocket::{WebSocketServer, WebSocketServerConfig};
use qollective::envelope::Context;
use qollective::types::mcp::McpData;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::error::{Result as QollectiveResult};
use holodeck_storybook::HolodeckStorybookServer;
use rmcp::model::CallToolResult;
use tracing::{info, warn, error};
use tracing_subscriber;
use std::sync::Arc;
use serde_json;
use async_trait::async_trait;

/// Bridge adapter that implements qollective ContextDataHandler for real LLM-powered storybook server
struct HolodeckStorybookMcpAdapter {
    /// Real LLM-powered storybook server from server.rs
    storybook_server: Arc<HolodeckStorybookServer>,
}

impl HolodeckStorybookMcpAdapter {
    async fn new() -> QollectiveResult<Self> {
        // Create the real LLM-powered storybook server with configuration
        let storybook_server = Arc::new(
            HolodeckStorybookServer::new_with_config_file().await
                .map_err(|e| qollective::error::QollectiveError::internal(format!("Failed to create storybook server: {}", e)))?
        );

        info!("🔖 Created LLM-powered HolodeckStorybookServer with dual-server architecture");

        Ok(Self {
            storybook_server,
        })
    }
}

#[async_trait]
impl ContextDataHandler<McpData, McpData> for HolodeckStorybookMcpAdapter {
    async fn handle(&self, context: Option<Context>, data: McpData) -> QollectiveResult<McpData> {
        info!("📨 HolodeckStorybookMcpAdapter received MCP data for processing with real LLM server");

        // Log the complete incoming MCP data for qollective demo
        info!("📦 QOLLECTIVE MCP DATA RECEIVED:");
        info!("📊 Tool Call: {}", if data.tool_call.is_some() { "Present" } else { "None" });
        info!("📊 Tool Response: {}", if data.tool_response.is_some() { "Present" } else { "None" });
        info!("📊 Discovery Data: {}", if data.discovery_data.is_some() { "Present" } else { "None" });

        // Handle MCP tool calls through the real storybook server
        if let Some(tool_call) = data.tool_call {
            info!("🔖 Executing real LLM-powered tool: '{}'", tool_call.params.name);
            if let Some(ref args) = tool_call.params.arguments {
                info!("🔧 Tool arguments: {}", serde_json::to_string_pretty(args).unwrap_or_else(|_| "Invalid args".to_string()));
            }

            // Delegate to the real LLM-powered storybook server based on tool name
            let tool_result = match tool_call.params.name.as_ref() {
                "health_check" => {
                    self.storybook_server.health_check().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Health check failed: {}", e)))?
                }
                "get_service_info" => {
                    self.storybook_server.get_service_info().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Service info failed: {}", e)))?
                }
                "serve_content" => {
                    info!("🔖 Routing to REAL LLM content serving - this will call configured LLM provider!");

                    // Convert MCP tool arguments to ContentRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to content request");

                        let story_id = args.get("story_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| uuid::Uuid::parse_str(s).ok())
                            .unwrap_or_else(|| uuid::Uuid::now_v7());

                        let content_type = args.get("content_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("story")
                            .to_string();

                        let include_validation = args.get("include_validation")
                            .and_then(|v| v.as_bool());

                        let include_realtime = args.get("include_realtime")
                            .and_then(|v| v.as_bool());

                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let user_id = args.get("user_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let request_id = args.get("request_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        info!("🔖 CALLING REAL LLM: Story ID='{}', Content Type='{}'", story_id, content_type);

                        // Call the actual LLM-powered content serving method
                        match self.storybook_server.serve_content(rmcp::handler::server::tool::Parameters(holodeck_storybook::server::ContentRequest {
                            tenant,
                            user_id,
                            request_id,
                            story_id: story_id.to_string(),
                            content_type,
                            include_validation,
                            include_realtime,
                        })).await {
                            Ok(result) => {
                                info!("🎉 REAL LLM CONTENT RESPONSE RECEIVED! Content length: {} items", result.content.len());

                                // Log the actual response content for visibility
                                info!("🔖 CONTENT RESPONSE DEBUG: {:?}", result);

                                result
                            }
                            Err(e) => {
                                error!("💥 Real LLM content serving failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Content serving failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("serve_content requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "manage_websocket" => {
                    info!("🔖 Routing to REAL LLM WebSocket management - this will configure real-time communication!");

                    // Convert MCP tool arguments to WebSocketRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to WebSocket request");

                        let session_id = args.get("session_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| uuid::Uuid::parse_str(s).ok())
                            .unwrap_or_else(|| uuid::Uuid::now_v7());

                        let user_id = args.get("user_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let event_types = args.get("event_types")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_else(|| vec!["session_events".to_string()]);

                        let connection_params = args.get("connection_params").cloned();

                        info!("🔖 CALLING REAL LLM: Session ID='{}', Event Types={:?}", session_id, event_types);

                        // Call the actual LLM-powered WebSocket management method
                        match self.storybook_server.manage_websocket(rmcp::handler::server::tool::Parameters(holodeck_storybook::server::WebSocketRequest {
                            session_id: session_id.to_string(),
                            user_id,
                            event_types,
                            connection_params,
                        })).await {
                            Ok(result) => {
                                info!("🎉 REAL LLM WEBSOCKET RESPONSE RECEIVED! Content length: {} items", result.content.len());

                                // Log the actual response content for visibility
                                info!("🔖 WEBSOCKET RESPONSE DEBUG: {:?}", result);

                                result
                            }
                            Err(e) => {
                                error!("💥 Real LLM WebSocket management failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("WebSocket management failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("manage_websocket requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "get_server_status" => {
                    info!("🔖 Routing to comprehensive server status check");

                    // Convert MCP tool arguments to ServerStatusRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        let detail_level = args.get("detail_level")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let include_services = args.get("include_services")
                            .and_then(|v| v.as_bool());

                        match self.storybook_server.get_server_status(rmcp::handler::server::tool::Parameters(holodeck_storybook::server::ServerStatusRequest {
                            detail_level,
                            include_services,
                        })).await {
                            Ok(result) => {
                                info!("🎉 Server status check completed successfully!");
                                result
                            }
                            Err(e) => {
                                error!("💥 Server status check failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Server status check failed: {}", e)));
                            }
                        }
                    } else {
                        // Call with default parameters
                        match self.storybook_server.get_server_status(rmcp::handler::server::tool::Parameters(holodeck_storybook::server::ServerStatusRequest {
                            detail_level: Some("basic".to_string()),
                            include_services: Some(true),
                        })).await {
                            Ok(result) => result,
                            Err(e) => {
                                error!("💥 Server status check failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Server status check failed: {}", e)));
                            }
                        }
                    }
                }
                _ => {
                    // For unmapped tools, return a helpful error
                    let error_content = rmcp::model::Content {
                        raw: rmcp::model::RawContent::text(format!("Tool '{}' is not mapped in the qollective MCP adapter. Available tools: health_check, get_service_info, serve_content, manage_websocket, get_server_status", tool_call.params.name)),
                        annotations: None,
                    };
                    CallToolResult {
                        content: vec![error_content],
                        is_error: Some(true),
                    }
                }
            };

            info!("✅ Real LLM tool execution completed successfully");

            // Log the response for qollective demo
            info!("📦 QOLLECTIVE MCP RESPONSE DATA:");
            info!("📋 Tool Result Content Count: {}", tool_result.content.len());
            info!("📋 Is Error: {:?}", tool_result.is_error);

            Ok(McpData {
                tool_call: None,
                tool_response: Some(tool_result),
                tool_registration: None,
                discovery_data: None,
            })
        } else {
            warn!("⚠️ Received MCP data without tool call");

            // Return empty response for non-tool-call requests
            Ok(McpData {
                tool_call: None,
                tool_response: None,
                tool_registration: None,
                discovery_data: None,
            })
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🔖 Starting Holodeck Storybook MCP Server with Qollective + Real LLM Integration");
    info!("📍 Service: holodeck-storybook");
    info!("🔧 Port: {}", HOLODECK_STORYBOOK_PORT);
    info!("🌐 Protocol: MCP over WebSocket (qollective envelope-first architecture)");
    info!("🤖 LLM Integration: Real LLM-powered content aggregation and server management from server.rs");
    info!("🏗️ Architecture: Dual-server (MCP + REST/WebSocket) for comprehensive content delivery");

    // Create the bridge adapter that connects qollective to our real LLM server
    let storybook_adapter = HolodeckStorybookMcpAdapter::new().await?;
    info!("✅ HolodeckStorybookMcpAdapter created with real LLM-powered dual-server architecture");

    // Configure the qollective WebSocket server
    let mut server_config = WebSocketServerConfig::default();
    server_config.base.bind_address = DEFAULT_HOST.to_string();
    server_config.base.port = HOLODECK_STORYBOOK_PORT;
    server_config.base.max_connections = 1000;

    // Create qollective WebSocket server
    let mut websocket_server = WebSocketServer::new(server_config).await?;
    info!("🌐 Qollective WebSocket Server created for MCP communication");
    info!("📦 Envelope integration enabled for qollective demo logging");

    // Register our MCP adapter at the /mcp endpoint
    websocket_server.receive_envelope_at("/mcp", storybook_adapter).await?;
    info!("📋 MCP handler registered at /mcp endpoint");

    // Note: The REST/WebSocket servers for external clients will be started separately
    // by the HolodeckStorybookServer when needed, following the dual-server architecture
    info!("🏗️ Dual-server architecture: MCP server (this process) + REST/WebSocket servers (managed by storybook server)");

    // Start the server with our real LLM-powered adapter
    info!("🚀 Starting qollective WebSocket server with real LLM integration...");

    // Start server and wait for shutdown
    tokio::select! {
        result = websocket_server.start() => {
            if let Err(e) = result {
                error!("❌ WebSocket server failed: {}", e);
                return Err(e.into());
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Shutdown signal received, stopping server...");
        }
    }

    Ok(())
}
