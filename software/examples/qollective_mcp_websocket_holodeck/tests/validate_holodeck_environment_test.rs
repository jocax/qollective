// ABOUTME: Integration tests for qollective WebSocket MCP server with real LLM-powered HolodeckEnvironmentServer
// ABOUTME: Tests complete envelope-first architecture with actual LLM-powered 3D environment generation and service integration

use std::time::Duration;
use uuid::Uuid;
use serde_json::{json, Value};
use tokio::time::timeout;
use shared_types::constants::{network, timeouts};
use qollective::envelope::{Envelope, Meta};
use qollective::types::mcp::McpData;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use rmcp::model::{CallToolRequest, CallToolRequestParam};

/// Integration test client for qollective WebSocket MCP server
struct QollectiveMcpTestClient {
    websocket_url: String,
}

impl QollectiveMcpTestClient {
    /// Initialize test client with WebSocket connection to qollective MCP server
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let websocket_url = format!("ws://{}:{}/mcp",
            network::DEFAULT_HOST,
            network::HOLODECK_ENVIRONMENT_PORT);

        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(1000)).await;

        Ok(Self {
            websocket_url,
        })
    }

    /// Send qollective MCP envelope over WebSocket and receive response
    async fn send_qollective_envelope(&self, envelope: Envelope<McpData>) -> Result<Envelope<McpData>, Box<dyn std::error::Error>> {
        // Connect to qollective WebSocket MCP server
        let (ws_stream, _) = connect_async(&self.websocket_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Wrap envelope in qollective WebSocket message format
        let envelope_value = serde_json::to_value(&envelope)?;
        let websocket_message = serde_json::json!({
            "type": "envelope",
            "payload": envelope_value
        });

        let message_json = serde_json::to_string(&websocket_message)?;
        println!("📤 Sending qollective WebSocket message: {}", message_json);
        ws_sender.send(Message::Text(message_json)).await?;

        // Receive qollective response envelope
        let response = if let Some(msg) = ws_receiver.next().await {
            match msg? {
                Message::Text(response_text) => {
                    println!("📥 Received qollective response: {}", response_text);

                    // Try to parse as wrapped WebSocket message first
                    if let Ok(websocket_response) = serde_json::from_str::<serde_json::Value>(&response_text) {
                        if let Some(data) = websocket_response.get("payload") {
                            println!("📦 Extracting envelope from WebSocket wrapper");
                            let response_envelope: Envelope<McpData> = serde_json::from_value(data.clone())?;
                            return Ok(response_envelope);
                        }
                    }

                    // Fall back to direct envelope parsing
                    let response_envelope: Envelope<McpData> = serde_json::from_str(&response_text)?;
                    Ok(response_envelope)
                }
                _ => {
                    Err("Unexpected WebSocket message type".into())
                }
            }
        } else {
            Err("No response received from qollective server".into())
        };

        // Properly close WebSocket connection with close handshake and timeout
        let close_result = tokio::time::timeout(
            std::time::Duration::from_millis(1000),
            async move {
                if let Err(e) = ws_sender.send(Message::Close(None)).await {
                    println!("⚠️  WebSocket close send failed: {}", e);
                    return;
                }

                // Wait for close frame response
                while let Some(msg) = ws_receiver.next().await {
                    match msg {
                        Ok(Message::Close(_)) => {
                            println!("✅ WebSocket close handshake completed");
                            return;
                        }
                        Ok(_) => continue,
                        Err(e) => {
                            println!("⚠️  WebSocket close error: {}", e);
                            return;
                        }
                    }
                }

                println!("⚠️  WebSocket close timeout - connection may not have closed gracefully");
            }
        ).await;

        if close_result.is_err() {
            println!("⚠️  WebSocket close operation timed out");
        }

        response
    }

    /// Create qollective MCP envelope with metadata
    fn create_envelope(&self, tool_call: CallToolRequest) -> Envelope<McpData> {
        let meta = Meta {
            timestamp: Some(chrono::Utc::now()),
            request_id: Some(Uuid::now_v7()),
            version: Some("1.0.0".to_string()),
            duration: None,
            tenant: Some("qollective.holodeck.environment.test".to_string()),
            on_behalf_of: None,
            security: None,
            debug: None,
            performance: None,
            monitoring: None,
            extensions: None,
            tracing: None,
        };

        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        Envelope::new(meta, mcp_data)
    }
}

