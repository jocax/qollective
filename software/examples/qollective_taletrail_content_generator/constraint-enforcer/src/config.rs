//! Constraint Enforcer configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use shared_types_llm::LlmConfig as SharedLlmConfig;
use figment::{Figment, providers::{Env, Format, Toml}};
use std::collections::HashMap;

/// Constraint Enforcer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintEnforcerConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
    pub llm: SharedLlmConfig,
    pub constraints: ConstraintsConfig,
    pub vocabulary: VocabularyConfig,
    pub themes: ThemesConfig,
    pub required_elements: RequiredElementsConfig,
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
pub struct ConstraintsConfig {
    pub vocabulary_check_enabled: bool,
    pub theme_consistency_enabled: bool,
    pub required_elements_check_enabled: bool,
    pub vocabulary_levels: Vec<String>,
    #[serde(default)]
    pub validation: ValidationConfig,
}

/// Validation configuration for hybrid keyword + LLM semantic matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Keyword match threshold (0.0 to 1.0) - percentage of keywords that must match
    pub keyword_match_threshold: f32,
    /// Enable LLM semantic fallback when keyword matching is insufficient
    pub enable_llm_fallback: bool,
    /// Minimum keyword length to consider (filters short words)
    pub min_keyword_length: usize,
    /// Maximum content length to send to LLM (truncated if longer)
    pub max_llm_content_length: usize,
    /// English stopwords to filter out during keyword extraction
    pub stopwords_en: Vec<String>,
    /// German stopwords to filter out during keyword extraction
    pub stopwords_de: Vec<String>,
    /// LLM prompt template for semantic checking
    pub llm_semantic_prompt: String,
}

/// Vocabulary configuration for multiple languages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyConfig {
    pub english: LanguageVocabulary,
    pub german: LanguageVocabulary,
}

/// Language-specific vocabulary organized by complexity level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageVocabulary {
    pub basic: VocabularyLevel,
    pub intermediate: VocabularyLevel,
    pub advanced: VocabularyLevel,
}

/// Vocabulary level with list of words
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyLevel {
    pub words: Vec<String>,
}

/// Theme consistency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemesConfig {
    pub min_consistency_score: f32,
    pub keywords: HashMap<String, ThemeKeywords>,
}

/// Keywords for a specific theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeKeywords {
    pub keywords: Vec<String>,
}

/// Required elements configuration for different story types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredElementsConfig {
    pub moral_keywords: Vec<String>,
    pub science_keywords: Vec<String>,
    pub educational_keywords: Vec<String>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "constraint-enforcer".to_string(),
            version: "0.1.0".to_string(),
            description: "TaleTrail Constraint Enforcer".to_string(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            nkey_file: Some("./nkeys/constraint-enforcer.nk".to_string()),
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
            subject: "mcp.constraint.enforce".to_string(),
            queue_group: "constraint-enforcer".to_string(),
            auth: AuthConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            keyword_match_threshold: 0.8,
            enable_llm_fallback: true,
            min_keyword_length: 3,
            max_llm_content_length: 2000,
            stopwords_en: vec![
                "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for"
            ].iter().map(|s| s.to_string()).collect(),
            stopwords_de: vec![
                "der", "die", "das", "den", "dem", "des", "ein", "eine", "und", "oder"
            ].iter().map(|s| s.to_string()).collect(),
            llm_semantic_prompt: "Does the following content convey or address the concept of '{element}'?\n\nContent:\n{content}\n\nAnswer with ONLY 'yes' or 'no' (no explanation needed).".to_string(),
        }
    }
}

impl Default for ConstraintsConfig {
    fn default() -> Self {
        Self {
            vocabulary_check_enabled: true,
            theme_consistency_enabled: true,
            required_elements_check_enabled: true,
            vocabulary_levels: vec!["basic".to_string(), "intermediate".to_string(), "advanced".to_string()],
            validation: ValidationConfig::default(),
        }
    }
}

impl ConstraintEnforcerConfig {
    /// Get vocabulary for a specific language
    pub fn get_vocabulary_for_language(&self, language: &Language) -> &LanguageVocabulary {
        match language {
            Language::En => &self.vocabulary.english,
            Language::De => &self.vocabulary.german,
        }
    }

    /// Get keywords for a specific theme
    pub fn get_theme_keywords(&self, theme: &str) -> Vec<String> {
        self.themes.keywords
            .get(theme)
            .map(|t| t.keywords.clone())
            .unwrap_or_default()
    }

    /// Get vocabulary words for a specific language and level
    pub fn get_vocabulary_words(&self, language: &Language, level: &str) -> Vec<String> {
        let lang_vocab = self.get_vocabulary_for_language(language);
        match level {
            "basic" => lang_vocab.basic.words.clone(),
            "intermediate" => lang_vocab.intermediate.words.clone(),
            "advanced" => lang_vocab.advanced.words.clone(),
            _ => Vec::new(),
        }
    }

