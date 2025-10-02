// ABOUTME: Binary entry point for holodeck-validator MCP server using qollective framework
// ABOUTME: Starts qollective MCP server with real LLM-powered HolodeckValidatorServer integration

use shared_types::constants::network::{HOLODECK_VALIDATOR_PORT, DEFAULT_HOST};
use qollective::server::websocket::{WebSocketServer, WebSocketServerConfig};
use qollective::envelope::Context;
use qollective::types::mcp::McpData;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::error::{Result as QollectiveResult};
use holodeck_validator::HolodeckValidatorServer;
use rmcp::model::CallToolResult;
use tracing::{info, warn, error};
use tracing_subscriber;
use std::sync::Arc;
use serde_json;
use async_trait::async_trait;

/// Bridge adapter that implements qollective ContextDataHandler for real LLM-powered validator server
struct HolodeckValidatorMcpAdapter {
    /// Real LLM-powered validator server from server.rs
    validator_server: Arc<HolodeckValidatorServer>,
}

impl HolodeckValidatorMcpAdapter {
    async fn new() -> QollectiveResult<Self> {
        // Create the real LLM-powered validator server with configuration
        let validator_server = Arc::new(
            HolodeckValidatorServer::new_with_config_file().await
                .map_err(|e| qollective::error::QollectiveError::internal(format!("Failed to create validator server: {}", e)))?
        );

        info!("🔍 Created LLM-powered HolodeckValidatorServer with configurable LLM integration");

        Ok(Self {
            validator_server,
        })
    }
}

