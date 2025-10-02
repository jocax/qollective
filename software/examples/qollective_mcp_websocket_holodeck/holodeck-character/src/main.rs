// ABOUTME: Binary entry point for holodeck-character MCP server using qollective framework
// ABOUTME: Starts qollective MCP server with real LLM-powered HolodeckCharacterServer integration

use shared_types::constants::network::{HOLODECK_CHARACTER_PORT, DEFAULT_HOST};
use qollective::server::websocket::{WebSocketServer, WebSocketServerConfig};
use qollective::server::common::ServerConfig;
use qollective::config::tls::TlsConfig;
use qollective::envelope::Context;
use qollective::types::mcp::McpData;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::error::{Result as QollectiveResult};
use holodeck_character::HolodeckCharacterServer;
use rmcp::model::CallToolResult;
use tracing::{info, warn, error};
use tracing_subscriber;
use std::sync::Arc;
use serde_json;
use async_trait::async_trait;

/// Bridge adapter that implements qollective ContextDataHandler for real LLM-powered server
struct HolodeckMcpAdapter {
    /// Real LLM-powered character server from server.rs
    character_server: Arc<HolodeckCharacterServer>,
}

impl HolodeckMcpAdapter {
    async fn new() -> QollectiveResult<Self> {
        // Create the real LLM-powered character server with configuration
        let character_server = Arc::new(
            HolodeckCharacterServer::new_with_config_file().await
                .map_err(|e| qollective::error::QollectiveError::internal(format!("Failed to create character server: {}", e)))?
        );
        
        info!("🎭 Created LLM-powered HolodeckCharacterServer with ollama integration");
        
        Ok(Self {
            character_server,
        })
    }
}

#[async_trait]
impl ContextDataHandler<McpData, McpData> for HolodeckMcpAdapter {
    async fn handle(&self, context: Option<Context>, data: McpData) -> QollectiveResult<McpData> {
        info!("📨 HolodeckMcpAdapter received MCP data for processing with real LLM server");
        
        // Log the complete incoming MCP data for qollective demo
        info!("📦 QOLLECTIVE MCP DATA RECEIVED:");
        info!("📊 Tool Call: {}", if data.tool_call.is_some() { "Present" } else { "None" });
        info!("📊 Tool Response: {}", if data.tool_response.is_some() { "Present" } else { "None" });
        info!("📊 Discovery Data: {}", if data.discovery_data.is_some() { "Present" } else { "None" });
        
        // Handle MCP tool calls through the real character server
        if let Some(tool_call) = data.tool_call {
            info!("🎭 Executing real LLM-powered tool: '{}'", tool_call.params.name);
            if let Some(ref args) = tool_call.params.arguments {
                info!("🔧 Tool arguments: {}", serde_json::to_string_pretty(args).unwrap_or_else(|_| "Invalid args".to_string()));
            }
            
            // Delegate to the real LLM-powered character server based on tool name
            let tool_result = match tool_call.params.name.as_ref() {
                "health_check" => {
                    self.character_server.health_check().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Health check failed: {}", e)))?
                }
                "get_service_info" => {
                    self.character_server.get_service_info().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Service info failed: {}", e)))?
                }
                "get_character_profile" => {
                    info!("🎭 Routing to character profile retrieval");
                    
                    // Convert MCP tool arguments to CharacterProfileRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to character profile request");
                        
                        let character_name = args.get("character_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Picard")
                            .to_string();
                        
                        let include_background = args.get("include_background")
                            .and_then(|v| v.as_bool());
                        
                        let include_speech_patterns = args.get("include_speech_patterns")
                            .and_then(|v| v.as_bool());
                        
                        info!("📋 Character profile request: Character='{}', Background={:?}, Speech={:?}", 
                            character_name, include_background, include_speech_patterns);
                        
                        // Call the actual character profile method
                        match self.character_server.get_character_profile(rmcp::handler::server::tool::Parameters(holodeck_character::server::CharacterProfileRequest {
                            character_name,
                            include_background,
                            include_speech_patterns,
                        })).await {
                            Ok(result) => {
                                info!("🎉 Character profile retrieved successfully! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 Character profile retrieval failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Character profile retrieval failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("get_character_profile requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "interact_with_character" => {
                    info!("🎭 Routing to REAL LLM character interaction - this will call ollama!");
                    
                    // Convert MCP tool arguments to CharacterInteractionRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to character interaction request");
                        
                        // Create CharacterInteractionRequest from MCP arguments
                        let character_name = args.get("character_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Picard")
                            .to_string();
                        
                        let user_message = args.get("user_message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Hello, Captain.")
                            .to_string();
                        
                        let player_action = args.get("player_action")
                            .and_then(|v| v.as_str())
                            .unwrap_or("asks a question")
                            .to_string();
                        
                        let scene_context = args.get("conversation_context")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let character_mood = args.get("character_mood")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let user_id = args.get("user_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let request_id = args.get("request_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        info!("🤖 CALLING REAL LLM: Character='{}', Message='{}'", character_name, user_message);
                        
                        // Call the actual LLM-powered character interaction method
                        match self.character_server.interact_with_character(rmcp::handler::server::tool::Parameters(holodeck_character::server::CharacterInteractionRequest {
                            tenant,
                            user_id,
                            request_id,
                            character_name,
                            user_message,
                            player_action,
                            scene_context,
                            character_mood,
                        })).await {
                            Ok(result) => {
                                info!("🎉 REAL LLM RESPONSE RECEIVED! Content length: {} items", result.content.len());
                                
                                // Log the actual response content for visibility  
                                info!("🎭 PICARD RESPONSE DEBUG: {:?}", result);
                                
                                result
                            }
                            Err(e) => {
                                error!("💥 Real LLM character interaction failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Character interaction failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("interact_with_character requires arguments".to_string()),
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
                        raw: rmcp::model::RawContent::text(format!("Tool '{}' is not mapped in the qollective MCP adapter. Available tools: health_check, get_service_info, get_character_profile, interact_with_character", tool_call.params.name)),
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
    
    info!("🎭 Starting Holodeck Character MCP Server with Qollective + Real LLM Integration");
    info!("📍 Service: holodeck-character");
    info!("🔧 Port: {}", HOLODECK_CHARACTER_PORT);
    info!("🌐 Protocol: MCP over WebSocket (qollective envelope-first architecture)");
    info!("🤖 LLM Integration: Real ollama-powered character AI from server.rs");
    
    // Create the bridge adapter that connects qollective to our real LLM server
    let holodeck_adapter = HolodeckMcpAdapter::new().await?;
    info!("✅ HolodeckMcpAdapter created with real LLM-powered character server");
    
    // Configure the qollective WebSocket server
    let mut server_config = WebSocketServerConfig::default();
    server_config.base.bind_address = DEFAULT_HOST.to_string();
    server_config.base.port = HOLODECK_CHARACTER_PORT;
    server_config.base.max_connections = 1000;
    
    // Create qollective WebSocket server 
    let mut websocket_server = WebSocketServer::new(server_config).await?;
    info!("🌐 Qollective WebSocket Server created");
    info!("📦 Envelope integration enabled for qollective demo logging");
    
    // Register our MCP adapter at the /mcp endpoint
    websocket_server.receive_envelope_at("/mcp", holodeck_adapter).await?;
    info!("📋 MCP handler registered at /mcp endpoint");
    
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

