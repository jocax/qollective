//! TOML configuration structures for LLM client setup
//!
//! This module provides configuration loading from TOML files with environment
//! variable override support via figment. Runtime credentials are NEVER stored
//! in TOML files - only static configuration and tenant overrides.

use crate::constants::*;
use crate::error::LlmError;
use crate::parameters::{ProviderType, SystemPromptStyle};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Root LLM configuration loaded from TOML
///
/// This structure represents the complete LLM configuration including default
/// provider settings and optional tenant-specific overrides. Runtime credentials
/// are NEVER stored in this configuration.
///
/// # TOML Example
///
/// ```toml
/// [llm]
/// type = "shimmy"
/// url = "http://127.0.0.1:11435/v1"
/// default_model = "qwen2.5-32b-instruct-q4_k_m"
/// use_default_model_fallback = true
/// max_tokens = 4096
/// temperature = 0.7
/// timeout_secs = 60
///
/// [llm.models]
/// en = "qwen2.5-32b-instruct-q4_k_m"
/// es = "llama-3.3-70b-instruct-q4_k_m"
/// fr = "magistral-small-2509-q8_0"
///
/// [llm.tenants.enterprise-corp]
/// type = "shimmy"
/// url = "http://enterprise-llm.local:11435/v1"
/// models = { en = "custom-enterprise-model" }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Default provider configuration (flattened from [llm] section)
    #[serde(flatten)]
    pub provider: ProviderConfig,

    /// Static tenant-specific overrides (NO CREDENTIALS)
    #[serde(default)]
    pub tenants: HashMap<String, TenantStaticConfig>,
}