#[tokio::test]
async fn test_qollective_websocket_connection() {
    println!("🧪 Testing qollective WebSocket MCP connection to holodeck-environment server");

    // Test WebSocket connection
    let websocket_url = format!("ws://{}:{}/mcp",
        network::DEFAULT_HOST,
        network::HOLODECK_ENVIRONMENT_PORT);

    let connection_result = tokio::time::timeout(
        Duration::from_secs(5),
        connect_async(&websocket_url)
    ).await;

    match connection_result {
        Ok(Ok((ws_stream, response))) => {
            println!("✅ Successfully connected to qollective holodeck-environment WebSocket server");
            println!("🔗 Connection status: {}", response.status());

            // Close connection gracefully
            let (mut sender, _receiver) = ws_stream.split();
            let _ = sender.send(Message::Close(None)).await;
        }
        Ok(Err(e)) => {
            panic!("❌ Failed to connect to qollective holodeck-environment server: {}. Make sure the server is running with: cargo run -p holodeck-environment", e);
        }
        Err(_) => {
            panic!("❌ Connection timeout to qollective holodeck-environment server. Make sure the server is running with: cargo run -p holodeck-environment");
        }
    }
}

#[tokio::test]
async fn test_health_check() {
    println!("🧪 Testing health check via qollective MCP envelope");

    let client = QollectiveMcpTestClient::new().await
        .expect("Failed to create qollective environment test client");

    // Create health check tool call
    let tool_call = CallToolRequest {
        method: Default::default(),
        params: CallToolRequestParam {
            name: "health_check".to_string().into(),
            arguments: None,
        },
        extensions: Default::default(),
    };

    let envelope = client.create_envelope(tool_call);

    let response = timeout(
        timeouts::DEFAULT_MCP_REQUEST_TIMEOUT,
        client.send_qollective_envelope(envelope)
    ).await
        .expect("Health check request timed out")
        .expect("Failed to send health check request");

    // Verify qollective envelope response
    assert!(response.data.tool_response.is_some(), "Expected tool response in qollective envelope");

    let tool_response = response.data.tool_response.unwrap();
    assert!(!tool_response.is_error.unwrap_or(true), "Health check should not return error");
    assert!(!tool_response.content.is_empty(), "Health check should return content");

    // Parse health check response
    let content = &tool_response.content[0];
    match &content.raw {
        rmcp::model::RawContent::Text(text_content) => {
            let health_data: Value = serde_json::from_str(&text_content.text)
                .expect("Failed to parse health check JSON");

            assert_eq!(health_data["service"], "holodeck-environment");
            assert_eq!(health_data["status"], "healthy");
            assert!(health_data["uptime_seconds"].is_number());

            println!("✅ Health check successful: {}", health_data);
        }
        _ => panic!("Expected text content in health check response"),
    }
}

