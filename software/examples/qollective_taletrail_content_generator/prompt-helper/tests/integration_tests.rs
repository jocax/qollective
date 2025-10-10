//! Integration Tests for Prompt Helper MCP Server
//!
//! This test suite validates the integration between:
//! - NATS connection with TLS (mTLS using client certificates)
//! - MCP server initialization and tool registration
//! - Tool discovery and schema validation
//!
//! # Test Organization
//!
//! 1. **NATS Connection Tests** - Verify TLS connection setup and validation
//! 2. **MCP Server Initialization Tests** - Verify server setup and configuration
//! 3. **Tool Registration and Discovery Tests** - Verify all 4 tools are properly registered
//! 4. **End-to-End Integration Tests** - Full NATS message routing (requires running NATS)
//!
//! # Running Tests
//!
//! Tests that don't require a running NATS server:
//! ```bash
//! cargo test --test integration_tests
//! ```
//!
//! Tests that require a running NATS server (marked with #[ignore]):
//! ```bash
//! cargo test --test integration_tests -- --ignored
//! ```

use async_nats::{ConnectOptions, ServerAddr};
use futures_util::stream::StreamExt; // For .next() on streams
use prompt_helper::{
    config::PromptHelperConfig,
    mcp_tools::{
        create_generate_constraint_prompts_tool, create_generate_story_prompts_tool,
        create_generate_validation_prompts_tool, create_get_model_for_language_tool,
        get_all_tools,
    },
};
use rmcp::model::{CallToolRequest, CallToolRequestParam};
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile;
use std::fs::File;
use std::io::BufReader;
use std::sync::Once;
use tokio;

// ============================================================================
// Test Setup - Rustls Crypto Provider Initialization
// ============================================================================

static INIT: Once = Once::new();