#[async_trait]
impl ContextDataHandler<McpData, McpData> for HolodeckValidatorMcpAdapter {
    async fn handle(&self, context: Option<Context>, data: McpData) -> QollectiveResult<McpData> {
        info!("📨 HolodeckValidatorMcpAdapter received MCP data for processing with real LLM server");

        // Log the complete incoming MCP data for qollective demo
        info!("📦 QOLLECTIVE MCP DATA RECEIVED:");
        info!("📊 Tool Call: {}", if data.tool_call.is_some() { "Present" } else { "None" });
        info!("📊 Tool Response: {}", if data.tool_response.is_some() { "Present" } else { "None" });
        info!("📊 Discovery Data: {}", if data.discovery_data.is_some() { "Present" } else { "None" });

        // Handle MCP tool calls through the real validator server
        if let Some(tool_call) = data.tool_call {
            info!("🔍 Executing real LLM-powered validation tool: '{}'", tool_call.params.name);
            if let Some(ref args) = tool_call.params.arguments {
                info!("🔧 Tool arguments: {}", serde_json::to_string_pretty(args).unwrap_or_else(|_| "Invalid args".to_string()));
            }

            // Delegate to the real LLM-powered validator server based on tool name
            let tool_result = match tool_call.params.name.as_ref() {
                "health_check" => {
                    self.validator_server.health_check().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Health check failed: {}", e)))?
                }
                "get_service_info" => {
                    self.validator_server.get_service_info().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Service info failed: {}", e)))?
                }
                "validate_content" => {
                    info!("🔍 Routing to REAL LLM content validation - this will call configurable LLM provider!");

                    // Convert MCP tool arguments to ContentValidationRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to content validation request");

                        // Create ContentValidationRequest from MCP arguments
                        let content_id = args.get("content_id")
                            .and_then(|v| v.as_str())
                            .map(|s| uuid::Uuid::parse_str(s).unwrap_or_else(|_| uuid::Uuid::now_v7()))
                            .unwrap_or_else(|| uuid::Uuid::now_v7());

                        let story_content = args.get("story_content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Test story content")
                            .to_string();

                        let story_type = args.get("story_type")
                            .and_then(|v| v.as_str())
                            .and_then(|s| match s {
                                "Adventure" => Some(shared_types::HolodeckStoryType::Adventure),
                                "Educational" => Some(shared_types::HolodeckStoryType::Educational),
                                "Mystery" => Some(shared_types::HolodeckStoryType::Mystery),
                                "Drama" => Some(shared_types::HolodeckStoryType::Drama),
                                _ => Some(shared_types::HolodeckStoryType::Adventure),
                            })
                            .unwrap_or(shared_types::HolodeckStoryType::Adventure);

                        let content_type = args.get("content_type")
                            .and_then(|v| v.as_str())
                            .and_then(|s| match s {
                                "Story" => Some(shared_types::HolodeckContentType::Story),
                                "Character" => Some(shared_types::HolodeckContentType::Character),
                                "Environment" => Some(shared_types::HolodeckContentType::Environment),
                                _ => Some(shared_types::HolodeckContentType::Story),
                            })
                            .unwrap_or(shared_types::HolodeckContentType::Story);

                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let user_id = args.get("user_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let request_id = args.get("request_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        info!("🤖 CALLING REAL LLM VALIDATION: Story type={:?}, Content type={:?}", story_type, content_type);

                        // Call the actual LLM-powered content validation method
                        match self.validator_server.validate_content(rmcp::handler::server::tool::Parameters(holodeck_validator::server::ContentValidationRequest {
                            tenant,
                            user_id,
                            request_id,
                            content_id: content_id.to_string(),
                            story_content,
                            story_type,
                            content_type,
                            characters: None, // Optional field
                        })).await {
                            Ok(result) => {
                                info!("🎉 REAL LLM VALIDATION RESPONSE RECEIVED! Content length: {} items", result.content.len());

                                // Log the actual response content for visibility
                                info!("🔍 VALIDATION RESULT DEBUG: {:?}", result);

                                result
                            }
                            Err(e) => {
                                error!("💥 Real LLM content validation failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Content validation failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("validate_content requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "validate_canon_consistency" => {
                    info!("🔍 Routing to REAL LLM canon consistency validation");

                    // Convert MCP tool arguments to CanonValidationRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        let content_id = args.get("content_id")
                            .and_then(|v| v.as_str())
                            .map(|s| uuid::Uuid::parse_str(s).unwrap_or_else(|_| uuid::Uuid::now_v7()))
                            .unwrap_or_else(|| uuid::Uuid::now_v7());

                        let story_content = args.get("story_content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Test story content")
                            .to_string();

                        let era = args.get("era")
                            .and_then(|v| v.as_str())
                            .unwrap_or("TNG")
                            .to_string();

                        let strictness_level = args.get("strictness_level")
                            .and_then(|v| v.as_str())
                            .and_then(|s| match s {
                                "Lenient" => Some(shared_types::CanonStrictnessLevel::Lenient),
                                "Standard" => Some(shared_types::CanonStrictnessLevel::Standard),
                                "Strict" => Some(shared_types::CanonStrictnessLevel::Strict),
                                _ => Some(shared_types::CanonStrictnessLevel::Standard),
                            })
                            .unwrap_or(shared_types::CanonStrictnessLevel::Standard);

                        match self.validator_server.validate_canon_consistency(rmcp::handler::server::tool::Parameters(holodeck_validator::server::CanonValidationRequest {
                            content_id: content_id.to_string(),
                            story_content,
                            era,
                            strictness_level,
                            universe_elements: vec![], // Optional field
                        })).await {
                            Ok(result) => result,
                            Err(e) => {
                                error!("💥 Canon validation failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Canon validation failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("validate_canon_consistency requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "assess_content_quality" => {
                    info!("🔍 Routing to REAL LLM quality assessment");

                    // Convert MCP tool arguments to QualityAssessmentRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        let content_id = args.get("content_id")
                            .and_then(|v| v.as_str())
                            .map(|s| uuid::Uuid::parse_str(s).unwrap_or_else(|_| uuid::Uuid::now_v7()))
                            .unwrap_or_else(|| uuid::Uuid::now_v7());

                        let story_content = args.get("story_content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Test story content")
                            .to_string();

                        let target_audience = args.get("target_audience")
                            .and_then(|v| v.as_str())
                            .and_then(|s| match s {
                                "Children" => Some(shared_types::TargetAudience::Children),
                                "Teens" => Some(shared_types::TargetAudience::Teens),
                                "Adults" => Some(shared_types::TargetAudience::Adults),
                                "General" => Some(shared_types::TargetAudience::General),
                                _ => Some(shared_types::TargetAudience::General),
                            })
                            .unwrap_or(shared_types::TargetAudience::General);

                        let story_type = args.get("story_type")
                            .and_then(|v| v.as_str())
                            .and_then(|s| match s {
                                "Adventure" => Some(shared_types::HolodeckStoryType::Adventure),
                                "Educational" => Some(shared_types::HolodeckStoryType::Educational),
                                "Mystery" => Some(shared_types::HolodeckStoryType::Mystery),
                                "Drama" => Some(shared_types::HolodeckStoryType::Drama),
                                _ => Some(shared_types::HolodeckStoryType::Adventure),
                            })
                            .unwrap_or(shared_types::HolodeckStoryType::Adventure);

                        match self.validator_server.assess_content_quality(rmcp::handler::server::tool::Parameters(holodeck_validator::server::QualityAssessmentRequest {
                            content_id: content_id.to_string(),
                            story_content,
                            target_audience,
                            story_type,
                            educational_objectives: None, // Optional field
                        })).await {
                            Ok(result) => result,
                            Err(e) => {
                                error!("💥 Quality assessment failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Quality assessment failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("assess_content_quality requires arguments".to_string()),
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
                        raw: rmcp::model::RawContent::text(format!("Tool '{}' is not mapped in the qollective MCP adapter. Available tools: health_check, get_service_info, validate_content, validate_canon_consistency, assess_content_quality", tool_call.params.name)),
                        annotations: None,
                    };
                    CallToolResult {
                        content: vec![error_content],
                        is_error: Some(true),
                    }
                }
            };

            info!("✅ Real LLM validation tool execution completed successfully");

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

    info!("🔍 Starting Holodeck Validator MCP Server with Qollective + Real LLM Integration");
    info!("📍 Service: holodeck-validator");
    info!("🔧 Port: {}", HOLODECK_VALIDATOR_PORT);
    info!("🌐 Protocol: MCP over WebSocket (qollective envelope-first architecture)");
    info!("🤖 LLM Integration: Real configurable LLM-powered validation AI from server.rs");

    // Create the bridge adapter that connects qollective to our real LLM server
    let holodeck_adapter = HolodeckValidatorMcpAdapter::new().await?;
    info!("✅ HolodeckValidatorMcpAdapter created with real LLM-powered validation server");

    // Configure the qollective WebSocket server
    let mut server_config = WebSocketServerConfig::default();
    server_config.base.bind_address = DEFAULT_HOST.to_string();
    server_config.base.port = HOLODECK_VALIDATOR_PORT;
    server_config.base.max_connections = 1000;

    // Create qollective WebSocket server
    let mut websocket_server = WebSocketServer::new(server_config).await?;
    info!("🌐 Qollective WebSocket Server created");
    info!("📦 Envelope integration enabled for qollective demo logging");

    // Register our MCP adapter at the /mcp endpoint
    websocket_server.receive_envelope_at("/mcp", holodeck_adapter).await?;
    info!("📋 MCP handler registered at /mcp endpoint");

    // Start the server with our real LLM-powered adapter
    info!("🚀 Starting qollective WebSocket server with real LLM validation integration...");

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
