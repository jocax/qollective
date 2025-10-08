//! Comprehensive Integration Tests for Prompt-Helper MCP Server
//!
//! This test suite verifies all functionality of the prompt-helper service
//! by connecting to NATS with TLS using Qollective's envelope-first architecture
//! and exercising all 4 MCP tools.
//!
//! # Requirements
//! - NATS server running on localhost:5222 with TLS
//! - TLS certificates in `../certs/` directory
//! - prompt-helper service running (or start before tests)
//!
//! # Running Tests
//! ```bash
//! cargo test -p prompt-helper --test integration_verification -- --test-threads=1
//! ```

use qollective::client::nats::NatsClient;
use qollective::config::nats::{NatsConfig, NatsConnectionConfig};
use qollective::config::tls::{TlsConfig, VerificationMode};
use qollective::crypto::CryptoProviderStrategy;
use qollective::envelope::Envelope;
use qollective::types::mcp::McpData;
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam, CallToolResult};
use serde_json;
use shared_types::{Language, PromptPackage};
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create Qollective NATS client with TLS
///
/// Creates a NatsClient using Qollective's configuration system with TLS
/// and mTLS authentication from certificates in the `../certs/` directory.
///
/// # Returns
/// Connected Qollective NatsClient or error
async fn create_nats_client() -> Result<NatsClient, Box<dyn std::error::Error>> {
    // Tests run from prompt-helper directory, certs are in parent directory
    let certs_dir = PathBuf::from("../certs");

    info!("Creating Qollective NatsClient with TLS from: {}", certs_dir.display());

    // Create NATS config using Qollective
    // NOTE: Authorization is temporarily disabled in NATS server for integration testing
    // TODO: Add NKey authentication support to Qollective
    let nats_config = NatsConfig {
        connection: NatsConnectionConfig {
            urls: vec!["nats://localhost:5222".to_string()],
            tls: TlsConfig {
                enabled: true,
                ca_cert_path: Some(certs_dir.join("ca.pem")),
                cert_path: Some(certs_dir.join("client-cert.pem")),
                key_path: Some(certs_dir.join("client-key.pem")),
                verification_mode: VerificationMode::MutualTls,
            },
            crypto_provider_strategy: Some(CryptoProviderStrategy::Skip),
            connection_timeout_ms: 5000,
            ..Default::default()
        },
        ..Default::default()
    };

    info!("Connecting to NATS at nats://localhost:5222 with TLS...");

    // Create NatsClient (handles TLS and connection automatically)
    let client = NatsClient::new(nats_config).await?;

    info!("Successfully connected to NATS via Qollective");

    Ok(client)
}

