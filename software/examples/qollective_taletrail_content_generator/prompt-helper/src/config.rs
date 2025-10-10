//! Prompt Helper configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use shared_types_llm::LlmConfig as SharedLlmConfig;
use figment::{Figment, providers::{Env, Format, Toml}};

/// Prompt Helper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptHelperConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
    pub llm: SharedLlmConfig,
    pub prompt: PromptConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    pub url: String,
    pub subject: String,
    pub queue_group: String,
    pub auth: AuthConfig,
    pub tls: TlsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nkey_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nkey_seed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub ca_cert: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,
}

// LlmConfig removed - now using shared-types-llm::LlmConfig

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    pub supported_languages: Vec<String>,
    pub default_language: String,
    pub models: ModelConfig,
    pub educational: EducationalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,  // Deprecated: use LlmConfig.models instead
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalConfig {
    pub min_educational_score: f32,
    pub required_elements: Vec<String>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "prompt-helper".to_string(),
            version: "0.1.0".to_string(),
            description: "TaleTrail Prompt Helper MCP Server".to_string(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            nkey_file: Some("./nkeys/prompt-helper.nk".to_string()),
            nkey_seed: None,
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            ca_cert: "./certs/ca.pem".to_string(),
            client_cert: None,
            client_key: None,
        }
    }
}

// LlmConfig::default() removed - now using shared-types-llm::LlmConfig

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:5222".to_string(),
            subject: "mcp.prompt.helper".to_string(),
            queue_group: "prompt-helper".to_string(),
            auth: AuthConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default_model: None,  // Deprecated: use LlmConfig.models instead
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

impl Default for EducationalConfig {
    fn default() -> Self {
        Self {
            min_educational_score: 0.6,
            required_elements: vec![
                "learning_objective".to_string(),
                "age_appropriate".to_string(),
                "safe_content".to_string(),
            ],
        }
    }
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            supported_languages: vec!["english".to_string(), "german".to_string()],
            default_language: "english".to_string(),
            models: ModelConfig::default(),
            educational: EducationalConfig::default(),
        }
    }
}

// PromptHelperConfig::default() removed - LlmConfig is now loaded from TOML

impl PromptHelperConfig {
    /// Load configuration using Figment merge strategy
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
    pub fn load() -> Result<Self> {
        // Load .env file from current directory
        // dotenvy loads .env vars into environment but does NOT override existing system env vars
        // This ensures: system env vars (highest) > .env vars (middle) > TOML (lowest)
        dotenvy::dotenv().ok();

        let config: Self = Figment::new()
            // Layer 1: Base config from TOML file (lowest priority)
            .merge(Toml::file("prompt-helper/config.toml"))
            // Layer 2: LLM-specific environment variables (map to nested llm.provider and llm.models sections)
            // Env::prefixed("LLM_") strips the "LLM_" prefix automatically
            // So LLM_TYPE becomes TYPE, LLM_MODELS_EN becomes MODELS_EN, LLM_GOOGLE_API_KEY becomes GOOGLE_API_KEY, etc.
            .merge(
                Env::prefixed("LLM_")
                    .map(|key| {
                        let key_str = key.as_str();

                        // Handle language-specific models: MODELS_EN -> llm.models.en, MODELS_DE -> llm.models.de
                        if let Some(lang) = key_str.strip_prefix("MODELS_") {
                            return format!("llm.models.{}", lang.to_lowercase()).into();
                        }

                        // Handle provider-level config
                        // Strip provider-specific prefixes (GOOGLE_, OPENAI_, ANTHROPIC_) to get generic field names
                        let generic_key = if let Some(suffix) = key_str.strip_prefix("GOOGLE_") {
                            suffix
                        } else if let Some(suffix) = key_str.strip_prefix("OPENAI_") {
                            suffix
                        } else if let Some(suffix) = key_str.strip_prefix("ANTHROPIC_") {
                            suffix
                        } else {
                            key_str
                        };
                        // Transform to llm.provider.{field}: GOOGLE_API_KEY -> llm.provider.api_key, TYPE -> llm.provider.type
                        format!("llm.provider.{}", generic_key.to_lowercase()).into()
                    })
            )
            // Layer 3: PROMPT_HELPER-specific environment variables (includes .env + system, system takes precedence)
            .merge(Env::prefixed("PROMPT_HELPER_"))
            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        Ok(config)
    }
}

// Tests removed - LlmConfig tests now in shared-types-llm crate
