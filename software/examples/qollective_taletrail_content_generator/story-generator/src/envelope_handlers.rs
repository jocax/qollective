//! Envelope-first handler for Story Generator MCP server
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
//! Route to handler (handle_generate_structure, handle_generate_nodes, handle_validate_paths)
//!   ↓
//! Wrap CallToolResult in Envelope<McpData> (response)
//! ```

use qollective::envelope::Envelope;
use qollective::types::mcp::McpData;
use qollective::server::EnvelopeHandler;
use qollective::error::Result;
use rmcp::model::{CallToolRequest, CallToolResult, Content};
use std::sync::Arc;
use std::future::Future;

use crate::config::StoryGeneratorConfig;
use crate::llm::StoryLlmClient;
use crate::mcp_tools::{
    GenerateStructureParams, GenerateNodesParams, ValidatePathsParams,
};
use crate::tool_handlers::{
    handle_generate_structure,
    handle_generate_nodes,
    handle_validate_paths,
};

/// Handler for story-generator MCP requests over NATS with envelope support
///
/// This handler implements the envelope-first architecture pattern by:
/// 1. Accepting `Envelope<McpData>` with complete metadata
/// 2. Extracting `CallToolRequest` from `envelope.payload.tool_call`
/// 3. Routing to appropriate tool handler
/// 4. Wrapping `CallToolResult` in response `Envelope<McpData>`
///
/// # Example
///
/// ```no_run
/// use story_generator::envelope_handlers::StoryGeneratorHandler;
/// use story_generator::config::StoryGeneratorConfig;
/// use story_generator::llm::StoryLlmClient;
/// use qollective::server::EnvelopeHandler;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = StoryGeneratorConfig::load()?;
/// let llm_client = StoryLlmClient::new(config.llm.clone())?;
/// let handler = StoryGeneratorHandler::new(config, llm_client);
///
/// // Handler is used by NatsServer.subscribe_queue_group()
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct StoryGeneratorHandler {
    #[allow(dead_code)]
    config: Arc<StoryGeneratorConfig>,
    llm_client: Arc<StoryLlmClient>,
}

impl StoryGeneratorHandler {
    /// Create a new StoryGeneratorHandler
    ///
    /// # Arguments
    ///
    /// * `config` - Story generator configuration
    /// * `llm_client` - LLM client for content generation
    pub fn new(config: StoryGeneratorConfig, llm_client: StoryLlmClient) -> Self {
        Self {
            config: Arc::new(config),
            llm_client: Arc::new(llm_client),
        }
    }

