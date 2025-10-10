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
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam, CallToolResult, Extensions};
use shared_types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, instrument, warn};

/// Orchestrates prompt generation for all MCP services
pub struct PromptOrchestrator {
    /// Qollective transport client for MCP communication
    transport: Arc<async_nats::Client>,

    /// NATS subjects for each prompt type (from constants)
    story_subject: String,
    validation_subject: String,
    constraint_subject: String,
    model_subject: String,

    /// Timeout for prompt generation (from config)
    timeout_secs: u64,
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
        Self {
            transport,
            story_subject: MCP_PROMPT_STORY.to_string(),
            validation_subject: MCP_PROMPT_VALIDATION.to_string(),
            constraint_subject: MCP_PROMPT_CONSTRAINT.to_string(),
            model_subject: MCP_PROMPT_MODEL.to_string(),
            timeout_secs: config.pipeline.generation_timeout_secs,
        }
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

        // Call MCP service via NATS
        self.call_mcp_tool(&self.story_subject, tool_request).await
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

        self.call_mcp_tool(&self.validation_subject, tool_request)
            .await
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

        self.call_mcp_tool(&self.constraint_subject, tool_request)
            .await
    }

    /// Call MCP tool via NATS transport
    ///
    /// Sends an MCP tool call request via NATS and waits for the response.
    /// Applies timeout from configuration.
    ///
    /// # Arguments
    ///
    /// * `subject` - NATS subject to send request to
    /// * `request` - MCP CallToolRequest to send
    ///
    /// # Returns
    ///
    /// PromptPackage containing the generated prompts
    ///
    /// # Errors
    ///
    /// - `TimeoutError` if request exceeds timeout
    /// - `SerializationError` if request/response serialization fails
    /// - `NetworkError` if NATS communication fails
    /// - `GenerationError` if MCP tool returns error
    async fn call_mcp_tool(
        &self,
        subject: &str,
        request: CallToolRequest,
    ) -> Result<PromptPackage> {
        // Serialize request to JSON
        let request_json = serde_json::to_vec(&request)
            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?;

        // Send via NATS and wait for response (with timeout)
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            self.transport.request(subject.to_string(), request_json.into()),
        )
        .await
        .map_err(|_| {
            TaleTrailError::TimeoutError
        })?
        .map_err(|e| TaleTrailError::NetworkError(e.to_string()))?;

        // Deserialize response
        let tool_result: CallToolResult = serde_json::from_slice(&response.payload)
            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?;

        // Check for MCP error
        if tool_result.is_error == Some(true) {
            return Err(TaleTrailError::GenerationError(format!(
                "MCP tool error: {:?}",
                tool_result.content
            )));
        }

        // Extract PromptPackage from result content
        // The content field is Vec<Content>, where Content is Annotated<RawContent>
        let first_content = tool_result
            .content
            .first()
            .ok_or_else(|| TaleTrailError::GenerationError("Empty MCP response".to_string()))?;

        // Extract text from Content - the raw field contains RawContent::Text tuple variant
        let json_str = match &first_content.raw {
            rmcp::model::RawContent::Text(text_content) => &text_content.text,
            _ => {
                return Err(TaleTrailError::GenerationError(
                    "Expected text content in MCP response".to_string(),
                ))
            }
        };

        // Parse JSON string to PromptPackage
        let prompt_package: PromptPackage = serde_json::from_str(json_str)
            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?;

        Ok(prompt_package)
    }

    /// Get model for language via MCP tool call
    ///
    /// Calls the `get_model_for_language` tool on the prompt-helper service.
    /// This is a utility method for future use.
    #[instrument(skip(self))]
    #[allow(dead_code)]
    async fn get_model_for_language(&self, language: Language) -> Result<String> {
        info!(?language, "Getting LLM model for language");

        let arguments = serde_json::json!({
            "language": language,
        });

        let arguments_map = arguments.as_object().cloned();

        let tool_request = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: "get_model_for_language".into(),
                arguments: arguments_map,
            },
            extensions: Extensions::default(),
        };

        let request_json = serde_json::to_vec(&tool_request)
            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?;

        let response = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            self.transport
                .request(self.model_subject.clone(), request_json.into()),
        )
        .await
        .map_err(|_| TaleTrailError::TimeoutError)?
        .map_err(|e| TaleTrailError::NetworkError(e.to_string()))?;

        let tool_result: CallToolResult = serde_json::from_slice(&response.payload)
            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?;

        if tool_result.is_error == Some(true) {
            return Err(TaleTrailError::GenerationError(format!(
                "MCP tool error: {:?}",
                tool_result.content
            )));
        }

        // Extract model name from response
        let first_content = tool_result
            .content
            .first()
            .ok_or_else(|| TaleTrailError::GenerationError("Empty MCP response".to_string()))?;

        // Extract text from Content
        let model_name = match &first_content.raw {
            rmcp::model::RawContent::Text(text_content) => text_content.text.clone(),
            _ => {
                return Err(TaleTrailError::GenerationError(
                    "Expected text content in MCP response".to_string(),
                ))
            }
        };

        Ok(model_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_used() {
        // Verify that constants from shared-types are used
        assert_eq!(MCP_PROMPT_STORY, "mcp.prompt.generate_story");
        assert_eq!(MCP_PROMPT_VALIDATION, "mcp.prompt.generate_validation");
        assert_eq!(MCP_PROMPT_CONSTRAINT, "mcp.prompt.generate_constraint");
        assert_eq!(MCP_PROMPT_MODEL, "mcp.prompt.get_model");
    }

    #[test]
    fn test_orchestrator_structure() {
        // Structural test - verify PromptOrchestrator has expected fields
        // Cannot instantiate without NATS client, but we can verify the type exists
        let _type_exists: Option<PromptOrchestrator> = None;
    }
}
