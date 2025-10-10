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
    pub nkey_file: String,
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
            nkey_file: "./nkeys/story-generator.nk".to_string(),
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
    /// Priority (lowest to highest): .env file → config.toml → Environment variables
    pub fn load() -> Result<Self> {
        // Load .env file from current directory (lowest priority)
        dotenvy::dotenv().ok();

        let config: Self = Figment::new()
            // Layer 1: config.toml file
            .merge(Toml::file("story-generator/config.toml"))

            // Layer 2: Environment variables (highest priority)
            .merge(Env::prefixed("STORY_GENERATOR_"))

            .extract()?;

        Ok(config)
    }
}
