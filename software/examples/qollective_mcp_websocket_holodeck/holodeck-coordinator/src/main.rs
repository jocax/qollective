// ABOUTME: Binary entry point for holodeck-coordinator MCP server using qollective framework
// ABOUTME: Starts qollective MCP server with real LLM-powered HolodeckCoordinatorServer integration

use shared_types::constants::network::{HOLODECK_COORDINATOR_PORT, DEFAULT_HOST};
use qollective::server::websocket::{WebSocketServer, WebSocketServerConfig};
use qollective::envelope::Context;
use qollective::types::mcp::McpData;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::error::{Result as QollectiveResult};
use holodeck_coordinator::HolodeckCoordinatorServer;
use rmcp::model::CallToolResult;
use tracing::{info, warn, error};
use tracing_subscriber;
use std::sync::Arc;
use serde_json;
use async_trait::async_trait;

/// Bridge adapter that implements qollective ContextDataHandler for real LLM-powered coordinator server
struct CoordinatorMcpAdapter {
    /// Real LLM-powered coordinator server from server.rs
    coordinator_server: Arc<HolodeckCoordinatorServer>,
}

impl CoordinatorMcpAdapter {
    async fn new() -> QollectiveResult<Self> {
        // Create the real LLM-powered coordinator server with configuration
        let coordinator_server = Arc::new(
            HolodeckCoordinatorServer::new_with_config_file().await
                .map_err(|e| qollective::error::QollectiveError::internal(format!("Failed to create coordinator server: {}", e)))?
        );
        
        info!("🎭 Created LLM-powered HolodeckCoordinatorServer with configurable provider integration");
        
        Ok(Self {
            coordinator_server,
        })
    }
}

