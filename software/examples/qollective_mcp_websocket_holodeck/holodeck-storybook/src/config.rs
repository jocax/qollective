// ABOUTME: Configuration loading system for holodeck-storybook service
// ABOUTME: Implements the multi-step config resolution: service config.toml → .env → error

use serde::{Deserialize, Serialize};
use shared_types::llm::{LlmProviderType, LlmConfig, LlmError};
use std::path::Path;
use std::env;

/// Service-specific configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub service: ServiceInfo,
    pub llm: LlmServiceConfig,
    pub storybook: StorybookConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmServiceConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub endpoint_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorybookConfig {
    pub rest_server_port: Option<u16>,
    pub websocket_port: Option<u16>,
    pub max_concurrent_sessions: u32,
    pub session_timeout_minutes: u32,
    pub content_cache_size_mb: u32,
    pub real_time_update_interval_ms: u64,
    pub story_persistence_enabled: bool,
    pub content_validation_timeout_seconds: u32,
}

impl ServiceConfig {
    /// Load configuration with the specified hierarchy:
    /// 1. Load service config.toml
    /// 2. If service has API key, use it
    /// 3. If not, try .env file
    /// 4. If no key found, error (no fallback)
    pub fn load_from_file<P: AsRef<Path>>(config_path: P, env_path: Option<P>) -> Result<Self, LlmError> {
        // Step 1: Load TOML config file
        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| LlmError::Configuration(format!("Failed to read config file {}: {}",
                config_path.as_ref().display(), e)))?;

        let mut config: ServiceConfig = toml::from_str(&config_content)
            .map_err(|e| LlmError::Configuration(format!("Failed to parse config TOML: {}", e)))?;

        // Step 2: Load .env file if provided
        if let Some(env_path) = env_path {
            dotenv::from_path(env_path)
                .map_err(|e| LlmError::Configuration(format!("Failed to load .env file: {}", e)))?;
        }

        // Step 3: Resolve API key using hierarchy
        if config.llm.api_key.is_none() {
            config.llm.api_key = Self::resolve_api_key(&config.llm.provider)?;
        }

        // Step 4: Set endpoint URL from environment if not in config
        if config.llm.endpoint_url.is_none() && config.llm.provider == "ollama" {
            config.llm.endpoint_url = env::var("OLLAMA_ENDPOINT").ok();
        }

        Ok(config)
    }

    /// Resolve API key for the configured provider
    /// Returns Some(key) if found, None if not needed (Ollama), or Error if required but missing
    fn resolve_api_key(provider: &str) -> Result<Option<String>, LlmError> {
        let env_var_name = match provider {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            "perplexity" => "PERPLEXITY_API_KEY",
            "ollama" => return Ok(None), // Ollama doesn't need API key
            _ => return Err(LlmError::UnsupportedProvider(format!("Unknown provider: {}", provider))),
        };

        match env::var(env_var_name) {
            Ok(key) => Ok(Some(key)),
            Err(_) => {
                if provider == "ollama" {
                    Ok(None)
                } else {
                    Err(LlmError::ApiKeyMissing(env_var_name.to_string()))
                }
            }
        }
    }

    /// Convert service config to shared LlmConfig
    pub fn to_llm_config(&self) -> Result<LlmConfig, LlmError> {
        let provider_type = match self.llm.provider.as_str() {
            "openai" => LlmProviderType::OpenAI,
            "ollama" => LlmProviderType::Ollama,
            "anthropic" => LlmProviderType::Anthropic,
            "perplexity" => LlmProviderType::Perplexity,
            _ => return Err(LlmError::UnsupportedProvider(self.llm.provider.clone())),
        };

        Ok(LlmConfig {
            provider: provider_type,
            model: self.llm.model.clone(),
            endpoint_url: self.llm.endpoint_url.clone(),
            temperature: Some(self.llm.temperature),
            max_tokens: Some(self.llm.max_tokens),
            timeout_seconds: Some(self.llm.timeout_seconds),
            fallback: None, // No fallback - explicit error if configured provider fails
        })
    }
}

impl Default for StorybookConfig {
    fn default() -> Self {
        Self {
            rest_server_port: None, // Will use constants::HOLODECK_STORYBOOK_PORT
            websocket_port: None,   // Will use same port for WebSocket upgrade
            max_concurrent_sessions: 1000,
            session_timeout_minutes: 480, // 8 hours
            content_cache_size_mb: 500,
            real_time_update_interval_ms: 100,
            story_persistence_enabled: true,
            content_validation_timeout_seconds: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_parsing() {
        let config_toml = r#"
[service]
name = "test-storybook"
version = "0.1.0"

[llm]
provider = "ollama"
model = "gemma2:2b"
temperature = 0.5
max_tokens = 2000
timeout_seconds = 30

[storybook]
max_concurrent_sessions = 500
session_timeout_minutes = 240
content_cache_size_mb = 250
real_time_update_interval_ms = 50
story_persistence_enabled = true
content_validation_timeout_seconds = 5
        "#;

        let config: ServiceConfig = toml::from_str(config_toml).unwrap();
        assert_eq!(config.service.name, "test-storybook");
        assert_eq!(config.llm.provider, "ollama");
        assert_eq!(config.llm.model, "gemma2:2b");
        assert_eq!(config.llm.temperature, 0.5);
        assert_eq!(config.storybook.max_concurrent_sessions, 500);
        assert_eq!(config.storybook.session_timeout_minutes, 240);
    }

    #[test]
    fn test_api_key_resolution_ollama() {
        // Ollama should not need API key
        let result = ServiceConfig::resolve_api_key("ollama").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_api_key_resolution_missing() {
        // Should error for providers that need API keys when not set
        env::remove_var("TEST_OPENAI_API_KEY");
        let result = ServiceConfig::resolve_api_key("openai");
        assert!(result.is_err());
    }

    #[test]
    fn test_llm_config_conversion() {
        let service_config = ServiceConfig {
            service: ServiceInfo {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
            },
            llm: LlmServiceConfig {
                provider: "ollama".to_string(),
                model: "gemma2:2b".to_string(),
                api_key: None,
                endpoint_url: None,
                temperature: 0.5,
                max_tokens: 2000,
                timeout_seconds: 30,
            },
            storybook: StorybookConfig::default(),
        };

        let llm_config = service_config.to_llm_config().unwrap();
        assert_eq!(llm_config.provider, LlmProviderType::Ollama);
        assert_eq!(llm_config.model, "gemma2:2b");
        assert_eq!(llm_config.temperature, Some(0.5));
        assert_eq!(llm_config.max_tokens, Some(2000));
    }

    #[test]
    fn test_storybook_config_defaults() {
        let config = StorybookConfig::default();
        assert_eq!(config.max_concurrent_sessions, 1000);
        assert_eq!(config.session_timeout_minutes, 480);
        assert_eq!(config.content_cache_size_mb, 500);
        assert!(config.story_persistence_enabled);
    }
}