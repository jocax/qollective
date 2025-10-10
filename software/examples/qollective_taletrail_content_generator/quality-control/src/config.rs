//! Quality Control configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use figment::{Figment, providers::{Env, Format, Toml}};
use std::collections::HashMap;

/// Quality Control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityControlConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
    pub validation: ValidationConfig,
    pub rubrics: RubricsConfig,
    pub safety: SafetyConfig,
    pub educational: EducationalConfig,
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
    pub thresholds: ValidationThresholds,
    pub correction: CorrectionConfig,
}

/// Age-specific rubrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricsConfig {
    pub age_6_8: AgeGroupConfig,
    pub age_9_11: AgeGroupConfig,
    pub age_12_14: AgeGroupConfig,
    pub age_15_17: AgeGroupConfig,
    pub age_18_plus: AgeGroupConfig,
}

/// Configuration for a specific age group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeGroupConfig {
    pub max_sentence_length: f32,
    pub vocabulary_level: String,
    pub allowed_themes: Vec<String>,
}

/// Safety keywords configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub violence_keywords: Vec<String>,
    pub fear_keywords: Vec<String>,
    pub inappropriate_keywords: Vec<String>,
}

/// Educational criteria configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalConfig {
    pub educational_keywords: Vec<String>,
    pub goals: HashMap<String, Vec<String>>,
}

/// Validation thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationThresholds {
    pub min_age_appropriate_score: f32,
    pub min_educational_value_score: f32,
    pub max_safety_violations: usize,
}

/// Correction capability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionConfig {
    pub can_fix_locally_min_score: f32,
    pub can_fix_locally_max_violations: usize,
    pub needs_revision_min_score: f32,
    pub needs_revision_max_violations: usize,
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
            thresholds: ValidationThresholds::default(),
            correction: CorrectionConfig::default(),
        }
    }
}

impl Default for ValidationThresholds {
    fn default() -> Self {
        Self {
            min_age_appropriate_score: 0.7,
            min_educational_value_score: 0.6,
            max_safety_violations: 0,
        }
    }
}

