// ABOUTME: Binary entry point for holodeck-safety MCP server with qollective WebSocket integration
// ABOUTME: Starts the safety server using qollective envelope-first architecture with configurable LLM providers

use shared_types::constants::network::{HOLODECK_SAFETY_PORT, DEFAULT_HOST};
use qollective::server::websocket::{WebSocketServer, WebSocketServerConfig};
use qollective::server::common::ServerConfig;
use qollective::config::tls::TlsConfig;
use qollective::envelope::Context;
use qollective::types::mcp::McpData;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::error::{Result as QollectiveResult, QollectiveError};
use holodeck_safety::HolodeckSafetyServer;
use rmcp::model::CallToolResult;
use tracing::{info, warn, error};
use tracing_subscriber;
use std::sync::Arc;
use serde_json;
use async_trait::async_trait;

struct HolodeckSafetyMcpAdapter {
    safety_server: Arc<HolodeckSafetyServer>,
}

impl HolodeckSafetyMcpAdapter {
    async fn new() -> QollectiveResult<Self> {
        // Create the real LLM-powered safety server with configuration
        let safety_server = Arc::new(
            HolodeckSafetyServer::new_with_config_file().await
                .map_err(|e| QollectiveError::internal(format!("Failed to create safety server: {}", e)))?
        );
        
        info!("🛡️ Created LLM-powered HolodeckSafetyServer with ollama integration");
        
        Ok(Self {
            safety_server,
        })
    }
}

