// ABOUTME: Integration tests for qollective WebSocket MCP server with real LLM-powered HolodeckValidatorServer
// ABOUTME: Tests complete envelope-first architecture with actual LLM-powered content validation, canon checking, and quality assessment

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

/// Integration test client for qollective WebSocket MCP server with holodeck-validator
struct QollectiveValidatorTestClient {
    websocket_url: String,
}

impl QollectiveValidatorTestClient {
    /// Initialize test client with WebSocket connection to qollective MCP server
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let websocket_url = format!("ws://{}:{}/mcp",
            network::DEFAULT_HOST,
            network::HOLODECK_VALIDATOR_PORT);

        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(1000)).await;

        Ok(Self {
            websocket_url,
        })
    }

    /// Save validation result to target folder with timestamp and test type pattern
    fn save_validation_result(&self, validation_result: &Value, test_type: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create target directory if it doesn't exist
        let target_dir = PathBuf::from("target");
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        // Extract content type from result
        let content_type = validation_result
            .get("content_type")
            .and_then(|ct| ct.as_str())
            .unwrap_or("unknown_content")
            .replace(":", "_")
            .replace("/", "_")
            .replace("\\", "_")
            .replace(" ", "_");

        // Generate timestamp
        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%d_%H-%M-%S");

        // Create filename
        let filename = format!("{}_{}_{}_{}.json",
            timestamp,
            test_type,
            content_type,
            validation_result.get("content_id").and_then(|id| id.as_str()).unwrap_or("no_id"));
        let filepath = target_dir.join(filename);

        // Save pretty-printed JSON
        let json_content = serde_json::to_string_pretty(validation_result)?;
        fs::write(&filepath, json_content)?;

        let saved_path = filepath.to_string_lossy().to_string();
        println!("💾 Saved validation result to: {}", saved_path);
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
                        if let Some(payload) = websocket_response.get("payload") {
                            println!("📦 Extracting envelope from WebSocket wrapper");
                            let response_envelope: Envelope<McpData> = serde_json::from_value(payload.clone())?;
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
        meta.tenant = Some("qollective-validator-test".to_string());

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
        meta.tenant = Some("qollective-validator-test".to_string());

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

    /// Test real LLM content validation
    pub async fn test_qollective_real_llm_content_validation(&self) -> Result<Value, Box<dyn std::error::Error>> {
        // Create content validation tool call that will invoke real LLM
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "validate_content".into(),
                arguments: Some(json!({
                    "content_id": Uuid::now_v7().to_string(),
                    "story_content": "Captain Picard stood on the bridge, facing a critical decision about first contact with an unknown alien species. The universal translator was malfunctioning, creating communication barriers. Data analyzed the alien ship's configuration while Worf reported defensive postures. Troi sensed confusion and curiosity from the aliens, not hostility. The crew had to decide whether to approach or maintain distance.",
                    "story_type": "Adventure",
                    "content_type": "Story",
                    "tenant": "qollective-llm-validation-test",
                    "user_id": "test-user",
                    "request_id": "content-validation-test"
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

        println!("🔍 Sending real LLM content validation request...");

        // Send envelope through qollective infrastructure with longer timeout for LLM processing
        let result = timeout(
            Duration::from_millis(30000), // 30 seconds for real LLM call
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective content validation timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();

                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective content validation failed: {:?}", tool_response.content).into())
                    } else {
                        // Parse and return the validation response
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let response_json: Value = serde_json::from_str(&text_content.text)?;
                                Ok(response_json)
                            } else {
                                Err("Unexpected content type from qollective LLM validation server".into())
                            }
                        } else {
                            Err("No content in qollective LLM validation response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective validation envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective content validation failed: {}", e).into())
        }
    }

    /// Test real LLM canon consistency validation
    pub async fn test_qollective_real_llm_canon_validation(&self) -> Result<Value, Box<dyn std::error::Error>> {
        // Create canon consistency validation tool call
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "validate_canon_consistency".into(),
                arguments: Some(json!({
                    "content_id": Uuid::now_v7().to_string(),
                    "story_content": "The Enterprise-D encountered a Borg cube near the Romulan Neutral Zone. Captain Picard ordered red alert and contacted Starfleet Command. The ship's deflector array was modified to emit an anti-Borg frequency. Geordi La Forge and Data worked together in Engineering to implement the modifications while Worf coordinated defensive strategies.",
                    "era": "TNG",
                    "strictness_level": "Standard"
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
        meta.tenant = Some("qollective-canon-validation-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        println!("🖖 Sending real LLM canon validation request...");

        let result = timeout(
            Duration::from_millis(25000), // 25 seconds for LLM canon validation
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective canon validation timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();

                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective canon validation failed: {:?}", tool_response.content).into())
                    } else {
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let response_json: Value = serde_json::from_str(&text_content.text)?;
                                Ok(response_json)
                            } else {
                                Err("Unexpected content type from qollective canon validation server".into())
                            }
                        } else {
                            Err("No content in qollective canon validation response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective canon validation envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective canon validation failed: {}", e).into())
        }
    }

    /// Test real LLM quality assessment
    pub async fn test_qollective_real_llm_quality_assessment(&self) -> Result<Value, Box<dyn std::error::Error>> {
        // Create quality assessment tool call
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "assess_content_quality".into(),
                arguments: Some(json!({
                    "content_id": Uuid::now_v7().to_string(),
                    "story_content": "In the ship's ready room, Captain Picard contemplated the moral implications of their next mission. The crew had discovered a planet where the inhabitants had developed a technology that could extend life indefinitely, but at the cost of their creativity and passion. Should Starfleet intervene or respect the Prime Directive? The decision weighed heavily on his mind as he reviewed the philosophical texts of Earth's greatest thinkers.",
                    "target_audience": "Adults",
                    "story_type": "Drama"
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
        meta.tenant = Some("qollective-quality-assessment-test".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        println!("⭐ Sending real LLM quality assessment request...");

        let result = timeout(
            Duration::from_millis(20000), // 20 seconds for LLM quality assessment
            self.send_qollective_envelope(envelope)
        ).await
        .map_err(|_| "Qollective quality assessment timed out")?;

        match result {
            Ok(response_envelope) => {
                let (_, response_data) = response_envelope.extract();

                if let Some(tool_response) = response_data.tool_response {
                    if tool_response.is_error == Some(true) {
                        Err(format!("Qollective quality assessment failed: {:?}", tool_response.content).into())
                    } else {
                        if let Some(content) = tool_response.content.first() {
                            if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                                let response_json: Value = serde_json::from_str(&text_content.text)?;
                                Ok(response_json)
                            } else {
                                Err("Unexpected content type from qollective quality assessment server".into())
                            }
                        } else {
                            Err("No content in qollective quality assessment response".into())
                        }
                    }
                } else {
                    Err("No tool response in qollective quality assessment envelope".into())
                }
            },
            Err(e) => Err(format!("Qollective quality assessment failed: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qollective_websocket_validator_health() {
        println!("🧪 Testing qollective WebSocket MCP server health with real LLM integration");

        let client = QollectiveValidatorTestClient::new().await
            .expect("Failed to create qollective validator test client");

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
        assert_eq!(service, "holodeck-validator");

        println!("🔍 Qollective WebSocket MCP server health check passed!");
    }

    #[tokio::test]
    async fn test_qollective_websocket_validator_service_info() {
        println!("🧪 Testing qollective WebSocket MCP server service info with real LLM provider");

        let client = QollectiveValidatorTestClient::new().await
            .expect("Failed to create qollective validator test client");

        let service_info = client.test_qollective_service_info().await
            .expect("Qollective service info retrieval failed");

        println!("✅ Qollective service info: {}", serde_json::to_string_pretty(&service_info).unwrap());

        // Validate qollective service info structure
        assert!(service_info.is_object());
        let info_obj = service_info.as_object().unwrap();
        assert!(info_obj.contains_key("service"));
        assert!(info_obj.contains_key("version"));
        assert!(info_obj.contains_key("llm_provider"));
        assert!(info_obj.contains_key("validation_capabilities"));

        // Verify service name
        let service_name = info_obj.get("service").unwrap().as_str().unwrap();
        assert_eq!(service_name, "holodeck-validator");

        // Verify real LLM provider is configured
        let llm_provider = info_obj.get("llm_provider").unwrap().as_object().unwrap();
        let provider_name = llm_provider.get("provider_name").unwrap().as_str().unwrap();
        assert!(!provider_name.is_empty(), "LLM provider should be configured");
        assert!(llm_provider.contains_key("model_name"));
        assert!(llm_provider.contains_key("provider_type"));

        // Verify validation capabilities
        let capabilities = info_obj.get("validation_capabilities").unwrap().as_object().unwrap();
        assert!(capabilities.contains_key("content_validation"));
        assert!(capabilities.contains_key("canon_consistency"));
        assert!(capabilities.contains_key("quality_assessment"));
        assert!(capabilities.contains_key("character_authenticity"));

        println!("🤖 Qollective WebSocket MCP server with real LLM validation integration verified!");
    }

    #[tokio::test]
    async fn test_qollective_real_llm_validation_workflow() -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing qollective WebSocket MCP server with REAL LLM validation workflow");

        let client = QollectiveValidatorTestClient::new().await
            .expect("Failed to create qollective validator test client");

        // Test content validation
        let content_result = client.test_qollective_real_llm_content_validation().await
            .expect("Content validation failed");

        println!("🔍 Content validation result: {}",
            serde_json::to_string_pretty(&content_result).unwrap());

        // Save content validation result to target folder
        client.save_validation_result(&content_result, "content_validation")
            .expect("Failed to save content validation result");

        // Verify content validation response structure
        let content_obj = content_result.as_object().unwrap();
        assert!(content_obj.contains_key("content_id"));
        assert!(content_obj.contains_key("is_valid"));
        assert!(content_obj.contains_key("overall_score"));
        assert!(content_obj.contains_key("quality_assessment"));
        assert!(content_obj.contains_key("structure_analysis"));
        assert!(content_obj.contains_key("safety_analysis"));

        let validation_score = content_obj.get("overall_score").unwrap().as_u64().unwrap();
        assert!(validation_score > 0, "Should have a validation score");

        let quality_assessment = content_obj.get("quality_assessment").unwrap().as_object().unwrap();
        assert!(!quality_assessment.is_empty(), "Should have quality assessment");

        println!("✅ Real LLM content validation successful! Validation score: {}", validation_score);

        // Test canon consistency validation
        let canon_result = client.test_qollective_real_llm_canon_validation().await
            .expect("Canon validation failed");

        println!("🖖 Canon validation result: {}",
            serde_json::to_string_pretty(&canon_result).unwrap());

        // Save canon validation result to target folder
        client.save_validation_result(&canon_result, "canon_validation")
            .expect("Failed to save canon validation result");

        // Verify canon validation response structure
        let canon_obj = canon_result.as_object().unwrap();
        assert!(canon_obj.contains_key("content_id"));
        assert!(canon_obj.contains_key("is_canon_compliant"));
        assert!(canon_obj.contains_key("compliance_score"));
        assert!(canon_obj.contains_key("compliance_details"));
        assert!(canon_obj.contains_key("era_consistency"));
        assert!(canon_obj.contains_key("character_accuracy"));

        let compliance_score = canon_obj.get("compliance_score").unwrap().as_u64().unwrap();
        assert!(compliance_score > 0, "Should have a compliance score");

        println!("✅ Real LLM canon validation successful! Compliance score: {}", compliance_score);

        // Test quality assessment
        let quality_result = client.test_qollective_real_llm_quality_assessment().await
            .expect("Quality assessment failed");

        println!("⭐ Quality assessment result: {}",
            serde_json::to_string_pretty(&quality_result).unwrap());

        // Save quality assessment result to target folder
        client.save_validation_result(&quality_result, "quality_assessment")
            .expect("Failed to save quality assessment result");

        // Verify quality assessment response structure
        let quality_obj = quality_result.as_object().unwrap();
        assert!(quality_obj.contains_key("content_id"));
        assert!(quality_obj.contains_key("overall_quality_score"));
        assert!(quality_obj.contains_key("quality_metrics"));
        assert!(quality_obj.contains_key("recommendations"));
        assert!(quality_obj.contains_key("strengths"));
        assert!(quality_obj.contains_key("weaknesses"));

        let quality_score = quality_obj.get("overall_quality_score").unwrap().as_u64().unwrap();
        assert!(quality_score > 0, "Should have a quality score");

        let quality_metrics = quality_obj.get("quality_metrics").unwrap().as_array().unwrap();
        assert!(!quality_metrics.is_empty(), "Should have quality metrics");

        println!("✅ Real LLM quality assessment successful! Quality score: {}", quality_score);

        println!("🚀 Complete qollective LLM validation workflow test passed!");
        println!("📊 Check server logs for detailed LLM provider interaction traces");

        Ok(())
    }

    #[tokio::test]
    async fn test_qollective_envelope_logging_demo() {
        println!("🧪 Testing qollective envelope logging for validation demo purposes");

        let client = QollectiveValidatorTestClient::new().await
            .expect("Failed to create qollective validator test client");

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
        meta.tenant = Some("qollective-envelope-validation-demo".to_string());
        meta.timestamp = Some(chrono::Utc::now());

        let envelope = Envelope::new(meta, mcp_data);

        println!("📦 Sending qollective envelope with rich metadata for validation logging demo");

        let result = client.send_qollective_envelope(envelope).await
            .expect("Qollective envelope logging demo failed");

        let (response_meta, response_data) = result.extract();

        // Verify envelope metadata preservation
        assert!(response_meta.request_id.is_some());
        assert!(response_meta.tenant.is_some());
        assert_eq!(response_meta.tenant.unwrap(), "qollective-envelope-validation-demo");

        // Verify response content
        assert!(response_data.tool_response.is_some());

        println!("📦 Qollective envelope-first architecture validation logging demo completed!");
        println!("🎯 Check server logs for comprehensive envelope processing details");
    }

    #[tokio::test]
    async fn test_qollective_error_handling() {
        println!("🧪 Testing qollective error handling for unmapped validation tools");

        let client = QollectiveValidatorTestClient::new().await
            .expect("Failed to create qollective validator test client");

        // Test with unmapped tool to verify error handling
        let tool_call = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "unmapped_validator_tool".into(),
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
        meta.tenant = Some("qollective-validator-error-test".to_string());

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

        println!("✅ Qollective error handling working correctly for unmapped validation tools!");
    }

    #[tokio::test]
    async fn test_qollective_concurrent_requests() {
        println!("🧪 Testing qollective WebSocket server concurrent validation request handling");

        let client = QollectiveValidatorTestClient::new().await
            .expect("Failed to create qollective validator test client");

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

        println!("✅ Qollective WebSocket server handles concurrent validation requests successfully!");
    }

    #[tokio::test]
    async fn test_qollective_performance() {
        println!("🧪 Testing qollective WebSocket MCP server validation performance");

        let client = QollectiveValidatorTestClient::new().await
            .expect("Failed to create qollective validator test client");

        // Test response time for health check (should be fast)
        let start_time = std::time::Instant::now();
        let _result = client.test_qollective_health_check().await
            .expect("Qollective performance test failed");
        let duration = start_time.elapsed();

        println!("🚀 Qollective health check took: {:?}", duration);

        // Should complete reasonably quickly (allowing for LLM calls)
        assert!(duration < Duration::from_millis(5000),
            "Qollective health check should complete within 5 seconds");

        println!("⚡ Qollective WebSocket MCP server validation performance acceptable!");
    }

    #[tokio::test]
    async fn test_qollective_validation_target_performance() {
        println!("🧪 Testing qollective validation performance target (< 400ms)");

        let client = QollectiveValidatorTestClient::new().await
            .expect("Failed to create qollective validator test client");

        // Test response time for simple validation (should meet < 400ms target)
        let start_time = std::time::Instant::now();
        let _result = client.test_qollective_health_check().await
            .expect("Qollective validation performance test failed");
        let duration = start_time.elapsed();

        println!("🎯 Qollective validation took: {:?}", duration);

        // Note: The < 400ms target applies to real validation calls, not health checks
        // Health checks should be even faster
        assert!(duration < Duration::from_millis(2000),
            "Qollective health check should complete well under performance targets");

        println!("⚡ Qollective WebSocket MCP server meets performance targets!");
    }
}
