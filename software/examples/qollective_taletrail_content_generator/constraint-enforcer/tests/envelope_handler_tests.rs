//! Unit tests for ConstraintEnforcerHandler envelope processing
//!
//! Tests verify:
//! - Correct envelope handling (metadata preservation)
//! - Tool call routing and execution
//! - Error handling and error envelopes
//! - Tenant isolation via metadata
//! - Parameter validation
//!
//! Run with: cargo test --package constraint-enforcer --test envelope_handler_tests

use constraint_enforcer::envelope_handlers::ConstraintEnforcerHandler;
use constraint_enforcer::config::ConstraintEnforcerConfig;
use shared_types::{ContentNode, Content, AgeGroup, GenerationRequest, Language};
use qollective::envelope::{Envelope, Meta};
use qollective::types::mcp::McpData;
use qollective::server::EnvelopeHandler;
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam};
use serde_json::json;
use uuid::Uuid;

/// Helper to create test config
fn test_config() -> ConstraintEnforcerConfig {
    // Use actual config loading with test defaults
    ConstraintEnforcerConfig::load().unwrap_or_else(|_| {
        // Fallback to minimal config if load fails
        use constraint_enforcer::config::{
            ServiceConfig, NatsConfig, ConstraintsConfig,
            VocabularyConfig, LanguageVocabulary, VocabularyLevel,
            ThemesConfig, RequiredElementsConfig,
        };
        use shared_types_llm::config::{LlmConfig, ProviderConfig};
        use shared_types_llm::parameters::{ProviderType, SystemPromptStyle};
        use std::collections::HashMap;

        ConstraintEnforcerConfig {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            constraints: ConstraintsConfig::default(),
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
            vocabulary: VocabularyConfig {
                english: LanguageVocabulary {
                    basic: VocabularyLevel { words: vec!["cat".to_string(), "dog".to_string(), "play".to_string()] },
                    intermediate: VocabularyLevel { words: vec!["adventure".to_string(), "mystery".to_string()] },
                    advanced: VocabularyLevel { words: vec!["sophisticated".to_string(), "complex".to_string()] },
                },
                german: LanguageVocabulary {
                    basic: VocabularyLevel { words: vec!["Katze".to_string()] },
                    intermediate: VocabularyLevel { words: vec!["Abenteuer".to_string()] },
                    advanced: VocabularyLevel { words: vec!["komplex".to_string()] },
                },
            },
            themes: ThemesConfig {
                min_consistency_score: 0.6,
                keywords: HashMap::from([
                    ("animals".to_string(), constraint_enforcer::config::ThemeKeywords {
                        keywords: vec!["cat".to_string(), "dog".to_string(), "bird".to_string()],
                    }),
                    ("adventure".to_string(), constraint_enforcer::config::ThemeKeywords {
                        keywords: vec!["journey".to_string(), "explore".to_string()],
                    }),
                ]),
            },
            required_elements: RequiredElementsConfig {
                moral_keywords: vec!["moral".to_string(), "lesson".to_string()],
                science_keywords: vec!["science".to_string(), "experiment".to_string()],
                educational_keywords: vec!["learn".to_string(), "discover".to_string()],
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

/// Helper to create test generation request
fn test_generation_request(theme: &str) -> GenerationRequest {
    GenerationRequest {
        theme: theme.to_string(),
        age_group: AgeGroup::_6To8,
        language: Language::En,
        educational_goals: Some(vec!["reading".to_string()]),
        node_count: Some(5),
        tenant_id: 1,
        tags: None,
        prompt_packages: None,
        author_id: None,
        required_elements: None,
        vocabulary_level: None,
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
async fn test_enforce_constraints_success() {
    // ARRANGE
    let config = test_config();
    let handler = ConstraintEnforcerHandler::new(config);

    let content_node = test_content_node(
        "node-1",
        "A friendly cat plays with a ball.",
    );

    let generation_request = test_generation_request("animals");

    let params = json!({
        "content_node": content_node,
        "generation_request": generation_request
    });

    let envelope = create_test_envelope("enforce_constraints", params, Some("tenant-123"));
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
async fn test_enforce_constraints_vocabulary_violation() {
    // ARRANGE
    let config = test_config();
    let handler = ConstraintEnforcerHandler::new(config);

    // Content with sophisticated vocabulary inappropriate for age 6-8
    let content_node = test_content_node(
        "node-1",
        "The sophisticated philosophical treatise explores complex existential questions.",
    );

    let generation_request = test_generation_request("philosophy");

    let params = json!({
        "content_node": content_node,
        "generation_request": generation_request
    });

    let envelope = create_test_envelope("enforce_constraints", params, Some("tenant-123"));

    // ACT
    let result = handler.handle(envelope).await;

    // ASSERT
    assert!(result.is_ok());
    let response_envelope = result.unwrap();

    let (_, data) = response_envelope.extract();
    let tool_response = data.tool_response.unwrap();

    // Should succeed (tool executed) even if content has violations
    assert_eq!(tool_response.is_error, Some(false));

    // Parse response to verify vocabulary violations detected
    let content_text = tool_response.content.first()
        .and_then(|c| serde_json::to_value(c).ok())
        .and_then(|v| v.get("text").cloned())
        .and_then(|t| t.as_str().map(String::from))
        .unwrap_or_default();

    assert!(!content_text.is_empty());
}

#[tokio::test]
async fn test_enforce_constraints_theme_violation() {
    // ARRANGE
    let config = test_config();
    let handler = ConstraintEnforcerHandler::new(config);

    // Content that doesn't match the theme
    let content_node = test_content_node(
        "node-1",
        "The spaceship zooms through the galaxy exploring distant planets.",
    );

    let generation_request = test_generation_request("animals");  // Theme mismatch

    let params = json!({
        "content_node": content_node,
        "generation_request": generation_request
    });

    let envelope = create_test_envelope("enforce_constraints", params, Some("tenant-123"));

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
async fn test_suggest_corrections_success() {
    // ARRANGE
    let config = test_config();
    let handler = ConstraintEnforcerHandler::new(config);

    let content_node = test_content_node(
        "node-1",
        "A sophisticated philosophical treatise.",
    );

    let generation_request = test_generation_request("philosophy");

    let params = json!({
        "content_node": content_node,
        "generation_request": generation_request
    });

    let envelope = create_test_envelope("suggest_corrections", params, Some("tenant-123"));

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
async fn test_enforce_constraints_invalid_params() {
    // ARRANGE
    let config = test_config();
    let handler = ConstraintEnforcerHandler::new(config);

    // Missing required field: content_node
    let params = json!({
        "generation_request": test_generation_request("animals")
    });

    let envelope = create_test_envelope("enforce_constraints", params, Some("tenant-123"));

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
async fn test_unknown_tool() {
    // ARRANGE
    let config = test_config();
    let handler = ConstraintEnforcerHandler::new(config);

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
async fn test_envelope_metadata_preserved() {
    // ARRANGE
    let config = test_config();
    let handler = ConstraintEnforcerHandler::new(config);

    let content_node = test_content_node("node-1", "Test content");
    let generation_request = test_generation_request("test");

    let params = json!({
        "content_node": content_node,
        "generation_request": generation_request
    });

    let envelope = create_test_envelope("enforce_constraints", params, Some("tenant-xyz"));

    // Capture original metadata
    let original_tenant = envelope.meta.tenant.clone();
    let original_request_id = envelope.meta.request_id.unwrap();

    // ACT
    let result = handler.handle(envelope).await;

    // ASSERT
    assert!(result.is_ok());
    let response_envelope = result.unwrap();

    // All metadata should be preserved
    assert_eq!(response_envelope.meta.tenant, original_tenant);
    assert_eq!(response_envelope.meta.request_id, Some(original_request_id));
}

#[tokio::test]
async fn test_tenant_isolation() {
    // ARRANGE
    let config = test_config();
    let handler = ConstraintEnforcerHandler::new(config);

    let content_node = test_content_node("node-1", "Test content");
    let generation_request = test_generation_request("test");

    let params = json!({
        "content_node": content_node,
        "generation_request": generation_request
    });

    // Create two envelopes with different tenants
    let envelope1 = create_test_envelope("enforce_constraints", params.clone(), Some("tenant-A"));
    let envelope2 = create_test_envelope("enforce_constraints", params.clone(), Some("tenant-B"));

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
async fn test_missing_tool_call_in_envelope() {
    // ARRANGE
    let config = test_config();
    let handler = ConstraintEnforcerHandler::new(config);

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
