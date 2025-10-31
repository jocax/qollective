//! Orchestrator configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use shared_types_llm::LlmConfig as SharedLlmConfig;
use figment::{Figment, providers::{Env, Format, Toml}};
use crate::execution_logger::ExecutionConfig;

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
    pub llm: SharedLlmConfig,
    pub pipeline: PipelineConfig,
    pub batch: BatchConfig,
    pub dag: DagConfig,
    pub negotiation: NegotiationConfig,
    pub retry: RetryConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub generation_timeout_secs: u64,
    pub validation_timeout_secs: u64,
    pub retry_max_attempts: u32,
    pub retry_base_delay_secs: u64,
    pub retry_max_delay_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub size_min: usize,
    pub size_max: usize,
    pub concurrent_batches: usize,
    pub concurrent_batches_max: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagConfig {
    pub default_node_count: usize,
    pub convergence_pattern: String,
    pub convergence_point_ratio: f32,
    pub max_depth: usize,
    pub branching_factor: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationConfig {
    pub max_rounds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "orchestrator".to_string(),
            version: "0.1.0".to_string(),
            description: "TaleTrail Orchestrator".to_string(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            nkey_file: Some("./nkeys/orchestrator.nk".to_string()),
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
            subject: "mcp.orchestrator.request".to_string(),
            queue_group: "orchestrator".to_string(),
            auth: AuthConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            generation_timeout_secs: 60,
            validation_timeout_secs: 10,
            retry_max_attempts: 3,
            retry_base_delay_secs: 1,
            retry_max_delay_secs: 30,
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            size_min: 4,
            size_max: 6,
            concurrent_batches: 3,
            concurrent_batches_max: 5,
        }
    }
}

impl Default for DagConfig {
    fn default() -> Self {
        Self {
            default_node_count: 16,
            convergence_pattern: "SingleConvergence".to_string(),
            convergence_point_ratio: 0.5,
            max_depth: 10,
            branching_factor: 2,
        }
    }
}

impl DagConfig {
    /// Convert orchestrator DagConfig to shared-types DagStructureConfig
    ///
    /// This method performs pattern parsing and ratio conversion according to DAG structure rules:
    /// - Parses convergence_pattern string to ConvergencePattern enum
    /// - Converts ratio to Option<f64> based on pattern requirements
    /// - PureBranching and ParallelPaths patterns get None for convergence_point_ratio
    /// - Other patterns get Some(ratio) for convergence_point_ratio
    pub fn to_dag_structure_config(&self) -> DagStructureConfig {
        use shared_types::ConvergencePattern;

        // Parse string to enum, with fallback to SingleConvergence
        let pattern = match self.convergence_pattern.as_str() {
            "SingleConvergence" => ConvergencePattern::SingleConvergence,
            "MultipleConvergence" => ConvergencePattern::MultipleConvergence,
            "EndOnly" => ConvergencePattern::EndOnly,
            "PureBranching" => ConvergencePattern::PureBranching,
            "ParallelPaths" => ConvergencePattern::ParallelPaths,
            unknown => {
                tracing::warn!(
                    pattern = unknown,
                    "Unknown convergence pattern, defaulting to SingleConvergence"
                );
                ConvergencePattern::SingleConvergence
            }
        };

        // Convert ratio to Option<f64>
        // PureBranching and ParallelPaths should have None
        let ratio = match pattern {
            ConvergencePattern::PureBranching | ConvergencePattern::ParallelPaths => None,
            _ => Some(self.convergence_point_ratio as f64),
        };

        DagStructureConfig {
            node_count: self.default_node_count as i64,
            convergence_pattern: pattern,
            convergence_point_ratio: ratio,
            max_depth: self.max_depth as i64,
            branching_factor: self.branching_factor as i64,
        }
    }
}