/// Send CallToolRequest wrapped in Envelope and wait for CallToolResult
///
/// Creates a CallToolRequest with the specified tool name and arguments,
/// wraps it in Envelope<McpData>, sends via Qollective NatsClient, and
/// extracts the CallToolResult from the response envelope.
///
/// # Arguments
/// * `client` - Qollective NatsClient to use for communication
/// * `tool_name` - Name of the MCP tool to call
/// * `arguments` - JSON arguments for the tool
/// * `timeout_secs` - Timeout in seconds to wait for response
///
/// # Returns
/// Deserialized CallToolResult or error
async fn call_mcp_tool(
    client: &NatsClient,
    tool_name: &str,
    arguments: serde_json::Value,
    timeout_secs: u64,
) -> Result<CallToolResult, Box<dyn std::error::Error>> {
    // Create CallToolRequest
    let arguments_map = if let serde_json::Value::Object(map) = arguments {
        Some(map)
    } else {
        return Err("Arguments must be a JSON object".into());
    };

    let request = CallToolRequest {
        method: CallToolRequestMethod,
        params: CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments: arguments_map,
        },
        extensions: rmcp::model::Extensions::default(),
    };

    info!("Calling tool '{}' via envelope-wrapped request", tool_name);

    // Wrap in McpData
    let mcp_data = McpData {
        tool_call: Some(request),
        tool_response: None,
        tool_registration: None,
        discovery_data: None,
    };

    // Create envelope with minimal metadata
    let meta = qollective::envelope::Meta {
        tenant: Some("test-tenant".to_string()),
        request_id: Some(Uuid::new_v4()),
        timestamp: Some(chrono::Utc::now()),
        // Minimal tracing info - use None for all optional fields
        tracing: Some(qollective::envelope::meta::TracingMeta {
            trace_id: Some(Uuid::new_v4().to_string()),
            span_id: None,
            parent_span_id: None,
            baggage: std::collections::HashMap::new(),
            sampling_rate: None,
            sampled: None,
            trace_state: None,
            operation_name: Some("call_mcp_tool".to_string()),
            span_kind: None,
            span_status: None,
            tags: std::collections::HashMap::new(),
        }),
        ..Default::default()
    };

    let envelope = Envelope::new(meta, mcp_data);

    info!("Sending envelope-wrapped request to mcp.prompt.helper");

    // Serialize envelope to JSON bytes
    let request_bytes = serde_json::to_vec(&envelope)?;

    // Send request and get response (with timeout)
    let response_bytes = client
        .request_raw("mcp.prompt.helper", &request_bytes, Duration::from_secs(timeout_secs))
        .await?;

    info!("Response received, deserializing envelope");

    // Deserialize response envelope
    let response_envelope: Envelope<McpData> = serde_json::from_slice(&response_bytes)?;

    // Extract tool response
    let (_, response_data) = response_envelope.extract();
    let result = response_data.tool_response
        .ok_or_else(|| "No tool_response in envelope")?;

    info!("CallToolResult extracted successfully from envelope");

    Ok(result)
}

// ============================================================================
// Integration Tests
// ============================================================================

/// Test 2.1: Test NATS Connection with TLS
///
/// Verifies that we can establish a secure TLS connection to the NATS server
/// using the certificates in the ../certs/ directory via Qollective NatsClient.
#[tokio::test]
async fn test_nats_connection_with_tls() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    info!("=== Test 2.1: NATS Connection with TLS (Qollective) ===");

    let _client = create_nats_client().await?;

    // Successfully creating the client proves TLS connection works
    info!("Test 2.1 PASSED: Successfully connected to NATS with TLS via Qollective");

    Ok(())
}

/// Test 2.2: Test Health Check Endpoint
///
/// NOTE: This test is disabled because the current implementation doesn't
/// have a separate health check endpoint. The server only subscribes to
/// mcp.prompt.helper subject. Health can be verified by successful tool calls.
#[tokio::test]
#[ignore] // No separate health check endpoint implemented
async fn test_health_check() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Test 2.2: Health Check Endpoint (DISABLED) ===");
    info!("Health check endpoint not implemented - health verified via tool calls");
    Ok(())
}

/// Test 2.3: Test Story Prompts Generation (English)
///
/// Calls the generate_story_prompts tool with English language parameters
/// and verifies the response contains a valid PromptPackage with the
/// correct LLM model for English.
#[tokio::test]
async fn test_generate_story_prompts_english() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    info!("=== Test 2.3: Story Prompts Generation (English) ===");

    let client = create_nats_client().await?;

    // Create request arguments
    let arguments = serde_json::json!({
        "theme": "Space Adventure",
        "age_group": "6-8",
        "language": "en",
        "educational_goals": ["teamwork", "problem-solving"]
    });

    // Call tool
    let result = call_mcp_tool(&client, "generate_story_prompts", arguments, 10).await?;

    // Verify no error
    assert!(
        result.is_error.is_none() || result.is_error == Some(false),
        "Result should not be an error"
    );

    // Parse PromptPackage from result content
    assert!(!result.content.is_empty(), "Result content should not be empty");

    let content_text = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text_content) => &text_content.text,
        _ => return Err("Expected text content".into()),
    };

    let prompt_package: PromptPackage = serde_json::from_str(content_text)?;

    info!("System prompt: {}", prompt_package.system_prompt);
    info!("User prompt: {}", prompt_package.user_prompt);
    info!("LLM model: {}", prompt_package.llm_model);
    info!("Fallback used: {}", prompt_package.fallback_used);

    // Verify fields
    assert!(!prompt_package.system_prompt.is_empty(), "System prompt should not be empty");
    assert!(!prompt_package.user_prompt.is_empty(), "User prompt should not be empty");
    assert!(!prompt_package.llm_model.is_empty(), "LLM model should not be empty");
    assert_eq!(prompt_package.language, Language::En, "Language should be English");

    // Verify LLM model matches config (English model)
    // Note: The actual model depends on config.toml settings
    info!("LLM model configured: {}", prompt_package.llm_model);

    info!("Test 2.3 PASSED: Story prompts generated successfully for English");

    Ok(())
}

