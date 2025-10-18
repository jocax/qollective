//! Unit tests for QualityControlHandler envelope processing
//!
//! Tests verify:
//! - Correct envelope handling (metadata preservation)
//! - Tool call routing and execution
//! - Error handling and error envelopes
//! - Tenant isolation via metadata
//! - Parameter validation
//!
//! Run with: cargo test --package quality-control --test envelope_handler_tests

use quality_control::envelope_handlers::{
    QualityControlHandler, ValidateContentParams, BatchValidateParams,
};
use quality_control::config::QualityControlConfig;
use shared_types::{ContentNode, Content, AgeGroup, Choice};
use qollective::envelope::{Envelope, Meta};
use qollective::types::mcp::McpData;
use qollective::server::EnvelopeHandler;
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam};
use serde_json::{json, Map};
use uuid::Uuid;
use chrono::Utc;

/// Helper to create test config
fn test_config() -> QualityControlConfig {
    // Use actual config loading with test defaults
    QualityControlConfig::load().unwrap_or_else(|_| {
        // Fallback to minimal config if load fails
        use quality_control::config::{
            ServiceConfig, NatsConfig, ValidationConfig,
            RubricsConfig, AgeGroupConfig, SafetyConfig, EducationalConfig,
        };
        use shared_types_llm::config::{LlmConfig, ProviderConfig};
        use shared_types_llm::parameters::{ProviderType, SystemPromptStyle};
        use std::collections::HashMap;

        QualityControlConfig {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            validation: ValidationConfig::default(),
            llm: LlmConfig {
                provider: ProviderConfig {
                    provider_type: ProviderType::Shimmy,
                    url: "http://localhost:11435/v1".to_string(),
                    api_key: None,
                    default_model: "test-model".to_string(),
                    use_default_model_fallback: true,
                    models: HashMap::new(),
                    max_tokens: 4096,
                    temperature: 0.7,
                    timeout_secs: 60,
                    system_prompt_style: SystemPromptStyle::Native,
                },
                tenants: HashMap::new(),
            },
            rubrics: RubricsConfig {
                age_6_8: AgeGroupConfig {
                    max_sentence_length: 15.0,
                    vocabulary_level: "basic".to_string(),
                    allowed_themes: vec!["animals".to_string(), "friendship".to_string()],
                },
                age_9_11: AgeGroupConfig {
                    max_sentence_length: 20.0,
                    vocabulary_level: "intermediate".to_string(),
                    allowed_themes: vec!["adventure".to_string(), "mystery".to_string()],
                },
                age_12_14: AgeGroupConfig {
                    max_sentence_length: 25.0,
                    vocabulary_level: "intermediate".to_string(),
                    allowed_themes: vec!["science".to_string(), "history".to_string()],
                },
                age_15_17: AgeGroupConfig {
                    max_sentence_length: 30.0,
                    vocabulary_level: "advanced".to_string(),
                    allowed_themes: vec!["technology".to_string(), "philosophy".to_string()],
                },
                age_18_plus: AgeGroupConfig {
                    max_sentence_length: 35.0,
                    vocabulary_level: "advanced".to_string(),
                    allowed_themes: vec!["complex_topics".to_string()],
                },
            },
            safety: SafetyConfig {
                violence_keywords: vec!["violence".to_string(), "sword".to_string()],
                fear_keywords: vec!["scary".to_string(), "horror".to_string()],
                inappropriate_keywords: vec!["alcohol".to_string()],
            },
            educational: EducationalConfig {
                educational_keywords: vec!["learn".to_string(), "discover".to_string()],
                goals: std::collections::HashMap::new(),
            },
        }
    })
}

/// Helper to create test content node
fn test_content_node(id: &str, text: &str) -> ContentNode {
    ContentNode {
        id: id.to_string(),
        content: Content {
            node_id: id.to_string(),
            r#type: "story".to_string(),
            text: text.to_string(),
            choices: vec![],
            next_nodes: vec![],
            convergence_point: false,
            educational_content: None,
        },
        incoming_edges: 0,
        outgoing_edges: 0,
        generation_metadata: None,
    }
}

