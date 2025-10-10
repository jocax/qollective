//! Test helper utilities for prompt-helper tests
//!
//! This module provides test configuration builders and utilities for
//! creating test instances of configs and services without using Default trait.

use crate::config::*;
use shared_types_llm::LlmConfig;

/// Create a test LlmConfig using TOML configuration
pub fn test_llm_config() -> LlmConfig {
    let toml = r#"
[llm]
type = "shimmy"
url = "http://localhost:11434/v1"
default_model = "test-model"
use_default_model_fallback = true
max_tokens = 4096
temperature = 0.7
timeout_secs = 60
system_prompt_style = "native"

[llm.models]
en = "test-model-en"
de = "test-model-de"
    "#;
    LlmConfig::from_toml_str(toml).expect("Failed to create test LLM config")
}

/// Create a test PromptHelperConfig for testing
pub fn test_prompt_helper_config() -> PromptHelperConfig {
    PromptHelperConfig {
        service: ServiceConfig::default(),
        nats: NatsConfig::default(),
        llm: test_llm_config(),
        prompt: PromptConfig::default(),
    }
}

/// Create a test ServiceConfig with custom values
pub fn test_service_config(name: &str, version: &str, description: &str) -> ServiceConfig {
    ServiceConfig {
        name: name.to_string(),
        version: version.to_string(),
        description: description.to_string(),
    }
}

/// Create a test NatsConfig with custom URL
pub fn test_nats_config(url: &str) -> NatsConfig {
    NatsConfig {
        url: url.to_string(),
        subject: "test.mcp.prompt.helper".to_string(),
        queue_group: "test-prompt-helper".to_string(),
        auth: AuthConfig::default(),
        tls: TlsConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_creation() {
        let config = test_llm_config();
        assert_eq!(config.provider.default_model, "test-model");
    }

    #[test]
    fn test_prompt_helper_config_creation() {
        let config = test_prompt_helper_config();
        assert_eq!(config.service.name, "prompt-helper");
        assert_eq!(config.llm.provider.default_model, "test-model");
    }
}
