//! Prompt Helper configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use figment::{Figment, providers::{Env, Format, Serialized, Toml}};

/// Prompt Helper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptHelperConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
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
    pub client_cert: String,
    pub client_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    pub supported_languages: Vec<String>,
    pub default_language: String,
    pub models: ModelConfig,
    pub educational: EducationalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub default_model: String,
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
            client_cert: "./certs/client-cert.pem".to_string(),
            client_key: "./certs/client-key.pem".to_string(),
        }
    }
}

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
            default_model: "llama-3.2-3b-instruct".to_string(),
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

impl Default for PromptHelperConfig {
    fn default() -> Self {
        Self {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            prompt: PromptConfig::default(),
        }
    }
}

impl PromptHelperConfig {
    /// Load configuration using Figment merge strategy
    /// Priority (lowest to highest): Defaults → config.toml → Environment variables
    pub fn load() -> Result<Self> {
        let config: Self = Figment::new()
            // Layer 1: Hardcoded defaults (fallback)
            .merge(Serialized::defaults(Self::default()))

            // Layer 2: config.toml file (overrides defaults)
            .merge(Toml::file("prompt-helper/config.toml"))

            // Layer 3: Environment variables (highest priority)
            .merge(Env::prefixed("PROMPT_HELPER_"))

            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        Ok(config)
    }
}