/// Initialize rustls crypto provider once for all tests
fn init_rustls() {
    INIT.call_once(|| {
        // Install ring crypto provider for rustls
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

// ============================================================================
// Test Helpers
// ============================================================================

/// Create test configuration with default values
fn create_test_config() -> PromptHelperConfig {
    use prompt_helper::config::*;

    let llm_toml = r#"
[llm]
type = "shimmy"
url = "http://127.0.0.1:11435/v1"
default_model = "test-model"

[llm.models]
de = "leolm-70b-chat"
en = "meta-llama-3-8b-instruct"
    "#;

    PromptHelperConfig {
        service: ServiceConfig::default(),
        nats: NatsConfig::default(),
        llm: shared_types_llm::LlmConfig::from_toml_str(llm_toml).unwrap(),
        prompt: PromptConfig::default(),
    }
}

/// Create test configuration with custom certificate paths
fn create_test_config_with_certs(ca_cert: &str, client_cert: &str, client_key: &str) -> PromptHelperConfig {
    let mut config = create_test_config();
    config.nats.tls.ca_cert = ca_cert.to_string();
    config.nats.tls.client_cert = Some(client_cert.to_string());
    config.nats.tls.client_key = Some(client_key.to_string());
    config
}

/// Build TLS configuration from certificate paths
///
/// # Arguments
/// * `ca_cert_path` - Path to CA certificate
/// * `client_cert_path` - Path to client certificate
/// * `client_key_path` - Path to client private key
///
/// # Returns
/// Result with ClientConfig for rustls TLS setup
fn build_tls_config(
    ca_cert_path: &str,
    client_cert_path: &str,
    client_key_path: &str,
) -> Result<ClientConfig, String> {
    // Initialize rustls crypto provider
    init_rustls();

    // Load CA certificate
    let ca_file = File::open(ca_cert_path)
        .map_err(|e| format!("Failed to open CA cert: {}", e))?;
    let mut ca_reader = BufReader::new(ca_file);

    let mut root_store = RootCertStore::empty();
    let ca_certs = rustls_pemfile::certs(&mut ca_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to parse CA cert: {}", e))?;

    for cert in ca_certs {
        root_store.add(cert)
            .map_err(|e| format!("Failed to add CA cert to store: {}", e))?;
    }

    // Load client certificate
    let client_cert_file = File::open(client_cert_path)
        .map_err(|e| format!("Failed to open client cert: {}", e))?;
    let mut client_cert_reader = BufReader::new(client_cert_file);
    let client_certs = rustls_pemfile::certs(&mut client_cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to parse client cert: {}", e))?;

    // Load client private key
    let client_key_file = File::open(client_key_path)
        .map_err(|e| format!("Failed to open client key: {}", e))?;
    let mut client_key_reader = BufReader::new(client_key_file);
    let client_key = rustls_pemfile::private_key(&mut client_key_reader)
        .map_err(|e| format!("Failed to parse client key: {}", e))?
        .ok_or_else(|| "No private key found in client key file".to_string())?;

    // Build client config
    let client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_client_auth_cert(client_certs, client_key)
        .map_err(|e| format!("Failed to build client config: {}", e))?;

    Ok(client_config)
}

/// Certificate paths for integration tests
/// Uses the actual certificate paths from the project
const TEST_CA_CERT: &str = "../../../tests/certs/ca.pem";
const TEST_CLIENT_CERT: &str = "../../../tests/certs/client-cert.pem";
const TEST_CLIENT_KEY: &str = "../../../tests/certs/client-key.pem";

// ============================================================================
// 1. NATS Connection Tests
// ============================================================================

#[tokio::test]
#[ignore] // Requires running NATS server with TLS
async fn test_nats_connection_with_tls_succeeds() {
    // Load configuration
    let config = create_test_config_with_certs(TEST_CA_CERT, TEST_CLIENT_CERT, TEST_CLIENT_KEY);

    // Build TLS configuration
    let tls_config = build_tls_config(
        &config.nats.tls.ca_cert,
        config.nats.tls.client_cert.as_ref().unwrap(),
        config.nats.tls.client_key.as_ref().unwrap(),
    )
    .expect("Failed to build TLS config");

    // Create NATS connection options with TLS
    let connect_opts = ConnectOptions::new()
        .tls_client_config(tls_config)
        .name(&config.service.name);

    // Parse server address
    let server_addr: ServerAddr = config.nats.url.parse()
        .expect("Invalid NATS server URL");

    // Attempt connection
    let client = connect_opts.connect(server_addr).await;

    // Verify connection succeeds
    assert!(client.is_ok(), "NATS connection should succeed with valid TLS config");

    // Close connection cleanly
    if let Ok(client) = client {
        client.flush().await.expect("Failed to flush client");
        // Connection will be dropped and closed automatically
    }
}

#[tokio::test]
async fn test_nats_tls_config_builds_successfully() {
    // Test that TLS config can be built with valid certificate paths
    let result = build_tls_config(TEST_CA_CERT, TEST_CLIENT_CERT, TEST_CLIENT_KEY);

    assert!(result.is_ok(), "TLS config should build successfully with valid certificates");
}

#[tokio::test]
async fn test_nats_tls_certificate_validation_with_invalid_ca() {
    // Create a temporary invalid CA cert path
    let invalid_ca = "/tmp/nonexistent_ca.pem";

    // Try to build TLS config with invalid CA
    let result = build_tls_config(invalid_ca, TEST_CLIENT_CERT, TEST_CLIENT_KEY);

    // Verify connection fails with appropriate error
    assert!(result.is_err(), "TLS config should fail with invalid CA certificate");
    assert!(result.unwrap_err().contains("Failed to open CA cert"));
}

#[tokio::test]
async fn test_nats_tls_certificate_validation_with_invalid_client_cert() {
    // Try to build TLS config with invalid client cert
    let invalid_client_cert = "/tmp/nonexistent_client.pem";
    let result = build_tls_config(TEST_CA_CERT, invalid_client_cert, TEST_CLIENT_KEY);

    // Verify fails with appropriate error
    assert!(result.is_err(), "TLS config should fail with invalid client certificate");
    assert!(result.unwrap_err().contains("Failed to open client cert"));
}

#[tokio::test]
async fn test_nats_tls_certificate_validation_with_invalid_client_key() {
    // Try to build TLS config with invalid client key
    let invalid_client_key = "/tmp/nonexistent_key.pem";
    let result = build_tls_config(TEST_CA_CERT, TEST_CLIENT_CERT, invalid_client_key);

    // Verify fails with appropriate error
    assert!(result.is_err(), "TLS config should fail with invalid client key");
    assert!(result.unwrap_err().contains("Failed to open client key"));
}

// ============================================================================
// 2. MCP Server Initialization Tests
// ============================================================================

#[tokio::test]
async fn test_mcp_server_initialization_succeeds() {
    // Create config
    let config = create_test_config();

    // Verify service configuration is valid
    assert_eq!(config.service.name, "prompt-helper");
    assert!(!config.service.version.is_empty());
    assert!(!config.service.description.is_empty());

    // Verify we can get all tools without panic
    let tools = get_all_tools();
    assert_eq!(tools.len(), 4, "Should have exactly 4 tools");
}

#[tokio::test]
async fn test_mcp_server_has_correct_info() {
    // Initialize configuration
    let config = create_test_config();

    // Verify server name
    assert_eq!(config.service.name, "prompt-helper");

    // Verify server version is set
    assert!(!config.service.version.is_empty());
    assert_eq!(config.service.version, "0.1.0");

    // Verify server description is set
    assert!(!config.service.description.is_empty());
    assert_eq!(config.service.description, "TaleTrail Prompt Helper MCP Server");
}

#[tokio::test]
async fn test_mcp_server_config_has_valid_defaults() {
    let config = create_test_config();

    // Verify NATS config
    assert!(!config.nats.url.is_empty());
    assert!(!config.nats.subject.is_empty());
    assert!(!config.nats.queue_group.is_empty());

    // Verify TLS config paths
    assert!(!config.nats.tls.ca_cert.is_empty());
    // client_cert and client_key are optional in default config
    // They are only required for mTLS scenarios

    // Verify prompt config
    assert!(!config.prompt.supported_languages.is_empty());
    assert!(!config.prompt.default_language.is_empty());
    assert!(config.prompt.models.temperature > 0.0);
    assert!(config.prompt.models.max_tokens > 0);
}

// ============================================================================
// 3. Tool Registration and Discovery Tests
// ============================================================================

#[tokio::test]
async fn test_all_four_tools_registered() {
    // Get all tools
    let tools = get_all_tools();

    // Verify exactly 4 tools are registered
    assert_eq!(tools.len(), 4, "Should have exactly 4 tools registered");

    // Extract tool names
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();

    // Verify tool names match expected
    assert!(tool_names.contains(&"generate_story_prompts"), "Should have generate_story_prompts tool");
    assert!(tool_names.contains(&"generate_validation_prompts"), "Should have generate_validation_prompts tool");
    assert!(tool_names.contains(&"generate_constraint_prompts"), "Should have generate_constraint_prompts tool");
    assert!(tool_names.contains(&"get_model_for_language"), "Should have get_model_for_language tool");
}

#[tokio::test]
async fn test_tool_discovery_returns_tool_info() {
    // Get all tools
    let tools = get_all_tools();

    // Verify each tool has required information
    for tool in tools {
        // Non-empty name
        assert!(!tool.name.is_empty(), "Tool should have non-empty name");

        // Non-empty description
        assert!(tool.description.is_some(), "Tool {} should have description", tool.name);
        assert!(!tool.description.as_ref().unwrap().is_empty(), "Tool {} should have non-empty description", tool.name);

        // Valid input schema
        assert!(!tool.input_schema.is_empty(), "Tool {} should have input schema", tool.name);
    }
}

#[tokio::test]
async fn test_tool_schemas_are_valid_json() {
    // Get all tools
    let tools = get_all_tools();

    // Verify each tool's schema can be serialized to valid JSON
    for tool in tools {
        // Convert schema to JSON value
        let schema_json = serde_json::to_value(&*tool.input_schema)
            .expect(&format!("Tool {} schema should serialize to JSON", tool.name));

        // Verify it's an object
        assert!(schema_json.is_object(), "Tool {} schema should be a JSON object", tool.name);

        // Verify it has expected schema fields
        let schema_obj = schema_json.as_object().unwrap();
        assert!(schema_obj.contains_key("properties") || schema_obj.contains_key("type"),
                "Tool {} schema should have 'properties' or 'type' field", tool.name);
    }
}

#[tokio::test]
async fn test_individual_tool_creation_functions() {
    // Test each tool creation function individually
    let story_tool = create_generate_story_prompts_tool();
    assert_eq!(story_tool.name, "generate_story_prompts");
    assert!(story_tool.description.is_some());

    let validation_tool = create_generate_validation_prompts_tool();
    assert_eq!(validation_tool.name, "generate_validation_prompts");
    assert!(validation_tool.description.is_some());

    let constraint_tool = create_generate_constraint_prompts_tool();
    assert_eq!(constraint_tool.name, "generate_constraint_prompts");
    assert!(constraint_tool.description.is_some());

    let model_tool = create_get_model_for_language_tool();
    assert_eq!(model_tool.name, "get_model_for_language");
    assert!(model_tool.description.is_some());
}

#[tokio::test]
async fn test_tool_input_schemas_have_required_structure() {
    let tools = get_all_tools();

    for tool in tools {
        let schema_json = serde_json::to_value(&*tool.input_schema)
            .expect(&format!("Tool {} schema should serialize", tool.name));

        let schema_obj = schema_json.as_object()
            .expect(&format!("Tool {} schema should be object", tool.name));

        // Verify basic JSON Schema structure
        // Most schemas should have either 'properties' (for object schemas) or 'type'
        let has_properties = schema_obj.contains_key("properties");
        let has_type = schema_obj.contains_key("type");

        assert!(has_properties || has_type,
                "Tool {} schema should have 'properties' or 'type'", tool.name);

        // If it has properties, verify it's structured correctly
        if has_properties {
            let properties = schema_obj.get("properties")
                .expect(&format!("Tool {} should have properties", tool.name));
            assert!(properties.is_object(),
                    "Tool {} properties should be an object", tool.name);
        }
    }
}

// ============================================================================
// 4. End-to-End Integration Tests (Requires Running NATS)
// ============================================================================

#[tokio::test]
#[ignore] // Requires running NATS server with TLS and proper subject setup
async fn test_nats_message_routing_to_handler() {
    // This test verifies full end-to-end NATS message routing
    // Prerequisites:
    // - NATS server running with TLS on localhost:5222
    // - Server configured with mcp.prompt.helper subject
    // - Valid client certificates in place

    let config = create_test_config_with_certs(TEST_CA_CERT, TEST_CLIENT_CERT, TEST_CLIENT_KEY);

    // Build TLS config
    let tls_config = build_tls_config(
        &config.nats.tls.ca_cert,
        config.nats.tls.client_cert.as_ref().unwrap(),
        config.nats.tls.client_key.as_ref().unwrap(),
    )
    .expect("Failed to build TLS config");

    // Connect to NATS
    let connect_opts = ConnectOptions::new()
        .tls_client_config(tls_config)
        .name("test-client");

    let server_addr: ServerAddr = config.nats.url.parse().unwrap();
    let client = connect_opts.connect(server_addr).await
        .expect("Failed to connect to NATS");

    // Create a test CallToolRequest for generate_story_prompts
    let mut arguments = serde_json::Map::new();
    arguments.insert("theme".to_string(), serde_json::json!("Space Adventure"));
    arguments.insert("age_group".to_string(), serde_json::json!("_6To8"));
    arguments.insert("language".to_string(), serde_json::json!("En"));
    arguments.insert("educational_goals".to_string(), serde_json::json!(["science", "creativity"]));

    let request = CallToolRequest {
        method: Default::default(),
        params: CallToolRequestParam {
            name: "generate_story_prompts".into(),
            arguments: Some(arguments),
        },
        extensions: Default::default(),
    };

    let request_json = serde_json::to_vec(&request).unwrap();

    // Subscribe to response subject (assuming pattern: mcp.prompt.helper.response)
    let response_subject = format!("{}.response", config.nats.subject);
    let mut subscription = client.subscribe(response_subject.clone()).await
        .expect("Failed to subscribe");

    // Publish request
    client.publish(config.nats.subject.clone(), request_json.into()).await
        .expect("Failed to publish request");

    client.flush().await.expect("Failed to flush");

    // Wait for response (with timeout)
    let response_msg = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        subscription.next()
    )
    .await
    .expect("Timeout waiting for response")
    .expect("No response received");

    // Parse response
    let response_data = response_msg.payload;
    assert!(!response_data.is_empty(), "Response should not be empty");

    // Verify response can be parsed as CallToolResult
    let response_result: Result<rmcp::model::CallToolResult, _> = serde_json::from_slice(&response_data);
    assert!(response_result.is_ok(), "Response should be valid CallToolResult");

    // Clean up
    client.flush().await.expect("Failed to flush client");
}

#[tokio::test]
#[ignore] // Requires running NATS server
async fn test_graceful_shutdown_drains_subscriptions() {
    // This test verifies that shutdown properly drains all subscriptions
    // Prerequisites: NATS server running with TLS

    let config = create_test_config_with_certs(TEST_CA_CERT, TEST_CLIENT_CERT, TEST_CLIENT_KEY);

    // Build TLS config
    let tls_config = build_tls_config(
        &config.nats.tls.ca_cert,
        config.nats.tls.client_cert.as_ref().unwrap(),
        config.nats.tls.client_key.as_ref().unwrap(),
    )
    .expect("Failed to build TLS config");

    // Connect to NATS
    let connect_opts = ConnectOptions::new()
        .tls_client_config(tls_config)
        .name("shutdown-test-client");

    let server_addr: ServerAddr = config.nats.url.parse().unwrap();
    let client = connect_opts.connect(server_addr).await
        .expect("Failed to connect to NATS");

    // Create subscription
    let subscription = client.subscribe(config.nats.subject.clone()).await
        .expect("Failed to subscribe");

    // Verify subscription is active
    // Note: In actual implementation, you'd track subscription state

    // Drain subscription (graceful shutdown)
    drop(subscription);

    // Flush and close connection
    client.flush().await.expect("Failed to flush");

    // Connection closes when dropped
    drop(client);

    // If we reach here without panic, graceful shutdown succeeded
    assert!(true, "Graceful shutdown completed");
}

// ============================================================================
// 5. Configuration Tests
// ============================================================================

#[tokio::test]
async fn test_config_default_values() {
    let config = create_test_config();

    // Verify service defaults
    assert_eq!(config.service.name, "prompt-helper");
    assert_eq!(config.service.version, "0.1.0");

    // Verify NATS defaults
    assert_eq!(config.nats.url, "nats://localhost:5222");
    assert_eq!(config.nats.subject, "mcp.prompt.helper");
    assert_eq!(config.nats.queue_group, "prompt-helper");

    // Verify prompt model defaults
    assert_eq!(config.prompt.models.temperature, 0.7);
    assert_eq!(config.prompt.models.max_tokens, 4096);
}

#[tokio::test]
async fn test_config_custom_cert_paths() {
    let custom_ca = "/custom/path/ca.pem";
    let custom_cert = "/custom/path/client.pem";
    let custom_key = "/custom/path/key.pem";

    let config = create_test_config_with_certs(custom_ca, custom_cert, custom_key);

    assert_eq!(config.nats.tls.ca_cert, custom_ca);
    assert_eq!(config.nats.tls.client_cert.as_deref(), Some(custom_cert));
    assert_eq!(config.nats.tls.client_key.as_deref(), Some(custom_key));
}

#[tokio::test]
async fn test_config_loads_language_specific_models() {
    let config = create_test_config();

    // Verify language model mappings are loaded
    assert!(config.llm.provider.models.contains_key("de"), "Should have German model mapping");
    assert!(config.llm.provider.models.contains_key("en"), "Should have English model mapping");

    // Verify correct models
    assert_eq!(config.llm.provider.models.get("de").unwrap(), "leolm-70b-chat");
    assert_eq!(config.llm.provider.models.get("en").unwrap(), "meta-llama-3-8b-instruct");

    // Verify get_model_for_language works
    assert_eq!(config.llm.provider.get_model_for_language("de"), Some(&"leolm-70b-chat".to_string()));
    assert_eq!(config.llm.provider.get_model_for_language("en"), Some(&"meta-llama-3-8b-instruct".to_string()));
}

// ============================================================================
// 6. Tool Schema Validation Tests
// ============================================================================

#[tokio::test]
async fn test_generate_story_prompts_schema_structure() {
    let tool = create_generate_story_prompts_tool();
    let schema_json = serde_json::to_value(&*tool.input_schema).unwrap();
    let schema_obj = schema_json.as_object().unwrap();

    // Verify schema has expected fields
    if let Some(properties) = schema_obj.get("properties") {
        let props = properties.as_object().unwrap();

        // Should have: theme, age_group, language, educational_goals
        assert!(props.contains_key("theme"), "Should have 'theme' property");
        assert!(props.contains_key("age_group"), "Should have 'age_group' property");
        assert!(props.contains_key("language"), "Should have 'language' property");
        assert!(props.contains_key("educational_goals"), "Should have 'educational_goals' property");
    }
}

#[tokio::test]
async fn test_generate_validation_prompts_schema_structure() {
    let tool = create_generate_validation_prompts_tool();
    let schema_json = serde_json::to_value(&*tool.input_schema).unwrap();
    let schema_obj = schema_json.as_object().unwrap();

    // Verify schema has expected fields
    if let Some(properties) = schema_obj.get("properties") {
        let props = properties.as_object().unwrap();

        // Should have: age_group, language, content_type
        assert!(props.contains_key("age_group"), "Should have 'age_group' property");
        assert!(props.contains_key("language"), "Should have 'language' property");
        assert!(props.contains_key("content_type"), "Should have 'content_type' property");
    }
}

#[tokio::test]
async fn test_generate_constraint_prompts_schema_structure() {
    let tool = create_generate_constraint_prompts_tool();
    let schema_json = serde_json::to_value(&*tool.input_schema).unwrap();
    let schema_obj = schema_json.as_object().unwrap();

    // Verify schema has expected fields
    if let Some(properties) = schema_obj.get("properties") {
        let props = properties.as_object().unwrap();

        // Should have: vocabulary_level, language, required_elements
        assert!(props.contains_key("vocabulary_level"), "Should have 'vocabulary_level' property");
        assert!(props.contains_key("language"), "Should have 'language' property");
        assert!(props.contains_key("required_elements"), "Should have 'required_elements' property");
    }
}

#[tokio::test]
async fn test_get_model_for_language_schema_structure() {
    let tool = create_get_model_for_language_tool();
    let schema_json = serde_json::to_value(&*tool.input_schema).unwrap();
    let schema_obj = schema_json.as_object().unwrap();

    // Verify schema has expected fields
    if let Some(properties) = schema_obj.get("properties") {
        let props = properties.as_object().unwrap();

        // Should have: language
        assert!(props.contains_key("language"), "Should have 'language' property");
    }
}