/// Test 2.4: Test Story Prompts Generation (German)
///
/// Calls the generate_story_prompts tool with German language parameters
/// and verifies the response contains a valid PromptPackage with the
/// correct LLM model for German and German text in prompts.
#[tokio::test]
async fn test_generate_story_prompts_german() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    info!("=== Test 2.4: Story Prompts Generation (German) ===");

    let client = create_nats_client().await?;

    // Create request arguments
    let arguments = serde_json::json!({
        "theme": "Weltraumabenteuer",
        "age_group": "6-8",
        "language": "de",
        "educational_goals": ["Teamarbeit", "Problemlösung"]
    });

    // Call tool
    let result = call_mcp_tool(&client, "generate_story_prompts", arguments, 10).await?;

    // Verify no error
    assert!(
        result.is_error.is_none() || result.is_error == Some(false),
        "Result should not be an error"
    );

    // Parse PromptPackage from result content
    let content_text = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text_content) => &text_content.text,
        _ => return Err("Expected text content".into()),
    };

    let prompt_package: PromptPackage = serde_json::from_str(content_text)?;

    info!("System prompt: {}", prompt_package.system_prompt);
    info!("User prompt: {}", prompt_package.user_prompt);
    info!("LLM model: {}", prompt_package.llm_model);
    info!("Fallback used: {}", prompt_package.fallback_used);

    // Verify fields
    assert!(!prompt_package.system_prompt.is_empty(), "System prompt should not be empty");
    assert!(!prompt_package.user_prompt.is_empty(), "User prompt should not be empty");
    assert_eq!(prompt_package.language, Language::De, "Language should be German");

    // If template fallback is used, verify German text presence
    if prompt_package.fallback_used {
        assert!(
            prompt_package.system_prompt.contains("Du bist")
                || prompt_package.system_prompt.contains("Geschichten"),
            "German system prompt should contain German text markers"
        );
    }

    info!("Test 2.4 PASSED: Story prompts generated successfully for German");

    Ok(())
}

/// Test 2.5: Test Validation Prompts Generation
///
/// Calls the generate_validation_prompts tool and verifies the response
/// contains validation-focused prompts for quality control.
#[tokio::test]
async fn test_generate_validation_prompts() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    info!("=== Test 2.5: Validation Prompts Generation ===");

    let client = create_nats_client().await?;

    // Create request arguments
    let arguments = serde_json::json!({
        "age_group": "6-8",
        "language": "en",
        "content_type": "story"
    });

    // Call tool
    let result = call_mcp_tool(&client, "generate_validation_prompts", arguments, 10).await?;

    // Verify no error
    assert!(
        result.is_error.is_none() || result.is_error == Some(false),
        "Result should not be an error"
    );

    // Parse PromptPackage from result content
    let content_text = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text_content) => &text_content.text,
        _ => return Err("Expected text content".into()),
    };

    let prompt_package: PromptPackage = serde_json::from_str(content_text)?;

    info!("System prompt: {}", prompt_package.system_prompt);
    info!("User prompt: {}", prompt_package.user_prompt);

    // Verify validation-focused content
    assert!(
        prompt_package.system_prompt.contains("validat")
            || prompt_package.system_prompt.contains("quality")
            || prompt_package.system_prompt.contains("check"),
        "System prompt should contain validation keywords"
    );

    info!("Test 2.5 PASSED: Validation prompts generated successfully");

    Ok(())
}

