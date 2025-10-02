// ABOUTME: Integration tests for qollective WebSocket MCP server with real LLM-powered HolodeckDesignerServer
// ABOUTME: Tests complete envelope-first architecture with actual LLM-powered story generation, enhancement, and validation

use std::time::Duration;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use serde_json::{json, Value};
use tokio::time::timeout;
use shared_types::constants::{network, timeouts};
use qollective::envelope::{Envelope, Meta};
use qollective::types::mcp::McpData;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use rmcp::model::{CallToolRequest, CallToolRequestParam};
use chrono::prelude::*;

/// Integration test client for qollective WebSocket MCP server with holodeck-designer
struct QollectiveDesignerTestClient {
    websocket_url: String,
}

impl QollectiveDesignerTestClient {
    /// Initialize test client with WebSocket connection to qollective MCP server
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let websocket_url = format!("ws://{}:{}/mcp",
            network::DEFAULT_HOST,
            network::HOLODECK_DESIGNER_PORT);

        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(1000)).await;

        Ok(Self {
            websocket_url,
        })
    }

    /// Save story result to target folder with timestamp and story title pattern
    fn save_story_result(&self, story_result: &Value, test_type: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create target directory if it doesn't exist
        let target_dir = PathBuf::from("target");
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        // Extract story title from result
        let story_title = story_result
            .get("story_template")
            .and_then(|template| template.get("name"))
            .and_then(|name| name.as_str())
            .unwrap_or("unknown_story")
            .replace(":", "_")
            .replace("/", "_")
            .replace("\\", "_")
            .replace(" ", "_");

        // Generate timestamp
        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%d_%H-%M-%S");

        // Create filename
        let filename = format!("{}_{}_{}_{}.json", timestamp, test_type, story_title, story_result.get("story_id").and_then(|id| id.as_str()).unwrap_or("no_id"));
        let filepath = target_dir.join(filename);

        // Save pretty-printed JSON
        let json_content = serde_json::to_string_pretty(story_result)?;
        fs::write(&filepath, json_content)?;

        let saved_path = filepath.to_string_lossy().to_string();
        println!("💾 Saved story result to: {}", saved_path);
        Ok(saved_path)
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

    /// Test real LLM story generation
    pub async fn test_qollective_real_llm_story_generation(&self) -> Result<Value, Box<dyn std::error::Error>> {
        // Create story generation tool call that will invoke real LLM
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "generate_story".into(),
                arguments: Some(json!({
                    "theme": "First Contact with a new alien civilization",
                    "story_type": "SciFi",
                    "duration_minutes": 60,
                    "max_participants": 4,
                    "characters": ["Picard", "Data", "Worf", "Troi"],
                    "safety_level": "Standard",
                    "tenant": "qollective-llm-test",
                    "user_id": "test-user",
                    "request_id": "story-generation-test"
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

        println!("🎨 Sending real LLM story generation request...");

        // Send envelope through qollective infrastructure with longer timeout for LLM processing
        let result = timeout(
            Duration::from_millis(30000), // 30 seconds for real LLM call
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective story generation timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();

                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective story generation failed: {:?}", tool_response.content).into())
                    } else {
                        // Parse and return the story response
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let response_json: Value = serde_json::from_str(&text_content.text)?;
                                Ok(response_json)
                            } else {
                                Err("Unexpected content type from qollective LLM story server".into())
                            }
                        } else {
                            Err("No content in qollective LLM story response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective story envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective story generation failed: {}", e).into())
        }
    }

    /// Test real LLM story enhancement
    pub async fn test_qollective_real_llm_story_enhancement(&self, story_id: Uuid) -> Result<Value, Box<dyn std::error::Error>> {
        // Create story enhancement tool call
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "enhance_story".into(),
                arguments: Some(json!({
                    "story_id": story_id.to_string(),
                    "enhancement_type": "character_development",
                    "focus_areas": ["dialogue", "character_arcs", "emotional_depth"]
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
        meta.tenant = Some("qollective-enhancement-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        println!("🎭 Sending real LLM story enhancement request...");

        let result = timeout(
            Duration::from_millis(25000), // 25 seconds for LLM enhancement
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective story enhancement timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();

                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective story enhancement failed: {:?}", tool_response.content).into())
                    } else {
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let response_json: Value = serde_json::from_str(&text_content.text)?;
                                Ok(response_json)
                            } else {
                                Err("Unexpected content type from qollective enhancement server".into())
                            }
                        } else {
                            Err("No content in qollective enhancement response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective enhancement envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective story enhancement failed: {}", e).into())
        }
    }

    /// Test real LLM story validation
    pub async fn test_qollective_real_llm_story_validation(&self, story_id: Uuid) -> Result<Value, Box<dyn std::error::Error>> {
        // Create story validation tool call
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "validate_story_consistency".into(),
                arguments: Some(json!({
                    "story_id": story_id.to_string()
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
        meta.tenant = Some("qollective-validation-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        println!("🔍 Sending real LLM story validation request...");

        let result = timeout(
            Duration::from_millis(20000), // 20 seconds for LLM validation
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective story validation timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();

                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective story validation failed: {:?}", tool_response.content).into())
                    } else {
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let response_json: Value = serde_json::from_str(&text_content.text)?;
                                Ok(response_json)
                            } else {
                                Err("Unexpected content type from qollective validation server".into())
                            }
                        } else {
                            Err("No content in qollective validation response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective validation envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective story validation failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qollective_websocket_designer_health() {
        println!("🧪 Testing qollective WebSocket MCP server health with real LLM integration");

        let client = QollectiveDesignerTestClient::new().await
            .expect("Failed to create qollective designer test client");

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
        assert_eq!(service, "holodeck-designer");

        println!("🎨 Qollective WebSocket MCP server health check passed!");
    }

    #[tokio::test]
    async fn test_qollective_websocket_designer_service_info() {
        println!("🧪 Testing qollective WebSocket MCP server service info with real LLM provider");

        let client = QollectiveDesignerTestClient::new().await
            .expect("Failed to create qollective designer test client");

        let service_info = client.test_qollective_service_info().await
            .expect("Qollective service info retrieval failed");

        println!("✅ Qollective service info: {}", serde_json::to_string_pretty(&service_info).unwrap());

        // Validate qollective service info structure
        assert!(service_info.is_object());
        let info_obj = service_info.as_object().unwrap();
        assert!(info_obj.contains_key("service"));
        assert!(info_obj.contains_key("version"));
        assert!(info_obj.contains_key("llm_provider"));
        assert!(info_obj.contains_key("story_capabilities"));

        // Verify service name
        let service_name = info_obj.get("service").unwrap().as_str().unwrap();
        assert_eq!(service_name, "holodeck-designer");

        // Verify real LLM provider is configured
        let llm_provider = info_obj.get("llm_provider").unwrap().as_object().unwrap();
        let provider_name = llm_provider.get("provider_name").unwrap().as_str().unwrap();
        assert!(!provider_name.is_empty(), "LLM provider should be configured");
        assert!(llm_provider.contains_key("model_name"));
        assert!(llm_provider.contains_key("provider_type"));

        // Verify story capabilities
        let capabilities = info_obj.get("story_capabilities").unwrap().as_object().unwrap();
        assert!(capabilities.contains_key("narrative_generation"));
        assert!(capabilities.contains_key("creative_enhancement"));
        assert!(capabilities.contains_key("safety_validation"));
        assert!(capabilities.contains_key("character_integration"));
        assert!(capabilities.contains_key("canon_consistency"));

        println!("🤖 Qollective WebSocket MCP server with real LLM integration verified!");
    }

    #[tokio::test]
    async fn test_qollective_real_llm_story_generation_workflow() -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing qollective WebSocket MCP server with REAL LLM story generation workflow");

        let client = QollectiveDesignerTestClient::new().await
            .expect("Failed to create qollective designer test client");

        // Test story generation
        let story_result = client.test_qollective_real_llm_story_generation().await
            .expect("Story generation failed");

        println!("🎨 Story generation result: {}",
            serde_json::to_string_pretty(&story_result).unwrap());

        // Save story generation result to target folder
        client.save_story_result(&story_result, "story_generation")
            .expect("Failed to save story generation result");

        // Verify story generation response structure
        let story_obj = story_result.as_object().unwrap();
        assert!(story_obj.contains_key("story_id"));
        assert!(story_obj.contains_key("story_content"));
        assert!(story_obj.contains_key("metadata"));

        let story_id_str = story_obj.get("story_id").unwrap().as_str().unwrap();
        let story_id = Uuid::parse_str(story_id_str).expect("Invalid story ID");

        let story_content = story_obj.get("story_content").unwrap().as_str().unwrap();
        assert!(!story_content.is_empty(), "Story should have content");

        println!("✅ Real LLM story generation successful! Story ID: {}, Content length: {} chars",
            story_id, story_content.len());

        // Test story enhancement
        let enhancement_result = client.test_qollective_real_llm_story_enhancement(story_id).await
            .expect("Story enhancement failed");

        println!("🎭 Story enhancement result: {}",
            serde_json::to_string_pretty(&enhancement_result).unwrap());

        // Save story enhancement result to target folder
        client.save_story_result(&enhancement_result, "story_enhancement")
            .expect("Failed to save story enhancement result");

        // Verify enhancement response structure
        let enhancement_obj = enhancement_result.as_object().unwrap();
        assert!(enhancement_obj.contains_key("story_id"));
        assert!(enhancement_obj.contains_key("enhancement_summary"));
        assert!(enhancement_obj.contains_key("enhancement_details"));

        println!("✅ Real LLM story enhancement successful!");

        // Test story validation
        let validation_result = client.test_qollective_real_llm_story_validation(story_id).await
            .expect("Story validation failed");

        println!("🔍 Story validation result: {}",
            serde_json::to_string_pretty(&validation_result).unwrap());

        // Save story validation result to target folder
        client.save_story_result(&validation_result, "story_validation")
            .expect("Failed to save story validation result");

        // Verify validation response structure
        let validation_obj = validation_result.as_object().unwrap();
        assert!(validation_obj.contains_key("story_id"));
        assert!(validation_obj.contains_key("is_consistent"));
        assert!(validation_obj.contains_key("consistency_score"));
        assert!(validation_obj.contains_key("validation_areas"));
        assert!(validation_obj.contains_key("recommendations"));
        assert!(validation_obj.contains_key("canon_compliance_rating"));

        let consistency_score = validation_obj.get("consistency_score").unwrap().as_u64().unwrap();
        assert!(consistency_score > 0, "Should have a consistency score");

        let validation_areas = validation_obj.get("validation_areas").unwrap().as_array().unwrap();
        assert_eq!(validation_areas.len(), 6, "Should have 6 validation areas");

        println!("✅ Real LLM story validation successful! Consistency score: {}", consistency_score);

        println!("🚀 Complete qollective LLM story workflow test passed!");
        println!("📊 Check server logs for detailed LLM provider interaction traces");

        Ok(())
    }

    #[tokio::test]
    async fn test_qollective_envelope_logging_demo() {
        println!("🧪 Testing qollective envelope logging for demo purposes");

        let client = QollectiveDesignerTestClient::new().await
            .expect("Failed to create qollective designer test client");

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

        let client = QollectiveDesignerTestClient::new().await
            .expect("Failed to create qollective designer test client");

        // Test with unmapped tool to verify error handling
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "unmapped_designer_tool".into(),
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

        let client = QollectiveDesignerTestClient::new().await
            .expect("Failed to create qollective designer test client");

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

        let client = QollectiveDesignerTestClient::new().await
            .expect("Failed to create qollective designer test client");

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

    #[tokio::test]
    async fn test_qollective_story_types_comprehensive() -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing all story types through qollective LLM integration");

        let client = QollectiveDesignerTestClient::new().await
            .expect("Failed to create qollective designer test client");

        let story_types = vec![
            ("Adventure", "Exploring a dangerous new nebula"),
            ("Mystery", "Solving the disappearance of a research station"),
            ("Drama", "Dealing with moral choices in diplomatic crisis"),
            ("Comedy", "Holodeck malfunction creates amusing situations"),
            ("Historical", "Witnessing the first warp flight"),
            ("SciFi", "Encountering Q and omnipotent beings"),
            ("Fantasy", "Medieval simulation with holographic characters"),
            ("Educational", "Learning about stellar cartography"),
        ];

        println!("🎨 Testing {} different story types...", story_types.len());

        for (story_type, theme) in story_types {
            println!("📖 Testing {} story: {}", story_type, theme);

            let tool_call = CallToolRequest {
                method: rmcp::model::CallToolRequestMethod::default(),
                params: CallToolRequestParam {
                    name: "generate_story".into(),
                    arguments: Some(json!({
                        "theme": theme,
                        "story_type": story_type,
                        "duration_minutes": 30,
                        "max_participants": 2,
                        "characters": ["Picard", "Data"],
                        "safety_level": "Standard",
                        "tenant": "qollective-story-types-test",
                        "user_id": "test-user",
                        "request_id": format!("{}-story-test", story_type.to_lowercase())
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
            meta.tenant = Some("qollective-story-types-test".to_string());

            let envelope = Envelope::new(meta, mcp_data);

            let result = timeout(
                Duration::from_millis(20000), // 20 seconds per story type
                client.send_qollective_envelope(envelope)
            ).await
            .map_err(|_| format!("{} story generation timed out", story_type))?;

            match result {
                Ok(response_envelope) => {
                    let (_, response_data) = response_envelope.extract();

                    if let Some(tool_response) = response_data.tool_response {
                        if tool_response.is_error == Some(true) {
                            panic!("{} story generation failed: {:?}", story_type, tool_response.content);
                        } else {
                            if let Some(content) = tool_response.content.first() {
                                if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                    let response_json: Value = serde_json::from_str(&text_content.text)
                                        .expect(&format!("Failed to parse {} story response", story_type));

                                    // Verify response structure
                                    let story_obj = response_json.as_object().unwrap();
                                    assert!(story_obj.contains_key("story_id"));
                                    assert!(story_obj.contains_key("story_content"));

                                    let story_content = story_obj.get("story_content").unwrap().as_str().unwrap();
                                    assert!(!story_content.is_empty(), "{} story should have content", story_type);

                                    println!("✅ {} story generated successfully - {} chars",
                                        story_type, story_content.len());
                                }
                            }
                        }
                    }
                },
                Err(e) => panic!("{} story generation failed: {}", story_type, e)
            }
        }

        println!("🎉 All story types tested successfully through qollective LLM integration!");
        Ok(())
    }
}
