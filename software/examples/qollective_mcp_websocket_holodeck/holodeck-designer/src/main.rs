// ABOUTME: Binary entry point for holodeck-designer MCP server using qollective framework
// ABOUTME: Starts qollective MCP server with real LLM-powered HolodeckDesignerServer integration

use shared_types::constants::network::{HOLODECK_DESIGNER_PORT, DEFAULT_HOST};
use qollective::server::websocket::{WebSocketServer, WebSocketServerConfig};
use qollective::server::common::ServerConfig;
use qollective::config::tls::TlsConfig;
use qollective::envelope::Context;
use qollective::types::mcp::McpData;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::error::{Result as QollectiveResult};
use holodeck_designer::HolodeckDesignerServer;
use holodeck_designer::config::ServiceConfig;
use rmcp::model::CallToolResult;
use tracing::{info, warn, error};
use tracing_subscriber;
use std::sync::Arc;
use serde_json;
use async_trait::async_trait;

/// Bridge adapter that implements qollective ContextDataHandler for real LLM-powered story generation server
struct HolodeckDesignerMcpAdapter {
    /// Real LLM-powered story generation server from server.rs
    designer_server: Arc<HolodeckDesignerServer>,
}

impl HolodeckDesignerMcpAdapter {
    async fn new() -> QollectiveResult<Self> {
        // Load configuration from file with .env fallback
        let config_path = "config.toml";
        let env_path = Some(".env");

        info!("📋 Loading holodeck-designer configuration from {}", config_path);
        let service_config = ServiceConfig::load_from_file(config_path, env_path)
            .map_err(|e| qollective::error::QollectiveError::internal(format!("Failed to load config: {}", e)))?;

        info!("🤖 LLM Provider: {} (model: {})", service_config.llm.provider, service_config.llm.model);
        info!("⚡ Story generation timeout: {}ms", service_config.story_design.story_generation_timeout_ms);
        info!("🎭 Character integration: {}", service_config.story_design.enable_character_integration);

        // Create the real LLM-powered designer server with full service configuration including character integration
        let designer_server = Arc::new(
            HolodeckDesignerServer::new_with_service_config(&service_config).await
                .map_err(|e| qollective::error::QollectiveError::internal(format!("Failed to create designer server: {}", e)))?
        );

        info!("🎨 Created LLM-powered HolodeckDesignerServer with configurable LLM integration");

        Ok(Self {
            designer_server,
        })
    }
}