impl Default for CorrectionConfig {
    fn default() -> Self {
        Self {
            can_fix_locally_min_score: 0.6,
            can_fix_locally_max_violations: 3,
            needs_revision_min_score: 0.3,
            needs_revision_max_violations: 6,
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

    /// Get age-specific configuration based on AgeGroup
    pub fn get_age_config(&self, age_group: &AgeGroup) -> &AgeGroupConfig {
        match age_group {
            AgeGroup::_6To8 => &self.rubrics.age_6_8,
            AgeGroup::_9To11 => &self.rubrics.age_9_11,
            AgeGroup::_12To14 => &self.rubrics.age_12_14,
            AgeGroup::_15To17 => &self.rubrics.age_15_17,
            AgeGroup::Plus18 => &self.rubrics.age_18_plus,
        }
    }

    /// Get all safety keywords combined
    pub fn get_all_safety_keywords(&self) -> Vec<&str> {
        self.safety.violence_keywords.iter()
            .chain(self.safety.fear_keywords.iter())
            .chain(self.safety.inappropriate_keywords.iter())
            .map(|s| s.as_str())
            .collect()
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

    /// Helper to get the standard test config sections (rubrics, safety, educational, validation.thresholds, validation.correction)
    fn get_test_config_sections() -> &'static str {
        r#"
[rubrics.age_6_8]
max_sentence_length = 15
vocabulary_level = "basic"
allowed_themes = ["animals", "friendship"]

[rubrics.age_9_11]
max_sentence_length = 20
vocabulary_level = "intermediate"
allowed_themes = ["adventure", "science"]

[rubrics.age_12_14]
max_sentence_length = 25
vocabulary_level = "intermediate"
allowed_themes = ["mystery", "history"]

[rubrics.age_15_17]
max_sentence_length = 30
vocabulary_level = "advanced"
allowed_themes = ["technology", "culture"]

[rubrics.age_18_plus]
max_sentence_length = 35
vocabulary_level = "advanced"
allowed_themes = ["philosophy", "ethics"]

[safety]
violence_keywords = ["sword", "fight"]
fear_keywords = ["scary", "monster"]
inappropriate_keywords = ["alcohol", "drugs"]

[educational]
educational_keywords = ["learn", "discover"]

[educational.goals]
science = ["observation", "experiment"]
math = ["counting", "patterns"]

[validation.thresholds]
min_age_appropriate_score = 0.7
min_educational_value_score = 0.6
max_safety_violations = 0

[validation.correction]
can_fix_locally_min_score = 0.6
can_fix_locally_max_violations = 3
needs_revision_min_score = 0.3
needs_revision_max_violations = 6
"#
    }

    #[test]
    fn test_default_toml_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let _guard = EnvGuard::new(vec!["QUALITY_CONTROL_NATS_URL"]);

        let config_content = format!(r#"
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
{}"#, get_test_config_sections());

        let _temp_dir = create_temp_config_dir(&config_content);

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

        let config_content = format!(r#"
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
{}"#, get_test_config_sections());

        let _temp_dir = create_temp_config_dir(&config_content);

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

        let config_content = format!(r#"
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
{}"#, get_test_config_sections());

        let _temp_dir = create_temp_config_dir(&config_content);

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

        let config_content = format!(r#"
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
{}"#, get_test_config_sections());

        let _temp_dir = create_temp_config_dir(&config_content);

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

        let config_content = format!(r#"
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
{}"#, get_test_config_sections());

        let _temp_dir = create_temp_config_dir(&config_content);

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("QUALITY_CONTROL_NATS__URL", "nats://override:4222");

        let config = QualityControlConfig::load().expect("Failed to load config");

        assert_eq!(config.nats.url, "nats://override:4222");

        restore_original_dir();
    }

    #[test]
    fn test_rubrics_and_safety_config_loading() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let _guard = EnvGuard::new(vec![]);

        let config_content = format!(r#"
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
{}"#, get_test_config_sections());

        let _temp_dir = create_temp_config_dir(&config_content);

        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = QualityControlConfig::load().expect("Failed to load config");

        // Test rubrics configuration
        assert_eq!(config.rubrics.age_6_8.max_sentence_length, 15.0);
        assert_eq!(config.rubrics.age_6_8.vocabulary_level, "basic");
        assert!(config.rubrics.age_6_8.allowed_themes.contains(&"animals".to_string()));

        assert_eq!(config.rubrics.age_9_11.max_sentence_length, 20.0);
        assert_eq!(config.rubrics.age_9_11.vocabulary_level, "intermediate");

        assert_eq!(config.rubrics.age_18_plus.max_sentence_length, 35.0);
        assert_eq!(config.rubrics.age_18_plus.vocabulary_level, "advanced");

        // Test safety configuration
        assert!(config.safety.violence_keywords.contains(&"sword".to_string()));
        assert!(config.safety.fear_keywords.contains(&"scary".to_string()));
        assert!(config.safety.inappropriate_keywords.contains(&"alcohol".to_string()));

        // Test educational configuration
        assert!(config.educational.educational_keywords.contains(&"learn".to_string()));
        assert!(config.educational.goals.contains_key("science"));
        assert!(config.educational.goals.contains_key("math"));

        // Test validation thresholds
        assert_eq!(config.validation.thresholds.min_age_appropriate_score, 0.7);
        assert_eq!(config.validation.thresholds.min_educational_value_score, 0.6);
        assert_eq!(config.validation.thresholds.max_safety_violations, 0);

        // Test correction config
        assert_eq!(config.validation.correction.can_fix_locally_min_score, 0.6);
        assert_eq!(config.validation.correction.can_fix_locally_max_violations, 3);
        assert_eq!(config.validation.correction.needs_revision_min_score, 0.3);
        assert_eq!(config.validation.correction.needs_revision_max_violations, 6);

        // Test helper methods
        let age_config = config.get_age_config(&AgeGroup::_6To8);
        assert_eq!(age_config.max_sentence_length, 15.0);

        let all_safety = config.get_all_safety_keywords();
        assert!(all_safety.contains(&"sword"));
        assert!(all_safety.contains(&"scary"));
        assert!(all_safety.contains(&"alcohol"));

        restore_original_dir();
    }
}