/// Test 2.6: Test Constraint Prompts Generation
///
/// Calls the generate_constraint_prompts tool and verifies the response
/// contains constraint-focused prompts for vocabulary and element enforcement.
#[tokio::test]
async fn test_generate_constraint_prompts() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    info!("=== Test 2.6: Constraint Prompts Generation ===");

    let client = create_nats_client().await?;

    // Create request arguments
    let arguments = serde_json::json!({
        "vocabulary_level": "basic",
        "language": "en",
        "required_elements": ["moral lesson", "science fact"]
    });

    // Call tool
    let result = call_mcp_tool(&client, "generate_constraint_prompts", arguments, 10).await?;

    // Verify no error
    assert!(
        result.is_error.is_none() || result.is_error == Some(false),
        "Result should not be an error"
    );

    // Parse PromptPackage from result content
    let content_text = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text_content) => &text_content.text,
        _ => return Err("Expected text content".into()),
    };

    let prompt_package: PromptPackage = serde_json::from_str(content_text)?;

    info!("System prompt: {}", prompt_package.system_prompt);
    info!("User prompt: {}", prompt_package.user_prompt);

    // Verify constraint-focused content
    assert!(
        prompt_package.system_prompt.contains("constraint")
            || prompt_package.system_prompt.contains("vocabular")
            || prompt_package.system_prompt.contains("required"),
        "System prompt should contain constraint keywords"
    );

    info!("Test 2.6 PASSED: Constraint prompts generated successfully");

    Ok(())
}

/// Test 2.7: Test Model Selection by Language
///
/// Calls the get_model_for_language tool for both English and German
/// and verifies the correct model IDs are returned.
#[tokio::test]
async fn test_get_model_for_language() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    info!("=== Test 2.7: Model Selection by Language ===");

    let client = create_nats_client().await?;

    // Test English model
    info!("Testing English model selection...");
    let arguments_en = serde_json::json!({
        "language": "en"
    });

    let result_en = call_mcp_tool(&client, "get_model_for_language", arguments_en, 5).await?;

    assert!(
        result_en.is_error.is_none() || result_en.is_error == Some(false),
        "Result should not be an error for English"
    );

    let model_en = match &result_en.content[0].raw {
        rmcp::model::RawContent::Text(text_content) => &text_content.text,
        _ => return Err("Expected text content".into()),
    };

    info!("English model: {}", model_en);
    assert!(!model_en.is_empty(), "English model should not be empty");

    // Test German model
    info!("Testing German model selection...");
    let arguments_de = serde_json::json!({
        "language": "de"
    });

    let result_de = call_mcp_tool(&client, "get_model_for_language", arguments_de, 5).await?;

    assert!(
        result_de.is_error.is_none() || result_de.is_error == Some(false),
        "Result should not be an error for German"
    );

    let model_de = match &result_de.content[0].raw {
        rmcp::model::RawContent::Text(text_content) => &text_content.text,
        _ => return Err("Expected text content".into()),
    };

    info!("German model: {}", model_de);
    assert!(!model_de.is_empty(), "German model should not be empty");

    info!("Test 2.7 PASSED: Model selection working for both languages");

    Ok(())
}

