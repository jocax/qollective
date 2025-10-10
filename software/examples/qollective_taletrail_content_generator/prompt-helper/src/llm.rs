//! LLM service implementation using shared-types-llm
//!
//! This module provides integration with the shared-types-llm crate for
//! dynamic multi-provider LLM support with tenant-aware configuration.

use async_trait::async_trait;
use shared_types::{
    traits::llm_service::NodeContext,
    traits::LlmService,
    PromptGenerationRequest,
    PromptPackage,
    TaleTrailError,
};
use shared_types_llm::{
    DefaultDynamicLlmClientProvider,
    DynamicLlmClientProvider,
    LlmConfig,
    LlmParameters,
    SystemPromptStyle,
};
use std::sync::Arc;
use tracing::{debug, error, info, warn, info_span, Instrument};

/// LLM service using shared-types-llm for multi-provider support
#[derive(Clone)]
pub struct SharedLlmService {
    provider: Arc<DefaultDynamicLlmClientProvider>,
}

impl std::fmt::Debug for SharedLlmService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedLlmService").finish()
    }
}

impl SharedLlmService {
    /// Create new LLM service from configuration
    pub fn new(config: LlmConfig) -> Result<Self, TaleTrailError> {
        let provider = DefaultDynamicLlmClientProvider::new(config);

        Ok(Self {
            provider: Arc::new(provider),
        })
    }

    /// Parse LLM response to extract system and user prompts
    fn parse_llm_response(response: &str) -> Result<(String, String), TaleTrailError> {
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

                debug!("Successfully parsed LLM response with separator: {}", separator);
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
    fn build_content_prompt(prompt_package: &PromptPackage, node_context: &NodeContext) -> String {
        let mut prompt = format!(
            "System: {}\n\nUser: {}\n\n",
            prompt_package.system_prompt, prompt_package.user_prompt
        );

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
impl LlmService for SharedLlmService {
    async fn generate_prompt(
        &self,
        meta_prompt: &str,
        context: &PromptGenerationRequest,
    ) -> Result<(String, String), TaleTrailError> {
        let span = info_span!(
            "llm_generation",
            service_target = ?context.service_target,
        );

        async move {
            let start_time = std::time::Instant::now();

            info!(
                "Generating prompts for theme='{}', age_group={:?}, language={:?}",
                context.generation_request.theme,
                context.generation_request.age_group,
                context.generation_request.language
            );

            // Build comprehensive meta-prompt with context
            let full_prompt = Self::build_meta_prompt(meta_prompt, context);

            debug!("Sending meta-prompt to LLM: {}", full_prompt);

            // Get language code for model selection
            let language_code = match context.generation_request.language {
                shared_types::Language::De => "de",
                shared_types::Language::En => "en",
            };

            // Create LLM parameters
            let params = LlmParameters {
                language_code: language_code.to_string(),
                model_name: None, // Use language mapping
                system_prompt_style: SystemPromptStyle::ChatML,
                tenant_id: None,
                tenant_config: None,
            };

            // Get dynamic client
            let client = self.provider.get_dynamic_llm_client(&params).await
                .map_err(|e| {
                    error!("Failed to get LLM client: {}", e);
                    TaleTrailError::LLMError(format!("Failed to get LLM client: {}", e))
                })?;

            info!("Using model: {}", client.model_name());

            // Send prompt and get response
            let response = client.prompt(&full_prompt).await
                .map_err(|e| {
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

        // Get language code for model selection
        let language_code = match prompt_package.language {
            shared_types::Language::De => "de",
            shared_types::Language::En => "en",
        };

        // Create LLM parameters
        let params = LlmParameters {
            language_code: language_code.to_string(),
            model_name: None,
            system_prompt_style: SystemPromptStyle::ChatML,
            tenant_id: None,
            tenant_config: None,
        };

        // Get dynamic client
        let client = self.provider.get_dynamic_llm_client(&params).await
            .map_err(|e| {
                error!("Failed to get LLM client: {}", e);
                TaleTrailError::LLMError(format!("Failed to get LLM client: {}", e))
            })?;

        // Send prompt and get response
        let content = client.prompt(&full_prompt).await
            .map_err(|e| {
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

    async fn list_models(&self) -> Result<Vec<String>, TaleTrailError> {
        // Not implemented in shared-types-llm yet
        Ok(vec![])
    }

    async fn model_exists(&self, _model_name: &str) -> Result<bool, TaleTrailError> {
        // Not implemented in shared-types-llm yet
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_llm_response_success() {
        let response = "System prompt here\n---SEPARATOR---\nUser prompt here";
        let result = SharedLlmService::parse_llm_response(response);

        assert!(result.is_ok());
        let (system, user) = result.unwrap();
        assert_eq!(system, "System prompt here");
        assert_eq!(user, "User prompt here");
    }

    #[test]
    fn test_parse_llm_response_missing_separator() {
        let response = "This has no separator";
        let result = SharedLlmService::parse_llm_response(response);

        assert!(result.is_err());
    }
}
