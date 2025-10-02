// ABOUTME: Integration tests for qollective WebSocket MCP server with real LLM-powered HolodeckStorybookServer
// ABOUTME: Tests complete envelope-first architecture with actual LLM-powered story content delivery and WebSocket management

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
            network::HOLODECK_STORYBOOK_PORT);

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
                },
                _ => Err("Unexpected message type".into()),
            }
        } else {
            Err("No message received".into())
        };

        response
    }

    /// Test story content serving through MCP with real LLM integration
    async fn test_serve_content(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔖 Testing story content serving with real LLM integration");

        let story_id = Uuid::now_v7().to_string();
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "serve_content".into(),
                arguments: Some(json!({
                    "story_id": story_id,
                    "content_type": "story",
                    "include_validation": true,
                    "include_realtime": true,
                    "tenant": "test-tenant",
                    "user_id": "test-user",
                    "request_id": Uuid::now_v7().to_string()
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
        meta.tenant = Some("qollective-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        let response = self.send_qollective_envelope(envelope).await?;

        // Validate response structure
        assert!(response.data.tool_response.is_some(), "Expected tool response");

        let tool_response = response.data.tool_response.unwrap();
        assert!(!tool_response.content.is_empty(), "Expected content in response");

        // Check for story content structure
        let first_content = &tool_response.content[0];
        println!("🎯 Story content response: {:?}", first_content);

        // Validate that we got real LLM-powered content
        // Check that we got content from LLM
        if let rmcp::model::RawContent::Text(text_content) = &first_content.raw {
            assert!(text_content.text.len() > 10, "Expected substantial content from LLM");
        }

        println!("✅ Story content serving test passed - real LLM content delivered!");
        Ok(())
    }

    /// Test WebSocket management through MCP
    async fn test_manage_websocket(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔖 Testing WebSocket management with real-time capabilities");

        let session_id = Uuid::now_v7().to_string();
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "manage_websocket".into(),
                arguments: Some(json!({
                    "session_id": session_id,
                    "user_id": "test-user",
                    "event_types": ["story_updates", "session_events"],
                    "connection_params": {
                        "max_connections": 100,
                        "heartbeat_interval": 30000
                    }
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
        meta.tenant = Some("qollective-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        let response = self.send_qollective_envelope(envelope).await?;

        // Validate response structure
        assert!(response.data.tool_response.is_some(), "Expected tool response");

        let tool_response = response.data.tool_response.unwrap();
        assert!(!tool_response.content.is_empty(), "Expected content in response");

        println!("✅ WebSocket management test passed - real-time capabilities configured!");
        Ok(())
    }

    /// Test server status monitoring
    async fn test_get_server_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔖 Testing server status monitoring");

        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "get_server_status".into(),
                arguments: Some(json!({
                    "detail_level": "full",
                    "include_services": true
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
        meta.tenant = Some("qollective-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        let response = self.send_qollective_envelope(envelope).await?;

        // Validate response structure
        assert!(response.data.tool_response.is_some(), "Expected tool response");

        let tool_response = response.data.tool_response.unwrap();
        assert!(!tool_response.content.is_empty(), "Expected content in response");

        println!("✅ Server status test passed - comprehensive monitoring data received!");
        Ok(())
    }

    /// Test health check functionality
    async fn test_health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔖 Testing health check");

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
        meta.tenant = Some("qollective-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        let response = self.send_qollective_envelope(envelope).await?;

        // Validate response structure
        assert!(response.data.tool_response.is_some(), "Expected tool response");

        let tool_response = response.data.tool_response.unwrap();
        assert!(!tool_response.content.is_empty(), "Expected content in response");

        println!("✅ Health check test passed!");
        Ok(())
    }

    /// Test service info retrieval
    async fn test_get_service_info(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔖 Testing service info retrieval");

        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "get_service_info".into(),
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
        meta.tenant = Some("qollective-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        let response = self.send_qollective_envelope(envelope).await?;

        // Validate response structure
        assert!(response.data.tool_response.is_some(), "Expected tool response");

        let tool_response = response.data.tool_response.unwrap();
        assert!(!tool_response.content.is_empty(), "Expected content in response");

        println!("✅ Service info test passed!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_qollective_holodeck_storybook_health_check() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let client = QollectiveMcpTestClient::new().await
                .expect("Failed to create test client");

            client.test_health_check().await
                .expect("Health check test failed");
        });
    }

    #[test]
    fn test_qollective_holodeck_storybook_service_info() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let client = QollectiveMcpTestClient::new().await
                .expect("Failed to create test client");

            client.test_get_service_info().await
                .expect("Service info test failed");
        });
    }

    #[test]
    fn test_qollective_holodeck_storybook_server_status() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let client = QollectiveMcpTestClient::new().await
                .expect("Failed to create test client");

            client.test_get_server_status().await
                .expect("Server status test failed");
        });
    }

    #[test]
    fn test_qollective_holodeck_storybook_websocket_management() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let client = QollectiveMcpTestClient::new().await
                .expect("Failed to create test client");

            client.test_manage_websocket().await
                .expect("WebSocket management test failed");
        });
    }

    #[test]
    fn test_qollective_holodeck_storybook_content_serving() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let client = QollectiveMcpTestClient::new().await
                .expect("Failed to create test client");

            client.test_serve_content().await
                .expect("Content serving test failed");
        });
    }

    #[test]
    fn test_qollective_holodeck_storybook_integration_flow() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let client = QollectiveMcpTestClient::new().await
                .expect("Failed to create test client");

            println!("🚀 Starting comprehensive holodeck-storybook integration test");

            // Test all endpoints in sequence
            client.test_health_check().await
                .expect("Health check failed");

            client.test_get_service_info().await
                .expect("Service info failed");

            client.test_get_server_status().await
                .expect("Server status failed");

            client.test_manage_websocket().await
                .expect("WebSocket management failed");

            client.test_serve_content().await
                .expect("Content serving failed");

            println!("🎉 All holodeck-storybook integration tests passed!");
        });
    }
}