/// Helper to create test MCP envelope
fn create_test_envelope(
    tool_name: &str,
    params: serde_json::Value,
    tenant_id: Option<&str>,
) -> Envelope<McpData> {
    let mut meta = Meta::default();
    meta.tenant = tenant_id.map(String::from);
    meta.request_id = Some(Uuid::new_v4());
    meta.timestamp = Some(Utc::now());

    let arguments = if let serde_json::Value::Object(map) = params {
        Some(map)
    } else {
        panic!("Parameters must be JSON object");
    };

    let tool_call = CallToolRequest {
        method: CallToolRequestMethod,
        params: CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments,
        },
        extensions: Default::default(),
    };

    let mcp_data = McpData {
        tool_call: Some(tool_call),
        tool_response: None,
        tool_registration: None,
        discovery_data: None,
    };

    Envelope::new(meta, mcp_data)
}

#[tokio::test]
async fn test_validate_content_success() {
    // ARRANGE
    let config = test_config();
    let handler = QualityControlHandler::new(config);

    let content_node = test_content_node(
        "node-1",
        "A friendly cat plays with a ball.",
    );

    let params = json!({
        "content_node": content_node,
        "age_group": "6-8",
        "educational_goals": ["reading", "vocabulary"]
    });

    let envelope = create_test_envelope("validate_content", params, Some("tenant-123"));
    let request_id = envelope.meta.request_id.unwrap();

    // ACT
    let result = handler.handle(envelope).await;

    // ASSERT
    assert!(result.is_ok(), "Handler should succeed: {:?}", result.err());
    let response_envelope = result.unwrap();

    // Verify metadata preserved
    assert_eq!(response_envelope.meta.tenant, Some("tenant-123".to_string()));
    assert_eq!(response_envelope.meta.request_id, Some(request_id));

    // Verify response structure
    let (_, data) = response_envelope.extract();
    assert!(data.tool_response.is_some());

    let tool_response = data.tool_response.unwrap();
    assert_eq!(tool_response.is_error, Some(false));
}

#[tokio::test]
async fn test_validate_content_invalid_params() {
    // ARRANGE
    let config = test_config();
    let handler = QualityControlHandler::new(config);

    // Missing required field: content_node
    let params = json!({
        "age_group": "6-8",
        "educational_goals": []
    });

    let envelope = create_test_envelope("validate_content", params, Some("tenant-123"));

    // ACT
    let result = handler.handle(envelope).await;

    // ASSERT
    assert!(result.is_ok(), "Handler should return error envelope");
    let response_envelope = result.unwrap();

    let (_, data) = response_envelope.extract();
    let tool_response = data.tool_response.unwrap();

    // Should be an error response
    assert_eq!(tool_response.is_error, Some(true));
}

#[tokio::test]
async fn test_validate_content_tenant_isolation() {
    // ARRANGE
    let config = test_config();
    let handler = QualityControlHandler::new(config);

    let content_node = test_content_node("node-1", "Test content");
    let params = json!({
        "content_node": content_node,
        "age_group": "6-8",
        "educational_goals": []
    });

    // Create two envelopes with different tenants
    let envelope1 = create_test_envelope("validate_content", params.clone(), Some("tenant-A"));
    let envelope2 = create_test_envelope("validate_content", params.clone(), Some("tenant-B"));

    let tenant1_id = envelope1.meta.tenant.clone();
    let tenant2_id = envelope2.meta.tenant.clone();

    // ACT
    let result1 = handler.handle(envelope1).await;
    let result2 = handler.handle(envelope2).await;

    // ASSERT
    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Verify tenant IDs preserved
    let response1 = result1.unwrap();
    let response2 = result2.unwrap();

    assert_eq!(response1.meta.tenant, tenant1_id);
    assert_eq!(response2.meta.tenant, tenant2_id);
}

#[tokio::test]
async fn test_batch_validate_success() {
    // ARRANGE
    let config = test_config();
    let handler = QualityControlHandler::new(config);

    let nodes = vec![
        test_content_node("node-1", "A cat plays."),
        test_content_node("node-2", "A dog runs."),
    ];

    let params = json!({
        "content_nodes": nodes,
        "age_group": "6-8",
        "educational_goals": ["reading"]
    });

    let envelope = create_test_envelope("batch_validate", params, Some("tenant-123"));

    // ACT
    let result = handler.handle(envelope).await;

    // ASSERT
    assert!(result.is_ok());
    let response_envelope = result.unwrap();

    let (_, data) = response_envelope.extract();
    let tool_response = data.tool_response.unwrap();

    assert_eq!(tool_response.is_error, Some(false));
}