#[tokio::test]
async fn test_get_service_info() {
    println!("🧪 Testing service info via qollective MCP envelope");

    let client = QollectiveMcpTestClient::new().await
        .expect("Failed to create qollective environment test client");

    // Create service info tool call request
    let tool_call = CallToolRequest {
        method: Default::default(),
        params: CallToolRequestParam {
            name: "get_service_info".to_string().into(),
            arguments: None,
        },
        extensions: Default::default(),
    };

    let envelope = client.create_envelope(tool_call);

    let response = timeout(
        timeouts::DEFAULT_MCP_REQUEST_TIMEOUT,
        client.send_qollective_envelope(envelope)
    ).await
        .expect("Service info request timed out")
        .expect("Failed to send service info request");

    // Verify qollective envelope response
    assert!(response.data.tool_response.is_some(), "Expected tool response in qollective envelope");

    let tool_response = response.data.tool_response.unwrap();
    assert!(!tool_response.is_error.unwrap_or(true), "Service info should not return error");
    assert!(!tool_response.content.is_empty(), "Service info should return content");

    // Parse service info response
    let content = &tool_response.content[0];
    match &content.raw {
        rmcp::model::RawContent::Text(text_content) => {
            let service_info: Value = serde_json::from_str(&text_content.text)
                .expect("Failed to parse service info JSON");

            assert_eq!(service_info["service_name"], "holodeck-environment");
            assert_eq!(service_info["version"], "0.1.0");
            assert_eq!(service_info["port"], network::HOLODECK_ENVIRONMENT_PORT);
            assert!(service_info["build_info"].as_str().unwrap().contains("Phase 5"));

            println!("✅ Service info successful: {}", service_info);
        }
        _ => panic!("Expected text content in service info response"),
    }
}

#[tokio::test]
async fn test_generate_environment() {
    println!("🧪 Testing 3D environment generation via qollective MCP envelope");

    let client = QollectiveMcpTestClient::new().await
        .expect("Failed to create qollective environment test client");

    // Create environment generation request
    let tool_call = CallToolRequest {
        method: Default::default(),
        params: CallToolRequestParam {
            name: "generate_environment".to_string().into(),
            arguments: Some({
                let mut map = serde_json::Map::new();
                map.insert("scene_description".to_string(), json!("A majestic Star Trek Enterprise bridge with holographic displays"));
                map.insert("environment_type".to_string(), json!("starship"));
                map.insert("safety_level".to_string(), json!("standard"));
                map.insert("tenant".to_string(), json!("test-tenant"));
                map.insert("user_id".to_string(), json!("test-user"));
                map.insert("request_id".to_string(), json!("test-env-gen-001"));
                map
            }),
        },
        extensions: Default::default(),
    };

    let envelope = client.create_envelope(tool_call);

    let response = timeout(
        Duration::from_secs(30), // Longer timeout for LLM generation
        client.send_qollective_envelope(envelope)
    ).await
        .expect("Environment generation request timed out")
        .expect("Failed to send environment generation request");

    // Verify qollective envelope response
    assert!(response.data.tool_response.is_some(), "Expected tool response in qollective envelope");

    let tool_response = response.data.tool_response.unwrap();
    assert!(!tool_response.is_error.unwrap_or(true), "Environment generation should not return error");
    assert!(!tool_response.content.is_empty(), "Environment generation should return content");

    // Parse environment generation response
    let content = &tool_response.content[0];
    match &content.raw {
        rmcp::model::RawContent::Text(text_content) => {
            let env_data: Value = serde_json::from_str(&text_content.text)
                .expect("Failed to parse environment generation JSON");

            // Verify 3D environment structure
            assert!(env_data["environment"].is_object(), "Should have environment object");
            assert!(env_data["environment"]["spatial_layout"].is_object(), "Should have spatial layout");
            assert!(env_data["environment"]["atmospheric_conditions"].is_object(), "Should have atmospheric conditions");
            assert!(env_data["environment"]["interactive_elements"].is_array(), "Should have interactive elements");
            assert!(env_data["environment"]["environment_type"].is_string(), "Should have environment type");
            assert!(env_data["environment"]["safety_level"].is_string(), "Should have safety level");

            // Verify generation metadata
            assert!(env_data["generation_metadata"].is_object(), "Should have generation metadata");
            assert!(env_data["generation_metadata"]["generation_time_ms"].is_number(), "Should have generation time");
            assert!(env_data["generation_metadata"]["llm_provider_used"].is_string(), "Should have LLM provider info");

            println!("✅ 3D Environment generation successful!");
            println!("🌍 Environment type: {}", env_data["environment"]["environment_type"]);
            println!("🛡️ Safety level: {}", env_data["environment"]["safety_level"]);
            println!("⏱️ Generation time: {}ms", env_data["generation_metadata"]["generation_time_ms"]);
            println!("🤖 LLM provider: {}", env_data["generation_metadata"]["llm_provider_used"]);
        }
        _ => panic!("Expected text content in environment generation response"),
    }
}

