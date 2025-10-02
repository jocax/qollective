// ABOUTME: Binary entry point for holodeck-environment MCP server using qollective framework
// ABOUTME: Starts qollective MCP server with real LLM-powered HolodeckEnvironmentServer integration

use shared_types::constants::network::{HOLODECK_ENVIRONMENT_PORT, DEFAULT_HOST};
use qollective::server::websocket::{WebSocketServer, WebSocketServerConfig};
use qollective::server::common::ServerConfig;
use qollective::config::tls::TlsConfig;
use qollective::envelope::Context;
use qollective::types::mcp::McpData;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::error::{Result as QollectiveResult};
use holodeck_environment::HolodeckEnvironmentServer;
use rmcp::model::CallToolResult;
use tracing::{info, warn, error};
use tracing_subscriber;
use std::sync::Arc;
use serde_json;
use async_trait::async_trait;

/// Bridge adapter that implements qollective ContextDataHandler for real LLM-powered environment server
struct HolodeckEnvironmentMcpAdapter {
    /// Real LLM-powered environment server from server.rs
    environment_server: Arc<HolodeckEnvironmentServer>,
}

impl HolodeckEnvironmentMcpAdapter {
    async fn new() -> QollectiveResult<Self> {
        // Create the real LLM-powered environment server with configuration
        let environment_server = Arc::new(
            HolodeckEnvironmentServer::new_with_config_file().await
                .map_err(|e| qollective::error::QollectiveError::internal(format!("Failed to create environment server: {}", e)))?
        );

        info!("🌍 Created LLM-powered HolodeckEnvironmentServer with configurable provider integration");

        Ok(Self {
            environment_server,
        })
    }
}

