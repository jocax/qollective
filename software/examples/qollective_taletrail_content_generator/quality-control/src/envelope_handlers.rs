//! Envelope-first handler for Quality Control MCP server
//!
//! This module implements the `EnvelopeHandler<McpData, McpData>` trait to process
//! MCP tool requests wrapped in Qollective envelopes. This enables:
//! - Tenant isolation via `tenant_id` in envelope metadata
//! - Distributed tracing via `trace_id` and `request_id`
//! - Compatibility with Phase 4 orchestrator
//!
//! # Architecture
//!
//! ```text
//! Envelope<McpData> (request)
//!   ↓
//! Extract CallToolRequest from envelope.payload.tool_call
//!   ↓
//! Route to handler (handle_validate_content, handle_batch_validate)
//!   ↓
//! Wrap CallToolResult in Envelope<McpData> (response)
//! ```

use qollective::envelope::Envelope;
use qollective::types::mcp::McpData;
use qollective::server::EnvelopeHandler;
use qollective::error::Result;
use rmcp::model::{CallToolRequest, CallToolResult, Content as McpContent, Tool};
use schemars::{schema_for, JsonSchema};
use serde_json::json;
use std::sync::Arc;
use std::future::Future;
use serde::{Deserialize, Serialize};

use crate::config::QualityControlConfig;
use crate::validation::validate_content_node;
use shared_types::*;
use shared_types::types::tool_registration::{ToolRegistration, ServiceCapabilities};

/// Request parameters for validate_content tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateContentParams {
    pub content_node: ContentNode,
    pub age_group: AgeGroup,
    #[serde(default)]
    pub educational_goals: Vec<String>,

    /// Language code for loading appropriate restricted words
    pub language: String,

    /// Optional validation policy for this request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_policy: Option<ValidationPolicy>,
}

/// Response for validate_content tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateContentResponse {
    pub node_id: String,
    pub validation_result: ValidationResult,
}

/// Request parameters for batch_validate tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchValidateParams {
    pub content_nodes: Vec<ContentNode>,
    pub age_group: AgeGroup,
    #[serde(default)]
    pub educational_goals: Vec<String>,

    /// Language code for loading appropriate restricted words
    pub language: String,

    /// Optional validation policy for this request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_policy: Option<ValidationPolicy>,
}

/// Response for batch_validate tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchValidateResponse {
    pub validations: Vec<ValidateContentResponse>,
    pub overall_pass: bool,
    pub failed_node_ids: Vec<String>,
}

/// Handler for quality-control MCP requests over NATS with envelope support
///
/// This handler implements the envelope-first architecture pattern by:
/// 1. Accepting `Envelope<McpData>` with complete metadata
/// 2. Extracting `CallToolRequest` from `envelope.payload.tool_call`
/// 3. Routing to appropriate validation handler
/// 4. Wrapping `CallToolResult` in response `Envelope<McpData>`
///
/// # Example
///
/// ```no_run
/// use quality_control::envelope_handlers::QualityControlHandler;
/// use quality_control::config::QualityControlConfig;
/// use qollective::server::EnvelopeHandler;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = QualityControlConfig::load()?;
/// let handler = QualityControlHandler::new(config);
///
/// // Handler is used by NatsServer.subscribe_queue_group()
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct QualityControlHandler {
    config: Arc<QualityControlConfig>,
}