#[tokio::test]
async fn test_manage_scene_update_lighting() {
    println!("🧪 Testing scene management (lighting update) via qollective MCP envelope");

    let client = QollectiveMcpTestClient::new().await
        .expect("Failed to create qollective environment test client");

    // Create scene management request for lighting update
    let tool_call = CallToolRequest {
        method: Default::default(),
        params: CallToolRequestParam {
            name: "manage_scene".to_string().into(),
            arguments: Some({
                let mut map = serde_json::Map::new();
                map.insert("scene_id".to_string(), json!("enterprise-bridge-001"));
                map.insert("operation_type".to_string(), json!("update_lighting"));
                map.insert("modification_parameters".to_string(), json!("Increase ambient lighting by 20% and add warm bridge ambiance"));
                map
            }),
        },
        extensions: Default::default(),
    };

    let envelope = client.create_envelope(tool_call);

    let response = timeout(
        Duration::from_secs(20), // Timeout for scene management
        client.send_qollective_envelope(envelope)
    ).await
        .expect("Scene management request timed out")
        .expect("Failed to send scene management request");

    // Verify qollective envelope response
    assert!(response.data.tool_response.is_some(), "Expected tool response in qollective envelope");

    let tool_response = response.data.tool_response.unwrap();
    assert!(!tool_response.is_error.unwrap_or(true), "Scene management should not return error");
    assert!(!tool_response.content.is_empty(), "Scene management should return content");

    // Parse scene management response
    let content = &tool_response.content[0];
    match &content.raw {
        rmcp::model::RawContent::Text(text_content) => {
            let scene_data: Value = serde_json::from_str(&text_content.text)
                .expect("Failed to parse scene management JSON");

            // Verify scene management structure
            assert!(scene_data["scene_id"].is_string(), "Should have scene ID");
            assert_eq!(scene_data["scene_id"], "enterprise-bridge-001");
            assert!(scene_data["operation_type"].is_string(), "Should have operation type");
            assert_eq!(scene_data["operation_type"], "update_lighting");
            assert!(scene_data["modifications_applied"].is_array(), "Should have modifications applied");
            assert!(scene_data["management_metadata"].is_object(), "Should have management metadata");

            println!("✅ Scene management (lighting) successful!");
            println!("🎬 Scene ID: {}", scene_data["scene_id"]);
            println!("🔧 Operation: {}", scene_data["operation_type"]);
            println!("✨ Modifications: {}", scene_data["modifications_applied"].as_array().unwrap().len());
        }
        _ => panic!("Expected text content in scene management response"),
    }
}

