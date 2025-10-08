//! Production LLM service implementation using rig-core
//!
//! This module provides the production implementation of the `LlmService` trait
//! using rig-core 0.21 to communicate with LM Studio or other OpenAI-compatible endpoints.
//!
//! # Architecture
//!
//! The `RigLlmService` is designed for:
//! - **Prompt Generation**: Using meta-prompts to dynamically generate system and user prompts
//! - **Content Generation**: Using pre-generated prompt packages to create story content
//! - **Model Management**: Querying available models and checking model availability
//!
//! # LM Studio Integration
//!
//! LM Studio provides an OpenAI-compatible API endpoint (default: http://127.0.0.1:1234/v1)
//! that rig-core can communicate with using standard OpenAI client patterns.
//!
//! # Prompt Parsing Strategy
//!
//! LLM responses are expected to contain two sections separated by a delimiter:
//! - **System Prompt**: Instructions for the LLM on how to behave
//! - **User Prompt**: The actual prompt template with placeholders
//!
//! Supported separators (case-insensitive):
//! - `---SEPARATOR---`
//! - `--- SEPARATOR ---`
//! - `---separator---`
//! - `--- separator ---`
//! - `\n---\n`
//! - `###SEPARATOR###`
//!
//! # Error Handling
//!
//! All errors are mapped to `TaleTrailError` variants:
//! - Network/connection errors → `NetworkError`
//! - Timeout errors → `TimeoutError`
//! - Parse/validation errors → `LLMError`
//! - Empty responses → `LLMError`
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use prompt_helper::llm::RigLlmService;
//! use shared_types::traits::LlmService;
//!
//! let llm = RigLlmService::new("http://127.0.0.1:1234/v1", "llama-3.2-3b-instruct")?;
//!
//! let (system_prompt, user_prompt) = llm
//!     .generate_prompt(meta_prompt, &context)
//!     .await?;
//!
//! println!("System: {}", system_prompt);
//! println!("User: {}", user_prompt);
//! ```

use async_trait::async_trait;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::openai;
use shared_types::{
    traits::llm_service::NodeContext, traits::LlmService, PromptGenerationRequest, PromptPackage,
    TaleTrailError,
};
use tracing::{debug, error, info, warn, info_span, Instrument};

/// Production LLM service using rig-core for OpenAI-compatible endpoints
///
/// This implementation communicates with LM Studio or other OpenAI-compatible
/// API servers to generate prompts and content.
///
/// # Configuration
///
/// - `base_url`: Full API endpoint URL (e.g., "http://127.0.0.1:1234/v1")
/// - `model_name`: Model identifier (e.g., "llama-3.2-3b-instruct")
///
/// # Thread Safety
///
/// `RigLlmService` is `Send + Sync` and can be safely shared across async tasks.
#[derive(Clone)]
pub struct RigLlmService {
    /// OpenAI-compatible client
    client: openai::Client,
    /// Model name for completions
    model_name: String,
    /// Base URL for API endpoint
    base_url: String,
}

