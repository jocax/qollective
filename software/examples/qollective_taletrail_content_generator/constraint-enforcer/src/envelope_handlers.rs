//! Envelope-first handler for Constraint Enforcer MCP server
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
//! Route to handler (handle_enforce_constraints, handle_suggest_corrections)
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

use crate::config::ConstraintEnforcerConfig;
use crate::constraints::enforce_constraints;
use shared_types::*;
use shared_types::types::tool_registration::{ToolRegistration, ServiceCapabilities};
use shared_types_llm::{DefaultDynamicLlmClientProvider, DynamicLlmClient, DynamicLlmClientProvider, LlmParameters};
use tracing::{debug, error};

/// Request parameters for enforce_constraints tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnforceConstraintsParams {
    pub content_node: ContentNode,
    pub generation_request: GenerationRequest,
}

/// Response for enforce_constraints tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnforceConstraintsResponse {
    pub node_id: String,
    pub constraint_result: ConstraintResult,
}

/// Request parameters for suggest_corrections tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SuggestCorrectionsParams {
    pub content_node: ContentNode,
    pub generation_request: GenerationRequest,
}

/// Response for suggest_corrections tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SuggestCorrectionsResponse {
    pub node_id: String,
    pub corrections: Vec<CorrectionSuggestion>,
    pub correction_capability: CorrectionCapability,
}

/// Handler for constraint-enforcer MCP requests over NATS with envelope support
///
/// This handler implements the envelope-first architecture pattern by:
/// 1. Accepting `Envelope<McpData>` with complete metadata
/// 2. Extracting `CallToolRequest` from `envelope.payload.tool_call`
/// 3. Routing to appropriate constraint enforcement handler
/// 4. Wrapping `CallToolResult` in response `Envelope<McpData>`
///
/// # Example
///
/// ```no_run
/// use constraint_enforcer::envelope_handlers::ConstraintEnforcerHandler;
/// use constraint_enforcer::config::ConstraintEnforcerConfig;
/// use qollective::server::EnvelopeHandler;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ConstraintEnforcerConfig::load()?;
/// let handler = ConstraintEnforcerHandler::new(config);
///
/// // Handler is used by NatsServer.subscribe_queue_group()
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ConstraintEnforcerHandler {
    config: Arc<ConstraintEnforcerConfig>,
    llm_provider: Option<Arc<DefaultDynamicLlmClientProvider>>,
}

impl ConstraintEnforcerHandler {
    /// Create a new ConstraintEnforcerHandler
    ///
    /// # Arguments
    ///
    /// * `config` - Constraint enforcer configuration
    pub fn new(config: ConstraintEnforcerConfig) -> Self {
        // Create LLM provider for on-demand client creation
        // We don't create the client here because we need language-specific models
        let llm_provider = Some(Arc::new(DefaultDynamicLlmClientProvider::new(config.llm.clone())));

        debug!("✅ LLM provider initialized for hybrid validation");

        Self {
            config: Arc::new(config),
            llm_provider,
        }
    }

