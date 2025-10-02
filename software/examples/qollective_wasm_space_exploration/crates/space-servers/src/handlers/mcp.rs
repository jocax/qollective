// ABOUTME: Space exploration MCP server for rover and probe tool execution
// ABOUTME: Provides remote tool execution capabilities for space exploration missions

use qollective::{
    error::Result,
    server::websocket::{WebSocketServer, WebSocketServerConfig},
    server::common::ServerConfig,
    prelude::{ContextDataHandler, UnifiedEnvelopeReceiver},
    envelope::Context,
};
use space_shared::{SpaceDataGenerator, Spacecraft, SpacecraftTelemetry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tracing::{info, debug};
use async_trait::async_trait;

/// MCP WebSocket server configuration for space operations
pub fn get_space_mcp_config() -> WebSocketServerConfig {
    WebSocketServerConfig {
        base: ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8445,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// MCP JSON-RPC request structure
#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Option<Value>,
}

/// MCP JSON-RPC response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    pub result: Option<Value>,
    pub error: Option<McpError>,
}

/// MCP JSON-RPC error structure
#[derive(Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// Space tools manager for MCP operations
pub struct SpaceToolsManager {
    pub spacecraft: Arc<RwLock<HashMap<String, Spacecraft>>>,
}

/// Main MCP handler for space tool operations
pub struct SpaceMcpHandler {
    pub manager: Arc<SpaceToolsManager>,
}

impl SpaceToolsManager {
    pub fn new() -> Self {
        // Initialize with demo spacecraft data
        let missions = SpaceDataGenerator::generate_missions();
        let mut spacecraft = HashMap::new();
        
        for mission in missions {
            for craft in mission.spacecraft {
                spacecraft.insert(craft.id.clone(), craft);
            }
        }
        
        Self {
            spacecraft: Arc::new(RwLock::new(spacecraft)),
        }
    }
    
    /// Execute space tool operation based on MCP call
    pub fn execute_tool(&self, tool_name: &str, arguments: &Value) -> Value {
        let start_time = std::time::Instant::now();
        
        match tool_name {
            "rover_drill" => {
                let rover_id = arguments.get("rover_id").and_then(|v| v.as_str()).unwrap_or("rover_1");
                let depth = arguments.get("depth").and_then(|v| v.as_f64()).unwrap_or(0.5);
                
                let result = format!(
                    "Drilled sample at depth {:.2}m. Sample composition: Iron oxide 45%, Silicon dioxide 30%, Aluminum oxide 15%, Other minerals 10%", 
                    depth
                );
                
                serde_json::json!({
                    "status": "success",
                    "rover_id": rover_id,
                    "result": result,
                    "execution_time_ms": start_time.elapsed().as_millis()
                })
            },
            "rover_photo" => {
                let rover_id = arguments.get("rover_id").and_then(|v| v.as_str()).unwrap_or("rover_1");
                let direction = arguments.get("direction").and_then(|v| v.as_str()).unwrap_or("forward");
                
                let result = format!("Captured high-resolution image facing {}. Resolution: 4096x3072, Lighting conditions: Optimal", direction);
                
                serde_json::json!({
                    "status": "success",
                    "rover_id": rover_id,
                    "result": result,
                    "execution_time_ms": start_time.elapsed().as_millis()
                })
            },
            "probe_analyze" => {
                let probe_id = arguments.get("probe_id").and_then(|v| v.as_str()).unwrap_or("probe_1");
                let analysis_type = arguments.get("analysis_type").and_then(|v| v.as_str()).unwrap_or("geological");
                
                let findings = match analysis_type {
                    "geological" => vec![
                        "Sedimentary rock layers detected",
                        "Evidence of ancient water activity",
                        "Crystalline structures present"
                    ],
                    "atmospheric" => vec![
                        "Trace methane detected",
                        "CO2 concentration: 95.3%",
                        "Temperature gradient anomaly identified"
                    ],
                    _ => vec!["General analysis completed"]
                };
                
                serde_json::json!({
                    "status": "success",
                    "probe_id": probe_id,
                    "analysis_type": analysis_type,
                    "findings": findings,
                    "confidence_score": 0.87,
                    "execution_time_ms": start_time.elapsed().as_millis()
                })
            },
            "orbital_scan" => {
                let satellite_id = arguments.get("satellite_id").and_then(|v| v.as_str()).unwrap_or("sat_1");
                let resolution = arguments.get("resolution").and_then(|v| v.as_str()).unwrap_or("medium");
                
                let images_captured = match resolution {
                    "high" => 45,
                    "medium" => 30,
                    "low" => 15,
                    _ => 20
                };
                
                serde_json::json!({
                    "status": "success",
                    "satellite_id": satellite_id,
                    "scan_id": format!("scan_{}_{}", satellite_id, chrono::Utc::now().timestamp()),
                    "images_captured": images_captured,
                    "coverage_percentage": 85.5,
                    "anomalies_detected": ["Unusual geological formation identified", "Potential landing site candidate found"],
                    "execution_time_ms": start_time.elapsed().as_millis()
                })
            },
            _ => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("Unknown tool: {}", tool_name),
                    "execution_time_ms": start_time.elapsed().as_millis()
                })
            }
        }
    }
}