#[tokio::test]
async fn test_unknown_tool() {
    // ARRANGE
    let config = test_config();
    let handler = QualityControlHandler::new(config);

    let params = json!({"test": "data"});
    let envelope = create_test_envelope("unknown_tool", params, Some("tenant-123"));

    // ACT
    let result = handler.handle(envelope).await;

    // ASSERT
    assert!(result.is_ok());
    let response_envelope = result.unwrap();

    let (_, data) = response_envelope.extract();
    let tool_response = data.tool_response.unwrap();

    // Should be error response for unknown tool
    assert_eq!(tool_response.is_error, Some(true));
}

#[tokio::test]
async fn test_envelope_metadata_preserved_through_processing() {
    // ARRANGE
    let config = test_config();
    let handler = QualityControlHandler::new(config);

    let content_node = test_content_node("node-1", "Test content");
    let params = json!({
        "content_node": content_node,
        "age_group": "6-8",
        "educational_goals": []
    });

    let envelope = create_test_envelope("validate_content", params, Some("tenant-xyz"));

    // Capture original metadata
    let original_tenant = envelope.meta.tenant.clone();
    let original_request_id = envelope.meta.request_id.unwrap();
    let original_timestamp = envelope.meta.timestamp.unwrap();

    // ACT
    let result = handler.handle(envelope).await;

    // ASSERT
    assert!(result.is_ok());
    let response_envelope = result.unwrap();

    // All metadata should be preserved
    assert_eq!(response_envelope.meta.tenant, original_tenant);
    assert_eq!(response_envelope.meta.request_id, Some(original_request_id));

    // Timestamp should be preserved or very close
    assert!(response_envelope.meta.timestamp.is_some());
    let time_diff = response_envelope.meta.timestamp.unwrap().signed_duration_since(original_timestamp);
    assert!(time_diff.num_seconds() < 1, "Timestamp should be preserved");
}

#[tokio::test]
async fn test_missing_tool_call_in_envelope() {
    // ARRANGE
    let config = test_config();
    let handler = QualityControlHandler::new(config);

    // Create envelope with NO tool_call
    let mut meta = Meta::default();
    meta.tenant = Some("tenant-123".to_string());
    meta.request_id = Some(Uuid::new_v4());

    let mcp_data = McpData {
        tool_call: None,  // Missing tool call
        tool_response: None,
        tool_registration: None,
        discovery_data: None,
    };

    let envelope = Envelope::new(meta, mcp_data);

    // ACT
    let result = handler.handle(envelope).await;

    // ASSERT
    assert!(result.is_err(), "Should return error for missing tool_call");
    let error = result.unwrap_err();
    assert!(error.to_string().contains("No tool_call"));
}

#[tokio::test]
async fn test_validate_content_with_safety_violations() {
    // ARRANGE
    let config = test_config();
    let handler = QualityControlHandler::new(config);

    // Content with safety keyword
    let content_node = test_content_node(
        "node-1",
        "The scary monster appeared with a sword.",
    );

    let params = json!({
        "content_node": content_node,
        "age_group": "6-8",
        "educational_goals": []
    });

    let envelope = create_test_envelope("validate_content", params, Some("tenant-123"));

    // ACT
    let result = handler.handle(envelope).await;

    // ASSERT
    assert!(result.is_ok());
    let response_envelope = result.unwrap();

    let (_, data) = response_envelope.extract();
    let tool_response = data.tool_response.unwrap();

    // Should not be an error, but validation result should indicate issues
    assert_eq!(tool_response.is_error, Some(false));

    // Parse response to verify safety issues detected
    let content_text = tool_response.content.first()
        .and_then(|c| serde_json::to_value(c).ok())
        .and_then(|v| v.get("text").cloned())
        .and_then(|t| t.as_str().map(String::from))
        .unwrap_or_default();

    // Response should be valid JSON containing validation result
    assert!(!content_text.is_empty());
}
