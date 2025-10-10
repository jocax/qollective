//! Configuration module for Story Generator MCP Server

use anyhow::Result;
use figment::{Figment, providers::{Env, Format, Toml}};
use serde::{Deserialize, Serialize};
use shared_types_llm::LlmConfig as SharedLlmConfig;

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
        // Load .env file from current directory
        // dotenvy loads .env vars into environment but does NOT override existing system env vars
        // This ensures: system env vars (highest) > .env vars (middle) > TOML (lowest)
        dotenvy::dotenv().ok();

        let config: Self = Figment::new()
            // Layer 1: Base config from TOML file (lowest priority)
            .merge(Toml::file("story-generator/config.toml"))
            // Layer 2: Environment variables (includes .env + system, system takes precedence)
            .merge(Env::prefixed("STORY_GENERATOR_"))
            .extract()?;

        Ok(config)
    }
}
