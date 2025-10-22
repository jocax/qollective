//! Prompt Orchestration Module
//!
//! Orchestrates prompt generation for all MCP services by calling the prompt-helper
//! service via NATS MCP transport. Executes prompt generation in parallel for
//! story, validation, and constraint services.
//!
//! # Architecture
//!
//! The prompt-helper service provides 4 MCP tools:
//! - `generate_story_prompts` - For story-generator service
//! - `generate_validation_prompts` - For quality-control service
//! - `generate_constraint_prompts` - For constraint-enforcer service
//! - `get_model_for_language` - Get LLM model for a language
//!
//! # Error Handling Strategy
//!
//! - **Story prompts**: Critical - failure returns error
//! - **Validation prompts**: Best effort - failure logs warning, continues
//! - **Constraint prompts**: Best effort - failure logs warning, continues

use crate::config::OrchestratorConfig;
use crate::mcp_client::McpEnvelopeClient;
use qollective::envelope::Meta;
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam, Extensions};
use shared_types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, instrument, warn};
use uuid::Uuid;

/// Helper function to create TracingMeta with minimal fields
fn create_tracing_meta(trace_id: String) -> qollective::envelope::meta::TracingMeta {
    qollective::envelope::meta::TracingMeta {
        trace_id: Some(trace_id),
        span_id: None,
        parent_span_id: None,
        baggage: std::collections::HashMap::new(),
        sampling_rate: None,
        sampled: Some(true),
        trace_state: None,
        operation_name: None,
        span_kind: None,
        span_status: None,
        tags: std::collections::HashMap::new(),
    }
}

/// Orchestrates prompt generation for all MCP services
pub struct PromptOrchestrator {
    /// MCP envelope client for envelope-first communication
    mcp_client: McpEnvelopeClient,

    /// NATS subjects for each prompt type (from constants)
    story_subject: String,
    validation_subject: String,
    constraint_subject: String,
    model_subject: String,
}

impl PromptOrchestrator {
    /// Create new prompt orchestrator
    ///
    /// # Arguments
    ///
    /// * `transport` - NATS client for MCP communication
    /// * `config` - Orchestrator configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use orchestrator::prompt_orchestration::PromptOrchestrator;
    /// use orchestrator::OrchestratorConfig;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = OrchestratorConfig::load()?;
    /// let nats_client = async_nats::connect(&config.nats.url).await?;
    /// let orchestrator = PromptOrchestrator::new(Arc::new(nats_client), &config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(transport: Arc<async_nats::Client>, config: &OrchestratorConfig) -> Self {
        let mcp_client = McpEnvelopeClient::new(
            transport,
            config.pipeline.generation_timeout_secs,
        );