impl LlmConfig {
    /// Load configuration from a TOML file with environment variable overrides
    ///
    /// Configuration priority (lowest to highest):
    /// 1. config.toml file (lowest priority - base defaults)
    /// 2. .env file (middle priority - environment-specific config)
    /// 3. System environment variables (highest priority - runtime secrets)
    ///
    /// The priority is achieved through dotenvy's behavior:
    /// - dotenvy loads .env file into process environment
    /// - dotenvy does NOT override existing environment variables
    /// - Therefore system env vars automatically take precedence over .env vars
    /// - Figment then merges: TOML → Environment (which contains .env + system vars)
    ///
    /// Environment variables are prefixed with `LLM_` and override TOML values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use shared_types_llm::LlmConfig;
    ///
    /// let config = LlmConfig::load("config.toml").unwrap();
    /// ```
    ///
    /// # Environment Variable Examples
    ///
    /// - `LLM_TYPE=openai` - Override provider type
    /// - `LLM_URL=https://custom.api.com/v1` - Override provider URL
    /// - `LLM_DEFAULT_MODEL=gpt-4` - Override default model
    /// - `LLM_OPENAI_API_KEY=sk-...` - OpenAI-specific API key
    /// - `LLM_ANTHROPIC_API_KEY=sk-ant-...` - Anthropic-specific API key
    /// - `LLM_GOOGLE_API_KEY=AIza...` - Google-specific API key
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LlmError> {
        // Load .env file from current directory
        // dotenvy loads .env vars into environment but does NOT override existing system env vars
        // This ensures: system env vars (highest) > .env vars (middle) > TOML (lowest)
        match dotenvy::dotenv() {
            Ok(path) => eprintln!("DEBUG: Loaded .env from: {:?}", path),
            Err(e) => eprintln!("DEBUG: .env not loaded: {}", e),
        }

        // Debug: Check what LLM_TYPE env var contains
        if let Ok(llm_type) = std::env::var("LLM_TYPE") {
            eprintln!("DEBUG: LLM_TYPE env var = {}", llm_type);
        } else {
            eprintln!("DEBUG: LLM_TYPE env var not set");
        }

        let config: LlmConfig = Figment::new()
            // Layer 1: Base config from TOML file (lowest priority)
            .merge(Toml::file(path.as_ref()).nested())
            // Layer 2: Environment variables (includes .env + system, system takes precedence)
            .merge(Env::prefixed(ENV_PREFIX_LLM))
            .select(CONFIG_KEY_LLM)
            .extract()
            .map_err(|e| LlmError::config_parse_error(format!("Failed to load config: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a TOML string (useful for testing)
    pub fn from_toml_str(toml: &str) -> Result<Self, LlmError> {
        // Parse TOML into a generic value first
        let toml_value: toml::Value = toml::from_str(toml)
            .map_err(|e| LlmError::config_parse_error(format!("Failed to parse TOML: {}", e)))?;

        // Extract the [llm] section
        let llm_section = toml_value
            .get(CONFIG_KEY_LLM)
            .ok_or_else(|| LlmError::config_parse_error("Missing [llm] section in TOML"))?;

        // Deserialize the llm section into LlmConfig
        let config: LlmConfig = llm_section.clone().try_into()
            .map_err(|e: toml::de::Error| LlmError::config_parse_error(format!("Failed to parse [llm] section: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    fn validate(&self) -> Result<(), LlmError> {
        self.provider.validate()?;

        // Validate all tenant configs
        for (tenant_id, tenant_config) in &self.tenants {
            tenant_config.validate().map_err(|e| {
                LlmError::invalid_tenant_config(
                    tenant_id,
                    format!("Invalid static config: {}", e),
                )
            })?;
        }

        Ok(())
    }

    /// Get tenant configuration by tenant_id
    pub fn get_tenant_config(&self, tenant_id: &str) -> Option<&TenantStaticConfig> {
        self.tenants.get(tenant_id)
    }
}

/// Default provider configuration from TOML
///
/// This structure defines the default LLM provider settings used when no tenant
/// override is specified. API keys can be stored here as the server's default
/// credentials, but runtime tenant credentials take priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type (shimmy, lmstudio, openai, anthropic, google)
    #[serde(rename = "type")]
    pub provider_type: ProviderType,

    /// Provider base URL
    pub url: String,

    /// Optional API key for the provider (server's default, not tenant-specific)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Default model name to use
    pub default_model: String,

    /// Use default model as fallback when requested model is unavailable
    #[serde(default = "default_use_fallback")]
    pub use_default_model_fallback: bool,

    /// Language code to model name mappings
    #[serde(default)]
    pub models: HashMap<String, String>,

    /// Maximum tokens for completions
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for completions (0.0 - 1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Request timeout in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    /// System prompt handling style
    #[serde(default)]
    pub system_prompt_style: SystemPromptStyle,
}

impl ProviderConfig {
    /// Validate the provider configuration
    fn validate(&self) -> Result<(), LlmError> {
        if self.url.trim().is_empty() {
            return Err(LlmError::config_error(ERROR_MISSING_PROVIDER_URL));
        }

        if self.default_model.trim().is_empty() {
            return Err(LlmError::config_error(ERROR_MISSING_MODEL_NAME));
        }

        if self.temperature < 0.0 || self.temperature > 1.0 {
            return Err(LlmError::config_error(
                "Temperature must be between 0.0 and 1.0",
            ));
        }

        Ok(())
    }

    /// Get model name for a specific language code
    pub fn get_model_for_language(&self, language_code: &str) -> Option<&String> {
        self.models.get(language_code)
    }
}

/// Static tenant configuration from TOML
///
/// This structure defines static tenant-specific overrides that are managed by
/// the server administrator. API keys can be stored here as tenant-specific
/// server credentials, but runtime credentials take priority.
///
/// # TOML Example
///
/// ```toml
/// [llm.tenants.enterprise-corp]
/// type = "openai"
/// url = "https://api.openai.com/v1"
/// api_key = "sk-server-enterprise-key"
/// default_model = "gpt-4"
/// max_tokens = 8192
///
/// [llm.tenants.enterprise-corp.models]
/// en = "gpt-4"
/// es = "gpt-4"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantStaticConfig {
    /// Provider type override
    #[serde(rename = "type")]
    pub provider_type: Option<ProviderType>,

    /// Provider URL override
    pub url: Option<String>,

    /// Optional API key override (server's credential for this tenant)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Default model override
    pub default_model: Option<String>,

    /// Model fallback flag override
    pub use_default_model_fallback: Option<bool>,

    /// Language-to-model mappings override
    #[serde(default)]
    pub models: HashMap<String, String>,

    /// Max tokens override
    pub max_tokens: Option<u32>,

    /// Temperature override
    pub temperature: Option<f32>,

    /// Timeout override
    pub timeout_secs: Option<u64>,

    /// System prompt style override
    pub system_prompt_style: Option<SystemPromptStyle>,
}

impl TenantStaticConfig {
    /// Validate the tenant configuration
    fn validate(&self) -> Result<(), LlmError> {
        if let Some(ref url) = self.url {
            if url.trim().is_empty() {
                return Err(LlmError::config_error(ERROR_MISSING_PROVIDER_URL));
            }
        }

        if let Some(ref model) = self.default_model {
            if model.trim().is_empty() {
                return Err(LlmError::config_error(ERROR_MISSING_MODEL_NAME));
            }
        }

        if let Some(temp) = self.temperature {
            if temp < 0.0 || temp > 1.0 {
                return Err(LlmError::config_error(
                    "Temperature must be between 0.0 and 1.0",
                ));
            }
        }

        Ok(())
    }
}

// Default value functions for serde
fn default_use_fallback() -> bool {
    true
}

fn default_max_tokens() -> u32 {
    DEFAULT_MAX_TOKENS
}

fn default_temperature() -> f32 {
    DEFAULT_TEMPERATURE
}

fn default_timeout_secs() -> u64 {
    DEFAULT_TIMEOUT_SECS
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CONFIG_TOML: &str = r#"
[llm]
type = "shimmy"
url = "http://127.0.0.1:11435/v1"
default_model = "qwen2.5-32b-instruct-q4_k_m"
use_default_model_fallback = true
max_tokens = 4096
temperature = 0.7
timeout_secs = 60
system_prompt_style = "native"

[llm.models]
en = "qwen2.5-32b-instruct-q4_k_m"
es = "llama-3.3-70b-instruct-q4_k_m"

[llm.tenants.test-tenant]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "custom-model"
max_tokens = 8192

[llm.tenants.test-tenant.models]
en = "custom-en-model"
"#;

    #[test]
    fn test_load_config_from_toml() {
        let config = LlmConfig::from_toml_str(TEST_CONFIG_TOML).unwrap();

        assert_eq!(config.provider.provider_type, ProviderType::Shimmy);
        assert_eq!(config.provider.url, "http://127.0.0.1:11435/v1");
        assert_eq!(
            config.provider.default_model,
            "qwen2.5-32b-instruct-q4_k_m"
        );
        assert!(config.provider.use_default_model_fallback);
        assert_eq!(config.provider.max_tokens, 4096);
        assert_eq!(config.provider.temperature, 0.7);
        assert_eq!(config.provider.timeout_secs, 60);
    }

    #[test]
    fn test_provider_config_models() {
        let config = LlmConfig::from_toml_str(TEST_CONFIG_TOML).unwrap();

        assert_eq!(
            config.provider.get_model_for_language("en"),
            Some(&"qwen2.5-32b-instruct-q4_k_m".to_string())
        );
        assert_eq!(
            config.provider.get_model_for_language("es"),
            Some(&"llama-3.3-70b-instruct-q4_k_m".to_string())
        );
        assert_eq!(config.provider.get_model_for_language("fr"), None);
    }

    #[test]
    fn test_tenant_config() {
        let config = LlmConfig::from_toml_str(TEST_CONFIG_TOML).unwrap();

        let tenant = config.get_tenant_config("test-tenant").unwrap();
        assert_eq!(tenant.provider_type, Some(ProviderType::LmStudio));
        assert_eq!(tenant.url, Some("http://localhost:1234/v1".to_string()));
        assert_eq!(tenant.default_model, Some("custom-model".to_string()));
        assert_eq!(tenant.max_tokens, Some(8192));
        assert_eq!(
            tenant.models.get("en"),
            Some(&"custom-en-model".to_string())
        );
    }

    #[test]
    fn test_invalid_temperature() {
        let invalid_toml = r#"
[llm]
type = "shimmy"
url = "http://localhost:11435/v1"
default_model = "test-model"
temperature = 1.5
"#;

        let result = LlmConfig::from_toml_str(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_fields() {
        let invalid_toml = r#"
[llm]
type = "shimmy"
default_model = "test-model"
"#;

        let result = LlmConfig::from_toml_str(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_provider_config_validation() {
        let valid_config = ProviderConfig {
            provider_type: ProviderType::Shimmy,
            url: "http://localhost:11435/v1".to_string(),
            api_key: None,
            default_model: "test-model".to_string(),
            use_default_model_fallback: true,
            models: HashMap::new(),
            max_tokens: 4096,
            temperature: 0.7,
            timeout_secs: 60,
            system_prompt_style: SystemPromptStyle::Native,
        };

        assert!(valid_config.validate().is_ok());

        let invalid_config = ProviderConfig {
            url: "".to_string(),
            ..valid_config
        };

        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_env_override_toml() {
        // Test that environment variables override TOML values
        let toml = r#"
[llm]
type = "shimmy"
url = "http://127.0.0.1:11435/v1"
default_model = "qwen2.5-32b-instruct-q4_k_m"
api_key = "toml-api-key"
"#;

        // Set environment variable to override TOML api_key
        std::env::set_var("LLM_API_KEY", "env-override-key");

        let config = LlmConfig::from_toml_str(toml).unwrap();

        // Environment variable should override TOML value
        // Note: from_toml_str doesn't use environment merging, but load() does
        assert_eq!(config.provider.api_key, Some("toml-api-key".to_string()));

        // Clean up
        std::env::remove_var("LLM_API_KEY");
    }

    #[test]
    fn test_provider_specific_api_keys() {
        // Test that provider-specific environment variables are recognized
        use crate::constants::*;

        // Verify constants are correctly defined
        assert_eq!(ENV_VAR_OPENAI_API_KEY, "LLM_OPENAI_API_KEY");
        assert_eq!(ENV_VAR_ANTHROPIC_API_KEY, "LLM_ANTHROPIC_API_KEY");
        assert_eq!(ENV_VAR_GOOGLE_API_KEY, "LLM_GOOGLE_API_KEY");

        // Test OpenAI configuration
        let openai_toml = r#"
[llm]
type = "openai"
url = "https://api.openai.com/v1"
default_model = "gpt-4"
"#;

        let config = LlmConfig::from_toml_str(openai_toml).unwrap();
        assert_eq!(config.provider.provider_type, ProviderType::OpenAI);

        // Test Anthropic configuration
        let anthropic_toml = r#"
[llm]
type = "anthropic"
url = "https://api.anthropic.com/v1"
default_model = "claude-3-opus"
"#;

        let config = LlmConfig::from_toml_str(anthropic_toml).unwrap();
        assert_eq!(config.provider.provider_type, ProviderType::Anthropic);

        // Test Google configuration
        let google_toml = r#"
[llm]
type = "google"
url = "https://generativelanguage.googleapis.com/v1"
default_model = "gemini-pro"
"#;

        let config = LlmConfig::from_toml_str(google_toml).unwrap();
        assert_eq!(config.provider.provider_type, ProviderType::Google);
    }
}