impl QualityControlHandler {
    /// Create a new QualityControlHandler
    ///
    /// # Arguments
    ///
    /// * `config` - Quality control configuration
    pub fn new(config: QualityControlConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Get tool registrations for discovery protocol
    ///
    /// Returns metadata about all tools provided by this service including
    /// JSON schemas, capabilities, and service version information.
    ///
    /// # Returns
    ///
    /// Vec<ToolRegistration> containing all available tools
    pub fn get_tool_registrations() -> Vec<ToolRegistration> {
        vec![
            ToolRegistration::new(
                "validate_content",
                json!(schema_for!(ValidateContentParams)),
                "quality-control",
                "0.0.1",
                vec![ServiceCapabilities::Batching, ServiceCapabilities::Retry],
            ),
            ToolRegistration::new(
                "batch_validate",
                json!(schema_for!(BatchValidateParams)),
                "quality-control",
                "0.0.1",
                vec![ServiceCapabilities::Batching, ServiceCapabilities::Retry],
            ),
        ]
    }

    /// Route tool call to appropriate handler
    ///
    /// This method dispatches the tool call based on the tool name to one of:
    /// - `handle_validate_content` - Validate a single content node
    /// - `handle_batch_validate` - Validate multiple content nodes
    ///
    /// # Arguments
    ///
    /// * `request` - MCP CallToolRequest with tool name and arguments
    ///
    /// # Returns
    ///
    /// CallToolResult with tool response JSON or error
    async fn execute_tool(&self, request: CallToolRequest) -> CallToolResult {
        match request.params.name.as_ref() {
            "validate_content" => {
                // Deserialize parameters from MCP CallToolRequest arguments
                // The arguments field is already a Map, convert to Value::Object for deserialization
                let args_value = serde_json::Value::Object(
                    request.params.arguments.clone().unwrap_or_default()
                );

                let params: ValidateContentParams = match serde_json::from_value(args_value) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Failed to deserialize validate_content params: {}", e);
                        return CallToolResult {
                            content: vec![McpContent::text(format!(
                                "Parameter deserialization error: {}",
                                e
                            ))],
                            is_error: Some(true),
                            structured_content: None,
                            meta: None,
                        };
                    }
                };

                // Call handler (now async)
                match handle_validate_content(params, &self.config).await {
                    Ok(response) => {
                        // Serialize response to JSON
                        match serde_json::to_string(&response) {
                            Ok(json) => CallToolResult {
                                content: vec![McpContent::text(json)],
                                is_error: Some(false),
                                structured_content: None,
                                meta: None,
                            },
                            Err(e) => {
                                tracing::error!("Failed to serialize response: {}", e);
                                CallToolResult {
                                    content: vec![McpContent::text(format!(
                                        "Response serialization error: {}",
                                        e
                                    ))],
                                    is_error: Some(true),
                                    structured_content: None,
                                    meta: None,
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("validate_content failed: {}", e);
                        CallToolResult {
                            content: vec![McpContent::text(format!(
                                "Tool execution error: {}",
                                e
                            ))],
                            is_error: Some(true),
                            structured_content: None,
                            meta: None,
                        }
                    }
                }
            }
            "batch_validate" => {
                // Deserialize parameters from MCP CallToolRequest arguments
                let args_value = serde_json::Value::Object(
                    request.params.arguments.clone().unwrap_or_default()
                );

                let params: BatchValidateParams = match serde_json::from_value(args_value) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Failed to deserialize batch_validate params: {}", e);
                        return CallToolResult {
                            content: vec![McpContent::text(format!(
                                "Parameter deserialization error: {}",
                                e
                            ))],
                            is_error: Some(true),
                            structured_content: None,
                            meta: None,
                        };
                    }
                };

                // Call handler (now async)
                match handle_batch_validate(params, &self.config).await {
                    Ok(response) => {
                        // Serialize response to JSON
                        match serde_json::to_string(&response) {
                            Ok(json) => CallToolResult {
                                content: vec![McpContent::text(json)],
                                is_error: Some(false),
                                structured_content: None,
                                meta: None,
                            },
                            Err(e) => {
                                tracing::error!("Failed to serialize response: {}", e);
                                CallToolResult {
                                    content: vec![McpContent::text(format!(
                                        "Response serialization error: {}",
                                        e
                                    ))],
                                    is_error: Some(true),
                                    structured_content: None,
                                    meta: None,
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("batch_validate failed: {}", e);
                        CallToolResult {
                            content: vec![McpContent::text(format!(
                                "Tool execution error: {}",
                                e
                            ))],
                            is_error: Some(true),
                            structured_content: None,
                            meta: None,
                        }
                    }
                }
            }
            _ => {
                // Unknown tool - return error
                tracing::error!("Unknown tool requested: {}", request.params.name);
                CallToolResult {
                    content: vec![McpContent::text(format!(
                        "Unknown tool: {}",
                        request.params.name
                    ))],
                    is_error: Some(true),
                    structured_content: None,
                    meta: None,
                }
            }
        }
    }
}

impl EnvelopeHandler<McpData, McpData> for QualityControlHandler {
    fn handle(&self, envelope: Envelope<McpData>) -> impl Future<Output = Result<Envelope<McpData>>> + Send {
        async move {
            // Extract metadata and payload
            let (meta, data) = envelope.extract();

            // Extract tool call from envelope
            let tool_call = data.tool_call.ok_or_else(|| {
                qollective::error::QollectiveError::mcp_tool_execution(
                    "No tool_call in envelope".to_string()
                )
            })?;

            // Extract trace_id from tracing metadata if present
            let trace_id = meta.tracing.as_ref()
                .and_then(|t| t.trace_id.clone());

            tracing::info!(
                "Processing tool: {} (tenant: {:?}, request_id: {:?}, trace_id: {:?})",
                tool_call.params.name,
                meta.tenant,
                meta.request_id,
                trace_id
            );

            // Execute tool
            let result = self.execute_tool(tool_call).await;

            tracing::info!(
                "Tool execution complete (is_error: {:?})",
                result.is_error
            );

            // Create response McpData
            let response_data = McpData {
                tool_call: None,
                tool_response: Some(result),
                tool_registration: None,
                discovery_data: None,
            };

            // Create response envelope (preserving metadata)
            Ok(Envelope::new(meta, response_data))
        }
    }
}

/// Handler for validate_content tool
///
/// Validates a single content node against age appropriateness, safety, and educational rubrics.
pub async fn handle_validate_content(
    params: ValidateContentParams,
    config: &QualityControlConfig,
) -> Result<ValidateContentResponse> {
    tracing::info!(
        "Validating node {} for age group {:?} | Content preview: {}",
        params.content_node.id,
        params.age_group,
        truncate_content(&params.content_node.content.text, 100)
    );

    let validation_result = validate_content_node(
        &params.content_node,
        params.age_group,
        &params.educational_goals,
        &params.language,                         // Pass language
        params.validation_policy.as_ref(),        // Pass validation policy
        config,                                   // Pass config
    ).await;

    tracing::info!(
        "Validation complete for node {}: is_valid={}",
        params.content_node.id,
        validation_result.is_valid
    );

    Ok(ValidateContentResponse {
        node_id: params.content_node.id.clone(),
        validation_result,
    })
}

/// Handler for batch_validate tool
///
/// Validates multiple content nodes in a batch.
pub async fn handle_batch_validate(
    params: BatchValidateParams,
    config: &QualityControlConfig,
) -> Result<BatchValidateResponse> {
    let node_count = params.content_nodes.len();

    tracing::debug!(
        "Batch validating {} content nodes for age group {:?}",
        node_count,
        params.age_group
    );

    let mut validations = Vec::new();
    let mut failed_node_ids = Vec::new();

    for content_node in params.content_nodes {
        let validation_result = validate_content_node(
            &content_node,
            params.age_group.clone(),
            &params.educational_goals,
            &params.language,                         // Pass language
            params.validation_policy.as_ref(),        // Pass validation policy
            config,                                   // Pass config
        ).await;

        if !validation_result.is_valid {
            failed_node_ids.push(content_node.id.clone());
        }

        validations.push(ValidateContentResponse {
            node_id: content_node.id.clone(),
            validation_result,
        });
    }

    let overall_pass = failed_node_ids.is_empty();

    tracing::info!(
        "Batch validation complete: {}/{} passed",
        node_count - failed_node_ids.len(),
        validations.len()
    );

    Ok(BatchValidateResponse {
        validations,
        overall_pass,
        failed_node_ids,
    })
}

/// Create the "validate_content" tool
///
/// This tool validates a single content node against age appropriateness, safety,
/// and educational rubrics.
#[allow(dead_code)]
pub fn create_validate_content_tool() -> Tool {
    let schema = schema_for!(ValidateContentParams);
    let schema_value =
        serde_json::to_value(schema).expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "validate_content".into(),
        description: Some(
            "Validate a single content node against age appropriateness, safety, and educational rubrics. \
             Performs comprehensive validation including vocabulary level, sentence complexity, \
             safety keywords detection, and educational value assessment. \
             Returns detailed ValidationResult with scores, issues, and correction suggestions."
                .into(),
        ),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Create the "batch_validate" tool
///
/// This tool validates multiple content nodes in batch.
#[allow(dead_code)]
pub fn create_batch_validate_tool() -> Tool {
    let schema = schema_for!(BatchValidateParams);
    let schema_value =
        serde_json::to_value(schema).expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "batch_validate".into(),
        description: Some(
            "Validate multiple content nodes in batch. \
             Performs the same comprehensive validation as validate_content but for multiple nodes. \
             Returns BatchValidateResponse with individual validation results and overall pass/fail status. \
             Useful for validating entire DAG structures or story branches at once."
                .into(),
        ),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Get all available MCP tools for quality control
#[allow(dead_code)]
pub fn get_all_tools() -> Vec<Tool> {
    vec![
        create_validate_content_tool(),
        create_batch_validate_tool(),
    ]
}

/// Truncate content for logging (first N characters)
///
/// This function safely truncates UTF-8 strings by respecting character boundaries,
/// not byte boundaries. This prevents panics when truncating strings containing
/// multi-byte UTF-8 characters (e.g., fancy quotes, emojis, accented characters).
///
/// # Arguments
///
/// * `text` - The text to truncate
/// * `max_len` - Maximum number of characters (not bytes) before truncation
///
/// # Returns
///
/// Truncated string with total character count indicator if content was truncated
///
/// # Examples
///
/// ```
/// use quality_control::envelope_handlers::truncate_content;
///
/// // ASCII text
/// assert_eq!(truncate_content("Hello, World!", 5), "Hello... (13 chars total)");
///
/// // Text with emoji (4-byte UTF-8 characters)
/// assert_eq!(truncate_content("Hello 🌍🚀✨", 7), "Hello 🌍... (9 chars total)");
///
/// // Short text - no truncation needed
/// assert_eq!(truncate_content("Hi!", 10), "Hi!");
/// ```
pub fn truncate_content(text: &str, max_len: usize) -> String {
    let char_count = text.chars().count();

    if char_count <= max_len {
        text.to_string()
    } else {
        // Take first max_len characters (not bytes) to avoid UTF-8 boundary issues
        let truncated: String = text.chars().take(max_len).collect();
        format!("{}... ({} chars total)", truncated, char_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServiceConfig, NatsConfig, QualityControlConfig};
    use qollective::envelope::Meta;
    use rmcp::model::{CallToolRequestParam, CallToolRequestMethod};
    use uuid::Uuid;

    /// Create a test QualityControlConfig for testing
    fn test_quality_control_config() -> QualityControlConfig {
        use shared_types_llm::config::{LlmConfig, ProviderConfig, DebugConfig};
        use shared_types_llm::parameters::{ProviderType, SystemPromptStyle};
        use std::collections::HashMap;

        QualityControlConfig {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            validation: crate::config::ValidationConfig::default(),
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
                    debug: DebugConfig::default(),  // Add missing debug field
                },
                tenants: HashMap::new(),
            },
            rubrics: crate::config::RubricsConfig {
                age_6_8: crate::config::AgeGroupConfig {
                    max_sentence_length: 15.0,
                    vocabulary_level: "basic".to_string(),
                    allowed_themes: vec!["animals".to_string()],
                },
                age_9_11: crate::config::AgeGroupConfig {
                    max_sentence_length: 20.0,
                    vocabulary_level: "intermediate".to_string(),
                    allowed_themes: vec!["adventure".to_string()],
                },
                age_12_14: crate::config::AgeGroupConfig {
                    max_sentence_length: 25.0,
                    vocabulary_level: "intermediate".to_string(),
                    allowed_themes: vec!["mystery".to_string()],
                },
                age_15_17: crate::config::AgeGroupConfig {
                    max_sentence_length: 30.0,
                    vocabulary_level: "advanced".to_string(),
                    allowed_themes: vec!["technology".to_string()],
                },
                age_18_plus: crate::config::AgeGroupConfig {
                    max_sentence_length: 35.0,
                    vocabulary_level: "advanced".to_string(),
                    allowed_themes: vec!["philosophy".to_string()],
                },
            },
            safety: crate::config::SafetyConfig {
                enable_keyword_validation: true,
                restricted_words_dir: "../quality-control/safety".to_string(),
                auto_load_by_language: true,
            },
            educational: crate::config::EducationalConfig {
                educational_keywords: vec!["learn".to_string()],
                goals: Default::default(),
            },
        }
    }

    #[tokio::test]
    async fn test_quality_control_handler_unknown_tool() {
        // ARRANGE: Create handler with test config
        let config = test_quality_control_config();
        let handler = QualityControlHandler::new(config);

        let request = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: "unknown_tool".into(),
                arguments: Some(serde_json::Map::new()),
            },
            extensions: Default::default(),
        };

        let mcp_data = McpData {
            tool_call: Some(request),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant".to_string());
        meta.request_id = Some(Uuid::new_v4());

        let envelope = Envelope::new(meta, mcp_data);

        // ACT: Handle envelope
        let result = handler.handle(envelope).await;

        // ASSERT: Should return error response
        assert!(result.is_ok());
        let response_envelope = result.unwrap();
        let (_, response_data) = response_envelope.extract();

        assert!(response_data.tool_response.is_some());
        let tool_response = response_data.tool_response.unwrap();
        assert_eq!(tool_response.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_quality_control_handler_missing_tool_call() {
        // ARRANGE: Create handler with test config
        let config = test_quality_control_config();
        let handler = QualityControlHandler::new(config);

        // Create envelope with NO tool_call
        let mcp_data = McpData {
            tool_call: None,
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant".to_string());
        meta.request_id = Some(Uuid::new_v4());

        let envelope = Envelope::new(meta, mcp_data);

        // ACT: Handle envelope
        let result = handler.handle(envelope).await;

        // ASSERT: Should return error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("No tool_call in envelope"));
    }

    // ===== Tests for truncate_content function =====

    #[test]
    fn test_truncate_content_ascii_no_truncation() {
        let text = "Hello";
        let max_len = 10;
        let result = truncate_content(text, max_len);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_truncate_content_ascii_exact_length() {
        let text = "Hello";
        let max_len = 5;
        let result = truncate_content(text, max_len);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_truncate_content_ascii_truncation() {
        let text = "Hello, World!";
        let max_len = 5;
        let result = truncate_content(text, max_len);
        assert_eq!(result, "Hello... (13 chars total)");
    }

    #[test]
    fn test_truncate_content_fancy_quotes_no_panic() {
        // This was the original panic case
        let mut text = String::from("Once upon a time in a magical forest, there lived a wise owl named Oliver. ");
        text.push('\u{201C}');
        text.push_str("Who dares disturb my slumber?");
        text.push('\u{201D}');
        text.push_str(" he hooted.");
        let max_len = 100;
        let result = truncate_content(&text, max_len);
        assert!(result.contains("..."));
        assert!(result.contains("chars total"));
        let truncated_part = result.split("...").next().unwrap();
        assert_eq!(truncated_part.chars().count(), 100);
    }

    #[test]
    fn test_truncate_content_emoji_characters() {
        let text = "Hello 🌍🚀✨";
        let max_len = 7;
        let result = truncate_content(text, max_len);
        // Actual char count is 9 (Hello=5, space=1, 3 emoji=3)
        assert_eq!(result, "Hello 🌍... (9 chars total)");
    }

    #[test]
    fn test_truncate_content_empty_string() {
        let text = "";
        let max_len = 10;
        let result = truncate_content(text, max_len);
        assert_eq!(result, "");
    }

    #[test]
    fn test_truncate_content_zero_max_len() {
        let text = "Hello";
        let max_len = 0;
        let result = truncate_content(text, max_len);
        assert_eq!(result, "... (5 chars total)");
    }

    #[test]
    fn test_truncate_content_single_emoji_truncation() {
        let text = "🌍🚀";
        let max_len = 1;
        let result = truncate_content(text, max_len);
        assert_eq!(result, "🌍... (2 chars total)");
    }
}
