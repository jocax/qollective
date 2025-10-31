//! Prompt Helper configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use shared_types_llm::LlmConfig as SharedLlmConfig;
use figment::{Figment, providers::{Env, Format, Toml}};
use crate::execution_logger::ExecutionConfig;

/// Prompt Helper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptHelperConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
    pub llm: SharedLlmConfig,
    pub prompt: PromptConfig,
    pub execution: ExecutionConfig,
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
        use tracing::debug;

        // Load .env file from current directory
        // dotenvy loads .env vars into environment but does NOT override existing system env vars
        // This ensures: system env vars (highest) > .env vars (middle) > TOML (lowest)
        match dotenvy::dotenv() {
            Ok(path) => debug!(
                path = ?path,
                "✅ Loaded .env file"
            ),
            Err(_) => debug!("ℹ️  No .env file found in current directory"),
        }

        let figment = Figment::new()
            // Layer 1: Base config from TOML file (lowest priority)
            .merge(Toml::file("config.toml"));

        debug!("📄 Layer 1: Loaded base config from config.toml");

        let figment = figment
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

                        // Special case for TYPE which maps to "type" (serde rename in ProviderConfig)
                        let field_name = if generic_key == "TYPE" {
                            "type"
                        } else {
                            &generic_key.to_lowercase()
                        };

                        // Transform to llm.{field}: GOOGLE_API_KEY -> llm.api_key, TYPE -> llm.type (provider fields are flattened)
                        format!("llm.{}", field_name).into()
                    })
            );

        debug!("🔧 Layer 2: Applied LLM_* environment variable overrides");

        let config: Self = figment
            // Layer 3: PROMPT_HELPER-specific environment variables (includes .env + system, system takes precedence)
            .merge(Env::prefixed("PROMPT_HELPER_"))
            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        debug!("🔧 Layer 3: Applied PROMPT_HELPER_* environment variable overrides");

        debug!(
            provider_type = ?config.llm.provider.provider_type,
            url = %config.llm.provider.url,
            default_model = %config.llm.provider.default_model,
            models_count = config.llm.provider.models.len(),
            "✅ Final merged configuration"
        );

        Ok(config)
    }
}

// Tests removed - LlmConfig tests now in shared-types-llm crate

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Global mutex to ensure tests run sequentially (avoid directory change conflicts)
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    /// Helper to clean up environment variables after test
    struct EnvGuard {
        vars: Vec<String>,
    }

    impl EnvGuard {
        fn new(vars: Vec<&str>) -> Self {
            Self {
                vars: vars.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for var in &self.vars {
                env::remove_var(var);
            }
        }
    }

    /// Helper to create a temporary directory with config.toml file
    fn create_temp_config_dir(content: &str) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        let mut file = fs::File::create(&config_path).expect("Failed to create config file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to config file");
        file.flush().expect("Failed to flush config file");
        temp_dir
    }

    #[test]
    fn test_default_toml_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        // Clean up any existing environment variables
        let _guard = EnvGuard::new(vec![
            "LLM_TYPE",
            "LLM_URL",
            "LLM_DEFAULT_MODEL",
            "LLM_GOOGLE_API_KEY",
            "PROMPT_HELPER_NATS_URL",
        ]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://localhost:4222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
api_key = "not-needed"
default_model = "default-model"

[llm.models]
en = "model-en-default"
de = "model-de-default"

[prompt]
supported_languages = ["english", "german"]
default_language = "english"

[prompt.models]
temperature = 0.7
max_tokens = 4096

[prompt.educational]
min_educational_score = 0.6
required_elements = ["learning_objective"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        // Change to temp directory to load config
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = PromptHelperConfig::load().expect("Failed to load config");

        // Verify default values
        assert_eq!(config.service.name, "test-service");
        assert_eq!(config.service.version, "1.0.0");
        assert_eq!(config.nats.url, "nats://localhost:4222");
        assert_eq!(config.llm.provider.provider_type.to_string(), "lmstudio");
        assert_eq!(config.llm.provider.models.get("en").unwrap(), "model-en-default");

        // Restore directory before temp_dir is dropped
        env::set_current_dir(original_dir).expect("Failed to restore dir");
        // _temp_dir is dropped here, cleaning up the directory
    }

    #[test]
    fn test_service_config_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["LLM_TYPE", "LLM_URL"]);

        let config_content = r#"
[service]
name = "custom-service"
version = "2.0.0"
description = "Custom Test Service"

[nats]
url = "nats://localhost:4222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
api_key = "not-needed"
default_model = "default-model"

[llm.models]
en = "model-en-default"

[prompt]
supported_languages = ["english"]
default_language = "english"

[prompt.models]
temperature = 0.7
max_tokens = 4096

[prompt.educational]
min_educational_score = 0.6
required_elements = ["learning_objective"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = PromptHelperConfig::load().expect("Failed to load config");

        // Verify service config is loaded correctly
        assert_eq!(config.service.name, "custom-service");
        assert_eq!(config.service.version, "2.0.0");
        assert_eq!(config.service.description, "Custom Test Service");

        env::set_current_dir(original_dir).expect("Failed to restore dir");
    }

    #[test]
    fn test_nats_config_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["PROMPT_HELPER_NATS_URL"]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://custom:5222"