impl std::fmt::Debug for RigLlmService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RigLlmService")
            .field("model_name", &self.model_name)
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl RigLlmService {
    /// Create new LLM service with specified endpoint and model
    ///
    /// # Arguments
    ///
    /// * `base_url` - Full API endpoint URL (must be non-empty)
    /// * `model_name` - Model identifier (must be non-empty)
    ///
    /// # Returns
    ///
    /// Configured `RigLlmService` instance or `ConfigError` if parameters are invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let llm = RigLlmService::new(
    ///     "http://127.0.0.1:1234/v1",
    ///     "llama-3.2-3b-instruct"
    /// )?;
    /// ```
    pub fn new(base_url: &str, model_name: &str) -> Result<Self, TaleTrailError> {
        if base_url.is_empty() {
            return Err(TaleTrailError::ConfigError(
                "Base URL cannot be empty".to_string(),
            ));
        }
        if model_name.is_empty() {
            return Err(TaleTrailError::ConfigError(
                "Model name cannot be empty".to_string(),
            ));
        }

        debug!(
            "Initializing RigLlmService with base_url={}, model={}",
            base_url, model_name
        );

        // Create OpenAI-compatible client
        // Note: LM Studio doesn't require an API key, so we use a dummy value
        let client = openai::Client::builder("dummy-key")
            .base_url(base_url)
            .build()
            .map_err(|e| {
                TaleTrailError::ConfigError(format!("Failed to build LLM client: {}", e))
            })?;

        Ok(Self {
            client,
            model_name: model_name.to_string(),
            base_url: base_url.to_string(),
        })
    }

    /// Parse LLM response to extract system and user prompts
    ///
    /// Expected format: "SYSTEM PROMPT\n---SEPARATOR---\nUSER PROMPT"
    ///
    /// # Arguments
    ///
    /// * `response` - Raw LLM response text
    ///
    /// # Returns
    ///
    /// Tuple of `(system_prompt, user_prompt)` or `LLMError` if parsing fails
    ///
    /// # Errors
    ///
    /// - Missing separator → `LLMError`
    /// - Empty system prompt → `LLMError`
    /// - Empty user prompt → `LLMError`
    fn parse_llm_response(response: &str) -> Result<(String, String), TaleTrailError> {
        // Try different separator variants (case-insensitive)
        let separators = vec![
            "---SEPARATOR---",
            "--- SEPARATOR ---",
            "---separator---",
            "--- separator ---",
            "\n---\n",
            "###SEPARATOR###",
        ];

        for separator in separators {
            if let Some((system, user)) = response.split_once(separator) {
                let system_prompt = system.trim().to_string();
                let user_prompt = user.trim().to_string();

                if system_prompt.is_empty() {
                    return Err(TaleTrailError::LLMError(
                        "System prompt is empty after parsing".to_string(),
                    ));
                }
                if user_prompt.is_empty() {
                    return Err(TaleTrailError::LLMError(
                        "User prompt is empty after parsing".to_string(),
                    ));
                }

                debug!(
                    "Successfully parsed LLM response with separator: {}",
                    separator
                );
                return Ok((system_prompt, user_prompt));
            }
        }

        error!("LLM response missing separator: {}", response);
        Err(TaleTrailError::LLMError(
            "LLM response missing separator. Expected format: 'SYSTEM PROMPT\n---SEPARATOR---\nUSER PROMPT'"
                .to_string(),
        ))
    }

    /// Build meta-prompt with context information
    ///
    /// Constructs a detailed meta-prompt that includes all context from the request
    /// to guide the LLM in generating appropriate system and user prompts.
    fn build_meta_prompt(meta_prompt: &str, context: &PromptGenerationRequest) -> String {
        format!(
            "{}\n\nContext:\n- Theme: {}\n- Age Group: {:?}\n- Language: {:?}\n- Educational Goals: {:?}\n- Node Count: {:?}",
            meta_prompt,
            context.generation_request.theme,
            context.generation_request.age_group,
            context.generation_request.language,
            context.generation_request.educational_goals,
            context.generation_request.node_count
        )
    }

    /// Build content generation prompt with node context
    ///
    /// Constructs a prompt that includes the system prompt, user prompt, and node context
    /// to generate story content at a specific node in the DAG.
    fn build_content_prompt(prompt_package: &PromptPackage, node_context: &NodeContext) -> String {
        let mut prompt = format!(
            "System: {}\n\nUser: {}\n\n",
            prompt_package.system_prompt, prompt_package.user_prompt
        );

        // Add node context
        prompt.push_str(&format!(
            "Context:\n- Node position: {}/{}\n",
            node_context.node_position, node_context.total_nodes
        ));

        if let Some(ref prev_content) = node_context.previous_content {
            prompt.push_str(&format!("- Previous content: {}\n", prev_content));
        }

        if !node_context.choices_made.is_empty() {
            prompt.push_str(&format!("- Choices made: {:?}\n", node_context.choices_made));
        }

        prompt
    }
}