    /// Route tool call to appropriate handler
    ///
    /// This method dispatches the tool call based on the tool name to one of:
    /// - `handle_generate_structure` - Create DAG structure with convergence points
    /// - `handle_generate_nodes` - Generate narrative content for nodes
    /// - `handle_validate_paths` - Validate DAG path connectivity
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
            "generate_structure" => {
                // Deserialize parameters
                let params: GenerateStructureParams = match serde_json::from_value(
                    serde_json::Value::Object(
                        request.params.arguments.clone().unwrap_or_default()
                    )
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Failed to deserialize generate_structure params: {}", e);
                        return CallToolResult {
                            content: vec![Content::text(format!(
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
                match handle_generate_structure(params) {
                    Ok(response) => {
                        // Serialize response to JSON
                        match serde_json::to_string(&response) {
                            Ok(json) => CallToolResult {
                                content: vec![Content::text(json)],
                                is_error: Some(false),
                                structured_content: None,
                                meta: None,
                            },
                            Err(e) => {
                                tracing::error!("Failed to serialize response: {}", e);
                                CallToolResult {
                                    content: vec![Content::text(format!(
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
                        tracing::error!("generate_structure failed: {}", e);
                        CallToolResult {
                            content: vec![Content::text(format!(
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
            "generate_nodes" => {
                // Deserialize parameters
                let params: GenerateNodesParams = match serde_json::from_value(
                    serde_json::Value::Object(
                        request.params.arguments.clone().unwrap_or_default()
                    )
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Failed to deserialize generate_nodes params: {}", e);
                        return CallToolResult {
                            content: vec![Content::text(format!(
                                "Parameter deserialization error: {}",
                                e
                            ))],
                            is_error: Some(true),
                            structured_content: None,
                            meta: None,
                        };
                    }
                };

                // Extract prompt_package from generation request
                // Note: prompt_packages is Option<Option<HashMap<String, serde_json::Value>>>
                let prompt_package_map = match &params.generation_request.prompt_packages {
                    Some(Some(packages)) if !packages.is_empty() => packages,
                    _ => {
                        tracing::error!("No prompt packages provided in generation request");
                        return CallToolResult {
                            content: vec![Content::text(
                                "Missing or empty prompt_packages in generation request. Call prompt-helper first.".to_string()
                            )],
                            is_error: Some(true),
                            structured_content: None,
                            meta: None,
                        };
                    }
                };

                // Get the story generation package (key: "story_generator")
                let prompt_package_json = match prompt_package_map.get("story_generator") {
                    Some(pkg) => pkg,
                    None => {
                        tracing::error!("No 'story_generator' package found in prompt_packages");
                        return CallToolResult {
                            content: vec![Content::text(
                                "Missing 'story_generator' package in prompt_packages".to_string()
                            )],
                            is_error: Some(true),
                            structured_content: None,
                            meta: None,
                        };
                    }
                };

                // Deserialize to PromptPackage
                let prompt_package: shared_types::PromptPackage = match serde_json::from_value(prompt_package_json.clone()) {
                    Ok(pkg) => pkg,
                    Err(e) => {
                        tracing::error!("Failed to deserialize PromptPackage: {}", e);
                        return CallToolResult {
                            content: vec![Content::text(format!(
                                "Invalid PromptPackage format: {}",
                                e
                            ))],
                            is_error: Some(true),
                            structured_content: None,
                            meta: None,
                        };
                    }
                };

                // Call async handler
                match handle_generate_nodes(
                    params,
                    self.llm_client.as_ref(),
                    &prompt_package,
                ).await {
                    Ok(response) => {
                        // Serialize response to JSON
                        match serde_json::to_string(&response) {
                            Ok(json) => CallToolResult {
                                content: vec![Content::text(json)],
                                is_error: Some(false),
                                structured_content: None,
                                meta: None,
                            },
                            Err(e) => {
                                tracing::error!("Failed to serialize response: {}", e);
                                CallToolResult {
                                    content: vec![Content::text(format!(
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
                        tracing::error!("generate_nodes failed: {}", e);
                        CallToolResult {
                            content: vec![Content::text(format!(
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
            "validate_paths" => {
                // Deserialize parameters
                let params: ValidatePathsParams = match serde_json::from_value(
                    serde_json::Value::Object(
                        request.params.arguments.clone().unwrap_or_default()
                    )
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Failed to deserialize validate_paths params: {}", e);
                        return CallToolResult {
                            content: vec![Content::text(format!(
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
                match handle_validate_paths(params) {
                    Ok(response) => {
                        // Serialize response to JSON
                        match serde_json::to_string(&response) {
                            Ok(json) => CallToolResult {
                                content: vec![Content::text(json)],
                                is_error: Some(false),
                                structured_content: None,
                                meta: None,
                            },
                            Err(e) => {
                                tracing::error!("Failed to serialize response: {}", e);
                                CallToolResult {
                                    content: vec![Content::text(format!(
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
                        tracing::error!("validate_paths failed: {}", e);
                        CallToolResult {
                            content: vec![Content::text(format!(
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
                    content: vec![Content::text(format!(
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

impl EnvelopeHandler<McpData, McpData> for StoryGeneratorHandler {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServiceConfig, NatsConfig, GenerationConfig};
    use qollective::envelope::Meta;
    use rmcp::model::{CallToolRequestParam, CallToolRequestMethod};
    use uuid::Uuid;
    use shared_types_llm::LlmConfig;

    /// Create a test LlmConfig using TOML configuration
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

    /// Create a test StoryGeneratorConfig for testing
    fn test_story_generator_config() -> StoryGeneratorConfig {
        StoryGeneratorConfig {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            generation: GenerationConfig::default(),
            llm: test_llm_config(),
        }
    }

    #[tokio::test]
    async fn test_story_generator_handler_unknown_tool() {
        // ARRANGE: Create handler with mock config
        let config = test_story_generator_config();
        let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
        let handler = StoryGeneratorHandler::new(config, llm_client);

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
    async fn test_story_generator_handler_missing_tool_call() {
        // ARRANGE: Create handler with mock config
        let config = test_story_generator_config();
        let llm_client = StoryLlmClient::new(test_llm_config()).unwrap();
        let handler = StoryGeneratorHandler::new(config, llm_client);

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
}