#[tokio::test]
async fn test_manage_scene_change_weather() {
    println!("🧪 Testing scene management (weather change) via qollective MCP envelope");

    let client = QollectiveMcpTestClient::new().await
        .expect("Failed to create qollective environment test client");

    // Create scene management request for weather change
    let tool_call = CallToolRequest {
        method: Default::default(),
        params: CallToolRequestParam {
            name: "manage_scene".to_string().into(),
            arguments: Some({
                let mut map = serde_json::Map::new();
                map.insert("scene_id".to_string(), json!("alien-world-forest"));
                map.insert("operation_type".to_string(), json!("change_weather"));
                map.insert("modification_parameters".to_string(), json!("Begin gentle rainfall with atmospheric mist and distant thunder"));
                map
            }),
        },
        extensions: Default::default(),
    };

    let envelope = client.create_envelope(tool_call);

    let response = timeout(
        Duration::from_secs(20),
        client.send_qollective_envelope(envelope)
    ).await
        .expect("Weather change request timed out")
        .expect("Failed to send weather change request");

    // Verify qollective envelope response
    assert!(response.data.tool_response.is_some(), "Expected tool response in qollective envelope");

    let tool_response = response.data.tool_response.unwrap();
    assert!(!tool_response.is_error.unwrap_or(true), "Weather change should not return error");
    assert!(!tool_response.content.is_empty(), "Weather change should return content");

    // Parse weather change response
    let content = &tool_response.content[0];
    match &content.raw {
        rmcp::model::RawContent::Text(text_content) => {
            let weather_data: Value = serde_json::from_str(&text_content.text)
                .expect("Failed to parse weather change JSON");

            assert_eq!(weather_data["operation_type"], "change_weather");
            assert!(weather_data["modifications_applied"].is_array(), "Should have weather modifications");

            println!("✅ Weather change successful!");
            println!("🌧️ Scene: {}", weather_data["scene_id"]);
            println!("🌦️ Weather modifications: {}", weather_data["modifications_applied"].as_array().unwrap().len());
        }
        _ => panic!("Expected text content in weather change response"),
    }
}

#[tokio::test]
async fn test_validate_environmental_safety() {
    println!("🧪 Testing environmental safety validation via qollective MCP envelope");

    let client = QollectiveMcpTestClient::new().await
        .expect("Failed to create qollective environment test client");

    // Create environmental safety validation request
    let tool_call = CallToolRequest {
        method: Default::default(),
        params: CallToolRequestParam {
            name: "validate_environmental_safety".to_string().into(),
            arguments: Some({
                let mut map = serde_json::Map::new();
                map.insert("environment_id".to_string(), json!("starship-bridge-test"));
                map.insert("safety_level".to_string(), json!("standard"));
                map
            }),
        },
        extensions: Default::default(),
    };

    let envelope = client.create_envelope(tool_call);

    let response = timeout(
        Duration::from_secs(15),
        client.send_qollective_envelope(envelope)
    ).await
        .expect("Safety validation request timed out")
        .expect("Failed to send safety validation request");

    // Verify qollective envelope response
    assert!(response.data.tool_response.is_some(), "Expected tool response in qollective envelope");

    let tool_response = response.data.tool_response.unwrap();
    assert!(!tool_response.is_error.unwrap_or(true), "Safety validation should not return error");
    assert!(!tool_response.content.is_empty(), "Safety validation should return content");

    // Parse safety validation response
    let content = &tool_response.content[0];
    match &content.raw {
        rmcp::model::RawContent::Text(text_content) => {
            let safety_data: Value = serde_json::from_str(&text_content.text)
                .expect("Failed to parse safety validation JSON");

            // Verify safety validation structure
            assert!(safety_data["environment_id"].is_string(), "Should have environment ID");
            assert!(safety_data["safety_level"].is_string(), "Should have safety level");
            assert!(safety_data["validation_result"].is_object(), "Should have validation result");
            assert!(safety_data["safety_constraints_checked"].is_array(), "Should have constraints checked");
            assert!(safety_data["compliance_report"].is_object(), "Should have compliance report");

            // Verify validation passed
            assert_eq!(safety_data["validation_result"]["status"], "compliant", "Environment should be safety compliant");

            println!("✅ Environmental safety validation successful!");
            println!("🛡️ Environment: {}", safety_data["environment_id"]);
            println!("📊 Safety level: {}", safety_data["safety_level"]);
            println!("✅ Status: {}", safety_data["validation_result"]["status"]);
            println!("🔍 Constraints checked: {}", safety_data["safety_constraints_checked"].as_array().unwrap().len());
        }
        _ => panic!("Expected text content in safety validation response"),
    }
}

