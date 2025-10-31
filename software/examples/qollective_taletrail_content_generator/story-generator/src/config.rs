//! Configuration module for Story Generator MCP Server

use anyhow::Result;
use figment::{Figment, providers::{Env, Format, Toml}};
use serde::{Deserialize, Serialize};
use shared_types_llm::LlmConfig as SharedLlmConfig;

use crate::execution_logger::ExecutionConfig;

/// Story Generator MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryGeneratorConfig {
    /// Server configuration
    pub service: ServiceConfig,

    /// NATS connection configuration
    pub nats: NatsConfig,

    /// Generation settings
    pub generation: GenerationConfig,

    /// LLM configuration
    pub llm: SharedLlmConfig,

    /// Execution logging configuration
    pub execution: ExecutionConfig,
}

/// Server-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,

    /// Service version
    pub version: String,

    /// Service description
    pub description: String,
}

/// NATS connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    /// NATS server URL
    pub url: String,

    /// NATS subject for story generation
    pub subject: String,

    /// NATS queue group for load balancing
    pub queue_group: String,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// TLS configuration
    pub tls: TlsConfig,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// NKey file path for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nkey_file: Option<String>,
    /// NKey seed value for authentication (alternative to nkey_file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nkey_seed: Option<String>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// CA certificate path
    pub ca_cert: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,
}

/// Content generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Generation timeout in seconds
    pub timeout_secs: u64,

    /// Minimum batch size
    pub batch_size_min: usize,

    /// Maximum batch size
    pub batch_size_max: usize,

    /// Target words per node
    pub target_words_per_node: usize,

    /// Delay between LLM requests in milliseconds (for rate limiting)
    pub request_delay_ms: u64,

    /// Delay for discovery responses in milliseconds
    pub discovery_delay_ms: u64,
}

// LlmConfig removed - now using shared-types-llm::LlmConfig

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "story-generator".to_string(),
            version: "0.1.0".to_string(),
            description: "TaleTrail Story Generator MCP Server".to_string(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            nkey_file: Some("./nkeys/story-generator.nk".to_string()),
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

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:5222".to_string(),
            subject: "mcp.story.generate".to_string(),
            queue_group: "story-generator".to_string(),
            auth: AuthConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 60,
            batch_size_min: 4,
            batch_size_max: 6,
            target_words_per_node: 400,
            request_delay_ms: 2000,
            discovery_delay_ms: 100,
        }
    }
}

// LlmConfig::default() and StoryGeneratorConfig::default() removed - now loading from TOML

impl StoryGeneratorConfig {
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
            .merge(Toml::file("story-generator/config.toml"));

        debug!("📄 Layer 1: Loaded base config from story-generator/config.toml");

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
            // Layer 3: STORY_GENERATOR-specific environment variables (includes .env + system, system takes precedence)
            .merge(Env::prefixed("STORY_GENERATOR_"))
            .extract()?;

        debug!("🔧 Layer 3: Applied STORY_GENERATOR_* environment variable overrides");

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

    /// Helper to create a temporary directory with story-generator/config.toml file
    fn create_temp_config_dir(content: &str) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create story-generator subdirectory
        let story_gen_dir = temp_dir.path().join("story-generator");
        fs::create_dir(&story_gen_dir).expect("Failed to create story-generator dir");

        let config_path = story_gen_dir.join("config.toml");
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
            "STORY_GENERATOR_NATS_URL",
        ]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://localhost:5222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[generation]
timeout_secs = 60
batch_size_min = 4
batch_size_max = 6
target_words_per_node = 400

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "default-model"

[llm.models]
en = "model-en-default"
de = "model-de-default"
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        // Change to temp directory to load config
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = StoryGeneratorConfig::load().expect("Failed to load config");

        // Verify default values
        assert_eq!(config.service.name, "test-service");
        assert_eq!(config.service.version, "1.0.0");
        assert_eq!(config.nats.url, "nats://localhost:5222");
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
url = "nats://localhost:5222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[generation]
timeout_secs = 90
batch_size_min = 5
batch_size_max = 10
target_words_per_node = 500

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "default-model"

