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
    pub nkey_file: String,
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
            nkey_file: "./nkeys/prompt-helper.nk".to_string(),
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
    /// Priority (lowest to highest): .env file → config.toml → Environment variables
    pub fn load() -> Result<Self> {
        // Load .env file from current directory (lowest priority)
        dotenvy::dotenv().ok();

        let config: Self = Figment::new()
            // Layer 1: config.toml file
            .merge(Toml::file("prompt-helper/config.toml"))

            // Layer 2: Environment variables (highest priority)
            .merge(Env::prefixed("PROMPT_HELPER_"))

            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        Ok(config)
    }
}

// Tests removed - LlmConfig tests now in shared-types-llm crate
