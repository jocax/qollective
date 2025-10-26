//! RigDynamicLlmClient implementation wrapping rig-core
//!
//! This module provides the concrete implementation of DynamicLlmClient
//! that wraps rig-core's OpenAI-compatible client with our configuration
//! and error handling.

use crate::config::DebugConfig;
use crate::constants::*;
use crate::error::LlmError;
use crate::parameters::{RequestContext, SystemPromptStyle};
use crate::rig_wrapper::RigClientWrapper;
use crate::traits::DynamicLlmClient;
use async_trait::async_trait;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::gemini::completion::gemini_api_types::{AdditionalParameters, GenerationConfig};
use tracing::{debug, error, info, warn};

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
///     Default::default(),
/// );
///
/// let response = client.prompt("Tell me a joke.", None).await?;
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
    /// Debug configuration
    debug_config: DebugConfig,
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
    /// * `debug_config` - Debug configuration for response logging
    pub fn new(
        rig_client: RigClientWrapper,
        model: String,
        provider: String,
        url: String,
        max_tokens: u32,
        temperature: f32,
        system_prompt_style: SystemPromptStyle,
        debug_config: DebugConfig,
    ) -> Self {
        debug!(
            model = %model,
            provider = %provider,
            url = %url,
            max_tokens = max_tokens,
            temperature = temperature,
            system_prompt_style = %system_prompt_style,
            debug_enabled = debug_config.dump_raw_response_enabled,
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
            debug_config,
        }
    }

    /// Truncate text for preview logging
    fn truncate_text(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}... ({} chars total)", &text[..max_len], text.len())
        }
    }

    /// Dump LLM response to file for debugging
    fn dump_response_to_file(&self, prompt: &str, response: &str, context: Option<&RequestContext>) -> Result<(), LlmError> {
        use std::fs;
        use std::io::Write;
        use std::path::Path;

        // Create dump directory if it doesn't exist
        let dump_dir = Path::new(&self.debug_config.dump_directory);
        fs::create_dir_all(dump_dir).map_err(|e| {
            LlmError::config_error(format!("Failed to create dump directory: {}", e))
        })?;

        // Generate filename with timestamp and model
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S_%3f");
        let safe_model_name = self.model.replace('/', "_").replace(':', "_");

        // Build context suffix for filename if context is provided
        let context_suffix = if let Some(ctx) = context {
            if !ctx.metadata.is_empty() {
                let suffix = ctx.metadata.iter()
                    .map(|(k, v)| format!("{}_{}", k, v.replace('/', "_").replace(':', "_")))
                    .collect::<Vec<_>>()
                    .join("_");
                format!("_{}", suffix)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let response_preview = &response
            .chars()
            .take(50)
            .collect::<String>()
            .replace(|c: char| !c.is_alphanumeric(), "_");

        let filename = format!(
            "{}_{}_{}{}_{}",
            LLM_DUMP_FILENAME_PREFIX,
            timestamp,
            safe_model_name,
            context_suffix,  // Will be empty if no context, or "_node_id_20_batch_id_abc" if context exists
            response_preview
        );

        let filepath = dump_dir.join(format!("{}.txt", filename));

        // Write prompt and response
        let mut file = fs::File::create(&filepath).map_err(|e| {
            LlmError::config_error(format!("Failed to create dump file: {}", e))
        })?;

        writeln!(file, "=== LLM REQUEST DUMP ===").map_err(|e| {
            LlmError::config_error(format!("Failed to write to dump file: {}", e))
        })?;
        writeln!(file, "Timestamp: {}", chrono::Utc::now().to_rfc3339()).map_err(|e| {
            LlmError::config_error(format!("Failed to write to dump file: {}", e))
        })?;
        writeln!(file, "Model: {}", self.model).map_err(|e| {
            LlmError::config_error(format!("Failed to write to dump file: {}", e))
        })?;
        writeln!(file, "Provider: {}", self.provider).map_err(|e| {
            LlmError::config_error(format!("Failed to write to dump file: {}", e))
        })?;

        // Write context metadata to file header
        if let Some(ctx) = context {
            if !ctx.metadata.is_empty() {
                writeln!(file, "\n=== REQUEST CONTEXT ===").map_err(|e| {
                    LlmError::config_error(format!("Failed to write to dump file: {}", e))
                })?;
                for (key, value) in &ctx.metadata {
                    writeln!(file, "{}: {}", key, value).map_err(|e| {
                        LlmError::config_error(format!("Failed to write to dump file: {}", e))
                    })?;
                }
            }
        }

        writeln!(file, "\n=== PROMPT ({} chars) ===\n", prompt.len()).map_err(|e| {
            LlmError::config_error(format!("Failed to write to dump file: {}", e))
        })?;
        writeln!(file, "{}", prompt).map_err(|e| {
            LlmError::config_error(format!("Failed to write to dump file: {}", e))
        })?;
        writeln!(file, "\n=== RESPONSE ({} chars) ===\n", response.len()).map_err(|e| {
            LlmError::config_error(format!("Failed to write to dump file: {}", e))
        })?;
        writeln!(file, "{}", response).map_err(|e| {
            LlmError::config_error(format!("Failed to write to dump file: {}", e))
        })?;

        info!(filepath = %filepath.display(), "Dumped LLM response to file");

        Ok(())
    }
}

#[async_trait]
impl DynamicLlmClient for RigDynamicLlmClient {
    async fn prompt(&self, prompt: &str, context: Option<RequestContext>) -> Result<String, LlmError> {
        // Log prompt preview at DEBUG level
        let context_info = if let Some(ref ctx) = context {
            ctx.metadata.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            "none".to_string()
        };

        debug!(
            prompt_preview = %Self::truncate_text(prompt, LLM_RESPONSE_PREVIEW_LENGTH),
            prompt_len = prompt.len(),
            model = %self.model,
            context = %context_info,
            "Sending prompt to LLM"
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
            error!(error = %e, "LLM request failed");
            LlmError::request_failed(format!("LLM request failed: {}", e))
        })?;

        // Log response preview at DEBUG level (ALWAYS)
        debug!(
            response_preview = %Self::truncate_text(&response, LLM_RESPONSE_PREVIEW_LENGTH),
            response_len = response.len(),
            model = %self.model,
            context = %context_info,
            "Received LLM response"
        );

        // Optionally dump full response to file (pass context)
        if self.debug_config.dump_raw_response_enabled {
            if let Err(e) = self.dump_response_to_file(prompt, &response, context.as_ref()) {
                warn!(error = %e, "Failed to dump LLM response to file");
            }
        }

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
            DebugConfig::default(),
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