[llm.models]
en = "model-en-default"
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = StoryGeneratorConfig::load().expect("Failed to load config");

        // Verify service config is loaded correctly
        assert_eq!(config.service.name, "custom-service");
        assert_eq!(config.service.version, "2.0.0");
        assert_eq!(config.service.description, "Custom Test Service");

        env::set_current_dir(original_dir).expect("Failed to restore dir");
    }

    #[test]
    fn test_nats_config_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["STORY_GENERATOR_NATS_URL"]);

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

[generation]
timeout_secs = 60
batch_size_min = 4
batch_size_max = 6
target_words_per_node = 400

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "default-model"

[llm.models]
en = "model-en"
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = StoryGeneratorConfig::load().expect("Failed to load config");

        // Verify NATS config is loaded correctly
        assert_eq!(config.nats.url, "nats://custom:5222");
        assert_eq!(config.nats.subject, "custom.subject");
        assert_eq!(config.nats.queue_group, "custom-group");

        env::set_current_dir(original_dir).expect("Failed to restore dir");
    }

    #[test]
    fn test_generation_config_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec![]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://localhost:5222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[generation]
timeout_secs = 120
batch_size_min = 8
batch_size_max = 12
target_words_per_node = 600

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "default-model"

[llm.models]
en = "model-en"
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = StoryGeneratorConfig::load().expect("Failed to load config");

        // Verify generation config is loaded correctly
        assert_eq!(config.generation.timeout_secs, 120);
        assert_eq!(config.generation.batch_size_min, 8);
        assert_eq!(config.generation.batch_size_max, 12);
        assert_eq!(config.generation.target_words_per_node, 600);

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
url = "nats://localhost:5222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[generation]
timeout_secs = 60
batch_size_min = 4
batch_size_max = 6
target_words_per_node = 400

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "default-model"

[llm.models]
en = "model-en"
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_TYPE", "google");

        let config = StoryGeneratorConfig::load().expect("Failed to load config");

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
url = "nats://localhost:5222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[generation]
timeout_secs = 60
batch_size_min = 4
batch_size_max = 6
target_words_per_node = 400

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "default-model"

[llm.models]
en = "model-en"
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_URL", "https://custom.url/v1");

        let config = StoryGeneratorConfig::load().expect("Failed to load config");

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
url = "nats://localhost:5222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[generation]
timeout_secs = 60
batch_size_min = 4
batch_size_max = 6
target_words_per_node = 400

[llm]
type = "google"
url = "https://generativelanguage.googleapis.com/v1"
default_model = "gemini-pro"

[llm.models]
en = "model-en"
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_GOOGLE_API_KEY", "test-api-key-12345");

        let config = StoryGeneratorConfig::load().expect("Failed to load config");

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
url = "nats://localhost:5222"
subject = "test.subject"
queue_group = "test-group"

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[generation]
timeout_secs = 60
batch_size_min = 4
batch_size_max = 6
target_words_per_node = 400

[llm]
type = "google"
url = "https://generativelanguage.googleapis.com/v1"
default_model = "gemini-pro"

[llm.models]
en = "old-model-en"
de = "old-model-de"
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        let original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_MODELS_EN", "gemini-1.5-flash");
        env::set_var("LLM_MODELS_DE", "gemini-1.5-pro");

        let config = StoryGeneratorConfig::load().expect("Failed to load config");

        // Verify MODELS_* env vars override TOML
        assert_eq!(config.llm.provider.models.get("en"), Some(&"gemini-1.5-flash".to_string()));
        assert_eq!(config.llm.provider.models.get("de"), Some(&"gemini-1.5-pro".to_string()));

        env::set_current_dir(original_dir).expect("Failed to restore dir");
    }
}