#[async_trait]
impl ContextDataHandler<McpData, McpData> for CoordinatorMcpAdapter {
    async fn handle(&self, context: Option<Context>, data: McpData) -> QollectiveResult<McpData> {
        info!("📨 CoordinatorMcpAdapter received MCP data for processing with configurable LLM server");
        
        // Log the complete incoming MCP data for qollective demo
        info!("📦 QOLLECTIVE MCP DATA RECEIVED:");
        info!("📊 Tool Call: {}", if data.tool_call.is_some() { "Present" } else { "None" });
        info!("📊 Tool Response: {}", if data.tool_response.is_some() { "Present" } else { "None" });
        info!("📊 Discovery Data: {}", if data.discovery_data.is_some() { "Present" } else { "None" });
        
        // Handle MCP tool calls through the real coordinator server
        if let Some(tool_call) = data.tool_call {
            info!("🎭 Executing real LLM-powered orchestration tool: '{}'", tool_call.params.name);
            if let Some(ref args) = tool_call.params.arguments {
                info!("🔧 Tool arguments: {}", serde_json::to_string_pretty(args).unwrap_or_else(|_| "Invalid args".to_string()));
            }
            
            // Delegate to the real LLM-powered coordinator server based on tool name
            let tool_result = match tool_call.params.name.as_ref() {
                "health_check" => {
                    self.coordinator_server.health_check().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Health check failed: {}", e)))?
                }
                "get_service_info" => {
                    self.coordinator_server.get_service_info().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Service info failed: {}", e)))?
                }
                "create_holodeck_session" => {
                    info!("🎭 Routing to holodeck session orchestration");
                    
                    // Convert MCP tool arguments to CreateHolodeckSessionRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to session creation request");
                        
                        let session_name = args.get("session_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Enterprise Bridge Session")
                            .to_string();
                        
                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let user_id = args.get("user_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let request_id = args.get("request_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        info!("🎯 ORCHESTRATING HOLODECK SESSION: Session='{}' with LLM coordination", session_name);
                        
                        // Call the actual LLM-powered session orchestration method
                        match self.coordinator_server.create_holodeck_session(rmcp::handler::server::tool::Parameters(holodeck_coordinator::server::CreateHolodeckSessionRequest {
                            tenant,
                            user_id,
                            request_id,
                            session_name,
                        })).await {
                            Ok(result) => {
                                info!("🎉 Holodeck session orchestration completed! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 Holodeck session orchestration failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Session orchestration failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("create_holodeck_session requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "check_system_health" => {
                    info!("🎭 Routing to system health aggregation with LLM analysis");
                    
                    // Convert MCP tool arguments to SystemHealthRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        let include_details = args.get("include_details")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true);
                        
                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        info!("🔍 ANALYZING SYSTEM HEALTH: Details={} with LLM coordination", include_details);
                        
                        match self.coordinator_server.check_system_health(rmcp::handler::server::tool::Parameters(holodeck_coordinator::server::SystemHealthRequest {
                            tenant,
                            include_details: Some(include_details),
                        })).await {
                            Ok(result) => {
                                info!("🎉 System health analysis completed! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 System health analysis failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Health analysis failed: {}", e)));
                            }
                        }
                    } else {
                        // Default request with basic health check
                        match self.coordinator_server.check_system_health(rmcp::handler::server::tool::Parameters(holodeck_coordinator::server::SystemHealthRequest {
                            tenant: None,
                            include_details: Some(true),
                        })).await {
                            Ok(result) => {
                                info!("🎉 System health analysis completed (default)! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 System health analysis failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Health analysis failed: {}", e)));
                            }
                        }
                    }
                }
                "discover_servers" => {
                    info!("🎭 Routing to server discovery with LLM coordination");
                    
                    // Convert MCP tool arguments to ServerDiscoveryRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        let discovery_mode = args.get("discovery_mode")
                            .and_then(|v| v.as_str())
                            .unwrap_or("automatic")
                            .to_string();
                        
                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        info!("🔍 DISCOVERING SERVERS: Mode='{}' with LLM coordination", discovery_mode);
                        
                        match self.coordinator_server.discover_servers(rmcp::handler::server::tool::Parameters(holodeck_coordinator::server::ServerDiscoveryRequest {
                            tenant,
                            discovery_mode,
                        })).await {
                            Ok(result) => {
                                info!("🎉 Server discovery completed! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 Server discovery failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Discovery failed: {}", e)));
                            }
                        }
                    } else {
                        // Default discovery request  
                        match self.coordinator_server.discover_servers(rmcp::handler::server::tool::Parameters(holodeck_coordinator::server::ServerDiscoveryRequest {
                            tenant: None,
                            discovery_mode: "automatic".to_string(),
                        })).await {
                            Ok(result) => {
                                info!("🎉 Server discovery completed (default)! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 Server discovery failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Discovery failed: {}", e)));
                            }
                        }
                    }
                }
                "orchestrate_validation" => {
                    info!("🎭 Routing to distributed validation orchestration with LLM coordination");
                    
                    // Convert MCP tool arguments to ValidationOrchestrationRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        let content_id = args.get("content_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("default-content")
                            .to_string();
                        
                        let validation_type = args.get("validation_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("comprehensive")
                            .to_string();
                        
                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        info!("🔍 ORCHESTRATING VALIDATION: Content='{}', Type='{}' with LLM coordination", content_id, validation_type);
                        
                        match self.coordinator_server.orchestrate_validation(rmcp::handler::server::tool::Parameters(holodeck_coordinator::server::ValidationOrchestrationRequest {
                            tenant,
                            content_id,
                            validation_type,
                        })).await {
                            Ok(result) => {
                                info!("🎉 Validation orchestration completed! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 Validation orchestration failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Validation orchestration failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("orchestrate_validation requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                _ => {
                    // For unmapped tools, return a helpful error
                    let error_content = rmcp::model::Content {
                        raw: rmcp::model::RawContent::text(format!("Tool '{}' is not mapped in the coordinator MCP adapter. Available tools: health_check, get_service_info, create_holodeck_session, check_system_health, discover_servers, orchestrate_validation", tool_call.params.name)),
                        annotations: None,
                    };
                    CallToolResult {
                        content: vec![error_content],
                        is_error: Some(true),
                    }
                }
            };
            
            info!("✅ Real LLM orchestration tool execution completed successfully");
            
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
    
    info!("🎭 Starting Holodeck Coordinator MCP Server with Qollective + Configurable LLM Integration");
    info!("📍 Service: holodeck-coordinator");
    info!("🔧 Port: {}", HOLODECK_COORDINATOR_PORT);
    info!("🌐 Protocol: MCP over WebSocket (qollective envelope-first architecture)");
    info!("🤖 LLM Integration: Configurable provider with orchestration capabilities");
    
    // Create the bridge adapter that connects qollective to our real LLM server
    let coordinator_adapter = CoordinatorMcpAdapter::new().await?;
    info!("✅ CoordinatorMcpAdapter created with configurable LLM-powered server");
    
    // Configure the qollective WebSocket server
    let mut server_config = WebSocketServerConfig::default();
    server_config.base.bind_address = DEFAULT_HOST.to_string();
    server_config.base.port = HOLODECK_COORDINATOR_PORT;
    server_config.base.max_connections = 1000;
    
    // Create qollective WebSocket server 
    let mut websocket_server = WebSocketServer::new(server_config).await?;
    info!("🌐 Qollective WebSocket Server created");
    info!("📦 Envelope integration enabled for coordinator orchestration");
    
    // Register our MCP adapter at the /mcp endpoint
    websocket_server.receive_envelope_at("/mcp", coordinator_adapter).await?;
    info!("📋 MCP handler registered at /mcp endpoint");
    
    // Start the server with our real LLM-powered adapter
    info!("🚀 Starting qollective WebSocket server with configurable LLM integration...");
    
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