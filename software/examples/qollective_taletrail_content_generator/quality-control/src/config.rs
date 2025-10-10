//! Quality Control configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use figment::{Figment, providers::{Env, Format, Toml}};

/// Quality Control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityControlConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
    pub validation: ValidationConfig,
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
    pub tls: TlsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub ca_cert: String,
    pub client_cert: String,
    pub client_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub min_quality_score: f32,
    pub timeout_secs: u64,
    pub max_negotiation_rounds: u32,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "quality-control".to_string(),
            version: "0.1.0".to_string(),
            description: "TaleTrail Quality Control MCP Server".to_string(),
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
            subject: "mcp.quality.validate".to_string(),
            queue_group: "quality-control".to_string(),
            tls: TlsConfig::default(),
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            min_quality_score: 0.7,
            timeout_secs: 10,
            max_negotiation_rounds: 3,
        }
    }
}

impl QualityControlConfig {
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
            .merge(Toml::file("quality-control/config.toml"));

        debug!("📄 Layer 1: Loaded base config from quality-control/config.toml");

        let config: Self = figment
            // Layer 2: QUALITY_CONTROL-specific environment variables (includes .env + system, system takes precedence)
            // Use double underscore (__) as the delimiter for nested config paths
            // Example: QUALITY_CONTROL_NATS__URL maps to nats.url
            .merge(Env::prefixed("QUALITY_CONTROL_").split("__"))
            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        debug!("🔧 Layer 2: Applied QUALITY_CONTROL_* environment variable overrides");

        debug!(
            service_name = %config.service.name,
            nats_url = %config.nats.url,
            min_quality_score = config.validation.min_quality_score,
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

    // Global mutex to ensure tests run sequentially (avoid directory change conflicts)
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    // Capture the original working directory before any tests modify it
    lazy_static! {
        static ref ORIGINAL_DIR: PathBuf = env::current_dir()
            .expect("Failed to get current directory at module initialization");
    }

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

    /// Helper to create a temporary directory with quality-control/config.toml file
    fn create_temp_config_dir(content: &str) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create quality-control subdirectory
        let qc_dir = temp_dir.path().join("quality-control");
        fs::create_dir(&qc_dir).expect("Failed to create quality-control dir");

        let config_path = qc_dir.join("config.toml");
        let mut file = fs::File::create(&config_path).expect("Failed to create config file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to config file");
        file.flush().expect("Failed to flush config file");
        temp_dir
    }

    /// Helper to safely restore to the original directory
    /// On macOS, if current directory is in a deleted temp dir, we must cd to an intermediate directory first
    fn restore_original_dir() {
        let _ = env::set_current_dir("/");  // Safe fallback directory
        env::set_current_dir(ORIGINAL_DIR.as_path()).expect("Failed to restore dir");
    }

    #[test]
    fn test_default_toml_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let _guard = EnvGuard::new(vec!["QUALITY_CONTROL_NATS_URL"]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://localhost:5222"
subject = "test.subject"
queue_group = "test-group"

[nats.tls]
ca_cert = "./test-ca.pem"
client_cert = "./test-client.pem"
client_key = "./test-key.pem"

[validation]
min_quality_score = 0.7
timeout_secs = 10
max_negotiation_rounds = 3
"#;

        let _temp_dir = create_temp_config_dir(config_content);

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = QualityControlConfig::load().expect("Failed to load config");

        assert_eq!(config.service.name, "test-service");
        assert_eq!(config.nats.url, "nats://localhost:5222");
        assert_eq!(config.validation.min_quality_score, 0.7);

        restore_original_dir();
    }

    #[test]
    fn test_service_config_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let _guard = EnvGuard::new(vec![]);

        let config_content = r#"
[service]
name = "custom-service"
version = "2.0.0"
description = "Custom Test Service"

[nats]
url = "nats://localhost:5222"
subject = "test.subject"
queue_group = "test-group"

[nats.tls]
ca_cert = "./test-ca.pem"
client_cert = "./test-client.pem"
client_key = "./test-key.pem"

[validation]
min_quality_score = 0.8
timeout_secs = 15
max_negotiation_rounds = 5
"#;

        let _temp_dir = create_temp_config_dir(config_content);

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = QualityControlConfig::load().expect("Failed to load config");

        assert_eq!(config.service.name, "custom-service");
        assert_eq!(config.service.version, "2.0.0");
        assert_eq!(config.service.description, "Custom Test Service");

        restore_original_dir();
    }

    #[test]
    fn test_nats_config_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let _guard = EnvGuard::new(vec![]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://custom:5222"
subject = "custom.subject"
queue_group = "custom-group"

[nats.tls]
ca_cert = "./custom-ca.pem"
client_cert = "./custom-client.pem"
client_key = "./custom-key.pem"

[validation]
min_quality_score = 0.7
timeout_secs = 10
max_negotiation_rounds = 3
"#;

        let _temp_dir = create_temp_config_dir(config_content);

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = QualityControlConfig::load().expect("Failed to load config");

        assert_eq!(config.nats.url, "nats://custom:5222");
        assert_eq!(config.nats.subject, "custom.subject");
        assert_eq!(config.nats.queue_group, "custom-group");

        restore_original_dir();
    }

    #[test]
    fn test_validation_config_loading() {
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

[nats.tls]
ca_cert = "./test-ca.pem"
client_cert = "./test-client.pem"
client_key = "./test-key.pem"

[validation]
min_quality_score = 0.9
timeout_secs = 20
max_negotiation_rounds = 10
"#;

        let _temp_dir = create_temp_config_dir(config_content);

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = QualityControlConfig::load().expect("Failed to load config");

        assert_eq!(config.validation.min_quality_score, 0.9);
        assert_eq!(config.validation.timeout_secs, 20);
        assert_eq!(config.validation.max_negotiation_rounds, 10);

        restore_original_dir();
    }

    #[test]
    fn test_env_var_override() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        // Figment uses double underscore (__) to represent nested config paths
        // QUALITY_CONTROL_NATS__URL maps to config.nats.url
        let _guard = EnvGuard::new(vec!["QUALITY_CONTROL_NATS__URL"]);

        let config_content = r#"
[service]
name = "test-service"
version = "1.0.0"
description = "Test Service"

[nats]
url = "nats://localhost:5222"
subject = "test.subject"
queue_group = "test-group"

[nats.tls]
ca_cert = "./test-ca.pem"
client_cert = "./test-client.pem"
client_key = "./test-key.pem"

[validation]
min_quality_score = 0.7
timeout_secs = 10
max_negotiation_rounds = 3
"#;

        let _temp_dir = create_temp_config_dir(config_content);

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("QUALITY_CONTROL_NATS__URL", "nats://override:4222");

        let config = QualityControlConfig::load().expect("Failed to load config");

        assert_eq!(config.nats.url, "nats://override:4222");

        restore_original_dir();
    }
}
