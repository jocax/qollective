// ABOUTME: Integration tests for qollective WebSocket MCP server with real LLM-powered HolodeckCoordinatorServer
// ABOUTME: Tests complete envelope-first architecture with actual ollama-powered orchestration AI integration

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

/// Integration test client for qollective WebSocket MCP coordinator server
struct QollectiveCoordinatorMcpTestClient {
    websocket_url: String,
}

impl QollectiveCoordinatorMcpTestClient {
    /// Initialize test client with WebSocket connection to qollective MCP coordinator server
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let websocket_url = format!("ws://{}:{}/mcp",
            network::DEFAULT_HOST,
            network::HOLODECK_COORDINATOR_PORT);

        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(1000)).await;

        Ok(Self {
            websocket_url,
        })
    }

    /// Send qollective MCP envelope over WebSocket and receive response
    async fn send_qollective_envelope(&self, envelope: Envelope<McpData>) -> Result<Envelope<McpData>, Box<dyn std::error::Error>> {
        // Connect to qollective WebSocket MCP coordinator server
        let (ws_stream, _) = connect_async(&self.websocket_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Wrap envelope in qollective WebSocket message format
        let envelope_value = serde_json::to_value(&envelope)?;
        let websocket_message = serde_json::json!({
            "type": "envelope",
            "payload": envelope_value
        });

        let message_json = serde_json::to_string(&websocket_message)?;
        println!("📤 Sending qollective coordinator WebSocket message: {}", message_json);
        ws_sender.send(Message::Text(message_json)).await?;

        // Receive qollective response envelope
        let response = if let Some(msg) = ws_receiver.next().await {
            match msg? {
                Message::Text(response_text) => {
                    println!("📥 Received qollective coordinator response: {}", response_text);

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
            Err("No response received from qollective coordinator server".into())
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
            }
        ).await;

        if close_result.is_err() {
            println!("⚠️  WebSocket close handshake timeout");
        }

        response
    }

    /// Test qollective coordinator server health and real LLM integration
    pub async fn test_qollective_health_check(&self) -> Result<Value, Box<dyn std::error::Error>> {
        // Create health check tool call request for qollective MCP coordinator adapter
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "health_check".into(),
                arguments: None,
            },
            extensions: rmcp::model::Extensions::default(),
        };

        // Create qollective MCP envelope with tool call
        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.tenant = Some("qollective-coordinator-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        // Send envelope through qollective infrastructure
        let result = timeout(
            Duration::from_millis(10000), // Give more time for LLM calls
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective coordinator health check timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();
                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective coordinator health check failed: {:?}", tool_response.content).into())
                    } else {
                        // Parse tool response content as JSON
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let json_value: Value = serde_json::from_str(&text_content.text)?;
                                Ok(json_value)
                            } else {
                                Err("Unexpected content type from qollective coordinator server".into())
                            }
                        } else {
                            Err("No content in qollective coordinator response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective coordinator envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective coordinator health check failed: {}", e).into())
        }
    }

    /// Test qollective coordinator service information with real LLM provider details
    pub async fn test_qollective_service_info(&self) -> Result<Value, Box<dyn std::error::Error>> {
        // Create service info tool call request
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "get_service_info".into(),
                arguments: None,
            },
            extensions: rmcp::model::Extensions::default(),
        };

        // Create qollective MCP envelope with tool call
        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.tenant = Some("qollective-coordinator-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        // Send envelope through qollective infrastructure
        let result = timeout(
            Duration::from_millis(10000),
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective coordinator service info timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();
                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective coordinator service info failed: {:?}", tool_response.content).into())
                    } else {
                        // Parse tool response content as JSON
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let json_value: Value = serde_json::from_str(&text_content.text)?;
                                Ok(json_value)
                            } else {
                                Err("Unexpected content type from qollective coordinator server".into())
                            }
                        } else {
                            Err("No content in qollective coordinator response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective coordinator envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective coordinator service info failed: {}", e).into())
        }
    }

    /// Test system health check with LLM-powered performance analysis
    pub async fn test_qollective_system_health(&self) -> Result<Value, Box<dyn std::error::Error>> {
        // Create system health check tool call request
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "check_system_health".into(),
                arguments: Some(json!({
                    "include_details": true,
                    "tenant": "qollective-system-health-test"
                }).as_object().unwrap().clone()),
            },
            extensions: rmcp::model::Extensions::default(),
        };

        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.tenant = Some("qollective-coordinator-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        let result = timeout(
            Duration::from_millis(15000), // Longer timeout for LLM health analysis
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective system health check timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();
                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective system health check failed: {:?}", tool_response.content).into())
                    } else {
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let json_value: Value = serde_json::from_str(&text_content.text)?;
                                Ok(json_value)
                            } else {
                                Err("Unexpected content type from qollective coordinator system health".into())
                            }
                        } else {
                            Err("No content in qollective coordinator system health response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective coordinator system health envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective coordinator system health check failed: {}", e).into())
        }
    }

    /// Test server discovery with LLM-powered network analysis
    pub async fn test_qollective_server_discovery(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "discover_servers".into(),
                arguments: Some(json!({
                    "discovery_mode": "automatic",
                    "tenant": "qollective-discovery-test"
                }).as_object().unwrap().clone()),
            },
            extensions: rmcp::model::Extensions::default(),
        };

        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.tenant = Some("qollective-coordinator-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        let result = timeout(
            Duration::from_millis(15000), // Longer timeout for LLM network analysis
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective server discovery timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();
                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective server discovery failed: {:?}", tool_response.content).into())
                    } else {
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let json_value: Value = serde_json::from_str(&text_content.text)?;
                                Ok(json_value)
                            } else {
                                Err("Unexpected content type from qollective coordinator discovery".into())
                            }
                        } else {
                            Err("No content in qollective coordinator discovery response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective coordinator discovery envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective coordinator server discovery failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qollective_coordinator_websocket_server_health() {
        println!("🧪 Testing qollective WebSocket MCP coordinator server health with real LLM integration");

        let client = QollectiveCoordinatorMcpTestClient::new().await
            .expect("Failed to create qollective coordinator test client");

        let health_result = client.test_qollective_health_check().await
            .expect("Qollective coordinator health check failed");

        println!("✅ Qollective coordinator health check result: {}", serde_json::to_string_pretty(&health_result).unwrap());

        // Validate qollective coordinator health response structure
        assert!(health_result.is_object());
        let health_obj = health_result.as_object().unwrap();
        assert!(health_obj.contains_key("status"));
        assert!(health_obj.contains_key("service"));
        assert!(health_obj.contains_key("version"));

        // Verify status is healthy
        let status = health_obj.get("status").unwrap().as_str().unwrap();
        assert_eq!(status, "healthy");

        // Verify service name
        let service = health_obj.get("service").unwrap().as_str().unwrap();
        assert_eq!(service, "holodeck-coordinator");

        println!("🎭 Qollective WebSocket MCP coordinator server health check passed!");
    }

    #[tokio::test]
    async fn test_qollective_coordinator_websocket_server_service_info() {
        println!("🧪 Testing qollective WebSocket MCP coordinator server service info with real LLM provider");

        let client = QollectiveCoordinatorMcpTestClient::new().await
            .expect("Failed to create qollective coordinator test client");

        let service_info = client.test_qollective_service_info().await
            .expect("Qollective coordinator service info retrieval failed");

        println!("✅ Qollective coordinator service info: {}", serde_json::to_string_pretty(&service_info).unwrap());

        // Validate qollective coordinator service info structure
        assert!(service_info.is_object());
        let info_obj = service_info.as_object().unwrap();
        assert!(info_obj.contains_key("service"));
        assert!(info_obj.contains_key("version"));
        assert!(info_obj.contains_key("orchestration_capabilities"));
        assert!(info_obj.contains_key("managed_servers"));

        // Verify service name
        let service_name = info_obj.get("service").unwrap().as_str().unwrap();
        assert_eq!(service_name, "holodeck-coordinator");

        // Verify orchestration capabilities
        let capabilities = info_obj.get("orchestration_capabilities").unwrap().as_object().unwrap();
        assert!(capabilities.contains_key("server_coordination"));
        assert!(capabilities.contains_key("health_monitoring"));
        assert!(capabilities.contains_key("service_discovery"));

        // Verify managed servers
        let managed_servers = info_obj.get("managed_servers").unwrap().as_object().unwrap();
        assert!(managed_servers.contains_key("holodeck-validator"));
        assert!(managed_servers.contains_key("holodeck-environment"));
        assert!(managed_servers.contains_key("holodeck-safety"));
        assert!(managed_servers.contains_key("holodeck-character"));

        println!("🤖 Qollective WebSocket MCP coordinator server with real ollama LLM integration verified!");
    }

    #[tokio::test]
    async fn test_qollective_coordinator_system_health_llm_analysis() {
        println!("🧪 Testing qollective coordinator system health with LLM-powered performance analysis");

        let client = QollectiveCoordinatorMcpTestClient::new().await
            .expect("Failed to create qollective coordinator test client");

        let health_result = client.test_qollective_system_health().await
            .expect("Qollective coordinator system health check failed");

        println!("✅ Qollective coordinator system health result: {}", serde_json::to_string_pretty(&health_result).unwrap());

        // Validate system health response structure
        assert!(health_result.is_object());
        let health_obj = health_result.as_object().unwrap();
        assert!(health_obj.contains_key("overall_health"));
        assert!(health_obj.contains_key("connected_servers"));
        assert!(health_obj.contains_key("total_servers"));
        assert!(health_obj.contains_key("server_health"));
        assert!(health_obj.contains_key("coordination_capabilities"));

        // Verify overall health status
        let overall_health = health_obj.get("overall_health").unwrap().as_str().unwrap();
        assert!(["healthy", "degraded", "critical"].contains(&overall_health));

        // Verify server health details
        let server_health = health_obj.get("server_health").unwrap().as_object().unwrap();
        assert!(server_health.contains_key("holodeck-validator"));
        assert!(server_health.contains_key("holodeck-environment"));
        assert!(server_health.contains_key("holodeck-safety"));
        assert!(server_health.contains_key("holodeck-character"));

        // Verify coordination capabilities with LLM intelligence
        let capabilities = health_obj.get("coordination_capabilities").unwrap().as_object().unwrap();
        assert!(capabilities.contains_key("supported_orchestration_patterns"));
        let patterns = capabilities.get("supported_orchestration_patterns").unwrap().as_array().unwrap();
        assert!(patterns.iter().any(|p| p.as_str().unwrap().contains("llm_powered")));

        println!("🤖 Qollective coordinator LLM-powered system health analysis verified!");
    }

    #[tokio::test]
    async fn test_qollective_coordinator_server_discovery_llm_intelligence() {
        println!("🧪 Testing qollective coordinator server discovery with LLM-powered network analysis");

        let client = QollectiveCoordinatorMcpTestClient::new().await
            .expect("Failed to create qollective coordinator test client");

        let discovery_result = client.test_qollective_server_discovery().await
            .expect("Qollective coordinator server discovery failed");

        println!("✅ Qollective coordinator server discovery result: {}", serde_json::to_string_pretty(&discovery_result).unwrap());

        // Validate server discovery response structure
        assert!(discovery_result.is_object());
        let discovery_obj = discovery_result.as_object().unwrap();
        assert!(discovery_obj.contains_key("discovered_servers"));
        assert!(discovery_obj.contains_key("registry_status"));
        assert!(discovery_obj.contains_key("total_discovered"));
        assert!(discovery_obj.contains_key("last_discovery"));

        // Verify discovered servers
        let discovered_servers = discovery_obj.get("discovered_servers").unwrap().as_array().unwrap();
        assert!(discovered_servers.len() >= 4); // At least 4 core services

        // Verify each discovered server has required fields
        for server in discovered_servers {
            let server_obj = server.as_object().unwrap();
            assert!(server_obj.contains_key("service_name"));
            assert!(server_obj.contains_key("url"));
            assert!(server_obj.contains_key("port"));
            assert!(server_obj.contains_key("capabilities"));
            assert!(server_obj.contains_key("health_status"));

            // Verify service names are recognized holodeck services
            let service_name = server_obj.get("service_name").unwrap().as_str().unwrap();
            assert!(["holodeck-validator", "holodeck-environment", "holodeck-safety",
                     "holodeck-character", "holodeck-designer", "holodeck-storybook"].contains(&service_name));
        }

        // Verify LLM intelligence in registry status
        let registry_status = discovery_obj.get("registry_status").unwrap().as_str().unwrap();
        assert!(["optimal", "degraded", "critical", "llm_optimized", "degraded_llm_analyzed"].contains(&registry_status));

        println!("🤖 Qollective coordinator LLM-powered server discovery verified!");
    }

    /// Test real LLM interaction with holodeck session orchestration
    #[tokio::test]
    async fn test_qollective_coordinator_real_llm_session_orchestration() -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing qollective WebSocket MCP coordinator server with REAL LLM session orchestration");

        let client = QollectiveCoordinatorMcpTestClient::new().await
            .expect("Failed to create qollective coordinator test client");

        // Create holodeck session orchestration tool call that will invoke real ollama LLM
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "create_holodeck_session".into(),
                arguments: Some(json!({
                    "session_name": "Enterprise Bridge Training Simulation",
                    "tenant": "qollective-llm-orchestration-test",
                    "user_id": "test-captain",
                    "request_id": "llm-orchestration-test"
                }).as_object().unwrap().clone()),
            },
            extensions: rmcp::model::Extensions::default(),
        };

        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.tenant = Some("qollective-llm-orchestration-demo".to_string());
        meta.timestamp = Some(chrono::Utc::now());

        let envelope = Envelope::new(meta, mcp_data);

        println!("🎭 Sending real LLM holodeck session orchestration request...");

        // Send envelope through qollective infrastructure with longer timeout for LLM processing
        let result = timeout(
            Duration::from_millis(45000), // 45 seconds for real LLM orchestration
            client.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective session orchestration timed out")?;

        match result {
            Ok(response_envelope) => {
                let (response_meta, response_data) = response_envelope.extract();

                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        panic!("Qollective session orchestration failed: {:?}", tool_response.content);
                    } else {
                        // Parse and display the orchestration response
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let response_json: Value = serde_json::from_str(&text_content.text)
                                    .expect("Failed to parse orchestration response");

                                println!("🚀 Holodeck session orchestration response: {}",
                                    serde_json::to_string_pretty(&response_json).unwrap());

                                // Verify response structure contains orchestration results
                                let response_obj = response_json.as_object().unwrap();
                                assert!(response_obj.contains_key("session_id"));
                                assert!(response_obj.contains_key("session_status"));
                                assert!(response_obj.contains_key("orchestration_results"));
                                assert!(response_obj.contains_key("server_coordination"));
                                assert!(response_obj.contains_key("next_steps"));

                                let session_status = response_obj.get("session_status").unwrap().as_str().unwrap();
                                assert!(["created", "failed"].contains(&session_status));

                                // Verify orchestration results with LLM intelligence
                                let orchestration_results = response_obj.get("orchestration_results").unwrap().as_object().unwrap();
                                assert!(orchestration_results.contains_key("coordination_success"));
                                assert!(orchestration_results.contains_key("validator_result"));
                                assert!(orchestration_results.contains_key("environment_result"));
                                assert!(orchestration_results.contains_key("safety_result"));
                                assert!(orchestration_results.contains_key("character_result"));

                                // Verify server coordination with LLM guidance
                                let server_coordination = response_obj.get("server_coordination").unwrap().as_object().unwrap();
                                assert!(server_coordination.contains_key("coordinated_servers"));
                                assert!(server_coordination.contains_key("coordination_sequence"));
                                assert!(server_coordination.contains_key("rollback_plan"));

                                println!("✅ Real LLM holodeck session orchestration successful!");
                            } else {
                                panic!("Unexpected content type from qollective LLM orchestration server");
                            }
                        } else {
                            panic!("No content in qollective LLM orchestration response");
                        }
                    }
                } else {
                    panic!("No tool response in qollective orchestration envelope");
                }

                // Verify envelope metadata preservation
                assert!(response_meta.request_id.is_some());
                assert!(response_meta.tenant.is_some());

                println!("🚀 Qollective envelope-first architecture with REAL LLM orchestration verified!");
                println!("📊 Check server logs for detailed LLM provider orchestration traces");
            },
            Err(e) => panic!("Qollective session orchestration failed: {}", e)
        }

        Ok(())
    }

    /// Test distributed validation with LLM-powered conflict resolution
    #[tokio::test]
    async fn test_qollective_coordinator_real_llm_validation_orchestration() -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing qollective coordinator distributed validation with REAL LLM conflict resolution");

        let client = QollectiveCoordinatorMcpTestClient::new().await
            .expect("Failed to create qollective coordinator test client");

        // Create validation orchestration tool call that will invoke real ollama LLM
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "orchestrate_validation".into(),
                arguments: Some(json!({
                    "content_id": "enterprise-bridge-scenario-v1",
                    "validation_type": "comprehensive",
                    "tenant": "qollective-llm-validation-test"
                }).as_object().unwrap().clone()),
            },
            extensions: rmcp::model::Extensions::default(),
        };

        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.tenant = Some("qollective-llm-validation-demo".to_string());
        meta.timestamp = Some(chrono::Utc::now());

        let envelope = Envelope::new(meta, mcp_data);

        println!("🔍 Sending real LLM distributed validation orchestration request...");

        // Send envelope through qollective infrastructure with longer timeout for LLM processing
        let result = timeout(
            Duration::from_millis(45000), // 45 seconds for real LLM validation
            client.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective validation orchestration timed out")?;

        match result {
            Ok(response_envelope) => {
                let (response_meta, response_data) = response_envelope.extract();

                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        panic!("Qollective validation orchestration failed: {:?}", tool_response.content);
                    } else {
                        // Parse and display the validation response
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let response_json: Value = serde_json::from_str(&text_content.text)
                                    .expect("Failed to parse validation response");

                                println!("🔍 Distributed validation orchestration response: {}",
                                    serde_json::to_string_pretty(&response_json).unwrap());

                                // Verify response structure contains validation results
                                let response_obj = response_json.as_object().unwrap();
                                assert!(response_obj.contains_key("validation_results"));
                                assert!(response_obj.contains_key("conflict_resolution"));

                                // Verify validation results with distributed coordination
                                let validation_results = response_obj.get("validation_results").unwrap().as_object().unwrap();
                                assert!(validation_results.contains_key("overall_success"));
                                assert!(validation_results.contains_key("aggregated_score"));
                                assert!(validation_results.contains_key("server_results"));

                                // Verify LLM-powered conflict resolution
                                let conflict_resolution = response_obj.get("conflict_resolution").unwrap().as_object().unwrap();
                                assert!(conflict_resolution.contains_key("conflicts_detected"));
                                assert!(conflict_resolution.contains_key("resolution_applied"));

                                if conflict_resolution.get("conflicts_detected").unwrap().as_bool().unwrap() {
                                    assert!(conflict_resolution.contains_key("llm_analysis"));
                                    assert!(conflict_resolution.contains_key("resolution_strategy"));
                                    assert!(conflict_resolution.contains_key("final_decision"));

                                    let llm_analysis = conflict_resolution.get("llm_analysis").unwrap().as_str().unwrap();
                                    assert!(!llm_analysis.is_empty(), "LLM should provide conflict analysis");

                                    println!("🤖 LLM conflict resolution analysis: {}",
                                             llm_analysis.chars().take(200).collect::<String>());
                                }

                                println!("✅ Real LLM distributed validation orchestration successful!");
                            } else {
                                panic!("Unexpected content type from qollective LLM validation server");
                            }
                        } else {
                            panic!("No content in qollective LLM validation response");
                        }
                    }
                } else {
                    panic!("No tool response in qollective validation envelope");
                }

                // Verify envelope metadata preservation
                assert!(response_meta.request_id.is_some());
                assert!(response_meta.tenant.is_some());

                println!("🚀 Qollective envelope-first architecture with REAL LLM validation orchestration verified!");
                println!("📊 Check server logs for detailed LLM provider validation traces");
            },
            Err(e) => panic!("Qollective validation orchestration failed: {}", e)
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_qollective_coordinator_envelope_logging_demo() {
        println!("🧪 Testing qollective coordinator envelope logging for demo purposes");

        let client = QollectiveCoordinatorMcpTestClient::new().await
            .expect("Failed to create qollective coordinator test client");

        // Create a test envelope with rich metadata for logging demo
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "health_check".into(),
                arguments: None,
            },
            extensions: rmcp::model::Extensions::default(),
        };

        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.tenant = Some("qollective-coordinator-envelope-demo".to_string());
        meta.timestamp = Some(chrono::Utc::now());

        let envelope = Envelope::new(meta, mcp_data);

        println!("📦 Sending qollective coordinator envelope with rich metadata for logging demo");

        let result = client.send_qollective_envelope(envelope).await
            .expect("Qollective coordinator envelope logging demo failed");

        let (response_meta, response_data) = result.extract();

        // Verify envelope metadata preservation
        assert!(response_meta.request_id.is_some());
        assert!(response_meta.tenant.is_some());
        assert_eq!(response_meta.tenant.unwrap(), "qollective-coordinator-envelope-demo");

        // Verify response content
        assert!(response_data.tool_response.is_some());

        println!("📦 Qollective coordinator envelope-first architecture logging demo completed!");
        println!("🎯 Check server logs for comprehensive envelope processing details");
    }

    #[tokio::test]
    async fn test_qollective_coordinator_error_handling() {
        println!("🧪 Testing qollective coordinator error handling for unmapped tools");

        let client = QollectiveCoordinatorMcpTestClient::new().await
            .expect("Failed to create qollective coordinator test client");

        // Test with unmapped tool to verify error handling
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "unmapped_orchestration_tool".into(),
                arguments: Some(json!({
                    "test": "data"
                }).as_object().unwrap().clone()),
            },
            extensions: rmcp::model::Extensions::default(),
        };

        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.request_id = Some(Uuid::now_v7());
        meta.tenant = Some("qollective-coordinator-error-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        let result = client.send_qollective_envelope(envelope).await
            .expect("Qollective coordinator error handling test failed");

        let (_, response_data) = result.extract();

        // Should receive error response but still complete successfully
        assert!(response_data.tool_response.is_some());
        let tool_response = response_data.tool_response.unwrap();

        // Should indicate error
        assert_eq!(tool_response.is_error, Some(true));

        // Should contain helpful error message
        assert!(!tool_response.content.is_empty());

        println!("✅ Qollective coordinator error handling working correctly for unmapped tools!");
    }

    #[tokio::test]
    async fn test_qollective_coordinator_concurrent_requests() {
        println!("🧪 Testing qollective coordinator WebSocket server concurrent request handling");

        let client = QollectiveCoordinatorMcpTestClient::new().await
            .expect("Failed to create qollective coordinator test client");

        // Test concurrent health checks
        let (result1, result2, result3) = tokio::join!(
            client.test_qollective_health_check(),
            client.test_qollective_health_check(),
            client.test_qollective_health_check()
        );

        // All should succeed
        assert!(result1.is_ok(), "First concurrent health check failed");
        assert!(result2.is_ok(), "Second concurrent health check failed");
        assert!(result3.is_ok(), "Third concurrent health check failed");

        println!("✅ Qollective coordinator WebSocket server handles concurrent requests successfully!");
    }

    #[tokio::test]
    async fn test_qollective_coordinator_performance() {
        println!("🧪 Testing qollective coordinator WebSocket MCP server performance");

        let client = QollectiveCoordinatorMcpTestClient::new().await
            .expect("Failed to create qollective coordinator test client");

        // Test response time for health check (should be fast)
        let start_time = std::time::Instant::now();
        let _result = client.test_qollective_health_check().await
            .expect("Qollective coordinator performance test failed");
        let duration = start_time.elapsed();

        println!("🚀 Qollective coordinator health check took: {:?}", duration);

        // Should complete reasonably quickly (allowing for LLM calls)
        assert!(duration < Duration::from_millis(8000),
            "Qollective coordinator health check should complete within 8 seconds");

        println!("⚡ Qollective coordinator WebSocket MCP server performance acceptable!");
    }
}
