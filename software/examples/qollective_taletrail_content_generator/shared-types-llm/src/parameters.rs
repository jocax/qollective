//! Request-time configuration structures for LLM operations
//!
//! This module defines structures used to configure LLM requests at runtime,
//! including tenant-specific overrides with runtime credentials support.

use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider type for LLM services
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// Shimmy local LLM server (docker-shimmy)
    Shimmy,
    /// LM Studio local LLM server
    LmStudio,
    /// OpenAI API
    OpenAI,
    /// Anthropic API
    Anthropic,
    /// Google Vertex AI / Gemini API
    Google,
}

impl ProviderType {
    /// Get the string representation of the provider type
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Shimmy => PROVIDER_TYPE_SHIMMY,
            Self::LmStudio => PROVIDER_TYPE_LMSTUDIO,
            Self::OpenAI => PROVIDER_TYPE_OPENAI,
            Self::Anthropic => PROVIDER_TYPE_ANTHROPIC,
            Self::Google => PROVIDER_TYPE_GOOGLE,
        }
    }

    /// Get the default base URL for this provider type
    pub fn default_url(&self) -> &'static str {
        match self {
            Self::Shimmy => SHIMMY_DEFAULT_URL,
            Self::LmStudio => LMSTUDIO_DEFAULT_URL,
            Self::OpenAI => OPENAI_DEFAULT_URL,
            Self::Anthropic => ANTHROPIC_DEFAULT_URL,
            Self::Google => GOOGLE_DEFAULT_URL,
        }
    }

    /// Check if this provider requires an API key
    pub fn requires_api_key(&self) -> bool {
        matches!(self, Self::OpenAI | Self::Anthropic | Self::Google)
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ProviderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            PROVIDER_TYPE_SHIMMY => Ok(Self::Shimmy),
            PROVIDER_TYPE_LMSTUDIO => Ok(Self::LmStudio),
            PROVIDER_TYPE_OPENAI => Ok(Self::OpenAI),
            PROVIDER_TYPE_ANTHROPIC => Ok(Self::Anthropic),
            PROVIDER_TYPE_GOOGLE => Ok(Self::Google),
            _ => Err(format!("{}: {}", ERROR_INVALID_PROVIDER_TYPE, s)),
        }
    }
}

/// System prompt handling style for different LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemPromptStyle {
    /// Model natively supports system prompts (OpenAI, Anthropic)
    Native,
    /// Prepend system prompt to user message
    Prepend,
    /// Use ChatML format with special tokens
    ChatML,
    /// No system prompt support
    None,
}

impl SystemPromptStyle {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Native => PROMPT_STYLE_NATIVE,
            Self::Prepend => PROMPT_STYLE_PREPEND,
            Self::ChatML => PROMPT_STYLE_CHATML,
            Self::None => PROMPT_STYLE_NONE,
        }
    }
}

impl Default for SystemPromptStyle {
    fn default() -> Self {
        Self::Native
    }
}

impl std::fmt::Display for SystemPromptStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SystemPromptStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            PROMPT_STYLE_NATIVE => Ok(Self::Native),
            PROMPT_STYLE_PREPEND => Ok(Self::Prepend),
            PROMPT_STYLE_CHATML => Ok(Self::ChatML),
            PROMPT_STYLE_NONE => Ok(Self::None),
            _ => Err(format!("Invalid system prompt style: {}", s)),
        }
    }
}

/// Google-specific credentials for Vertex AI / Gemini API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCredentials {
    /// Google Cloud project ID
    pub project_id: String,
    /// Google API key
    pub api_key: String,
}

/// Runtime tenant configuration with credentials support (Premium Feature)
///
/// This structure allows tenants to provide their own LLM credentials and configuration
/// at request time, enabling multi-tenant systems where each tenant can use their own
/// LLM providers without storing credentials in server configuration files.
///
/// # Security Notes
///
/// - Credentials are NEVER stored in TOML configuration files
/// - Credentials are only passed in runtime requests
/// - Usage is logged for audit purposes
///
/// # Example
///
/// ```
/// use shared_types_llm::{TenantLlmConfig, ProviderType, SystemPromptStyle};
/// use std::collections::HashMap;
///
/// let config = TenantLlmConfig {
///     tenant_id: "premium-corp".to_string(),
///     provider_type: ProviderType::OpenAI,
///     api_key: Some("sk-...".to_string()),
///     google_credentials: None,
///     base_url: Some("https://api.openai.com/v1".to_string()),
///     model_overrides: HashMap::from([
///         ("en".to_string(), "gpt-4".to_string()),
///         ("es".to_string(), "gpt-4".to_string()),
///     ]),
///     max_tokens: Some(8192),
///     temperature: Some(0.8),
///     timeout_secs: Some(120),
///     system_prompt_style: Some(SystemPromptStyle::Native),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantLlmConfig {
    /// Tenant identifier
    pub tenant_id: String,

    /// Provider type (Shimmy, LmStudio, OpenAI, Anthropic, Google)
    pub provider_type: ProviderType,

    /// API key for the provider (required for OpenAI, Anthropic, Google)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Google-specific credentials (optional, for full Google Cloud integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_credentials: Option<GoogleCredentials>,

    /// Base URL for the provider (overrides default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// Language-specific model overrides (language_code -> model_name)
    #[serde(default)]
    pub model_overrides: HashMap<String, String>,

    /// Maximum tokens for completions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Temperature for completions (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Request timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,

    /// System prompt handling style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt_style: Option<SystemPromptStyle>,
}

