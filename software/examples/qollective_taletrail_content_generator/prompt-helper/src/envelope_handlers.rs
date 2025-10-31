//! Envelope-first handler for Prompt Helper MCP server
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
//! Route to handler (handle_generate_story_prompts, etc.)
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
use std::time::Instant;

use crate::config::PromptHelperConfig;
use crate::execution_logger::ExecutionLogger;
use crate::llm::SharedLlmService;
use crate::tool_handlers::{
    handle_generate_story_prompts,
    handle_generate_validation_prompts,
    handle_generate_constraint_prompts,
    handle_get_model_for_language,
};

/// Handler for prompt-helper MCP requests over NATS with envelope support
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
/// use prompt_helper::handler::PromptHelperHandler;
/// use prompt_helper::config::PromptHelperConfig;
/// use prompt_helper::llm::RigLlmService;
/// use qollective::traits::handlers::EnvelopeHandler;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = PromptHelperConfig::load()?;
/// let llm_service = RigLlmService::new(&config.llm.base_url, &config.llm.default_model)?;
/// let handler = PromptHelperHandler::new(config, llm_service);
///
/// // Handler is used by NatsServer.subscribe_queue_group()
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct PromptHelperHandler {
    config: Arc<PromptHelperConfig>,
    llm_service: Arc<SharedLlmService>,
}

impl PromptHelperHandler {
    /// Create a new PromptHelperHandler
    ///
    /// # Arguments
    ///
    /// * `config` - Prompt helper configuration for template fallback
    /// * `llm_service` - LLM service for prompt generation
    pub fn new(config: PromptHelperConfig, llm_service: SharedLlmService) -> Self {
        Self {
            config: Arc::new(config),
            llm_service: Arc::new(llm_service),
        }
    }

    /// Route tool call to appropriate handler
    ///
    /// This method dispatches the tool call based on the tool name to one of:
    /// - `handle_generate_story_prompts` - Generate story prompts
    /// - `handle_generate_validation_prompts` - Generate validation prompts
    /// - `handle_generate_constraint_prompts` - Generate constraint prompts
    /// - `handle_get_model_for_language` - Get LLM model for language
    ///
    /// # Arguments
    ///
    /// * `request` - MCP CallToolRequest with tool name and arguments
    ///
    /// # Returns
    ///
    /// CallToolResult with PromptPackage JSON or error
    async fn execute_tool(&self, request: CallToolRequest) -> CallToolResult {
        match request.params.name.as_ref() {
            "generate_story_prompts" => {
                handle_generate_story_prompts(
                    request,
                    self.llm_service.as_ref(),
                    &self.config,
                ).await
            }
            "generate_validation_prompts" => {
                handle_generate_validation_prompts(
                    request,
                    self.llm_service.as_ref(),
                    &self.config,
                ).await
            }
            "generate_constraint_prompts" => {
                handle_generate_constraint_prompts(
                    request,
                    self.llm_service.as_ref(),
                    &self.config,
                ).await
            }
            "get_model_for_language" => {
                handle_get_model_for_language(
                    request,
                    self.llm_service.as_ref(),
                    &self.config,
                ).await
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

impl EnvelopeHandler<McpData, McpData> for PromptHelperHandler {
    fn handle(&self, envelope: Envelope<McpData>) -> impl Future<Output = Result<Envelope<McpData>>> + Send {
        async move {
        // Start timing
        let start_time = Instant::now();

        // Extract metadata and payload
        let (meta, data) = envelope.extract();

        // Extract tool call from envelope
        let tool_call = data.tool_call.ok_or_else(|| {
            qollective::error::QollectiveError::mcp_tool_execution(
                "No tool_call in envelope".to_string()
            )
        })?;

        // Extract request_id
        let request_id = meta.request_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| {
                eprintln!("[WARN] No request_id in envelope metadata, using UUID");
                uuid::Uuid::new_v4().to_string()
            });

        // Store tool name for logging
        let tool_name = tool_call.params.name.clone();

        // Initialize execution logger (non-failing)
        let mut logger: Option<ExecutionLogger> = match ExecutionLogger::new(
            request_id.clone(),
            "prompt-helper".to_string(),
            &self.config.execution,
        ) {
            Ok(l) => {
                eprintln!("[DEBUG] Execution logger initialized for request: {}", request_id);
                Some(l)
            }
            Err(e) => {
                eprintln!("[WARN] Failed to initialize execution logger: {}", e);
                None
            }
        };

        // Log request
        if let Some(ref mut logger) = logger {
            let tool_args = serde_json::to_value(&tool_call.params.arguments).unwrap_or_default();
            let _ = logger.log_request(&tool_name, &tool_args);
        }

        // Extract trace_id from tracing metadata if present
        let trace_id = meta.tracing.as_ref()
            .and_then(|t| t.trace_id.clone());

        tracing::info!(
            "Processing tool: {} (tenant: {:?}, request_id: {:?}, trace_id: {:?})",
            tool_name,
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

        // Log response and write metadata
        if let Some(ref mut logger) = logger {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let success = !result.is_error.unwrap_or(false);

            let result_json = serde_json::to_value(&result).unwrap_or_default();
            let _ = logger.log_response(&result_json, duration_ms);
            let _ = logger.write_metadata(&tool_name, success, duration_ms);
        }

        // Create response McpData
        let response_data = McpData {
            tool_call: None,
            tool_response: Some(result),
            tool_registration: None,
            discovery_data: None,
        };

        // Create response envelope (preserving metadata including tracing)
        use qollective::envelope::Meta;
        let response_meta = Meta::preserve_for_response(Some(&meta));
        Ok(Envelope::new(response_meta, response_data))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::SharedLlmService;
    use crate::config::{ServiceConfig, NatsConfig, PromptConfig, PromptHelperConfig};
    use shared_types_llm::LlmConfig;
    use qollective::envelope::Meta;
    use rmcp::model::{CallToolRequestParam, CallToolRequestMethod};
    use serde_json::json;
    use uuid::Uuid;

    /// Create a test LlmConfig using TOML configuration
    fn test_llm_config() -> LlmConfig {
        let toml = r#"
[llm]
type = "shimmy"
url = "http://localhost:11434/v1"
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

    /// Create a test PromptHelperConfig for testing
    fn test_prompt_helper_config() -> PromptHelperConfig {
        PromptHelperConfig {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            llm: test_llm_config(),
            prompt: PromptConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_prompt_helper_handler_unknown_tool() {
        // ARRANGE: Create handler with mock config
        let config = test_prompt_helper_config();
        let llm_service = SharedLlmService::new(test_llm_config()).unwrap();
        let handler = PromptHelperHandler::new(config, llm_service);

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
    async fn test_prompt_helper_handler_missing_tool_call() {
        // ARRANGE: Create handler with mock config
        let config = test_prompt_helper_config();
        let llm_service = SharedLlmService::new(test_llm_config()).unwrap();
        let handler = PromptHelperHandler::new(config, llm_service);

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
