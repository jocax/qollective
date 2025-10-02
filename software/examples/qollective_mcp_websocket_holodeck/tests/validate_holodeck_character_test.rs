// ABOUTME: Integration tests for qollective WebSocket MCP server with real LLM-powered HolodeckCharacterServer
// ABOUTME: Tests complete envelope-first architecture with actual ollama-powered character AI integration

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
            network::HOLODECK_CHARACTER_PORT);

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
            }
        ).await;

        if close_result.is_err() {
            println!("⚠️  WebSocket close handshake timeout");
        }

        response
    }

    /// Test qollective server health and real LLM integration
    pub async fn test_qollective_health_check(&self) -> Result<Value, Box<dyn std::error::Error>> {
        // Create health check tool call request for qollective MCP adapter
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
        meta.tenant = Some("qollective-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        // Send envelope through qollective infrastructure
        let result = timeout(
            Duration::from_millis(10000), // Give more time for LLM calls
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective health check timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();
                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective health check failed: {:?}", tool_response.content).into())
                    } else {
                        // Parse tool response content as JSON
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let json_value: Value = serde_json::from_str(&text_content.text)?;
                                Ok(json_value)
                            } else {
                                Err("Unexpected content type from qollective server".into())
                            }
                        } else {
                            Err("No content in qollective response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective health check failed: {}", e).into())
        }
    }

    /// Test qollective service information with real LLM provider details
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
        meta.tenant = Some("qollective-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        // Send envelope through qollective infrastructure
        let result = timeout(
            Duration::from_millis(10000),
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective service info timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();
                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective service info failed: {:?}", tool_response.content).into())
                    } else {
                        // Parse tool response content as JSON
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let json_value: Value = serde_json::from_str(&text_content.text)?;
                                Ok(json_value)
                            } else {
                                Err("Unexpected content type from qollective server".into())
                            }
                        } else {
                            Err("No content in qollective response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective service info failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qollective_websocket_server_health() {
        println!("🧪 Testing qollective WebSocket MCP server health with real LLM integration");

        let client = QollectiveMcpTestClient::new().await
            .expect("Failed to create qollective test client");

        let health_result = client.test_qollective_health_check().await
            .expect("Qollective health check failed");

        println!("✅ Qollective health check result: {}", serde_json::to_string_pretty(&health_result).unwrap());

        // Validate qollective health response structure
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
        assert_eq!(service, "holodeck-character");

        println!("🎭 Qollective WebSocket MCP server health check passed!");
    }

    #[tokio::test]
    async fn test_qollective_websocket_server_service_info() {
        println!("🧪 Testing qollective WebSocket MCP server service info with real LLM provider");

        let client = QollectiveMcpTestClient::new().await
            .expect("Failed to create qollective test client");

        let service_info = client.test_qollective_service_info().await
            .expect("Qollective service info retrieval failed");

        println!("✅ Qollective service info: {}", serde_json::to_string_pretty(&service_info).unwrap());

        // Validate qollective service info structure
        assert!(service_info.is_object());
        let info_obj = service_info.as_object().unwrap();
        assert!(info_obj.contains_key("service"));
        assert!(info_obj.contains_key("version"));
        assert!(info_obj.contains_key("llm_provider"));
        assert!(info_obj.contains_key("character_capabilities"));

        // Verify service name
        let service_name = info_obj.get("service").unwrap().as_str().unwrap();
        assert_eq!(service_name, "holodeck-character");

        // Verify real LLM provider (ollama)
        let llm_provider = info_obj.get("llm_provider").unwrap().as_str().unwrap();
        assert_eq!(llm_provider, "ollama");

        // Verify character capabilities
        let capabilities = info_obj.get("character_capabilities").unwrap().as_object().unwrap();
        assert!(capabilities.contains_key("personality_modeling"));
        assert!(capabilities.contains_key("dialogue_generation"));
        assert!(capabilities.contains_key("consistency_validation"));

        println!("🤖 Qollective WebSocket MCP server with real ollama LLM integration verified!");
    }

    #[tokio::test]
    async fn test_qollective_envelope_logging_demo() {
        println!("🧪 Testing qollective envelope logging for demo purposes");

        let client = QollectiveMcpTestClient::new().await
            .expect("Failed to create qollective test client");

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
        meta.tenant = Some("qollective-envelope-demo".to_string());
        meta.timestamp = Some(chrono::Utc::now());

        let envelope = Envelope::new(meta, mcp_data);

        println!("📦 Sending qollective envelope with rich metadata for logging demo");

        let result = client.send_qollective_envelope(envelope).await
            .expect("Qollective envelope logging demo failed");

        let (response_meta, response_data) = result.extract();

        // Verify envelope metadata preservation
        assert!(response_meta.request_id.is_some());
        assert!(response_meta.tenant.is_some());
        assert_eq!(response_meta.tenant.unwrap(), "qollective-envelope-demo");

        // Verify response content
        assert!(response_data.tool_response.is_some());

        println!("📦 Qollective envelope-first architecture logging demo completed!");
        println!("🎯 Check server logs for comprehensive envelope processing details");
    }

    #[tokio::test]
    async fn test_qollective_error_handling() {
        println!("🧪 Testing qollective error handling for unmapped tools");

        let client = QollectiveMcpTestClient::new().await
            .expect("Failed to create qollective test client");

        // Test with unmapped tool to verify error handling
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "unmapped_character_tool".into(),
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
        meta.tenant = Some("qollective-error-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        let result = client.send_qollective_envelope(envelope).await
            .expect("Qollective error handling test failed");

        let (_, response_data) = result.extract();

        // Should receive error response but still complete successfully
        assert!(response_data.tool_response.is_some());
        let tool_response = response_data.tool_response.unwrap();

        // Should indicate error
        assert_eq!(tool_response.is_error, Some(true));

        // Should contain helpful error message
        assert!(!tool_response.content.is_empty());

        println!("✅ Qollective error handling working correctly for unmapped tools!");
    }

    #[tokio::test]
    async fn test_qollective_concurrent_requests() {
        println!("🧪 Testing qollective WebSocket server concurrent request handling");

        let client = QollectiveMcpTestClient::new().await
            .expect("Failed to create qollective test client");

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

        println!("✅ Qollective WebSocket server handles concurrent requests successfully!");
    }

    #[tokio::test]
    async fn test_qollective_performance() {
        println!("🧪 Testing qollective WebSocket MCP server performance");

        let client = QollectiveMcpTestClient::new().await
            .expect("Failed to create qollective test client");

        // Test response time for health check (should be fast)
        let start_time = std::time::Instant::now();
        let _result = client.test_qollective_health_check().await
            .expect("Qollective performance test failed");
        let duration = start_time.elapsed();

        println!("🚀 Qollective health check took: {:?}", duration);

        // Should complete reasonably quickly (allowing for LLM calls)
        assert!(duration < Duration::from_millis(5000),
            "Qollective health check should complete within 5 seconds");

        println!("⚡ Qollective WebSocket MCP server performance acceptable!");
    }

    /// Test real LLM interaction with character dialogue generation
    #[tokio::test]
    async fn test_qollective_real_llm_character_interaction() -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing qollective WebSocket MCP server with REAL LLM character interaction");

        let client = QollectiveMcpTestClient::new().await
            .expect("Failed to create qollective test client");

        // Create character interaction tool call that will invoke real ollama LLM
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "interact_with_character".into(),
                arguments: Some(json!({
                    "character_name": "Picard",
                    "user_message": "Captain, what do you think about our current mission?",
                    "player_action": "asks for captain's opinion",
                    "conversation_context": "Bridge discussion about exploration",
                    "character_mood": "thoughtful",
                    "tenant": "qollective-llm-test",
                    "user_id": "test-user",
                    "request_id": "llm-interaction-test"
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
        meta.tenant = Some("qollective-llm-demo".to_string());
        meta.timestamp = Some(chrono::Utc::now());

        let envelope = Envelope::new(meta, mcp_data);

        println!("🎭 Sending real LLM character interaction request to Captain Picard...");

        // Send envelope through qollective infrastructure with longer timeout for LLM processing
        let result = timeout(
            Duration::from_millis(30000), // 30 seconds for real LLM call
            client.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective character interaction timed out")?;

        match result {
            Ok(response_envelope) => {
                let (response_meta, response_data) = response_envelope.extract();

                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        panic!("Qollective character interaction failed: {:?}", tool_response.content);
                    } else {
                        // Parse and display the character response
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let response_json: Value = serde_json::from_str(&text_content.text)
                                    .expect("Failed to parse character response");

                                println!("🖖 Captain Picard's response: {}",
                                    serde_json::to_string_pretty(&response_json).unwrap());

                                // Verify response structure contains character dialogue
                                let response_obj = response_json.as_object().unwrap();
                                assert!(response_obj.contains_key("character_name"));
                                assert!(response_obj.contains_key("character_response"));
                                assert!(response_obj.contains_key("response_metadata"));

                                let character_name = response_obj.get("character_name").unwrap().as_str().unwrap();
                                assert_eq!(character_name, "Picard");

                                let character_response = response_obj.get("character_response").unwrap().as_str().unwrap();
                                assert!(!character_response.is_empty(), "Character should provide a response");

                                println!("✅ Real LLM character interaction successful! Response length: {} chars",
                                    character_response.len());
                            } else {
                                panic!("Unexpected content type from qollective LLM character server");
                            }
                        } else {
                            panic!("No content in qollective LLM character response");
                        }
                    }
                } else {
                    panic!("No tool response in qollective character envelope");
                }

                // Verify envelope metadata preservation
                assert!(response_meta.request_id.is_some());
                assert!(response_meta.tenant.is_some());

                println!("🚀 Qollective envelope-first architecture with REAL LLM integration verified!");
                println!("📊 Check server logs for detailed LLM provider interaction traces");
            },
            Err(e) => panic!("Qollective character interaction failed: {}", e)
        }

        Ok(())
    }
}
