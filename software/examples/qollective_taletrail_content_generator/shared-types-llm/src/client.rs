//! RigDynamicLlmClient implementation wrapping rig-core
//!
//! This module provides the concrete implementation of DynamicLlmClient
//! that wraps rig-core's OpenAI-compatible client with our configuration
//! and error handling.

use crate::error::LlmError;
use crate::parameters::SystemPromptStyle;
use crate::rig_wrapper::RigClientWrapper;
use crate::traits::DynamicLlmClient;
use async_trait::async_trait;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::gemini::completion::gemini_api_types::{AdditionalParameters, GenerationConfig};
use tracing::{debug, error, info, trace};

/// Concrete implementation of DynamicLlmClient using rig-core
///
/// This client wraps rig-core provider clients (OpenAI, Anthropic, Google) and implements
/// our DynamicLlmClient trait, handling system prompt styles and configuration.
///
/// # Example
///
/// ```no_run
/// use shared_types_llm::{RigDynamicLlmClient, SystemPromptStyle, DynamicLlmClient};
/// use shared_types_llm::rig_wrapper::RigClientWrapper;
/// use rig::providers::openai;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let openai_client = openai::Client::builder("sk-test-key")
///     .base_url("http://127.0.0.1:11435/v1")
///     .build()?;
///
/// let wrapper = RigClientWrapper::OpenAI(Arc::new(openai_client));
///
/// let client = RigDynamicLlmClient::new(
///     wrapper,
///     "qwen2.5-32b-instruct-q4_k_m".to_string(),
///     "shimmy".to_string(),
///     "http://127.0.0.1:11435/v1".to_string(),
///     4096,
///     0.7,
///     SystemPromptStyle::Native,
/// );
///
/// let response = client.prompt("Tell me a joke.").await?;
/// println!("Response: {}", response);
/// # Ok(())
/// # }
/// ```
pub struct RigDynamicLlmClient {
    /// Underlying rig-core client wrapper
    rig_client: RigClientWrapper,
    /// Model name being used
    model: String,
    /// Provider type string
    provider: String,
    /// Base URL for API
    url: String,
    /// Maximum tokens limit
    max_tokens: u32,
    /// Temperature setting
    temperature: f32,
    /// System prompt style
    system_prompt_style: SystemPromptStyle,
}

impl RigDynamicLlmClient {
    /// Create a new RigDynamicLlmClient
    ///
    /// # Arguments
    ///
    /// * `rig_client` - Configured rig-core client wrapper (OpenAI, Anthropic, or Google)
    /// * `model` - Model name to use
    /// * `provider` - Provider type string (for logging/debugging)
    /// * `url` - Base URL for API
    /// * `max_tokens` - Maximum tokens limit
    /// * `temperature` - Temperature setting (0.0 - 1.0)
    /// * `system_prompt_style` - How to handle system prompts
    pub fn new(
        rig_client: RigClientWrapper,
        model: String,
        provider: String,
        url: String,
        max_tokens: u32,
        temperature: f32,
        system_prompt_style: SystemPromptStyle,
    ) -> Self {
        debug!(
            model = %model,
            provider = %provider,
            url = %url,
            max_tokens = max_tokens,
            temperature = temperature,
            system_prompt_style = %system_prompt_style,
            "Creating RigDynamicLlmClient"
        );

        Self {
            rig_client,
            model,
            provider,
            url,
            max_tokens,
            temperature,
            system_prompt_style,
        }
    }
}

