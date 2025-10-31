//! Envelope-first handler for Orchestrator MCP server
//!
//! This module implements the `EnvelopeHandler<McpData, McpData>` trait to process
//! MCP tool requests wrapped in Qollective envelopes. This enables:
//! - Tenant isolation via `tenant_id` in envelope metadata
//! - Distributed tracing via `trace_id` and `request_id`
//! - Compatibility with downstream MCP services
//!
//! # Architecture
//!
//! ```text
//! Envelope<McpData> (request)
//!   ↓
//! Extract CallToolRequest from envelope.payload.tool_call
//!   ↓
//! Route to handler (handle_orchestrate_generation)
//!   ↓
//! Call Orchestrator.orchestrate_generation()
//!   ↓
//! Wrap CallToolResult in Envelope<McpData> (response)
//! ```

use qollective::envelope::Envelope;
use qollective::types::mcp::McpData;
use qollective::server::EnvelopeHandler;
use qollective::error::Result;
use rmcp::model::{CallToolRequest, CallToolResult, Content as McpContent, Tool};
use schemars::{schema_for, JsonSchema};
use std::sync::Arc;
use std::future::Future;
use std::time::Instant;
use serde::{Deserialize, Serialize};

use crate::orchestrator::Orchestrator;
use crate::execution_logger::ExecutionLogger;
use shared_types::*;

/// Request parameters for orchestrate_generation tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrchestrateGenerationParams {
    pub generation_request: GenerationRequest,
}

/// Response for orchestrate_generation tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrchestrateGenerationResponse {
    pub generation_response: GenerationResponse,
}

/// Handler for orchestrator MCP requests over NATS with envelope support
///
/// This handler implements the envelope-first architecture pattern by:
/// 1. Accepting `Envelope<McpData>` with complete metadata
/// 2. Extracting `CallToolRequest` from `envelope.payload.tool_call`
/// 3. Routing to orchestration handler
/// 4. Wrapping `CallToolResult` in response `Envelope<McpData>`
///
/// # Example
///
/// ```no_run
/// use orchestrator::envelope_handlers::OrchestratorHandler;
/// use orchestrator::{Orchestrator, OrchestratorConfig};
/// use qollective::server::EnvelopeHandler;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = OrchestratorConfig::load()?;
/// let nats_client = async_nats::connect(&config.nats.url).await?;
/// let orchestrator = Orchestrator::new(Arc::new(nats_client), config);
/// let handler = OrchestratorHandler::new(Arc::new(orchestrator));
///
/// // Handler is used by NatsServer.subscribe_queue_group()
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct OrchestratorHandler {
    orchestrator: Arc<Orchestrator>,
}

impl OrchestratorHandler {
    /// Create a new OrchestratorHandler
    ///
    /// # Arguments
    ///
    /// * `orchestrator` - Orchestrator instance for generation pipeline
    pub fn new(orchestrator: Arc<Orchestrator>) -> Self {
        Self { orchestrator }
    }