#[async_trait]
impl ContextDataHandler<McpData, McpData> for HolodeckDesignerMcpAdapter {
    async fn handle(&self, context: Option<Context>, data: McpData) -> QollectiveResult<McpData> {
        info!("📨 HolodeckDesignerMcpAdapter received MCP data for processing with real LLM server");

        // Log the complete incoming MCP data for qollective demo
        info!("📦 QOLLECTIVE MCP DATA RECEIVED:");
        info!("📊 Tool Call: {}", if data.tool_call.is_some() { "Present" } else { "None" });
        info!("📊 Tool Response: {}", if data.tool_response.is_some() { "Present" } else { "None" });
        info!("📊 Discovery Data: {}", if data.discovery_data.is_some() { "Present" } else { "None" });

        // Handle MCP tool calls through the real designer server
        if let Some(tool_call) = data.tool_call {
            info!("🎨 Executing real LLM-powered tool: '{}'", tool_call.params.name);
            if let Some(ref args) = tool_call.params.arguments {
                info!("🔧 Tool arguments: {}", serde_json::to_string_pretty(args).unwrap_or_else(|_| "Invalid args".to_string()));
            }

            // Delegate to the real LLM-powered designer server based on tool name
            let tool_result = match tool_call.params.name.as_ref() {
                "health_check" => {
                    self.designer_server.health_check().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Health check failed: {}", e)))?
                }
                "get_service_info" => {
                    self.designer_server.get_service_info().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Service info failed: {}", e)))?
                }
                "generate_story" => {
                    info!("🎨 Routing to REAL LLM story generation - this will call the configurable LLM provider!");

                    // Convert MCP tool arguments to StoryGenerationRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to story generation request");

                        // Create StoryGenerationRequest from MCP arguments
                        let theme = args.get("theme")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Adventure in the Alpha Quadrant")
                            .to_string();

                        let story_type = args.get("story_type")
                            .and_then(|v| v.as_str())
                            .and_then(|s| match s {
                                "Adventure" => Some(shared_types::holodeck::HolodeckStoryType::Adventure),
                                "Mystery" => Some(shared_types::holodeck::HolodeckStoryType::Mystery),
                                "Drama" => Some(shared_types::holodeck::HolodeckStoryType::Drama),
                                "Comedy" => Some(shared_types::holodeck::HolodeckStoryType::Comedy),
                                "Historical" => Some(shared_types::holodeck::HolodeckStoryType::Historical),
                                "SciFi" => Some(shared_types::holodeck::HolodeckStoryType::SciFi),
                                "Fantasy" => Some(shared_types::holodeck::HolodeckStoryType::Fantasy),
                                "Educational" => Some(shared_types::holodeck::HolodeckStoryType::Educational),
                                _ => None,
                            })
                            .unwrap_or(shared_types::holodeck::HolodeckStoryType::Adventure);

                        let duration_minutes = args.get("duration_minutes")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32);

                        let max_participants = args.get("max_participants")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32);

                        let characters = args.get("characters")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_else(Vec::new);

                        let safety_level = args.get("safety_level")
                            .and_then(|v| v.as_str())
                            .and_then(|s| match s {
                                "Training" => Some(shared_types::holodeck::SafetyLevel::Training),
                                "Standard" => Some(shared_types::holodeck::SafetyLevel::Standard),
                                "Reduced" => Some(shared_types::holodeck::SafetyLevel::Reduced),
                                "Disabled" => Some(shared_types::holodeck::SafetyLevel::Disabled),
                                _ => None,
                            });

                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let user_id = args.get("user_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let request_id = args.get("request_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        info!("🤖 CALLING REAL LLM: Theme='{}', Type={:?}", theme, story_type);

                        // Call the actual LLM-powered story generation method
                        match self.designer_server.generate_story(rmcp::handler::server::tool::Parameters(holodeck_designer::server::StoryGenerationRequest {
                            tenant,
                            user_id,
                            request_id,
                            theme,
                            story_type: format!("{:?}", story_type),
                            duration_minutes,
                            max_participants,
                            characters,
                            safety_level: safety_level.map(|sl| format!("{:?}", sl)),
                            participant_experience_level: None,
                            environment_constraints: None,
                            narrative_complexity: None,
                        })).await {
                            Ok(result) => {
                                info!("🎉 REAL LLM STORY GENERATION RESPONSE RECEIVED! Content length: {} items", result.content.len());

                                // Log the actual response content for visibility
                                info!("🎨 STORY GENERATION DEBUG: {:?}", result);

                                result
                            }
                            Err(e) => {
                                error!("💥 Real LLM story generation failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Story generation failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("generate_story requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "enhance_story" => {
                    info!("🎨 Routing to REAL LLM story enhancement - this will call the configurable LLM provider!");

                    // Convert MCP tool arguments to StoryEnhancementRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to story enhancement request");

                        let story_id = args.get("story_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| uuid::Uuid::parse_str(s).ok())
                            .unwrap_or_else(uuid::Uuid::now_v7);

                        let enhancement_type = args.get("enhancement_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("general")
                            .to_string();

                        let focus_areas = args.get("focus_areas")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_else(Vec::new);

                        info!("🤖 CALLING REAL LLM ENHANCEMENT: Story ID={}", story_id);

                        // Call the actual LLM-powered story enhancement method
                        match self.designer_server.enhance_story(rmcp::handler::server::tool::Parameters(holodeck_designer::server::StoryEnhancementRequest {
                            story_template: story_id.to_string(), // Use story_id as template content
                            enhancement_areas: focus_areas,
                            target_improvements: vec![enhancement_type],
                            preserve_elements: Vec::new(),
                        })).await {
                            Ok(result) => {
                                info!("🎉 REAL LLM STORY ENHANCEMENT RESPONSE RECEIVED!");
                                result
                            }
                            Err(e) => {
                                error!("💥 Real LLM story enhancement failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Story enhancement failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("enhance_story requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "validate_story_consistency" => {
                    info!("🎨 Routing to REAL LLM story validation - this will call the configurable LLM provider!");

                    // Convert MCP tool arguments to StoryValidationRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        let story_id = args.get("story_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| uuid::Uuid::parse_str(s).ok())
                            .unwrap_or_else(uuid::Uuid::now_v7);

                        info!("🤖 CALLING REAL LLM VALIDATION: Story ID={}", story_id);

                        // Call the actual LLM-powered story validation method
                        match self.designer_server.validate_story_consistency(rmcp::handler::server::tool::Parameters(holodeck_designer::server::StoryValidationRequest {
                            story_content: story_id.to_string(), // Use story_id as content to validate
                            validation_criteria: vec!["consistency".to_string(), "canon".to_string()],
                        })).await {
                            Ok(result) => {
                                info!("🎉 REAL LLM STORY VALIDATION RESPONSE RECEIVED!");
                                result
                            }
                            Err(e) => {
                                error!("💥 Real LLM story validation failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Story validation failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("validate_story_consistency requires arguments".to_string()),
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
                        raw: rmcp::model::RawContent::text(format!("Tool '{}' is not mapped in the qollective MCP adapter. Available tools: health_check, get_service_info, generate_story, enhance_story, validate_story_consistency", tool_call.params.name)),
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

    info!("🎨 Starting Holodeck Designer MCP Server with Qollective + Real LLM Integration");
    info!("📍 Service: holodeck-designer");
    info!("🔧 Port: {}", HOLODECK_DESIGNER_PORT);
    info!("🌐 Protocol: MCP over WebSocket (qollective envelope-first architecture)");
    info!("🤖 LLM Integration: Real LLM-powered story generation from server.rs");

    // Create the bridge adapter that connects qollective to our real LLM server
    let holodeck_adapter = HolodeckDesignerMcpAdapter::new().await?;
    info!("✅ HolodeckDesignerMcpAdapter created with real LLM-powered story generation server");

    // Configure the qollective WebSocket server
    let mut server_config = WebSocketServerConfig::default();
    server_config.base.bind_address = DEFAULT_HOST.to_string();
    server_config.base.port = HOLODECK_DESIGNER_PORT;
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
