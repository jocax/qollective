//! Prompt Helper configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use figment::{Figment, providers::{Env, Format, Serialized, Toml}};
use std::collections::HashMap;

/// Prompt Helper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptHelperConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
    pub llm: LlmConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub default_model: String,
    #[serde(default)]
    pub models: HashMap<String, String>,
}

impl LlmConfig {
    /// Get the appropriate model for a given language
    pub fn get_model_for_language(&self, language: &Language) -> String {
        let lang_code = match language {
            Language::De => "de",
            Language::En => "en",
        };

        self.models
            .get(lang_code)
            .cloned()
            .unwrap_or_else(|| self.default_model.clone())
    }
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

impl Default for LlmConfig {
    fn default() -> Self {
        let mut models = HashMap::new();
        models.insert("de".to_string(), "leolm-70b-chat".to_string());
        models.insert("en".to_string(), "meta-llama-3-8b-instruct".to_string());

        Self {
            base_url: "http://127.0.0.1:1234/v1".to_string(),
            default_model: "meta-llama-3-8b-instruct".to_string(),
            models,
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

impl Default for PromptHelperConfig {
    fn default() -> Self {
        Self {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            llm: LlmConfig::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_for_language_returns_german_model() {
        let config = LlmConfig::default();
        let model = config.get_model_for_language(&Language::De);
        assert_eq!(model, "leolm-70b-chat");
    }

    #[test]
    fn test_get_model_for_language_returns_english_model() {
        let config = LlmConfig::default();
        let model = config.get_model_for_language(&Language::En);
        assert_eq!(model, "meta-llama-3-8b-instruct");
    }

    #[test]
    fn test_get_model_for_language_falls_back_to_default() {
        let mut config = LlmConfig::default();
        config.models.clear(); // Remove all language mappings
        let model_de = config.get_model_for_language(&Language::De);
        let model_en = config.get_model_for_language(&Language::En);
        assert_eq!(model_de, config.default_model);
        assert_eq!(model_en, config.default_model);
    }

    #[test]
    fn test_llm_config_loads_custom_models() {
        let mut models = HashMap::new();
        models.insert("de".to_string(), "test-german-model".to_string());
        models.insert("en".to_string(), "test-english-model".to_string());

        let config = LlmConfig {
            base_url: "http://test".to_string(),
            default_model: "test-default".to_string(),
            models,
        };

        assert_eq!(config.get_model_for_language(&Language::De), "test-german-model");
        assert_eq!(config.get_model_for_language(&Language::En), "test-english-model");
    }

    #[test]
    fn test_llm_config_default_has_language_models() {
        let config = LlmConfig::default();
        assert!(config.models.contains_key("de"), "Should have German model mapping");
        assert!(config.models.contains_key("en"), "Should have English model mapping");
        assert_eq!(config.models.len(), 2, "Should have exactly 2 language mappings");
    }
}