    /// Route tool call to appropriate handler
    ///
    /// This method dispatches the tool call based on the tool name to:
    /// - `handle_orchestrate_generation` - Orchestrate complete content generation
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
            "orchestrate_generation" => {
                // Deserialize parameters
                let params: OrchestrateGenerationParams = match serde_json::from_value(
                    serde_json::Value::Object(
                        request.params.arguments.clone().unwrap_or_default()
                    )
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Failed to deserialize orchestrate_generation params: {}", e);
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
                match handle_orchestrate_generation(params, &self.orchestrator).await {
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
                        tracing::error!("orchestrate_generation failed: {}", e);
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

impl EnvelopeHandler<McpData, McpData> for OrchestratorHandler {
    fn handle(&self, envelope: Envelope<McpData>) -> impl Future<Output = Result<Envelope<McpData>>> + Send {
        async move {
            let start_time = Instant::now();

            // Extract metadata and payload
            let (meta, data) = envelope.extract();

            // Extract tool call from envelope
            let tool_call = data.tool_call.ok_or_else(|| {
                qollective::error::QollectiveError::mcp_tool_execution(
                    "No tool_call in envelope".to_string()
                )
            })?;

            // Extract request_id from envelope metadata
            let request_id = meta.request_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| {
                    eprintln!("[WARN] No request_id in envelope metadata, using UUID");
                    uuid::Uuid::new_v4().to_string()
                });

            // Initialize execution logger (non-failing)
            let mut logger = match ExecutionLogger::new(
                request_id.clone(),
                "orchestrator".to_string(),
                &self.orchestrator.config.execution,
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
            let tool_name = tool_call.params.name.clone();
            let tool_arguments = serde_json::to_value(&tool_call.params.arguments).unwrap_or_default();
            if let Some(ref mut logger) = logger {
                let _ = logger.log_request(&tool_name, &tool_arguments);
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

                let _ = logger.log_response(&serde_json::to_value(&result).unwrap_or_default(), duration_ms);
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

/// Handler for orchestrate_generation tool
///
/// Orchestrates the complete content generation pipeline through all MCP services.
pub async fn handle_orchestrate_generation(
    params: OrchestrateGenerationParams,
    orchestrator: &Orchestrator,
) -> Result<OrchestrateGenerationResponse> {
    tracing::debug!(
        "Starting orchestration for theme: {}, age_group: {:?}, language: {:?}",
        params.generation_request.theme,
        params.generation_request.age_group,
        params.generation_request.language
    );

    let generation_response = orchestrator
        .orchestrate_generation(params.generation_request)
        .await
        .map_err(|e| qollective::error::QollectiveError::mcp_tool_execution(e.to_string()))?;

    tracing::info!(
        "Orchestration complete: status={:?}, progress={}%",
        generation_response.status,
        generation_response.progress_percentage
    );

    Ok(OrchestrateGenerationResponse {
        generation_response,
    })
}

/// Create the "orchestrate_generation" tool
///
/// This tool orchestrates the complete content generation pipeline.
#[allow(dead_code)]
pub fn create_orchestrate_generation_tool() -> Tool {
    let schema = schema_for!(OrchestrateGenerationParams);
    let schema_value =
        serde_json::to_value(schema).expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "orchestrate_generation".into(),
        description: Some(
            "Orchestrate complete content generation pipeline for TaleTrail stories. \
             Coordinates prompt-helper, story-generator, quality-control, and constraint-enforcer services \
             to generate, validate, and refine interactive story content. \
             Supports multiple age groups, languages, and educational goals. \
             Returns GenerationResponse with complete DAG structure and metadata."
                .into(),
        ),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Get all available MCP tools for orchestrator
#[allow(dead_code)]
pub fn get_all_tools() -> Vec<Tool> {
    vec![create_orchestrate_generation_tool()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ServiceConfig, NatsConfig, OrchestratorConfig, PipelineConfig,
        BatchConfig, DagConfig, NegotiationConfig
    };
    use shared_types_llm::LlmConfig;
    use qollective::envelope::Meta;
    use rmcp::model::{CallToolRequestParam, CallToolRequestMethod};
    use uuid::Uuid;

    /// Check if infrastructure tests should run
    fn should_run_infra_tests() -> bool {
        std::env::var("ENABLE_INFRA_TESTS").is_ok()
    }

    /// Create a test OrchestratorConfig for testing
    fn test_orchestrator_config() -> OrchestratorConfig {
        let llm_toml = r#"
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
        let llm_config = LlmConfig::from_toml_str(llm_toml).expect("Failed to create test LLM config");

        OrchestratorConfig {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            llm: llm_config,
            pipeline: PipelineConfig {
                generation_timeout_secs: 60,
                validation_timeout_secs: 30,
                retry_max_attempts: 3,
                retry_base_delay_secs: 5,
                retry_max_delay_secs: 30,
            },
            batch: BatchConfig {
                size_min: 3,
                size_max: 5,
                concurrent_batches: 2,
                concurrent_batches_max: 4,
            },
            dag: DagConfig {
                default_node_count: 10,
                convergence_pattern: "SingleConvergence".to_string(),
                convergence_point_ratio: 0.6,
                max_depth: 20,
                branching_factor: 2,
            },
            negotiation: NegotiationConfig {
                max_rounds: 3,
            },
            retry: crate::config::RetryConfig::default(),
        }
    }

    /// Create a test GenerationRequest for testing
    #[allow(dead_code)]
    fn test_generation_request() -> GenerationRequest {
        GenerationRequest {
            theme: "Space Adventure".to_string(),
            age_group: AgeGroup::_9To11,
            language: Language::En,
            node_count: Some(10),
            tenant_id: 123,
            educational_goals: Some(vec!["Explore space concepts".to_string()]),
            vocabulary_level: Some(VocabularyLevel::Intermediate),
            required_elements: Some(vec!["planets".to_string()]),
            tags: Some(vec!["space".to_string(), "science".to_string()]),
            prompt_packages: None,
            author_id: Some(Some(1)),
            story_structure: None,
            dag_config: None,
            validation_policy: None,
        }
    }

    #[tokio::test]
    async fn infra_orchestrator_handler_unknown_tool() {
        if !should_run_infra_tests() {
            eprintln!("Skipping: Set ENABLE_INFRA_TESTS=1 to run infrastructure tests");
            return;
        }

        // ARRANGE: Create handler with mock orchestrator
        let config = test_orchestrator_config();
        let nats_client = Arc::new(
            async_nats::connect("nats://localhost:4222")
                .await
                .expect("Failed to connect to NATS for test")
        );
        let orchestrator = Arc::new(
            Orchestrator::new(nats_client, config)
                .await
                .expect("Failed to create orchestrator")
        );
        let handler = OrchestratorHandler::new(orchestrator);

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
    async fn infra_orchestrator_handler_missing_tool_call() {
        if !should_run_infra_tests() {
            eprintln!("Skipping: Set ENABLE_INFRA_TESTS=1 to run infrastructure tests");
            return;
        }

        // ARRANGE: Create handler with mock orchestrator
        let config = test_orchestrator_config();
        let nats_client = Arc::new(
            async_nats::connect("nats://localhost:4222")
                .await
                .expect("Failed to connect to NATS for test")
        );
        let orchestrator = Arc::new(
            Orchestrator::new(nats_client, config)
                .await
                .expect("Failed to create orchestrator")
        );
        let handler = OrchestratorHandler::new(orchestrator);

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