#[async_trait]
impl ContextDataHandler<McpData, McpData> for HolodeckSafetyMcpAdapter {
    async fn handle(&self, context: Option<Context>, data: McpData) -> QollectiveResult<McpData> {
        info!("📨 HolodeckSafetyMcpAdapter received MCP data for processing with real LLM server");
        
        // Log the complete incoming MCP data for qollective demo
        info!("📦 QOLLECTIVE MCP DATA RECEIVED:");
        info!("📊 Tool Call: {}", if data.tool_call.is_some() { "Present" } else { "None" });
        info!("📊 Tool Response: {}", if data.tool_response.is_some() { "Present" } else { "None" });
        info!("📊 Discovery Data: {}", if data.discovery_data.is_some() { "Present" } else { "None" });
        
        // Handle MCP tool calls through the real safety server
        if let Some(tool_call) = data.tool_call {
            info!("🛡️ Executing real LLM-powered safety tool: '{}'", tool_call.params.name);
            if let Some(ref args) = tool_call.params.arguments {
                info!("🔧 Tool arguments: {}", serde_json::to_string_pretty(args).unwrap_or_else(|_| "Invalid args".to_string()));
            }
            
            // Delegate to the real LLM-powered safety server based on tool name
            let tool_result = match tool_call.params.name.as_ref() {
                "health_check" => {
                    self.safety_server.health_check().await
                        .map_err(|e| QollectiveError::internal(format!("Health check failed: {}", e)))?
                }
                "get_service_info" => {
                    self.safety_server.get_service_info().await
                        .map_err(|e| QollectiveError::internal(format!("Service info failed: {}", e)))?
                }
                "analyze_content_safety" => {
                    info!("🛡️ Routing to REAL LLM content safety analysis - this will call ollama!");
                    
                    // Convert MCP tool arguments to SafetyAnalysisRequest format
                    if let Some(ref args) = tool_call.params.arguments {
                        info!("🔄 Converting MCP arguments to safety analysis request");
                        
                        let content_id = args.get("content_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let content = args.get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("No content provided")
                            .to_string();
                        
                        let content_type = args.get("content_type")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let safety_level = args.get("safety_level")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Standard")
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
                        
                        info!("🛡️ Safety analysis request: Content='{}...', Safety Level='{}'", 
                            content.chars().take(50).collect::<String>(), safety_level);
                        
                        // Call the actual safety analysis method
                        match self.safety_server.analyze_content_safety(rmcp::handler::server::tool::Parameters(holodeck_safety::server::SafetyAnalysisRequest {
                            content_id,
                            content,
                            content_type,
                            safety_level,
                            tenant,
                            user_id,
                            request_id,
                        })).await {
                            Ok(result) => {
                                info!("🎉 Safety analysis completed successfully! Content length: {} items", result.content.len());
                                result
                            }
                            Err(e) => {
                                error!("💥 Safety analysis failed: {}", e);
                                return Err(QollectiveError::internal(format!("Safety analysis failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("analyze_content_safety requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "validate_compliance" => {
                    info!("🛡️ Routing to REAL LLM compliance validation - this will call ollama!");
                    
                    if let Some(ref args) = tool_call.params.arguments {
                        let content_id = args.get("content_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        let content = args.get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("No content provided")
                            .to_string();
                        
                        let content_type = args.get("content_type")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let regulations = args.get("regulations")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_else(|| vec!["General Starfleet Regulations".to_string()]);
                        
                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let user_id = args.get("user_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        match self.safety_server.validate_compliance(rmcp::handler::server::tool::Parameters(holodeck_safety::server::ComplianceValidationRequest {
                            content_id,
                            content,
                            content_type,
                            regulations,
                            tenant,
                            user_id,
                        })).await {
                            Ok(result) => result,
                            Err(e) => {
                                error!("💥 Compliance validation failed: {}", e);
                                return Err(QollectiveError::internal(format!("Compliance validation failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("validate_compliance requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                "assess_risk_factors" => {
                    info!("🛡️ Routing to REAL LLM risk assessment - this will call ollama!");
                    
                    if let Some(ref args) = tool_call.params.arguments {
                        let scenario_id = args.get("scenario_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        let scenario_description = args.get("scenario_description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("No scenario description provided")
                            .to_string();
                        
                        let safety_level = args.get("safety_level")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Standard")
                            .to_string();
                        
                        let participants = args.get("participants")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_else(|| vec!["Unknown participant".to_string()]);
                        
                        let environment_type = args.get("environment_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown environment")
                            .to_string();
                        
                        let tenant = args.get("tenant")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        match self.safety_server.assess_risk_factors(rmcp::handler::server::tool::Parameters(holodeck_safety::server::RiskAssessmentRequest {
                            scenario_id,
                            scenario_description,
                            safety_level,
                            participants,
                            environment_type,
                            tenant,
                        })).await {
                            Ok(result) => result,
                            Err(e) => {
                                error!("💥 Risk assessment failed: {}", e);
                                return Err(QollectiveError::internal(format!("Risk assessment failed: {}", e)));
                            }
                        }
                    } else {
                        let error_content = rmcp::model::Content {
                            raw: rmcp::model::RawContent::text("assess_risk_factors requires arguments".to_string()),
                            annotations: None,
                        };
                        CallToolResult {
                            content: vec![error_content],
                            is_error: Some(true),
                        }
                    }
                }
                _ => {
                    warn!("🚫 Unknown tool requested: {}", tool_call.params.name);
                    let error_content = rmcp::model::Content {
                        raw: rmcp::model::RawContent::text(format!("Tool '{}' not found", tool_call.params.name)),
                        annotations: None,
                    };
                    CallToolResult {
                        content: vec![error_content],
                        is_error: Some(true),
                    }
                }
            };
            
            info!("🎯 Real LLM tool execution completed, returning results through qollective envelope");
            
            // Return MCP response data in qollective envelope structure
            Ok(McpData {
                tool_call: None,
                tool_response: Some(tool_result),
                tool_registration: None,
                discovery_data: None,
            })
        } else {
            // Handle non-tool MCP requests (discovery, etc.)
            info!("🔍 Processing non-tool MCP request");
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
    
    info!("🛡️ Starting Holodeck Safety MCP Server with Qollective + Real LLM Integration");
    info!("📍 Service: holodeck-safety");
    info!("🔧 Port: {}", HOLODECK_SAFETY_PORT);
    info!("🌐 Protocol: MCP over WebSocket (qollective envelope-first architecture)");
    info!("🤖 LLM Integration: Real ollama-powered safety AI from server.rs");
    
    // Create the bridge adapter that connects qollective to our real LLM server
    let holodeck_adapter = HolodeckSafetyMcpAdapter::new().await?;
    info!("✅ HolodeckSafetyMcpAdapter created with real LLM-powered safety server");
    
    // Configure the qollective WebSocket server
    let mut server_config = WebSocketServerConfig::default();
    server_config.base.bind_address = DEFAULT_HOST.to_string();
    server_config.base.port = HOLODECK_SAFETY_PORT;
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
    info!("🛡️ Available safety tools: analyze_content_safety, validate_compliance, assess_risk_factors, health_check, get_service_info");
    info!("🌐 MCP URL: ws://{}:{}/mcp", DEFAULT_HOST, HOLODECK_SAFETY_PORT);
    
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