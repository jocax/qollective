//! Unit tests for StoryGeneratorHandler envelope processing
//!
//! Tests verify:
//! - Correct envelope handling (metadata preservation)
//! - Tool call routing and execution
//! - Error handling and error envelopes
//! - Tenant isolation via metadata
//! - Parameter validation
//!
//! Run with: cargo test --package story-generator --test envelope_handler_tests

use story_generator::envelope_handlers::StoryGeneratorHandler;
use story_generator::config::StoryGeneratorConfig;
use story_generator::llm::StoryLlmClient;
use shared_types::{ContentNode, Content, AgeGroup, GenerationRequest, Language, DAG, Edge};
use qollective::envelope::{Envelope, Meta};
use qollective::types::mcp::McpData;
use qollective::server::EnvelopeHandler;
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam};
use serde_json::json;
use uuid::Uuid;
use shared_types_llm::config::LlmConfig;
use std::collections::HashMap;

/// Helper to create test LLM config
fn test_llm_config() -> LlmConfig {
    let toml = r#"
[llm]
type = "shimmy"
url = "http://localhost:1234/v1"
default_model = "test-model"
use_default_model_fallback = true
max_tokens = 4096
temperature = 0.7
timeout_secs = 60
system_prompt_style = "native"

[llm.models]
en = "test-model-en"
de = "test-model-de"
    "#;
    LlmConfig::from_toml_str(toml).expect("Failed to create test LLM config")
}

/// Helper to create test config
fn test_config() -> StoryGeneratorConfig {
    // Use actual config loading with test defaults
    StoryGeneratorConfig::load().unwrap_or_else(|_| {
        // Fallback to minimal config if load fails
        use story_generator::config::{ServiceConfig, NatsConfig, GenerationConfig};

        StoryGeneratorConfig {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            generation: GenerationConfig::default(),
            llm: test_llm_config(),
        }
    })
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

/// Helper to create test DAG structure
fn test_dag_structure() -> DAG {
    let node = ContentNode {
        id: "node-1".to_string(),
        content: Content {
            node_id: "node-1".to_string(),
            r#type: "story".to_string(),
            text: String::new(),
            choices: vec![],
            next_nodes: vec![],
            convergence_point: false,
            educational_content: None,
        },
        incoming_edges: 0,
        outgoing_edges: 0,
        generation_metadata: None,
    };

    let mut nodes = HashMap::new();
    nodes.insert("node-1".to_string(), node);

    DAG {
        nodes,
        convergence_points: vec![],
        edges: vec![],
        start_node_id: "node-1".to_string(),
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
async fn test_generate_structure_success() {
    // ARRANGE
    let config = test_config();
    let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
    let handler = StoryGeneratorHandler::new(config, llm_client);

    let generation_request = test_generation_request("animals");

    let params = json!({
        "generation_request": generation_request
    });

    let envelope = create_test_envelope("generate_structure", params, Some("tenant-123"));
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
async fn test_validate_paths_success() {
    // ARRANGE
    let config = test_config();
    let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
    let handler = StoryGeneratorHandler::new(config, llm_client);

    let dag_structure = test_dag_structure();

    let params = json!({
        "dag": dag_structure
    });

    let envelope = create_test_envelope("validate_paths", params, Some("tenant-123"));

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
async fn test_generate_structure_invalid_params() {
    // ARRANGE
    let config = test_config();
    let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
    let handler = StoryGeneratorHandler::new(config, llm_client);

    // Missing required field: generation_request
    let params = json!({
        "invalid_field": "test"
    });

    let envelope = create_test_envelope("generate_structure", params, Some("tenant-123"));

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
    let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
    let handler = StoryGeneratorHandler::new(config, llm_client);

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
    let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
    let handler = StoryGeneratorHandler::new(config, llm_client);

    let generation_request = test_generation_request("test");

    let params = json!({
        "generation_request": generation_request
    });

    let envelope = create_test_envelope("generate_structure", params, Some("tenant-xyz"));

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
    let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
    let handler = StoryGeneratorHandler::new(config, llm_client);

    let generation_request = test_generation_request("test");

    let params = json!({
        "generation_request": generation_request
    });

    // Create two envelopes with different tenants
    let envelope1 = create_test_envelope("generate_structure", params.clone(), Some("tenant-A"));
    let envelope2 = create_test_envelope("generate_structure", params.clone(), Some("tenant-B"));

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
    let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
    let handler = StoryGeneratorHandler::new(config, llm_client);

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