impl Default for NegotiationConfig {
    fn default() -> Self {
        Self {
            max_rounds: 3,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

impl OrchestratorConfig {
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
            .merge(
                Env::prefixed("LLM_")
                    .map(|key| {
                        let key_str = key.as_str();

                        // Handle language-specific models: MODELS_EN -> llm.models.en
                        if let Some(lang) = key_str.strip_prefix("MODELS_") {
                            return format!("llm.models.{}", lang.to_lowercase()).into();
                        }

                        // Handle provider-level config
                        let generic_key = if let Some(suffix) = key_str.strip_prefix("GOOGLE_") {
                            suffix
                        } else if let Some(suffix) = key_str.strip_prefix("OPENAI_") {
                            suffix
                        } else if let Some(suffix) = key_str.strip_prefix("ANTHROPIC_") {
                            suffix
                        } else {
                            key_str
                        };

                        let field_name = if generic_key == "TYPE" {
                            "type"
                        } else {
                            &generic_key.to_lowercase()
                        };

                        format!("llm.{}", field_name).into()
                    })
            );

        debug!("🔧 Layer 2: Applied LLM_* environment variable overrides");

        let config: Self = figment
            // Layer 3: ORCHESTRATOR-specific environment variables (includes .env + system, system takes precedence)
            .merge(Env::prefixed("ORCHESTRATOR_").split("__"))
            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        debug!("🔧 Layer 3: Applied ORCHESTRATOR_* environment variable overrides");

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
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tempfile::TempDir;
    use lazy_static::lazy_static;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    lazy_static! {
        static ref ORIGINAL_DIR: PathBuf = env::current_dir()
            .expect("Failed to get current directory at module initialization");
    }

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

    fn create_temp_config_dir(content: &str) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        // Create config.toml directly in temp_dir (not in orchestrator subdir)
        // because OrchestratorConfig::load() looks for config.toml in current directory
        let config_path = temp_dir.path().join("config.toml");
        let mut file = fs::File::create(&config_path).expect("Failed to create config file");
        file.write_all(content.as_bytes()).expect("Failed to write to config file");
        file.flush().expect("Failed to flush config file");
        temp_dir
    }

    fn restore_original_dir() {
        let _ = env::set_current_dir("/");
        env::set_current_dir(ORIGINAL_DIR.as_path()).expect("Failed to restore dir");
    }

    #[test]
    fn test_default_toml_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["LLM_TYPE", "ORCHESTRATOR_NATS_URL"]);

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

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "default-model"

[llm.models]
en = "model-en-default"

[pipeline]
generation_timeout_secs = 60
validation_timeout_secs = 10
retry_max_attempts = 3
retry_base_delay_secs = 1
retry_max_delay_secs = 30

[batch]
size_min = 4
size_max = 6
concurrent_batches = 3
concurrent_batches_max = 5

[dag]
default_node_count = 16
convergence_pattern = "SingleConvergence"
convergence_point_ratio = 0.5
max_depth = 10
branching_factor = 2

[negotiation]
max_rounds = 3

[retry]
max_attempts = 3
initial_delay_ms = 100
max_delay_ms = 5000
backoff_multiplier = 2.0
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = OrchestratorConfig::load().expect("Failed to load config");

        assert_eq!(config.service.name, "test-service");
        assert_eq!(config.nats.url, "nats://localhost:5222");
        assert_eq!(config.llm.provider.provider_type.to_string(), "lmstudio");
        assert_eq!(config.pipeline.generation_timeout_secs, 60);

        restore_original_dir();
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

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "default-model"

[llm.models]
en = "model-en"

[pipeline]
generation_timeout_secs = 60
validation_timeout_secs = 10
retry_max_attempts = 3
retry_base_delay_secs = 1
retry_max_delay_secs = 30

[batch]
size_min = 4
size_max = 6
concurrent_batches = 3
concurrent_batches_max = 5

[dag]
default_node_count = 16
convergence_pattern = "SingleConvergence"
convergence_point_ratio = 0.5
max_depth = 10
branching_factor = 2

[negotiation]
max_rounds = 3

[retry]
max_attempts = 3
initial_delay_ms = 100
max_delay_ms = 5000
backoff_multiplier = 2.0
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_TYPE", "google");

        let config = OrchestratorConfig::load().expect("Failed to load config");

        assert_eq!(config.llm.provider.provider_type.to_string(), "google");

        restore_original_dir();
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

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "default-model"

[llm.models]
en = "model-en"

[pipeline]
generation_timeout_secs = 60
validation_timeout_secs = 10
retry_max_attempts = 3
retry_base_delay_secs = 1
retry_max_delay_secs = 30

[batch]
size_min = 4
size_max = 6
concurrent_batches = 3
concurrent_batches_max = 5

[dag]
default_node_count = 16
convergence_pattern = "SingleConvergence"
convergence_point_ratio = 0.5
max_depth = 10
branching_factor = 2

[negotiation]
max_rounds = 3

[retry]
max_attempts = 3
initial_delay_ms = 100
max_delay_ms = 5000
backoff_multiplier = 2.0
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_URL", "https://custom.url/v1");

        let config = OrchestratorConfig::load().expect("Failed to load config");

        assert_eq!(config.llm.provider.url, "https://custom.url/v1");

        restore_original_dir();
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

[llm]
type = "google"
url = "https://generativelanguage.googleapis.com/v1"
default_model = "gemini-pro"

[llm.models]
en = "model-en"

[pipeline]
generation_timeout_secs = 60
validation_timeout_secs = 10
retry_max_attempts = 3
retry_base_delay_secs = 1
retry_max_delay_secs = 30

[batch]
size_min = 4
size_max = 6
concurrent_batches = 3
concurrent_batches_max = 5

[dag]
default_node_count = 16
convergence_pattern = "SingleConvergence"
convergence_point_ratio = 0.5
max_depth = 10
branching_factor = 2

[negotiation]
max_rounds = 3

[retry]
max_attempts = 3
initial_delay_ms = 100
max_delay_ms = 5000
backoff_multiplier = 2.0
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_GOOGLE_API_KEY", "test-api-key-12345");

        let config = OrchestratorConfig::load().expect("Failed to load config");

        assert_eq!(config.llm.provider.api_key, Some("test-api-key-12345".to_string()));

        restore_original_dir();
    }