impl Default for TenantLlmConfig {
    fn default() -> Self {
        Self {
            tenant_id: String::new(),
            provider_type: ProviderType::Shimmy,
            api_key: None,
            google_credentials: None,
            base_url: None,
            model_overrides: HashMap::new(),
            max_tokens: None,
            temperature: None,
            timeout_secs: None,
            system_prompt_style: None,
        }
    }
}

/// Request-time LLM parameters
///
/// This structure defines the parameters for a single LLM request, including
/// language code, optional model override, and optional tenant-specific configuration.
///
/// # Configuration Priority
///
/// When resolving configuration, the following priority order is used:
/// 1. Runtime `tenant_config` (highest priority)
/// 2. Static TOML tenant configuration
/// 3. Default TOML configuration (lowest priority)
///
/// # Example
///
/// ```
/// use shared_types_llm::{LlmParameters, SystemPromptStyle};
///
/// // Basic usage
/// let params = LlmParameters {
///     language_code: "en".to_string(),
///     model_name: None,
///     system_prompt_style: SystemPromptStyle::Native,
///     tenant_id: None,
///     tenant_config: None,
/// };
///
/// // With model override
/// let params = LlmParameters {
///     language_code: "es".to_string(),
///     model_name: Some("gpt-4".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmParameters {
    /// Language code for content generation (e.g., "en", "es", "fr")
    pub language_code: String,

    /// Optional explicit model name override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,

    /// System prompt handling style
    #[serde(default)]
    pub system_prompt_style: SystemPromptStyle,

    /// Optional tenant identifier for static TOML config lookup
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    /// Optional runtime tenant configuration (Premium Feature)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_config: Option<TenantLlmConfig>,
}

