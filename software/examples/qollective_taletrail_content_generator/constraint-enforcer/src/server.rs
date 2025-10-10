//! Constraint Enforcer MCP Server Tool Handlers
//!
//! This module implements the envelope-first handler for constraint enforcement.
//! It provides MCP tools for enforcing narrative constraints including vocabulary
//! levels, theme consistency, and required story elements.

use qollective::envelope::Envelope;
use qollective::types::mcp::McpData;
use qollective::server::EnvelopeHandler;
use qollective::error::Result;
use rmcp::model::{CallToolRequest, CallToolResult, Content as McpContent, Tool};
use schemars::{schema_for, JsonSchema};
use std::sync::Arc;
use std::future::Future;
use serde::{Deserialize, Serialize};

use crate::config::ConstraintEnforcerConfig;
use crate::constraints::enforce_constraints;
use shared_types::*;

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
#[derive(Clone)]
pub struct ConstraintEnforcerHandler {
    config: Arc<ConstraintEnforcerConfig>,
}

impl ConstraintEnforcerHandler {
    /// Create a new ConstraintEnforcerHandler
    ///
    /// # Arguments
    ///
    /// * `config` - Constraint enforcer configuration
    pub fn new(config: ConstraintEnforcerConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
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

                // Call handler
                match handle_enforce_constraints(params, &self.config) {
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

                // Call handler
                match handle_suggest_corrections(params, &self.config) {
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
pub fn handle_enforce_constraints(
    params: EnforceConstraintsParams,
    _config: &ConstraintEnforcerConfig,
) -> Result<EnforceConstraintsResponse> {
    tracing::debug!(
        "Enforcing constraints on node {} for theme: {}",
        params.content_node.id,
        params.generation_request.theme
    );

    let constraint_result = enforce_constraints(&params.content_node, &params.generation_request);

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
pub fn handle_suggest_corrections(
    params: SuggestCorrectionsParams,
    _config: &ConstraintEnforcerConfig,
) -> Result<SuggestCorrectionsResponse> {
    tracing::debug!(
        "Generating correction suggestions for node {}",
        params.content_node.id
    );

    // Use full enforcement to get corrections
    let constraint_result = enforce_constraints(&params.content_node, &params.generation_request);

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