    #[test]
    fn test_env_var_models_override() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["LLM_MODELS_EN"]);

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

[llm]
type = "google"
url = "https://generativelanguage.googleapis.com/v1"
default_model = "gemini-pro"

[llm.models]
en = "old-model-en"

[pipeline]
generation_timeout_secs = 60
validation_timeout_secs = 10
retry_max_attempts = 3
retry_base_delay_secs = 1
retry_max_delay_secs = 30

[batch]
size_min = 4
size_max = 6
concurrent_batches = 3
concurrent_batches_max = 5

[dag]
default_node_count = 16
convergence_pattern = "SingleConvergence"
convergence_point_ratio = 0.5
max_depth = 10
branching_factor = 2

[negotiation]
max_rounds = 3

[retry]
max_attempts = 3
initial_delay_ms = 100
max_delay_ms = 5000
backoff_multiplier = 2.0
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("LLM_MODELS_EN", "gemini-1.5-flash");

        let config = OrchestratorConfig::load().expect("Failed to load config");

        assert_eq!(config.llm.provider.models.get("en"), Some(&"gemini-1.5-flash".to_string()));

        restore_original_dir();
    }

    #[test]
    fn test_orchestrator_env_var_override() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["ORCHESTRATOR_NATS__URL"]);

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

[llm]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "default-model"

[llm.models]
en = "model-en"

[pipeline]
generation_timeout_secs = 60
validation_timeout_secs = 10
retry_max_attempts = 3
retry_base_delay_secs = 1
retry_max_delay_secs = 30

[batch]
size_min = 4
size_max = 6
concurrent_batches = 3
concurrent_batches_max = 5

[dag]
default_node_count = 16
convergence_pattern = "SingleConvergence"
convergence_point_ratio = 0.5
max_depth = 10
branching_factor = 2

[negotiation]
max_rounds = 3

[retry]
max_attempts = 3
initial_delay_ms = 100
max_delay_ms = 5000
backoff_multiplier = 2.0
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("ORCHESTRATOR_NATS__URL", "nats://override:4222");

        let config = OrchestratorConfig::load().expect("Failed to load config");

        assert_eq!(config.nats.url, "nats://override:4222");

        restore_original_dir();
    }
}