#[async_trait]
impl DynamicLlmClient for RigDynamicLlmClient {
    async fn prompt(&self, prompt: &str) -> Result<String, LlmError> {
        trace!(
            prompt_len = prompt.len(),
            model = %self.model,
            provider = %self.provider,
            "Executing LLM prompt"
        );

        // Use the agent() builder method directly on the client,
        // which creates a simple agent without tool configuration.
        // This avoids the 'missing field tools' error with LM Studio.
        let response = match &self.rig_client {
            RigClientWrapper::OpenAI(client) => {
                let agent = client.agent(&self.model).build();
                agent.prompt(prompt).await
            }
            RigClientWrapper::Anthropic(client) => {
                let agent = client.agent(&self.model).build();
                agent.prompt(prompt).await
            }
            RigClientWrapper::Google(client) => {
                info!(
                    model = %self.model,
                    prompt_length = prompt.len(),
                    "=== GOOGLE GEMINI REQUEST ==="
                );

                // Create GenerationConfig (required by Gemini API)
                let gen_cfg = GenerationConfig {
                    top_k: Some(1),
                    top_p: Some(0.95),
                    candidate_count: Some(1),
                    temperature: Some(self.temperature as f64),
                    max_output_tokens: Some(self.max_tokens as u64),
                    ..Default::default()
                };
                let cfg = AdditionalParameters::default().with_config(gen_cfg);

                // Build agent with additional_params containing GenerationConfig
                let agent = client
                    .agent(&self.model)
                    .temperature(self.temperature as f64)
                    .additional_params(serde_json::to_value(cfg)
                        .expect("Failed to serialize GenerationConfig"))
                    .build();

                match agent.prompt(prompt).await {
                    Ok(response) => {
                        info!(
                            response_length = response.len(),
                            "=== GOOGLE GEMINI RESPONSE (SUCCESS) ==="
                        );
                        Ok(response)
                    }
                    Err(e) => {
                        error!(error = %e, "=== GOOGLE GEMINI RESPONSE (ERROR) ===");
                        Err(e)
                    }
                }
            }

        }
        .map_err(|e| {
            LlmError::request_failed(format!("LLM request failed: {}", e))
        })?;

        debug!(
            response_len = response.len(),
            model = %self.model,
            provider = %self.provider,
            "Received LLM response"
        );

        Ok(response)
    }

    fn format_prompt(&self, system_prompt: &str, user_prompt: &str) -> String {
        match self.system_prompt_style {
            SystemPromptStyle::Native => {
                // For Native style, we prepend system prompt as a separate message
                // but since rig-core agent.prompt() doesn't have explicit system role support
                // in completions API, we prepend it to the user message
                format!("System: {}\n\nUser: {}", system_prompt, user_prompt)
            }
            SystemPromptStyle::Prepend => {
                // Prepend system prompt to user message
                format!("{}\n\n{}", system_prompt, user_prompt)
            }
            SystemPromptStyle::ChatML => {
                // Use ChatML format with special tokens
                format!(
                    "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>",
                    system_prompt, user_prompt
                )
            }
            SystemPromptStyle::None => {
                // Ignore system prompt, only use user prompt
                user_prompt.to_string()
            }
        }
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn provider_type(&self) -> &str {
        &self.provider
    }

    fn base_url(&self) -> &str {
        &self.url
    }

    fn max_tokens(&self) -> u32 {
        self.max_tokens
    }

    fn temperature(&self) -> f32 {
        self.temperature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client(style: SystemPromptStyle) -> RigDynamicLlmClient {
        use rig::providers::openai;
        use std::sync::Arc;

        let openai_client = openai::Client::builder("test-key")
            .base_url("http://localhost:11435/v1")
            .build()
            .unwrap();

        let wrapper = RigClientWrapper::OpenAI(Arc::new(openai_client));

        RigDynamicLlmClient::new(
            wrapper,
            "test-model".to_string(),
            "shimmy".to_string(),
            "http://localhost:11435/v1".to_string(),
            4096,
            0.7,
            style,
        )
    }

    #[test]
    fn test_format_prompt_native() {
        let client = create_test_client(SystemPromptStyle::Native);
        let formatted = client.format_prompt("System prompt", "User prompt");

        assert!(formatted.contains("System: System prompt"));
        assert!(formatted.contains("User: User prompt"));
    }

    #[test]
    fn test_format_prompt_prepend() {
        let client = create_test_client(SystemPromptStyle::Prepend);
        let formatted = client.format_prompt("System prompt", "User prompt");

        assert_eq!(formatted, "System prompt\n\nUser prompt");
    }

    #[test]
    fn test_format_prompt_chatml() {
        let client = create_test_client(SystemPromptStyle::ChatML);
        let formatted = client.format_prompt("System prompt", "User prompt");

        assert!(formatted.contains("<|im_start|>system"));
        assert!(formatted.contains("System prompt"));
        assert!(formatted.contains("<|im_start|>user"));
        assert!(formatted.contains("User prompt"));
        assert!(formatted.contains("<|im_end|>"));
    }

    #[test]
    fn test_format_prompt_none() {
        let client = create_test_client(SystemPromptStyle::None);
        let formatted = client.format_prompt("System prompt", "User prompt");

        assert_eq!(formatted, "User prompt");
        assert!(!formatted.contains("System prompt"));
    }

    #[test]
    fn test_client_getters() {
        let client = create_test_client(SystemPromptStyle::Native);

        assert_eq!(client.model_name(), "test-model");
        assert_eq!(client.provider_type(), "shimmy");
        assert_eq!(client.base_url(), "http://localhost:11435/v1");
        assert_eq!(client.max_tokens(), 4096);
        assert_eq!(client.temperature(), 0.7);
    }
}