#[tokio::test]
async fn test_invalid_tool_call() {
    println!("🧪 Testing invalid tool call error handling via qollective MCP envelope");

    let client = QollectiveMcpTestClient::new().await
        .expect("Failed to create qollective environment test client");

    // Create invalid tool call
    let tool_call = CallToolRequest {
        method: Default::default(),
        params: CallToolRequestParam {
            name: "nonexistent_tool".to_string().into(),
            arguments: None,
        },
        extensions: Default::default(),
    };

    let envelope = client.create_envelope(tool_call);

    let response = timeout(
        timeouts::DEFAULT_MCP_REQUEST_TIMEOUT,
        client.send_qollective_envelope(envelope)
    ).await
        .expect("Invalid tool call request timed out")
        .expect("Failed to send invalid tool call request");

    // Verify qollective envelope response contains error
    assert!(response.data.tool_response.is_some(), "Expected tool response in qollective envelope");

    let tool_response = response.data.tool_response.unwrap();
    assert!(tool_response.is_error.unwrap_or(false), "Invalid tool call should return error");
    assert!(!tool_response.content.is_empty(), "Error response should return content");

    // Parse error response
    let content = &tool_response.content[0];
    match &content.raw {
        rmcp::model::RawContent::Text(text_content) => {
            assert!(text_content.text.contains("not mapped"), "Error should mention tool not mapped");
            assert!(text_content.text.contains("Available tools"), "Error should list available tools");

            println!("✅ Invalid tool call correctly handled with error");
            println!("❌ Error message: {}", text_content.text);
        }
        _ => panic!("Expected text content in error response"),
    }
}

#[tokio::test]
async fn test_environment_generation_with_different_types() {
    println!("🧪 Testing environment generation with different environment types");

    let client = QollectiveMcpTestClient::new().await
        .expect("Failed to create qollective environment test client");

    let environment_types = vec![
        ("alien", "An exotic alien jungle with bioluminescent flora"),
        ("fantasy", "A magical wizard's tower with floating books and mystical artifacts"),
        ("historical", "A Victorian-era London street scene with cobblestones and gas lamps"),
        ("space", "A cosmic nebula observation deck with swirling galaxies visible"),
    ];

    for (env_type, description) in environment_types {
        println!("🌍 Testing {} environment generation", env_type);

        let tool_call = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "generate_environment".to_string().into(),
                arguments: Some({
                    let mut map = serde_json::Map::new();
                    map.insert("scene_description".to_string(), json!(description));
                    map.insert("environment_type".to_string(), json!(env_type));
                    map.insert("safety_level".to_string(), json!("standard"));
                    map.insert("request_id".to_string(), json!(format!("test-{}-env", env_type)));
                    map
                }),
            },
            extensions: Default::default(),
        };

        let envelope = client.create_envelope(tool_call);

        let response = timeout(
            Duration::from_secs(30),
            client.send_qollective_envelope(envelope)
        ).await
            .expect(&format!("{} environment generation timed out", env_type))
            .expect(&format!("Failed to generate {} environment", env_type));

        // Verify response
        assert!(response.data.tool_response.is_some());
        let tool_response = response.data.tool_response.unwrap();
        assert!(!tool_response.is_error.unwrap_or(true), "Environment generation should succeed");

        let content = &tool_response.content[0];
        match &content.raw {
            rmcp::model::RawContent::Text(text_content) => {
                let env_data: Value = serde_json::from_str(&text_content.text)
                    .expect("Failed to parse environment JSON");

                assert!(env_data["environment"]["environment_type"].as_str().unwrap().contains(env_type) ||
                        env_data["environment"]["environment_type"].as_str() == Some("TrainingFacility"),
                        "Environment type should match request or be TrainingFacility fallback");

                println!("✅ {} environment generated successfully!", env_type);
            }
            _ => panic!("Expected text content in environment response"),
        }
    }
}
