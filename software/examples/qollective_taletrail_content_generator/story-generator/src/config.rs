//! Configuration module for Story Generator MCP Server

use anyhow::Result;
use figment::{Figment, providers::{Env, Format, Serialized, Toml}};
use serde::{Deserialize, Serialize};

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
    pub llm: LlmConfig,
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

    /// TLS configuration
    pub tls: TlsConfig,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// CA certificate path
    pub ca_cert: String,

    /// Client certificate path
    pub client_cert: String,

    /// Client key path
    pub client_key: String,
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

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// LM Studio URL
    pub url: String,

    /// LLM model name
    pub model_name: String,

    /// Maximum tokens per node
    pub max_tokens_per_node: u32,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "story-generator".to_string(),
            version: "0.1.0".to_string(),
            description: "TaleTrail Story Generator MCP Server".to_string(),
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
            subject: "mcp.story.generate".to_string(),
            queue_group: "story-generator".to_string(),
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

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:1234".to_string(),
            model_name: "local-model".to_string(),
            max_tokens_per_node: 600,
        }
    }
}

impl Default for StoryGeneratorConfig {
    fn default() -> Self {
        Self {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            generation: GenerationConfig::default(),
            llm: LlmConfig::default(),
        }
    }
}

impl StoryGeneratorConfig {
    /// Load configuration using Figment merge strategy
    /// Priority (lowest to highest): Defaults → config.toml → Environment variables
    pub fn load() -> Result<Self> {
        let config: Self = Figment::new()
            // Layer 1: Hardcoded defaults (fallback)
            .merge(Serialized::defaults(Self::default()))

            // Layer 2: config.toml file (overrides defaults)
            .merge(Toml::file("story-generator/config.toml"))

            // Layer 3: Environment variables (highest priority)
            .merge(Env::prefixed("STORY_GENERATOR_"))

            .extract()?;

        Ok(config)
    }
}