    /// Get LLM client for a specific language
    ///
    /// # Arguments
    ///
    /// * `language` - Language code for selecting appropriate model
    ///
    /// # Returns
    ///
    /// LLM client or None if provider not available
    async fn get_llm_client(&self, language: &Language) -> Option<Box<dyn DynamicLlmClient>> {
        let provider = self.llm_provider.as_ref()?;

        let language_code = match language {
            Language::En => "en",
            Language::De => "de",
        };

        let params = LlmParameters {
            language_code: language_code.to_string(),
            ..Default::default()
        };

        match provider.get_dynamic_llm_client(&params).await {
            Ok(client) => Some(client),
            Err(e) => {
                error!(
                    error = %e,
                    language = language_code,
                    "Failed to create LLM client for language"
                );
                None
            }
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
                "enforce_constraints",
                json!(schema_for!(EnforceConstraintsParams)),
                "constraint-enforcer",
                "0.0.1",
                vec![ServiceCapabilities::Batching, ServiceCapabilities::Retry],
            ),
            ToolRegistration::new(
                "suggest_corrections",
                json!(schema_for!(SuggestCorrectionsParams)),
                "constraint-enforcer",
                "0.0.1",
                vec![ServiceCapabilities::Retry],
            ),
        ]
    }

    /// Route tool call to appropriate handler
    ///
    /// This method dispatches the tool call based on the tool name to one of:
    /// - `handle_enforce_constraints` - Enforce constraints on a content node
    /// - `handle_suggest_corrections` - Suggest corrections for violations
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
            "enforce_constraints" => {
                // Deserialize parameters
                let params: EnforceConstraintsParams = match serde_json::from_value(
                    serde_json::Value::Object(
                        request.params.arguments.clone().unwrap_or_default()
                    )
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Failed to deserialize enforce_constraints params: {}", e);
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

                // Get LLM client for the request language
                let llm_client = self.get_llm_client(&params.generation_request.language).await;

                // Call async handler
                match handle_enforce_constraints(
                    params,
                    &self.config,
                    llm_client.as_ref().map(|b| b.as_ref()),
                ).await {
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
                        tracing::error!("enforce_constraints failed: {}", e);
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
            "suggest_corrections" => {
                // Deserialize parameters
                let params: SuggestCorrectionsParams = match serde_json::from_value(
                    serde_json::Value::Object(
                        request.params.arguments.clone().unwrap_or_default()
                    )
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Failed to deserialize suggest_corrections params: {}", e);
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

                // Get LLM client for the request language
                let llm_client = self.get_llm_client(&params.generation_request.language).await;

                // Call async handler
                match handle_suggest_corrections(
                    params,
                    &self.config,
                    llm_client.as_ref().map(|b| b.as_ref()),
                ).await {
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
                        tracing::error!("suggest_corrections failed: {}", e);
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

impl EnvelopeHandler<McpData, McpData> for ConstraintEnforcerHandler {
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

/// Handler for enforce_constraints tool
///
/// Enforces all constraints (vocabulary, theme, required elements) on a content node
/// and generates correction suggestions.
pub async fn handle_enforce_constraints(
    params: EnforceConstraintsParams,
    config: &ConstraintEnforcerConfig,
    llm_client: Option<&dyn DynamicLlmClient>,
) -> shared_types::Result<EnforceConstraintsResponse> {
    tracing::debug!(
        "Enforcing constraints on node {} for theme: {}",
        params.content_node.id,
        params.generation_request.theme
    );

    let constraint_result = match enforce_constraints(
        &params.content_node,
        &params.generation_request,
        &config.constraints.validation,
        llm_client,
    ).await {
        Ok(result) => result,
        Err(e) => return Err(TaleTrailError::ValidationError(format!("Constraint enforcement failed: {}", e))),
    };

    tracing::info!(
        "Constraint enforcement complete: {} vocab violations, theme score: {:.2}, {} missing elements",
        constraint_result.vocabulary_violations.len(),
        constraint_result.theme_consistency_score,
        constraint_result.missing_elements.len()
    );

    Ok(EnforceConstraintsResponse {
        node_id: params.content_node.id.clone(),
        constraint_result,
    })
}

/// Handler for suggest_corrections tool
///
/// Generates correction suggestions without full constraint analysis.
/// This is a lighter-weight version that focuses on actionable corrections.
pub async fn handle_suggest_corrections(
    params: SuggestCorrectionsParams,
    config: &ConstraintEnforcerConfig,
    llm_client: Option<&dyn DynamicLlmClient>,
) -> shared_types::Result<SuggestCorrectionsResponse> {
    tracing::debug!(
        "Generating correction suggestions for node {}",
        params.content_node.id
    );

    // Use full enforcement to get corrections
    let constraint_result = match enforce_constraints(
        &params.content_node,
        &params.generation_request,
        &config.constraints.validation,
        llm_client,
    ).await {
        Ok(result) => result,
        Err(e) => return Err(TaleTrailError::ValidationError(format!("Constraint enforcement failed: {}", e))),
    };

    tracing::info!(
        "Generated {} correction suggestions for node {}",
        constraint_result.corrections.len(),
        params.content_node.id
    );

    Ok(SuggestCorrectionsResponse {
        node_id: params.content_node.id.clone(),
        corrections: constraint_result.corrections,
        correction_capability: constraint_result.correction_capability,
    })
}

/// Create the "enforce_constraints" tool
///
/// This tool enforces vocabulary, theme, and required element constraints on a content node.
#[allow(dead_code)]
pub fn create_enforce_constraints_tool() -> Tool {
    let schema = schema_for!(EnforceConstraintsParams);
    let schema_value =
        serde_json::to_value(schema).expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "enforce_constraints".into(),
        description: Some(
            "Enforce vocabulary, theme, and required element constraints on content node. \
             Performs comprehensive constraint validation including: \
             - Vocabulary level checking with word-level suggestions \
             - Theme consistency validation across story context \
             - Required elements presence checking \
             Returns detailed ConstraintResult with violations, scores, correction capability, \
             and actionable correction suggestions."
                .into(),
        ),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Create the "suggest_corrections" tool
///
/// This tool generates correction suggestions for constraint violations.
#[allow(dead_code)]
pub fn create_suggest_corrections_tool() -> Tool {
    let schema = schema_for!(SuggestCorrectionsParams);
    let schema_value =
        serde_json::to_value(schema).expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "suggest_corrections".into(),
        description: Some(
            "Generate correction suggestions for content node based on constraint violations. \
             This is a lighter-weight version that focuses on actionable corrections without \
             full constraint analysis overhead. \
             Returns list of CorrectionSuggestions with issue, severity, suggestion, and field. \
             Useful for iterative content refinement and LLM-guided corrections."
                .into(),
        ),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Get all available MCP tools for constraint enforcement
#[allow(dead_code)]
pub fn get_all_tools() -> Vec<Tool> {
    vec![
        create_enforce_constraints_tool(),
        create_suggest_corrections_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServiceConfig, NatsConfig, ConstraintsConfig, ConstraintEnforcerConfig};
    use qollective::envelope::Meta;
    use rmcp::model::{CallToolRequestParam, CallToolRequestMethod};
    use uuid::Uuid;
    use shared_types_llm::{LlmConfig, ProviderConfig, ProviderType};

    /// Create a test ConstraintEnforcerConfig for testing
    fn test_constraint_enforcer_config() -> ConstraintEnforcerConfig {
        use crate::config::{VocabularyConfig, LanguageVocabulary, VocabularyLevel, ThemesConfig, RequiredElementsConfig};
        use std::collections::HashMap;

        ConstraintEnforcerConfig {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            llm: LlmConfig {
                provider: ProviderConfig {
                    provider_type: ProviderType::Shimmy,
                    url: "".to_string(),
                    api_key: None,
                    default_model: "".to_string(),
                    use_default_model_fallback: false,
                    models: Default::default(),
                    max_tokens: 0,
                    temperature: 0.0,
                    timeout_secs: 0,
                    system_prompt_style: Default::default(),
                    debug: Default::default(),
                },
                tenants: Default::default() },
            constraints: ConstraintsConfig::default(),
            vocabulary: VocabularyConfig {
                english: LanguageVocabulary {
                    basic: VocabularyLevel { words: vec!["test".to_string()] },
                    intermediate: VocabularyLevel { words: vec!["test".to_string()] },
                    advanced: VocabularyLevel { words: vec!["test".to_string()] },
                },
                german: LanguageVocabulary {
                    basic: VocabularyLevel { words: vec!["test".to_string()] },
                    intermediate: VocabularyLevel { words: vec!["test".to_string()] },
                    advanced: VocabularyLevel { words: vec!["test".to_string()] },
                },
            },
            themes: ThemesConfig {
                min_consistency_score: 0.6,
                keywords: HashMap::new(),
            },
            required_elements: RequiredElementsConfig {
                moral_keywords: vec!["moral".to_string()],
                science_keywords: vec!["science".to_string()],
                educational_keywords: vec!["education".to_string()],
            },
        }
    }

    #[tokio::test]
    async fn test_constraint_enforcer_handler_unknown_tool() {
        // ARRANGE: Create handler with test config
        let config = test_constraint_enforcer_config();
        let handler = ConstraintEnforcerHandler::new(config);

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
    async fn test_constraint_enforcer_handler_missing_tool_call() {
        // ARRANGE: Create handler with test config
        let config = test_constraint_enforcer_config();
        let handler = ConstraintEnforcerHandler::new(config);

        // Create envelope with NO tool_call
        let mcp_data = McpData {
            tool_call: None,
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant".to_string());
        meta.request_id = Some(Uuid::now_v7());

        let envelope = Envelope::new(meta, mcp_data);

        // ACT: Handle envelope
        let result = handler.handle(envelope).await;

        // ASSERT: Should return error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("No tool_call in envelope"));
    }
}