#[async_trait]
impl ContextDataHandler<McpData, McpData> for HolodeckEnvironmentMcpAdapter {
    async fn handle(&self, context: Option<Context>, data: McpData) -> QollectiveResult<McpData> {
        info!("📨 HolodeckEnvironmentMcpAdapter received MCP data for processing with real LLM server");

        // Log the complete incoming MCP data for qollective demo
        info!("📦 QOLLECTIVE MCP DATA RECEIVED:");
        info!("📊 Tool Call: {}", if data.tool_call.is_some() { "Present" } else { "None" });
        info!("📊 Tool Response: {}", if data.tool_response.is_some() { "Present" } else { "None" });
        info!("📊 Discovery Data: {}", if data.discovery_data.is_some() { "Present" } else { "None" });

        // Handle MCP tool calls through the real environment server
        if let Some(tool_call) = data.tool_call {
            info!("🌍 Executing real LLM-powered environment tool: '{}'", tool_call.params.name);
            if let Some(ref args) = tool_call.params.arguments {
                info!("🔧 Tool arguments: {}", serde_json::to_string_pretty(args).unwrap_or_else(|_| "Invalid args".to_string()));
            }

            // Delegate to the real LLM-powered environment server based on tool name
            let tool_result = match tool_call.params.name.as_ref() {
                "health_check" => {
                    self.environment_server.health_check().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Health check failed: {}", e)))?
                }
                "get_service_info" => {
                    self.environment_server.get_service_info().await
                        .map_err(|e| qollective::error::QollectiveError::internal(format!("Service info failed: {}", e)))?
                }
                "generate_environment" => {
                    info!("🌍 Routing to 3D environment generation with real LLM integration");

                    // Convert MCP tool arguments to EnvironmentGenerationRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to environment generation request");

                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let user_id = args.get("user_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let request_id = args.get("request_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let scene_description = args.get("scene_description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("A beautiful Star Trek holodeck environment")
                            .to_string();

                        let environment_type = args.get("environment_type")
                            .and_then(|v| v.as_str())
                            .map(|s| match s {
                                "starship" => holodeck_environment::server::EnvironmentType::StarshipInterior,
                                "alien" => holodeck_environment::server::EnvironmentType::AlienWorld,
                                "historical" => holodeck_environment::server::EnvironmentType::HistoricalSetting,
                                "fantasy" => holodeck_environment::server::EnvironmentType::FantasyRealm,
                                "space" => holodeck_environment::server::EnvironmentType::SpaceEnvironment,
                                _ => holodeck_environment::server::EnvironmentType::TrainingFacility,
                            });

                        let safety_level = args.get("safety_level")
                            .and_then(|v| v.as_str())
                            .map(|s| match s {
                                "training" => shared_types::holodeck::SafetyLevel::Training,
                                "standard" => shared_types::holodeck::SafetyLevel::Standard,
                                "reduced" => shared_types::holodeck::SafetyLevel::Reduced,
                                "disabled" => shared_types::holodeck::SafetyLevel::Disabled,
                                _ => shared_types::holodeck::SafetyLevel::Standard,
                            })
                            .unwrap_or(shared_types::holodeck::SafetyLevel::Standard);

                        info!("🏗️ Environment generation request: Scene='{}', Type={:?}, Safety={:?}",
                            scene_description, environment_type, safety_level);

                        // Call the actual environment generation method
                        match self.environment_server.generate_environment(rmcp::handler::server::tool::Parameters(holodeck_environment::server::EnvironmentGenerationRequest {
                            tenant,
                            user_id,
                            request_id,
                            scene_description,
                            environment_type,
                            safety_level,
                        })).await {
                            Ok(result) => {
                                info!("🎉 3D environment generated successfully! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 Environment generation failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Environment generation failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("generate_environment requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "manage_scene" => {
                    info!("🎬 Routing to REAL LLM scene management - dynamic environment modification!");

                    // Convert MCP tool arguments to SceneManagementRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to scene management request");

                        let scene_id = args.get("scene_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());

                        let operation_type = args.get("operation_type")
                            .and_then(|v| v.as_str())
                            .map(|s| match s {
                                "update_lighting" => holodeck_environment::server::SceneOperationType::UpdateLighting,
                                "change_weather" => holodeck_environment::server::SceneOperationType::ChangeWeather,
                                "modify_physics" => holodeck_environment::server::SceneOperationType::ModifyPhysics,
                                "add_objects" => holodeck_environment::server::SceneOperationType::AddObjects,
                                "update_atmosphere" => holodeck_environment::server::SceneOperationType::UpdateAtmosphere,
                                "emergency_shutdown" => holodeck_environment::server::SceneOperationType::EmergencyShutdown,
                                _ => holodeck_environment::server::SceneOperationType::UpdateAtmosphere,
                            })
                            .unwrap_or(holodeck_environment::server::SceneOperationType::UpdateAtmosphere);

                        let modification_parameters = args.get("modification_parameters")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or("Standard environmental modification".to_string());

                        info!("🎬 REAL LLM SCENE MANAGEMENT: Scene={}, Operation={:?}", scene_id, operation_type);

                        // Call the actual scene management method
                        match self.environment_server.manage_scene(rmcp::handler::server::tool::Parameters(holodeck_environment::server::SceneManagementRequest {
                            scene_id,
                            operation_type,
                            modification_parameters,
                        })).await {
                            Ok(result) => {
                                info!("🎉 REAL LLM SCENE MANAGEMENT COMPLETED! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 Real LLM scene management failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Scene management failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("manage_scene requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "validate_environmental_safety" => {
                    info!("🛡️ Routing to environmental safety validation");

                    // Convert MCP tool arguments to EnvironmentalSafetyRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        let environment_id = args.get("environment_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("default-environment")
                            .to_string();

                        let safety_level = args.get("safety_level")
                            .and_then(|v| v.as_str())
                            .map(|s| match s {
                                "training" => shared_types::holodeck::SafetyLevel::Training,
                                "standard" => shared_types::holodeck::SafetyLevel::Standard,
                                "reduced" => shared_types::holodeck::SafetyLevel::Reduced,
                                "disabled" => shared_types::holodeck::SafetyLevel::Disabled,
                                _ => shared_types::holodeck::SafetyLevel::Standard,
                            })
                            .unwrap_or(shared_types::holodeck::SafetyLevel::Standard);

                        info!("🛡️ Environmental safety validation: Environment={}, Safety={:?}", environment_id, safety_level);

                        // Call the actual environmental safety validation method
                        match self.environment_server.validate_environmental_safety(rmcp::handler::server::tool::Parameters(holodeck_environment::server::EnvironmentalSafetyRequest {
                            environment_id,
                            safety_level,
                        })).await {
                            Ok(result) => {
                                info!("🎉 Environmental safety validation completed successfully! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 Environmental safety validation failed: {}", e);
                                return Err(qollective::error::QollectiveError::internal(format!("Safety validation failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("validate_environmental_safety requires arguments".to_string()),
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
                        raw: rmcp::model::RawContent::text(format!("Tool '{}' is not mapped in the qollective MCP adapter. Available tools: health_check, get_service_info, generate_environment, manage_scene, validate_environmental_safety", tool_call.params.name)),
                        annotations: None,
                    };
                    CallToolResult {
                        content: vec![error_content],
                        is_error: Some(true),
                    }
                }
            };

            info!("✅ Real LLM environment tool execution completed successfully");

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

    info!("🌍 Starting Holodeck Environment MCP Server with Qollective + Real LLM Integration");
    info!("📍 Service: holodeck-environment");
    info!("🔧 Port: {}", HOLODECK_ENVIRONMENT_PORT);
    info!("🌐 Protocol: MCP over WebSocket (qollective envelope-first architecture)");
    info!("🤖 LLM Integration: Real configurable LLM-powered 3D environment AI from server.rs");

    // Create the bridge adapter that connects qollective to our real LLM environment server
    let holodeck_adapter = HolodeckEnvironmentMcpAdapter::new().await?;
    info!("✅ HolodeckEnvironmentMcpAdapter created with real LLM-powered environment server");

    // Configure the qollective WebSocket server
    let mut server_config = WebSocketServerConfig::default();
    server_config.base.bind_address = DEFAULT_HOST.to_string();
    server_config.base.port = HOLODECK_ENVIRONMENT_PORT;
    server_config.base.max_connections = 1000;

    // Create qollective WebSocket server
    let mut websocket_server = WebSocketServer::new(server_config).await?;
    info!("🌐 Qollective WebSocket Server created");
    info!("📦 Envelope integration enabled for qollective demo logging");

    // Register our MCP adapter at the /mcp endpoint
    websocket_server.receive_envelope_at("/mcp", holodeck_adapter).await?;
    info!("📋 MCP handler registered at /mcp endpoint");

    // Start the server with our real LLM-powered adapter
    info!("🚀 Starting qollective WebSocket server with real LLM environment integration...");

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