    /// Check if theme consistency is enabled
    pub fn is_theme_consistency_enabled(&self) -> bool {
        self.constraints.theme_consistency_enabled
    }

    /// Check if vocabulary check is enabled
    pub fn is_vocabulary_check_enabled(&self) -> bool {
        self.constraints.vocabulary_check_enabled
    }

    /// Check if required elements check is enabled
    pub fn is_required_elements_check_enabled(&self) -> bool {
        self.constraints.required_elements_check_enabled
    }

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
            .merge(Toml::file("constraint-enforcer/config.toml"));

        debug!("📄 Layer 1: Loaded base config from constraint-enforcer/config.toml");

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
            // Layer 3: CONSTRAINT_ENFORCER-specific environment variables (includes .env + system, system takes precedence)
            // Use double underscore (__) as the delimiter for nested config paths
            // Example: CONSTRAINT_ENFORCER_NATS__URL maps to nats.url
            .merge(Env::prefixed("CONSTRAINT_ENFORCER_").split("__"))
            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        debug!("🔧 Layer 3: Applied CONSTRAINT_ENFORCER_* environment variable overrides");

        debug!(
            service_name = %config.service.name,
            nats_url = %config.nats.url,
            vocabulary_check_enabled = config.constraints.vocabulary_check_enabled,
            provider_type = ?config.llm.provider.provider_type,
            llm_url = %config.llm.provider.url,
            default_model = %config.llm.provider.default_model,
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

    /// Helper to create a temporary directory with constraint-enforcer/config.toml file
    fn create_temp_config_dir(content: &str) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create constraint-enforcer subdirectory
        let ce_dir = temp_dir.path().join("constraint-enforcer");
        fs::create_dir(&ce_dir).expect("Failed to create constraint-enforcer dir");

        let config_path = ce_dir.join("config.toml");
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
        let _guard = EnvGuard::new(vec!["CONSTRAINT_ENFORCER_NATS_URL"]);

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

[constraints]
vocabulary_check_enabled = true
theme_consistency_enabled = true
required_elements_check_enabled = true
vocabulary_levels = ["basic", "intermediate"]

[vocabulary.english.basic]
words = ["test", "word"]

[vocabulary.english.intermediate]
words = ["intermediate"]

[vocabulary.english.advanced]
words = ["advanced"]

[vocabulary.german.basic]
words = ["test", "wort"]

[vocabulary.german.intermediate]
words = ["mittel"]

[vocabulary.german.advanced]
words = ["fortgeschritten"]

[themes]
min_consistency_score = 0.6

[themes.keywords.test]
keywords = ["test", "keyword"]

[required_elements]
moral_keywords = ["moral"]
science_keywords = ["science"]
educational_keywords = ["education"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = ConstraintEnforcerConfig::load().expect("Failed to load config");

        assert_eq!(config.service.name, "test-service");
        assert_eq!(config.nats.url, "nats://localhost:5222");
        assert_eq!(config.constraints.vocabulary_check_enabled, true);

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

[nats.auth]
nkey_file = "./test.nk"

[nats.tls]
ca_cert = "./test-ca.pem"

[constraints]
vocabulary_check_enabled = true
theme_consistency_enabled = true
required_elements_check_enabled = true
vocabulary_levels = ["basic"]

[vocabulary.english.basic]
words = ["test"]
[vocabulary.english.intermediate]
words = ["test"]
[vocabulary.english.advanced]
words = ["test"]
[vocabulary.german.basic]
words = ["test"]
[vocabulary.german.intermediate]
words = ["test"]
[vocabulary.german.advanced]
words = ["test"]

[themes]
min_consistency_score = 0.6
[themes.keywords.test]
keywords = ["test"]

[required_elements]
moral_keywords = ["moral"]
science_keywords = ["science"]
educational_keywords = ["education"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = ConstraintEnforcerConfig::load().expect("Failed to load config");

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

[nats.auth]
nkey_file = "./custom.nk"

[nats.tls]
ca_cert = "./custom-ca.pem"

[constraints]
vocabulary_check_enabled = true
theme_consistency_enabled = true
required_elements_check_enabled = true
vocabulary_levels = ["advanced"]

[vocabulary.english.basic]
words = ["test"]
[vocabulary.english.intermediate]
words = ["test"]
[vocabulary.english.advanced]
words = ["test"]
[vocabulary.german.basic]
words = ["test"]
[vocabulary.german.intermediate]
words = ["test"]
[vocabulary.german.advanced]
words = ["test"]

[themes]
min_consistency_score = 0.6
[themes.keywords.test]
keywords = ["test"]

[required_elements]
moral_keywords = ["moral"]
science_keywords = ["science"]
educational_keywords = ["education"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = ConstraintEnforcerConfig::load().expect("Failed to load config");

        assert_eq!(config.nats.url, "nats://custom:5222");
        assert_eq!(config.nats.subject, "custom.subject");
        assert_eq!(config.nats.queue_group, "custom-group");

        restore_original_dir();
    }

    #[test]
    fn test_constraints_config_loading() {
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

[constraints]
vocabulary_check_enabled = false
theme_consistency_enabled = false
required_elements_check_enabled = false
vocabulary_levels = ["expert", "master"]

[vocabulary.english.basic]
words = ["test"]
[vocabulary.english.intermediate]
words = ["test"]
[vocabulary.english.advanced]
words = ["test"]
[vocabulary.german.basic]
words = ["test"]
[vocabulary.german.intermediate]
words = ["test"]
[vocabulary.german.advanced]
words = ["test"]

[themes]
min_consistency_score = 0.6
[themes.keywords.test]
keywords = ["test"]

[required_elements]
moral_keywords = ["moral"]
science_keywords = ["science"]
educational_keywords = ["education"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = ConstraintEnforcerConfig::load().expect("Failed to load config");

        assert_eq!(config.constraints.vocabulary_check_enabled, false);
        assert_eq!(config.constraints.theme_consistency_enabled, false);
        assert_eq!(config.constraints.required_elements_check_enabled, false);
        assert_eq!(config.constraints.vocabulary_levels, vec!["expert", "master"]);

        restore_original_dir();
    }

    #[test]
    fn test_env_var_override() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(vec!["CONSTRAINT_ENFORCER_NATS__URL"]);

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

[constraints]
vocabulary_check_enabled = true
theme_consistency_enabled = true
required_elements_check_enabled = true
vocabulary_levels = ["basic"]

[vocabulary.english.basic]
words = ["test"]
[vocabulary.english.intermediate]
words = ["test"]
[vocabulary.english.advanced]
words = ["test"]
[vocabulary.german.basic]
words = ["test"]
[vocabulary.german.intermediate]
words = ["test"]
[vocabulary.german.advanced]
words = ["test"]

[themes]
min_consistency_score = 0.6
[themes.keywords.test]
keywords = ["test"]

[required_elements]
moral_keywords = ["moral"]
science_keywords = ["science"]
educational_keywords = ["education"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");
        env::set_var("CONSTRAINT_ENFORCER_NATS__URL", "nats://override:4222");

        let config = ConstraintEnforcerConfig::load().expect("Failed to load config");

        assert_eq!(config.nats.url, "nats://override:4222");

        restore_original_dir();
    }

    #[test]
    fn test_vocabulary_helper_methods() {
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

[constraints]
vocabulary_check_enabled = true
theme_consistency_enabled = true
required_elements_check_enabled = true
vocabulary_levels = ["basic"]

[vocabulary.english.basic]
words = ["hello", "world"]
[vocabulary.english.intermediate]
words = ["adventure"]
[vocabulary.english.advanced]
words = ["sophisticated"]
[vocabulary.german.basic]
words = ["hallo", "welt"]
[vocabulary.german.intermediate]
words = ["abenteuer"]
[vocabulary.german.advanced]
words = ["ausgefeilt"]

[themes]
min_consistency_score = 0.7
[themes.keywords.test]
keywords = ["test", "keyword"]
[themes.keywords.ocean]
keywords = ["ocean", "water"]

[required_elements]
moral_keywords = ["moral"]
science_keywords = ["science"]
educational_keywords = ["education"]
"#;

        let _temp_dir = create_temp_config_dir(config_content);
        env::set_current_dir(_temp_dir.path()).expect("Failed to change dir");

        let config = ConstraintEnforcerConfig::load().expect("Failed to load config");

        // Test get_vocabulary_for_language
        let english_vocab = config.get_vocabulary_for_language(&Language::En);
        assert_eq!(english_vocab.basic.words, vec!["hello", "world"]);

        let german_vocab = config.get_vocabulary_for_language(&Language::De);
        assert_eq!(german_vocab.basic.words, vec!["hallo", "welt"]);

        // Test get_theme_keywords
        let ocean_keywords = config.get_theme_keywords("ocean");
        assert_eq!(ocean_keywords, vec!["ocean", "water"]);

        let missing_keywords = config.get_theme_keywords("nonexistent");
        assert!(missing_keywords.is_empty());

        // Test get_vocabulary_words
        let basic_words = config.get_vocabulary_words(&Language::En, "basic");
        assert_eq!(basic_words, vec!["hello", "world"]);

        let advanced_words = config.get_vocabulary_words(&Language::De, "advanced");
        assert_eq!(advanced_words, vec!["ausgefeilt"]);

        // Test boolean helper methods
        assert!(config.is_vocabulary_check_enabled());
        assert!(config.is_theme_consistency_enabled());
        assert!(config.is_required_elements_check_enabled());

        // Test theme consistency score
        assert_eq!(config.themes.min_consistency_score, 0.7);

        restore_original_dir();
    }
}