impl SpaceMcpHandler {
    pub fn new() -> Self {
        let manager = Arc::new(SpaceToolsManager::new());
        Self { manager }
    }
    
    /// Handle MCP JSON-RPC requests
    pub fn handle_mcp_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "tools/list" => {
                let tools = serde_json::json!([
                    {
                        "name": "rover_drill",
                        "description": "Drill a sample at specified depth using rover",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "rover_id": {"type": "string", "description": "ID of the rover"},
                                "depth": {"type": "number", "description": "Drilling depth in meters"}
                            },
                            "required": ["rover_id"]
                        }
                    },
                    {
                        "name": "rover_photo",
                        "description": "Take a photo in specified direction using rover",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "rover_id": {"type": "string", "description": "ID of the rover"},
                                "direction": {"type": "string", "description": "Photo direction"}
                            },
                            "required": ["rover_id"]
                        }
                    },
                    {
                        "name": "probe_analyze",
                        "description": "Perform analysis using probe",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "probe_id": {"type": "string", "description": "ID of the probe"},
                                "analysis_type": {"type": "string", "description": "Type of analysis"}
                            },
                            "required": ["probe_id"]
                        }
                    },
                    {
                        "name": "orbital_scan",
                        "description": "Perform orbital scan using satellite",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "satellite_id": {"type": "string", "description": "ID of the satellite"},
                                "resolution": {"type": "string", "description": "Scan resolution"}
                            },
                            "required": ["satellite_id"]
                        }
                    }
                ]);
                
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({"tools": tools})),
                    error: None,
                }
            },
            "tools/call" => {
                if let Some(params) = request.params {
                    if let (Some(tool_name), Some(arguments)) = (
                        params.get("name").and_then(|v| v.as_str()),
                        params.get("arguments")
                    ) {
                        let result = self.manager.execute_tool(tool_name, arguments);
                        
                        McpResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: Some(serde_json::json!({"content": [{"type": "text", "text": result.to_string()}]})),
                            error: None,
                        }
                    } else {
                        McpResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(McpError {
                                code: -32602,
                                message: "Invalid params".to_string(),
                                data: None,
                            }),
                        }
                    }
                } else {
                    McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(McpError {
                            code: -32602,
                            message: "Missing params".to_string(),
                            data: None,
                        }),
                    }
                }
            },
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            }
        }
    }
}

/// Create and configure space MCP WebSocket server
pub async fn create_space_mcp_server() -> Result<WebSocketServer> {
    info!("🛠️ Creating Space MCP WebSocket server...");
    
    // Initialize MCP handler
    let handler = SpaceMcpHandler::new();
    let spacecraft_count = handler.manager.spacecraft.read().unwrap().len();
    
    info!("🔧 Loaded {} spacecraft for tool operations", spacecraft_count);
    
    // Create WebSocket server
    let config = get_space_mcp_config();
    let mut server = WebSocketServer::new(config).await?;
    
    // Register MCP handler
    info!("🔨 Registering MCP endpoint...");
    server.receive_envelope_at("/mcp", handler).await?;
    
    info!("✅ Registered MCP endpoint");
    
    // Display server information
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                      🛠️ SPACE EXPLORATION MCP SERVER 🛠️                   ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  🔧 Endpoint: ws://127.0.0.1:8445/mcp                                      ║");
    println!("║                                                                              ║");
    println!("║  Available Tools:                                                            ║");
    println!("║    rover_drill          - Drill sample using rover                         ║");
    println!("║    rover_photo          - Take photo using rover                           ║");
    println!("║    probe_analyze        - Analyze target using probe                       ║");
    println!("║    orbital_scan         - Perform orbital scan using satellite             ║");
    println!("║                                                                              ║");
    println!("║  MCP Protocol:                                                               ║");
    println!("║    • JSON-RPC 2.0 over WebSocket                                           ║");
    println!("║    • Standard MCP tool calling interface                                   ║");
    println!("║    • Real-time space mission tool execution                                ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    Ok(server)
}

// Handler implementation for MCP JSON-RPC over WebSocket

#[async_trait]
impl ContextDataHandler<Value, Value> for SpaceMcpHandler {
    async fn handle(
        &self,
        _context: Option<Context>,
        data: Value,
    ) -> Result<Value> {
        info!("🛠️ MCP REQUEST - Processing MCP request");
        info!("📊 Request details: {:?}", data);
        
        // Parse MCP JSON-RPC request
        match serde_json::from_value::<McpRequest>(data) {
            Ok(mcp_request) => {
                let mcp_response = self.handle_mcp_request(mcp_request);
                let response_value = serde_json::to_value(mcp_response)?;
                
                info!("🔧 MCP RESPONSE - Processing completed");
                info!("📤 Response details: {:?}", response_value);
                Ok(response_value)
            },
            Err(e) => {
                let error_response = McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(McpError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                
                let error_value = serde_json::to_value(error_response)?;
                Ok(error_value)
            }
        }
    }
}