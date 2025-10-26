//! Core traits for dynamic LLM client provisioning
//!
//! This module defines the trait-based abstraction for creating LLM clients
//! with runtime configuration support. Traits are mockable for testing via
//! the `mockall` crate when the `mocking` feature is enabled.

use crate::error::LlmError;
use crate::parameters::{LlmParameters, RequestContext};
use async_trait::async_trait;

/// Provider trait for creating dynamic LLM clients
///
/// This trait abstracts the creation of LLM clients based on runtime parameters,
/// enabling three-tier configuration priority:
/// 1. Runtime tenant config (from request)
/// 2. Static TOML tenant config
/// 3. Default TOML config
///
/// # Mockability
///
/// This trait is automatically mockable when the `mocking` feature is enabled
/// or when compiling tests, enabling comprehensive unit testing without real
/// LLM provider dependencies.
///
/// # Example
///
/// ```no_run
/// use shared_types_llm::{DynamicLlmClientProvider, LlmParameters};
///
/// # async fn example(provider: &dyn DynamicLlmClientProvider) -> Result<(), Box<dyn std::error::Error>> {
/// let params = LlmParameters {
///     language_code: "en".to_string(),
///     ..Default::default()
/// };
///
/// let client = provider.get_dynamic_llm_client(&params).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
pub trait DynamicLlmClientProvider: Send + Sync {
    /// Create a dynamic LLM client based on request parameters
    ///
    /// This method resolves configuration using the three-tier priority system
    /// and returns a client that implements the `DynamicLlmClient` trait.
    ///
    /// # Arguments
    ///
    /// * `params` - Request parameters including language code, tenant info, and optional overrides
    ///
    /// # Returns
    ///
    /// A boxed dynamic client or an error if configuration is invalid or provider is unreachable
    ///
    /// # Errors
    ///
    /// - `LlmError::MissingCredentials` - Required API key is missing
    /// - `LlmError::InvalidTenantConfig` - Tenant configuration is invalid
    /// - `LlmError::ConfigError` - General configuration error
    async fn get_dynamic_llm_client(
        &self,
        params: &LlmParameters,
    ) -> Result<Box<dyn DynamicLlmClient>, LlmError>;
}

/// Dynamic LLM client trait with prompt completion capabilities
///
/// This trait represents a configured LLM client that can execute prompts
/// and return completion results. It wraps rig-core's functionality with
/// our configuration and error handling.
///
/// # Mockability
///
/// This trait is automatically mockable when the `mocking` feature is enabled
/// or when compiling tests.
///
/// # Example
///
/// ```no_run
/// use shared_types_llm::DynamicLlmClient;
///
/// # async fn example(client: &dyn DynamicLlmClient) -> Result<(), Box<dyn std::error::Error>> {
/// let prompt = "Tell me a joke about programming.";
/// let response = client.prompt(prompt, None).await?;
/// println!("LLM response: {}", response);
/// # Ok(())
/// # }
/// ```
#[async_trait]
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
pub trait DynamicLlmClient: Send + Sync {
    /// Execute a prompt and get completion result
    ///
    /// This method sends a prompt to the configured LLM and returns the
    /// completion response. System prompts should be prepended to the prompt
    /// string before calling this method.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt text to send to the LLM
    /// * `context` - Optional request context for debug dumps (node_id, batch_id, etc.)
    ///
    /// # Returns
    ///
    /// The completion response as a string
    ///
    /// # Errors
    ///
    /// - `LlmError::RequestFailed` - Failed to execute prompt
    /// - `LlmError::ProviderUnreachable` - Cannot reach LLM provider
    /// - `LlmError::RigError` - Error from underlying rig-core library
    async fn prompt(&self, prompt: &str, context: Option<RequestContext>) -> Result<String, LlmError>;

    /// Format prompts according to the system prompt style
    ///
    /// This method takes separate system and user prompts and combines them
    /// according to the configured system prompt style (Native, Prepend, ChatML, None).
    ///
    /// # Arguments
    ///
    /// * `system_prompt` - System prompt to guide LLM behavior
    /// * `user_prompt` - User's input prompt
    ///
    /// # Returns
    ///
    /// Formatted prompt string ready to send to the LLM
    fn format_prompt(&self, system_prompt: &str, user_prompt: &str) -> String;

    /// Get the model name being used by this client
    fn model_name(&self) -> &str;

    /// Get the provider type string
    fn provider_type(&self) -> &str;

    /// Get the base URL being used
    fn base_url(&self) -> &str;

    /// Get the maximum tokens limit
    fn max_tokens(&self) -> u32;

    /// Get the temperature setting
    fn temperature(&self) -> f32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_dynamic_llm_client_provider() {
        let mut mock_provider = MockDynamicLlmClientProvider::new();

        mock_provider
            .expect_get_dynamic_llm_client()
            .returning(|_params| {
                let mut mock_client = MockDynamicLlmClient::new();
                mock_client
                    .expect_model_name()
                    .return_const("test-model".to_string());
                mock_client
                    .expect_provider_type()
                    .return_const("shimmy".to_string());
                mock_client
                    .expect_base_url()
                    .return_const("http://localhost:11435/v1".to_string());
                mock_client.expect_max_tokens().return_const(4096u32);
                mock_client.expect_temperature().return_const(0.7f32);
                mock_client
                    .expect_prompt()
                    .returning(|_prompt, _context| Box::pin(async { Ok("Mock response".to_string()) }));

                Box::pin(async move { Ok(Box::new(mock_client) as Box<dyn DynamicLlmClient>) })
            });

        let params = LlmParameters {
            language_code: "en".to_string(),
            ..Default::default()
        };

        let client = mock_provider.get_dynamic_llm_client(&params).await.unwrap();
        assert_eq!(client.model_name(), "test-model");
        assert_eq!(client.provider_type(), "shimmy");
        assert_eq!(client.max_tokens(), 4096);
        assert_eq!(client.temperature(), 0.7);
    }

    #[test]
    fn test_mock_dynamic_llm_client() {
        let mut mock_client = MockDynamicLlmClient::new();

        mock_client
            .expect_model_name()
            .return_const("test-model".to_string());
        mock_client
            .expect_provider_type()
            .return_const("openai".to_string());
        mock_client
            .expect_base_url()
            .return_const("https://api.openai.com/v1".to_string());
        mock_client.expect_max_tokens().return_const(8192u32);
        mock_client.expect_temperature().return_const(0.8f32);
        mock_client
            .expect_format_prompt()
            .return_const("formatted prompt".to_string());

        assert_eq!(mock_client.model_name(), "test-model");
        assert_eq!(mock_client.provider_type(), "openai");
        assert_eq!(mock_client.base_url(), "https://api.openai.com/v1");
        assert_eq!(mock_client.max_tokens(), 8192);
        assert_eq!(mock_client.temperature(), 0.8);
        assert_eq!(mock_client.format_prompt("sys", "user"), "formatted prompt");
    }
}