impl Default for LlmParameters {
    fn default() -> Self {
        Self {
            language_code: String::new(),
            model_name: None,
            system_prompt_style: SystemPromptStyle::default(),
            tenant_id: None,
            tenant_config: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_as_str() {
        assert_eq!(ProviderType::Shimmy.as_str(), PROVIDER_TYPE_SHIMMY);
        assert_eq!(ProviderType::LmStudio.as_str(), PROVIDER_TYPE_LMSTUDIO);
        assert_eq!(ProviderType::OpenAI.as_str(), PROVIDER_TYPE_OPENAI);
        assert_eq!(ProviderType::Anthropic.as_str(), PROVIDER_TYPE_ANTHROPIC);
        assert_eq!(ProviderType::Google.as_str(), PROVIDER_TYPE_GOOGLE);
    }

    #[test]
    fn test_provider_type_default_url() {
        assert_eq!(ProviderType::Shimmy.default_url(), SHIMMY_DEFAULT_URL);
        assert_eq!(ProviderType::LmStudio.default_url(), LMSTUDIO_DEFAULT_URL);
        assert_eq!(ProviderType::OpenAI.default_url(), OPENAI_DEFAULT_URL);
        assert_eq!(
            ProviderType::Anthropic.default_url(),
            ANTHROPIC_DEFAULT_URL
        );
        assert_eq!(ProviderType::Google.default_url(), GOOGLE_DEFAULT_URL);
    }

    #[test]
    fn test_provider_type_requires_api_key() {
        assert!(!ProviderType::Shimmy.requires_api_key());
        assert!(!ProviderType::LmStudio.requires_api_key());
        assert!(ProviderType::OpenAI.requires_api_key());
        assert!(ProviderType::Anthropic.requires_api_key());
        assert!(ProviderType::Google.requires_api_key());
    }

    #[test]
    fn test_provider_type_from_str() {
        assert_eq!(
            "shimmy".parse::<ProviderType>().unwrap(),
            ProviderType::Shimmy
        );
        assert_eq!(
            "lmstudio".parse::<ProviderType>().unwrap(),
            ProviderType::LmStudio
        );
        assert_eq!(
            "openai".parse::<ProviderType>().unwrap(),
            ProviderType::OpenAI
        );
        assert_eq!(
            "anthropic".parse::<ProviderType>().unwrap(),
            ProviderType::Anthropic
        );
        assert_eq!(
            "google".parse::<ProviderType>().unwrap(),
            ProviderType::Google
        );
        assert!("invalid".parse::<ProviderType>().is_err());
    }

    #[test]
    fn test_system_prompt_style_as_str() {
        assert_eq!(SystemPromptStyle::Native.as_str(), PROMPT_STYLE_NATIVE);
        assert_eq!(SystemPromptStyle::Prepend.as_str(), PROMPT_STYLE_PREPEND);
        assert_eq!(SystemPromptStyle::ChatML.as_str(), PROMPT_STYLE_CHATML);
        assert_eq!(SystemPromptStyle::None.as_str(), PROMPT_STYLE_NONE);
    }

    #[test]
    fn test_system_prompt_style_from_str() {
        assert_eq!(
            "native".parse::<SystemPromptStyle>().unwrap(),
            SystemPromptStyle::Native
        );
        assert_eq!(
            "prepend".parse::<SystemPromptStyle>().unwrap(),
            SystemPromptStyle::Prepend
        );
        assert_eq!(
            "chatml".parse::<SystemPromptStyle>().unwrap(),
            SystemPromptStyle::ChatML
        );
        assert_eq!(
            "none".parse::<SystemPromptStyle>().unwrap(),
            SystemPromptStyle::None
        );
        assert!("invalid".parse::<SystemPromptStyle>().is_err());
    }

    #[test]
    fn test_tenant_llm_config_serialization() {
        let config = TenantLlmConfig {
            tenant_id: "test-tenant".to_string(),
            provider_type: ProviderType::OpenAI,
            api_key: Some("sk-test".to_string()),
            google_credentials: None,
            base_url: Some("https://api.openai.com/v1".to_string()),
            model_overrides: HashMap::from([("en".to_string(), "gpt-4".to_string())]),
            max_tokens: Some(2048),
            temperature: Some(0.7),
            timeout_secs: Some(60),
            system_prompt_style: Some(SystemPromptStyle::Native),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TenantLlmConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tenant_id, "test-tenant");
        assert_eq!(deserialized.provider_type, ProviderType::OpenAI);
        assert_eq!(deserialized.api_key, Some("sk-test".to_string()));
    }

    #[test]
    fn test_google_credentials_serialization() {
        let creds = GoogleCredentials {
            project_id: "my-project-123".to_string(),
            api_key: "AIza-test-key".to_string(),
        };

        let json = serde_json::to_string(&creds).unwrap();
        let deserialized: GoogleCredentials = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.project_id, "my-project-123");
        assert_eq!(deserialized.api_key, "AIza-test-key");
    }

    #[test]
    fn test_tenant_llm_config_with_google_credentials() {
        let config = TenantLlmConfig {
            tenant_id: "google-tenant".to_string(),
            provider_type: ProviderType::Google,
            api_key: None,
            google_credentials: Some(GoogleCredentials {
                project_id: "my-project-123".to_string(),
                api_key: "AIza-test-key".to_string(),
            }),
            base_url: Some("https://generativelanguage.googleapis.com/v1".to_string()),
            model_overrides: HashMap::from([("en".to_string(), "gemini-pro".to_string())]),
            max_tokens: Some(8192),
            temperature: Some(0.8),
            timeout_secs: Some(120),
            system_prompt_style: Some(SystemPromptStyle::Native),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TenantLlmConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tenant_id, "google-tenant");
        assert_eq!(deserialized.provider_type, ProviderType::Google);
        assert!(deserialized.google_credentials.is_some());
        let creds = deserialized.google_credentials.unwrap();
        assert_eq!(creds.project_id, "my-project-123");
        assert_eq!(creds.api_key, "AIza-test-key");
    }

    #[test]
    fn test_tenant_llm_config_google_with_simple_api_key() {
        let config = TenantLlmConfig {
            tenant_id: "google-simple".to_string(),
            provider_type: ProviderType::Google,
            api_key: Some("AIza-simple-key".to_string()),
            google_credentials: None,
            base_url: Some("https://generativelanguage.googleapis.com/v1".to_string()),
            model_overrides: HashMap::from([("en".to_string(), "gemini-pro".to_string())]),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            timeout_secs: Some(60),
            system_prompt_style: Some(SystemPromptStyle::Native),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TenantLlmConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tenant_id, "google-simple");
        assert_eq!(deserialized.provider_type, ProviderType::Google);
        assert_eq!(deserialized.api_key, Some("AIza-simple-key".to_string()));
        assert!(deserialized.google_credentials.is_none());
    }

    #[test]
    fn test_llm_parameters_default() {
        let params = LlmParameters::default();
        assert_eq!(params.language_code, "");
        assert!(params.model_name.is_none());
        assert_eq!(params.system_prompt_style, SystemPromptStyle::Native);
        assert!(params.tenant_id.is_none());
        assert!(params.tenant_config.is_none());
    }

    #[test]
    fn test_llm_parameters_serialization() {
        let params = LlmParameters {
            language_code: "en".to_string(),
            model_name: Some("gpt-4".to_string()),
            system_prompt_style: SystemPromptStyle::ChatML,
            tenant_id: Some("tenant-123".to_string()),
            tenant_config: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: LlmParameters = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.language_code, "en");
        assert_eq!(deserialized.model_name, Some("gpt-4".to_string()));
        assert_eq!(deserialized.system_prompt_style, SystemPromptStyle::ChatML);
    }
}