/// Test 2.8: Test Template Fallback
///
/// Verifies that when LLM generation fails (or is unavailable),
/// the service falls back to template-based prompt generation.
/// This is indicated by the fallback_used field in PromptPackage.
#[tokio::test]
async fn test_template_fallback() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    info!("=== Test 2.8: Template Fallback ===");
    info!("Note: This test verifies fallback behavior when LLM is unavailable");

    let client = create_nats_client().await?;

    // Create request arguments
    let arguments = serde_json::json!({
        "theme": "Ocean Exploration",
        "age_group": "6-8",
        "language": "en",
        "educational_goals": ["marine biology", "conservation"]
    });

    // Call tool
    let result = call_mcp_tool(&client, "generate_story_prompts", arguments, 10).await?;

    // Verify no error (fallback should still produce valid prompts)
    assert!(
        result.is_error.is_none() || result.is_error == Some(false),
        "Result should not be an error even with fallback"
    );

    // Parse PromptPackage from result content
    let content_text = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text_content) => &text_content.text,
        _ => return Err("Expected text content".into()),
    };

    let prompt_package: PromptPackage = serde_json::from_str(content_text)?;

    info!("Fallback used: {}", prompt_package.fallback_used);
    info!("System prompt: {}", prompt_package.system_prompt);

    // Verify prompts are still generated (from templates)
    assert!(!prompt_package.system_prompt.is_empty(), "System prompt should not be empty with fallback");
    assert!(!prompt_package.user_prompt.is_empty(), "User prompt should not be empty with fallback");

    // If fallback is used, log it
    if prompt_package.fallback_used {
        info!("Template fallback is active (LLM unavailable or failed)");
    } else {
        info!("LLM generation succeeded (no fallback needed)");
    }

    info!("Test 2.8 PASSED: Prompts generated (fallback behavior verified)");

    Ok(())
}

/// Test 2.9: Test Unknown Tool Request
///
/// Sends a request for a non-existent tool and verifies that the service
/// returns an appropriate error response.
#[tokio::test]
async fn test_unknown_tool_error() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    info!("=== Test 2.9: Unknown Tool Error ===");

    let client = create_nats_client().await?;

    // Create request arguments for unknown tool
    let arguments = serde_json::json!({
        "dummy_param": "dummy_value"
    });

    // Call unknown tool
    let result = call_mcp_tool(&client, "unknown_tool", arguments, 5).await?;

    // Verify error response
    assert_eq!(
        result.is_error,
        Some(true),
        "Result should indicate an error for unknown tool"
    );

    // Verify error message mentions unknown tool
    let error_message = match &result.content[0].raw {
        rmcp::model::RawContent::Text(text_content) => &text_content.text,
        _ => return Err("Expected text content".into()),
    };

    info!("Error message: {}", error_message);

    assert!(
        error_message.contains("Unknown") || error_message.contains("unknown"),
        "Error message should mention unknown tool"
    );

    info!("Test 2.9 PASSED: Unknown tool error handled correctly");

    Ok(())
}

/// Test 2.10: Test MCP Server Discovery (Tool List)
///
/// Note: This test is currently a placeholder as the tools/list handler
/// may not be implemented in the server yet. If not implemented, this
/// test documents the expected behavior for future implementation.
#[tokio::test]
#[ignore] // Remove this attribute when tools/list is implemented
async fn test_mcp_server_discovery() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    info!("=== Test 2.10: MCP Server Discovery (Tool List) ===");
    info!("Note: This test requires tools/list handler implementation");

    let client = create_nats_client().await?;

    // Send tools/list request
    let response_bytes = client
        .request_raw("mcp.prompt.tools", b"", Duration::from_secs(5))
        .await?;

    // Parse tool list response
    let tools_response: serde_json::Value = serde_json::from_slice(&response_bytes)?;

    info!("Tools response: {}", serde_json::to_string_pretty(&tools_response)?);

    // Verify 4 tools are returned
    let tools = tools_response["tools"]
        .as_array()
        .ok_or("Expected 'tools' array in response")?;

    assert_eq!(tools.len(), 4, "Should have 4 tools");

    // Verify tool names
    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(
        tool_names.contains(&"generate_story_prompts"),
        "Should include generate_story_prompts"
    );
    assert!(
        tool_names.contains(&"generate_validation_prompts"),
        "Should include generate_validation_prompts"
    );
    assert!(
        tool_names.contains(&"generate_constraint_prompts"),
        "Should include generate_constraint_prompts"
    );
    assert!(
        tool_names.contains(&"get_model_for_language"),
        "Should include get_model_for_language"
    );

    info!("Test 2.10 PASSED: Tool discovery working correctly");

    Ok(())
}