#[async_trait]
impl LlmService for RigLlmService {
    /// Generate prompts using meta-prompt (for prompt-helper service)
    ///
    /// Uses LLM to dynamically create system and user prompts based on meta-prompt instructions.
    ///
    /// # Arguments
    ///
    /// * `meta_prompt` - Instructions telling LLM how to generate prompts
    /// * `context` - Request context (age group, language, theme, educational goals)
    ///
    /// # Returns
    ///
    /// Tuple of `(system_prompt, user_prompt)`
    ///
    /// # Errors
    ///
    /// - `TaleTrailError::LLMError`: LLM API communication failure or parse error
    /// - `TaleTrailError::NetworkError`: Network connection failure
    /// - `TaleTrailError::TimeoutError`: Request timeout
    async fn generate_prompt(
        &self,
        meta_prompt: &str,
        context: &PromptGenerationRequest,
    ) -> Result<(String, String), TaleTrailError> {
        let span = info_span!(
            "llm_generation",
            model = %self.model_name,
            service_target = ?context.service_target,
        );

        async move {
            let start_time = std::time::Instant::now();

            info!(
                "Generating prompts for theme='{}', age_group={:?}",
                context.generation_request.theme, context.generation_request.age_group
            );

            // Build comprehensive meta-prompt with context
            let full_prompt = Self::build_meta_prompt(meta_prompt, context);

            debug!("Sending meta-prompt to LLM: {}", full_prompt);

            // Create agent for the model using completions API
            let agent = self
                .client
                .completion_model(&self.model_name)
                .completions_api()
                .into_agent_builder()
                .build();

            // Send prompt and get response
            let response = agent.prompt(&full_prompt).await.map_err(|e| {
                error!("LLM API request failed: {}", e);
                TaleTrailError::LLMError(format!("LLM request failed: {}", e))
            })?;

            debug!("Received LLM response: {}", response);

            // Parse response to extract system and user prompts
            let result = Self::parse_llm_response(&response);

            let duration_ms = start_time.elapsed().as_millis();
            info!("LLM generation completed in {}ms", duration_ms);

            result
        }
        .instrument(span)
        .await
    }

    /// Generate content using prepared prompt package
    ///
    /// # Arguments
    ///
    /// * `prompt_package` - Pre-generated prompts with LLM configuration
    /// * `node_context` - Story context (previous content, choices made)
    ///
    /// # Returns
    ///
    /// Generated content string
    ///
    /// # Errors
    ///
    /// - `TaleTrailError::LLMError`: Content generation failure
    /// - `TaleTrailError::NetworkError`: Network connection failure
    /// - `TaleTrailError::TimeoutError`: Request timeout
    async fn generate_content(
        &self,
        prompt_package: &PromptPackage,
        node_context: &NodeContext,
    ) -> Result<String, TaleTrailError> {
        info!(
            "Generating content at node {}/{}",
            node_context.node_position, node_context.total_nodes
        );

        // Build content generation prompt
        let full_prompt = Self::build_content_prompt(prompt_package, node_context);

        debug!("Sending content prompt to LLM: {}", full_prompt);

        // Create agent for the model using completions API
        let agent = self
            .client
            .completion_model(&self.model_name)
            .completions_api()
            .into_agent_builder()
            .build();

        // Send prompt and get response
        let content = agent.prompt(&full_prompt).await.map_err(|e| {
            error!("LLM content generation failed: {}", e);
            TaleTrailError::LLMError(format!("Content generation failed: {}", e))
        })?;

        if content.trim().is_empty() {
            warn!("LLM returned empty content");
            return Err(TaleTrailError::LLMError(
                "LLM returned empty content".to_string(),
            ));
        }

        debug!("Generated content length: {} characters", content.len());

        Ok(content)
    }