subject = "custom.subject"
queue_group = "custom-group"

[nats.auth]
nkey_file = "./custom.nk"

[nats.tls]
ca_cert = "./custom-ca.pem"

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
api_key = "not-needed"
default_model = "default-model"

[llm.models]
en = "model-en"

[prompt]
supported_languages = ["english"]
default_language = "english"

[prompt.models]
temperature = 0.8
max_tokens = 2048

[prompt.educational]
min_educational_score = 0.7
required_elements = ["test_element"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = PromptHelperConfig::load().expect("Failed to load config");

        // Verify NATS config is loaded correctly
        assert_eq!(config.nats.url, "nats://custom:5222");
        assert_eq!(config.nats.subject, "custom.subject");
        assert_eq!(config.nats.queue_group, "custom-group");

        env::set_current_dir(original_dir).expect("Failed to restore dir");
    }

    #[test]
    fn test_prompt_config_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec![]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://localhost:4222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
api_key = "not-needed"
default_model = "default-model"

[llm.models]
en = "model-en"

[prompt]
supported_languages = ["english", "german", "french"]
default_language = "german"

[prompt.models]
temperature = 0.9
max_tokens = 8192

[prompt.educational]
min_educational_score = 0.8
required_elements = ["element1", "element2"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = PromptHelperConfig::load().expect("Failed to load config");

        // Verify prompt config is loaded correctly
        assert_eq!(config.prompt.supported_languages, vec!["english", "german", "french"]);
        assert_eq!(config.prompt.default_language, "german");
        assert_eq!(config.prompt.models.temperature, 0.9);
        assert_eq!(config.prompt.models.max_tokens, 8192);
        assert_eq!(config.prompt.educational.min_educational_score, 0.8);

        env::set_current_dir(original_dir).expect("Failed to restore dir");
    }

    #[test]
    fn test_env_var_override_type() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["LLM_TYPE"]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://localhost:4222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
api_key = "not-needed"
default_model = "default-model"

[llm.models]
en = "model-en"

[prompt]
supported_languages = ["english"]
default_language = "english"

[prompt.models]
temperature = 0.7
max_tokens = 4096

[prompt.educational]
min_educational_score = 0.6
required_elements = ["learning_objective"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_TYPE", "google");

        let config = PromptHelperConfig::load().expect("Failed to load config");

        // Verify TYPE env var overrides TOML
        assert_eq!(config.llm.provider.provider_type.to_string(), "google");

        env::set_current_dir(original_dir).expect("Failed to restore dir");
    }

    #[test]
    fn test_env_var_override_url() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["LLM_URL"]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://localhost:4222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
api_key = "not-needed"
default_model = "default-model"

[llm.models]
en = "model-en"

[prompt]
supported_languages = ["english"]
default_language = "english"

[prompt.models]
temperature = 0.7
max_tokens = 4096

[prompt.educational]
min_educational_score = 0.6
required_elements = ["learning_objective"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_URL", "https://custom.url/v1");

        let config = PromptHelperConfig::load().expect("Failed to load config");

        // Verify URL env var overrides TOML
        assert_eq!(config.llm.provider.url, "https://custom.url/v1");

        env::set_current_dir(original_dir).expect("Failed to restore dir");
    }

    #[test]
    fn test_env_var_provider_specific_api_key() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["LLM_GOOGLE_API_KEY"]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://localhost:4222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[llm]
type = "google"
url = "https://generativelanguage.googleapis.com/v1"
default_model = "gemini-pro"

[llm.models]
en = "model-en"

[prompt]
supported_languages = ["english"]
default_language = "english"

[prompt.models]
temperature = 0.7
max_tokens = 4096

[prompt.educational]
min_educational_score = 0.6
required_elements = ["learning_objective"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_GOOGLE_API_KEY", "test-api-key-12345");

        let config = PromptHelperConfig::load().expect("Failed to load config");

        // Verify GOOGLE_API_KEY env var sets api_key
        assert_eq!(config.llm.provider.api_key, Some("test-api-key-12345".to_string()));

        env::set_current_dir(original_dir).expect("Failed to restore dir");
    }

    #[test]
    fn test_env_var_models_override() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["LLM_MODELS_EN", "LLM_MODELS_DE"]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://localhost:4222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[llm]
type = "google"
url = "https://generativelanguage.googleapis.com/v1"
default_model = "gemini-pro"

[llm.models]
en = "old-model-en"
de = "old-model-de"

[prompt]
supported_languages = ["english", "german"]
default_language = "english"

[prompt.models]
temperature = 0.7
max_tokens = 4096

[prompt.educational]
min_educational_score = 0.6
required_elements = ["learning_objective"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_MODELS_EN", "gemini-1.5-flash");
        env::set_var("LLM_MODELS_DE", "gemini-1.5-pro");

        let config = PromptHelperConfig::load().expect("Failed to load config");

        // Verify MODELS_* env vars override TOML
        assert_eq!(config.llm.provider.models.get("en"), Some(&"gemini-1.5-flash".to_string()));
        assert_eq!(config.llm.provider.models.get("de"), Some(&"gemini-1.5-pro".to_string()));

        env::set_current_dir(original_dir).expect("Failed to restore dir");
    }
}
