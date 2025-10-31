//! Quality Control configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use shared_types_llm::LlmConfig as SharedLlmConfig;
use figment::{Figment, providers::{Env, Format, Toml}};
use std::collections::HashMap;

use crate::execution_logger::ExecutionConfig;

/// Quality Control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityControlConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
    pub llm: SharedLlmConfig,
    pub validation: ValidationConfig,
    pub rubrics: RubricsConfig,
    /// Safety validation configuration
    #[serde(default)]
    pub safety: SafetyConfig,
    pub educational: EducationalConfig,
    /// Execution logging configuration
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
pub struct ValidationConfig {
    pub min_quality_score: f32,
    pub timeout_secs: u64,
    pub max_negotiation_rounds: u32,
    pub thresholds: ValidationThresholds,
    pub correction: CorrectionConfig,
    /// Delay for discovery responses in milliseconds
    pub discovery_delay_ms: u64,
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

/// Safety validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Enable keyword-based safety validation
    #[serde(default = "default_enable_keyword_validation")]
    pub enable_keyword_validation: bool,

    /// Directory containing per-language restricted word files
    pub restricted_words_dir: String,

    /// Automatically load restricted words based on request language
    #[serde(default = "default_auto_load")]
    pub auto_load_by_language: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            enable_keyword_validation: true,
            restricted_words_dir: "../quality-control/safety".to_string(),
            auto_load_by_language: true,
        }
    }
}

fn default_enable_keyword_validation() -> bool {
    true
}

fn default_auto_load() -> bool {
    true
}

/// Structure of restricted words TOML file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestrictedWordsFile {
    restricted_words: Vec<String>,
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

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            nkey_file: Some("./nkeys/quality-control.nk".to_string()),
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
            subject: "mcp.quality.validate".to_string(),
            queue_group: "quality-control".to_string(),
            auth: AuthConfig::default(),
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
            discovery_delay_ms: 100,
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
            // Layer 3: QUALITY_CONTROL-specific environment variables (includes .env + system, system takes precedence)
            // Use double underscore (__) as the delimiter for nested config paths
            // Example: QUALITY_CONTROL_NATS__URL maps to nats.url
            .merge(Env::prefixed("QUALITY_CONTROL_").split("__"))
            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        debug!("🔧 Layer 3: Applied QUALITY_CONTROL_* environment variable overrides");

        debug!(
            service_name = %config.service.name,
            nats_url = %config.nats.url,
            min_quality_score = config.validation.min_quality_score,
            provider_type = ?config.llm.provider.provider_type,
            llm_url = %config.llm.provider.url,
            default_model = %config.llm.provider.default_model,
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

    /// Load restricted words for a specific language from config file
    ///
    /// Returns empty Vec if file doesn't exist or keyword validation is disabled.
    pub fn load_restricted_words(&self, language: &str) -> Result<Vec<String>> {
        if !self.safety.enable_keyword_validation || !self.safety.auto_load_by_language {
            return Ok(Vec::new());
        }

        let file_path = format!("{}/{}.toml", self.safety.restricted_words_dir, language);
        let path = std::path::Path::new(&file_path);

        // If file doesn't exist, return empty list (not an error)
        if !path.exists() {
            tracing::debug!(
                language = %language,
                path = %file_path,
                "No restricted words file found, using empty list"
            );
            return Ok(Vec::new());
        }

        // Load and parse TOML file
        let content = std::fs::read_to_string(path)
            .map_err(|e| TaleTrailError::ConfigError(format!("Failed to read restricted words file {}: {}", file_path, e)))?;

        let config: RestrictedWordsFile = toml::from_str(&content)
            .map_err(|e| TaleTrailError::ConfigError(format!("Failed to parse restricted words file {}: {}", file_path, e)))?;

        tracing::info!(
            language = %language,
            word_count = config.restricted_words.len(),
            "Loaded restricted words from config file"
        );

        Ok(config.restricted_words)
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

    /// Helper to get the standard test config sections (llm, rubrics, safety, educational, validation.thresholds, validation.correction)
    fn get_test_config_sections() -> &'static str {
        r#"
[llm]
type = "google"
url = "https://generativelanguage.googleapis.com"
default_model = "gemini-2.0-flash-lite"
use_default_model_fallback = true
timeout_secs = 180
max_tokens = 4096
temperature = 0.7
system_prompt_style = "chatml"

[llm.models]
en = "gemini-2.0-flash-lite"
de = "gemini-2.0-flash-lite"

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
enable_keyword_validation = true
restricted_words_dir = "../quality-control/safety"
auto_load_by_language = true

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

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

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

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

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

[nats.auth]
nkey_file = "./custom.nk"

[nats.tls]
ca_cert = "./custom-ca.pem"

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

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

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

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

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

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

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
        assert_eq!(config.safety.enable_keyword_validation, true);
        assert_eq!(config.safety.restricted_words_dir, "../quality-control/safety");
        assert_eq!(config.safety.auto_load_by_language, true);

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

        restore_original_dir();
    }
}
