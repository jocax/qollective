//! Shared LLM client types and configuration for qollective_taletrail_content_generator
//!
//! This crate provides a unified abstraction for working with multiple LLM providers
//! (Shimmy, LM Studio, OpenAI, Anthropic, Google) with runtime configuration support and
//! three-tier configuration priority.
//!
//! # Architecture
//!
//! The crate implements a three-tier configuration priority system:
//! 1. **Runtime Tenant Config** (Premium) - Credentials and config provided at request time
//! 2. **Static TOML Tenant Config** - Server-managed tenant overrides (no credentials)
//! 3. **Default TOML Config** - Fallback configuration
//!
//! # Features
//!
//! - **Multi-Provider Support**: Shimmy, LM Studio, OpenAI, Anthropic, Google
//! - **Runtime Credentials**: Premium tenants can provide their own API keys at request time
//! - **Language Model Mapping**: Map language codes to specific models
//! - **System Prompt Styles**: Native, Prepend, ChatML, None
//! - **Mockable Traits**: Full mockall support for testing
//! - **Audit Logging**: Automatic logging when runtime config is used
//!
//! # Example
//!
//! ```no_run
//! use shared_types_llm::*;
//! use std::collections::HashMap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration from TOML
//! let config = LlmConfig::load("config.toml")?;
//! let provider = DefaultDynamicLlmClientProvider::new(config);
//!
//! // Basic usage with default config
//! let params = LlmParameters {
//!     language_code: "en".to_string(),
//!     model_name: None,
//!     system_prompt_style: SystemPromptStyle::Native,
//!     tenant_id: None,
//!     tenant_config: None,
//! };
//!
//! let client = provider.get_dynamic_llm_client(&params).await?;
//! let formatted = client.format_prompt(
//!     "You are a helpful assistant.",
//!     "Tell me a joke."
//! );
//! let response = client.prompt(&formatted).await?;
//!
//! // Premium tenant with runtime credentials
//! let tenant_config = TenantLlmConfig {
//!     tenant_id: "premium-corp".to_string(),
//!     provider_type: ProviderType::OpenAI,
//!     api_key: Some("sk-...".to_string()),
//!     google_credentials: None,
//!     base_url: Some("https://api.openai.com/v1".to_string()),
//!     model_overrides: HashMap::from([
//!         ("en".to_string(), "gpt-4".to_string()),
//!     ]),
//!     max_tokens: Some(8192),
//!     temperature: Some(0.8),
//!     timeout_secs: Some(120),
//!     system_prompt_style: Some(SystemPromptStyle::Native),
//! };
//!
//! let params = LlmParameters {
//!     language_code: "en".to_string(),
//!     tenant_config: Some(tenant_config),
//!     ..Default::default()
//! };
//!
//! let client = provider.get_dynamic_llm_client(&params).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! ## TOML Example
//!
//! ```toml
//! [llm]
//! type = "shimmy"
//! url = "http://127.0.0.1:11435/v1"
//! default_model = "qwen2.5-32b-instruct-q4_k_m"
//! use_default_model_fallback = true
//! max_tokens = 4096
//! temperature = 0.7
//! timeout_secs = 60
//! system_prompt_style = "native"
//!
//! [llm.models]
//! en = "qwen2.5-32b-instruct-q4_k_m"
//! es = "llama-3.3-70b-instruct-q4_k_m"
//! fr = "magistral-small-2509-q8_0"
//!
//! [llm.tenants.enterprise-corp]
//! type = "shimmy"
//! url = "http://enterprise-llm.local:11435/v1"
//! default_model = "custom-enterprise-model"
//!
//! [llm.tenants.enterprise-corp.models]
//! en = "custom-en-model"
//! ```
//!
//! # Security
//!
//! - **Runtime credentials are NEVER stored in TOML files**
//! - API keys are only passed in request-time `TenantLlmConfig`
//! - Usage of runtime config is automatically logged for audit purposes
//! - No credential validation - assumed valid if provided
//!
//! # Testing
//!
//! Enable the `mocking` feature to use mockall for testing:
//!
//! ```toml
//! [dev-dependencies]
//! shared-types-llm = { path = "../shared-types-llm", features = ["mocking"] }
//! ```
//!
//! Example test:
//!
//! ```
//! use shared_types_llm::*;
//!
//! #[tokio::test]
//! async fn test_with_mock() {
//!     let mut mock_provider = MockDynamicLlmClientProvider::new();
//!
//!     mock_provider
//!         .expect_get_dynamic_llm_client()
//!         .returning(|_params| {
//!             let mut mock_client = MockDynamicLlmClient::new();
//!             mock_client.expect_model_name().return_const("test-model".to_string());
//!             Ok(Box::new(mock_client))
//!         });
//!
//!     let params = LlmParameters::default();
//!     let client = mock_provider.get_dynamic_llm_client(&params).await.unwrap();
//!     assert_eq!(client.model_name(), "test-model");
//! }
//! ```

// Public modules
pub mod config;
pub mod constants;
pub mod error;
pub mod parameters;
pub mod rig_wrapper;
pub mod traits;

// Internal modules
mod client;
mod model_selector;
mod provider;

// Re-export commonly used types
pub use client::RigDynamicLlmClient;
pub use config::{LlmConfig, ProviderConfig, TenantStaticConfig};
pub use error::LlmError;
pub use model_selector::{merge_model_mappings, select_model_for_language};
pub use parameters::{GoogleCredentials, LlmParameters, ProviderType, SystemPromptStyle, TenantLlmConfig};
pub use provider::DefaultDynamicLlmClientProvider;
pub use traits::{DynamicLlmClient, DynamicLlmClientProvider};

// Re-export mockall mocks when mocking feature is enabled
#[cfg(any(test, feature = "mocking"))]
pub use traits::{MockDynamicLlmClient, MockDynamicLlmClientProvider};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_exports() {
        // Verify that all public types are accessible
        let _: Option<LlmConfig> = None;
        let _: Option<LlmError> = None;
        let _: Option<LlmParameters> = None;
        let _: Option<ProviderType> = None;
        let _: Option<SystemPromptStyle> = None;
        let _: Option<TenantLlmConfig> = None;
    }

    #[test]
    fn test_provider_type_roundtrip() {
        let types = vec![
            ProviderType::Shimmy,
            ProviderType::LmStudio,
            ProviderType::OpenAI,
            ProviderType::Anthropic,
        ];

        for provider_type in types {
            let str_repr = provider_type.as_str();
            let parsed: ProviderType = str_repr.parse().unwrap();
            assert_eq!(parsed, provider_type);
        }
    }

    #[test]
    fn test_system_prompt_style_roundtrip() {
        let styles = vec![
            SystemPromptStyle::Native,
            SystemPromptStyle::Prepend,
            SystemPromptStyle::ChatML,
            SystemPromptStyle::None,
        ];

        for style in styles {
            let str_repr = style.as_str();
            let parsed: SystemPromptStyle = str_repr.parse().unwrap();
            assert_eq!(parsed, style);
        }
    }
}