    /// List available LLM models
    ///
    /// # Returns
    ///
    /// Vector of model identifiers (e.g., "gpt-4", "llama-3.2-3b-instruct")
    ///
    /// # Errors
    ///
    /// - `TaleTrailError::LLMError`: Failed to fetch model list
    /// - `TaleTrailError::NetworkError`: Network connection failure
    async fn list_models(&self) -> Result<Vec<String>, TaleTrailError> {
        info!("Fetching available models from {}", self.base_url);

        // Query models endpoint
        // Note: LM Studio exposes models via /v1/models endpoint (OpenAI-compatible)
        let url = format!("{}/models", self.base_url);

        let response = reqwest::get(&url).await.map_err(|e| {
            error!("Failed to fetch model list: {}", e);
            TaleTrailError::NetworkError(format!("Failed to fetch model list: {}", e))
        })?;

        if !response.status().is_success() {
            error!("Model list request failed with status: {}", response.status());
            return Err(TaleTrailError::LLMError(format!(
                "Model list request failed with status: {}",
                response.status()
            )));
        }

        let body = response.text().await.map_err(|e| {
            error!("Failed to read model list response: {}", e);
            TaleTrailError::NetworkError(format!("Failed to read response: {}", e))
        })?;

        // Parse JSON response
        let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            error!("Failed to parse model list JSON: {}", e);
            TaleTrailError::LLMError(format!("Failed to parse model list: {}", e))
        })?;

        // Extract model IDs from response
        // Expected format: {"data": [{"id": "model-name"}, ...]}
        let models = json
            .get("data")
            .and_then(|data| data.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("id"))
                    .filter_map(|id| id.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        debug!("Found {} available models", models.len());

        Ok(models)
    }

    /// Check if specific model is available
    ///
    /// # Arguments
    ///
    /// * `model_name` - Model identifier to check
    ///
    /// # Returns
    ///
    /// `true` if model exists and is available
    ///
    /// # Errors
    ///
    /// - `TaleTrailError::LLMError`: Failed to check model availability
    /// - `TaleTrailError::NetworkError`: Network connection failure
    async fn model_exists(&self, model_name: &str) -> Result<bool, TaleTrailError> {
        debug!("Checking if model '{}' exists", model_name);

        let models = self.list_models().await?;
        let exists = models.contains(&model_name.to_string());

        if exists {
            debug!("Model '{}' is available", model_name);
        } else {
            warn!("Model '{}' not found in available models", model_name);
        }

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_llm_response_success() {
        let response = "System prompt here\n---SEPARATOR---\nUser prompt here";
        let result = RigLlmService::parse_llm_response(response);

        assert!(result.is_ok());
        let (system, user) = result.unwrap();
        assert_eq!(system, "System prompt here");
        assert_eq!(user, "User prompt here");
    }

    #[test]
    fn test_parse_llm_response_with_variant_separator() {
        let response = "System\n--- SEPARATOR ---\nUser";
        let result = RigLlmService::parse_llm_response(response);

        assert!(result.is_ok());
        let (system, user) = result.unwrap();
        assert_eq!(system, "System");
        assert_eq!(user, "User");
    }

    #[test]
    fn test_parse_llm_response_missing_separator() {
        let response = "This has no separator";
        let result = RigLlmService::parse_llm_response(response);

        assert!(result.is_err());
        match result.unwrap_err() {
            TaleTrailError::LLMError(msg) => {
                assert!(msg.contains("missing separator"));
            }
            _ => panic!("Expected LLMError"),
        }
    }

    #[test]
    fn test_parse_llm_response_empty_system_prompt() {
        let response = "\n---SEPARATOR---\nUser prompt";
        let result = RigLlmService::parse_llm_response(response);

        assert!(result.is_err());
        match result.unwrap_err() {
            TaleTrailError::LLMError(msg) => {
                assert!(msg.contains("System prompt is empty"));
            }
            _ => panic!("Expected LLMError"),
        }
    }

    #[test]
    fn test_parse_llm_response_empty_user_prompt() {
        let response = "System prompt\n---SEPARATOR---\n";
        let result = RigLlmService::parse_llm_response(response);

        assert!(result.is_err());
        match result.unwrap_err() {
            TaleTrailError::LLMError(msg) => {
                assert!(msg.contains("User prompt is empty"));
            }
            _ => panic!("Expected LLMError"),
        }
    }
}