        Self {
            mcp_client,
            story_subject: MCP_PROMPT_STORY.to_string(),
            validation_subject: MCP_PROMPT_VALIDATION.to_string(),
            constraint_subject: MCP_PROMPT_CONSTRAINT.to_string(),
            model_subject: MCP_PROMPT_MODEL.to_string(),
        }
    }

    /// Create metadata for MCP requests
    ///
    /// Constructs envelope metadata with tenant_id, request_id, and trace_id.
    ///
    /// # Arguments
    ///
    /// * `request` - Generation request containing tenant_id
    ///
    /// # Returns
    ///
    /// Meta struct populated with tenant and tracing information
    fn create_meta(&self, request: &GenerationRequest) -> Meta {
        let mut meta = Meta::default();

        // Set tenant ID for multi-tenancy isolation
        meta.tenant = Some(format!("tenant-{}", request.tenant_id));

        // Generate a request ID for this prompt generation
        meta.request_id = Some(Uuid::new_v4());

        // Set trace ID for distributed tracing
        let request_id_str = meta.request_id.map(|id| id.to_string()).unwrap_or_default();
        meta.tracing = Some(create_tracing_meta(request_id_str));

        meta
    }

    /// Generate prompts for all MCP services in parallel
    ///
    /// Executes prompt generation for story, validation, and constraint services
    /// concurrently using `tokio::join!`. Story prompts are critical (failure returns error),
    /// while validation and constraint prompts are best-effort (failures are logged).
    ///
    /// # Arguments
    ///
    /// * `request` - Generation request containing theme, age group, language, etc.
    ///
    /// # Returns
    ///
    /// HashMap mapping MCPServiceType to PromptPackage for each successfully generated prompt.
    ///
    /// # Errors
    ///
    /// Returns error if story prompt generation fails (critical path).
    /// Validation and constraint prompt failures are logged but don't cause failure.
    #[instrument(skip(self, request))]
    pub async fn generate_all_prompts(
        &self,
        request: &GenerationRequest,
    ) -> Result<HashMap<MCPServiceType, PromptPackage>> {
        info!("Generating prompts for all services in parallel");

        // Create futures for parallel execution
        let story_future = self.generate_story_prompts(request);
        let validation_future = self.generate_validation_prompts(request);
        let constraint_future = self.generate_constraint_prompts(request);

        // Execute in parallel with tokio::join!
        let (story_result, validation_result, constraint_result) =
            tokio::join!(story_future, validation_future, constraint_future);

        // Build HashMap with results
        let mut prompts = HashMap::new();

        // Handle story prompts (required)
        match story_result {
            Ok(pkg) => {
                prompts.insert(MCPServiceType::StoryGenerator, pkg);
            }
            Err(e) => {
                warn!("Story prompt generation failed: {}, returning error", e);
                // Return error - story prompts are critical
                return Err(e);
            }
        }

        // Handle validation prompts (best effort)
        match validation_result {
            Ok(pkg) => {
                prompts.insert(MCPServiceType::QualityControl, pkg);
            }
            Err(e) => warn!(
                "Validation prompt generation failed: {}, using fallback",
                e
            ),
        }

        // Handle constraint prompts (best effort)
        match constraint_result {
            Ok(pkg) => {
                prompts.insert(MCPServiceType::ConstraintEnforcer, pkg);
            }
            Err(e) => warn!(
                "Constraint prompt generation failed: {}, using fallback",
                e
            ),
        }

        info!(
            prompts_generated = prompts.len(),
            "Prompt generation completed"
        );

        Ok(prompts)
    }

    /// Generate story prompts via MCP tool call
    ///
    /// Calls the `generate_story_prompts` tool on the prompt-helper service.
    #[instrument(skip(self, request))]
    async fn generate_story_prompts(&self, request: &GenerationRequest) -> Result<PromptPackage> {
        info!("Generating story prompts");

        // Build CallToolRequest for generate_story_prompts tool
        let arguments = serde_json::json!({
            "theme": request.theme,
            "age_group": request.age_group,
            "language": request.language,
            "educational_goals": request.educational_goals,
        });

        let arguments_map = arguments.as_object().cloned();

        let tool_request = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: "generate_story_prompts".into(),
                arguments: arguments_map,
            },
            extensions: Extensions::default(),
        };

        // Create metadata for this request
        let meta = self.create_meta(request);

        // Call MCP service via NATS with envelope
        self.mcp_client.call_tool(&self.story_subject, tool_request, meta).await
    }

    /// Generate validation prompts via MCP tool call
    ///
    /// Calls the `generate_validation_prompts` tool on the prompt-helper service.
    #[instrument(skip(self, request))]
    async fn generate_validation_prompts(
        &self,
        request: &GenerationRequest,
    ) -> Result<PromptPackage> {
        info!("Generating validation prompts");

        let arguments = serde_json::json!({
            "age_group": request.age_group,
            "language": request.language,
            "content_type": "story",
        });

        let arguments_map = arguments.as_object().cloned();

        let tool_request = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: "generate_validation_prompts".into(),
                arguments: arguments_map,
            },
            extensions: Extensions::default(),
        };

        // Create metadata for this request
        let meta = self.create_meta(request);

        // Call MCP service via NATS with envelope
        self.mcp_client.call_tool(&self.validation_subject, tool_request, meta).await
    }

    /// Generate constraint prompts via MCP tool call
    ///
    /// Calls the `generate_constraint_prompts` tool on the prompt-helper service.
    #[instrument(skip(self, request))]
    async fn generate_constraint_prompts(
        &self,
        request: &GenerationRequest,
    ) -> Result<PromptPackage> {
        info!("Generating constraint prompts");

        let arguments = serde_json::json!({
            "vocabulary_level": request.vocabulary_level,
            "language": request.language,
            "required_elements": request.required_elements,
        });

        let arguments_map = arguments.as_object().cloned();

        let tool_request = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: "generate_constraint_prompts".into(),
                arguments: arguments_map,
            },
            extensions: Extensions::default(),
        };

        // Create metadata for this request
        let meta = self.create_meta(request);

        // Call MCP service via NATS with envelope
        self.mcp_client.call_tool(&self.constraint_subject, tool_request, meta).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_used() {
        // Verify that constants from shared-types are used
        // All constants use single subject pattern per architectural decision
        assert_eq!(MCP_PROMPT_STORY, "mcp.prompt.helper");
        assert_eq!(MCP_PROMPT_VALIDATION, "mcp.prompt.helper");
        assert_eq!(MCP_PROMPT_CONSTRAINT, "mcp.prompt.helper");
        assert_eq!(MCP_PROMPT_MODEL, "mcp.prompt.helper");

        // Verify they're all the same (single subject pattern)
        assert_eq!(MCP_PROMPT_STORY, MCP_PROMPT_HELPER);
        assert_eq!(MCP_PROMPT_VALIDATION, MCP_PROMPT_HELPER);
        assert_eq!(MCP_PROMPT_CONSTRAINT, MCP_PROMPT_HELPER);
    }

    #[test]
    fn test_orchestrator_structure() {
        // Structural test - verify PromptOrchestrator has expected fields
        // Cannot instantiate without NATS client, but we can verify the type exists
        let _type_exists: Option<PromptOrchestrator> = None;
    }
}
