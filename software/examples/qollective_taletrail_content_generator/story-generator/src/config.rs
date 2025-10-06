//! Configuration module for Story Generator MCP Server

use anyhow::Result;
use figment::{Figment, providers::{Env, Format, Toml}};
use serde::{Deserialize, Serialize};
use shared_types::constants::*;

/// Story Generator MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryGeneratorConfig {
    /// Server configuration
    pub server: ServerConfig,
    
    /// NATS connection configuration
    pub nats: NatsConfig,
    
    /// Generation settings
    pub generation: GenerationConfig,
    
    /// LLM configuration
    pub llm: LlmConfig,
}

/// Server-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
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
    /// NATS server URL (from environment/constants)
    #[serde(skip)]
    pub url: String,
    
    /// TLS CA certificate path (from environment/constants)
    #[serde(skip)]
    pub tls_ca_cert: String,
    
    /// TLS client certificate path (from environment/constants)
    #[serde(skip)]
    pub tls_client_cert: String,
    
    /// TLS client key path (from environment/constants)
    #[serde(skip)]
    pub tls_client_key: String,
}

/// Content generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Generation timeout in seconds (from environment/constants)
    #[serde(skip)]
    pub timeout_secs: u64,
    
    /// Minimum batch size (from environment/constants)
    #[serde(skip)]
    pub batch_size_min: usize,
    
    /// Maximum batch size (from environment/constants)
    #[serde(skip)]
    pub batch_size_max: usize,
    
    /// Target words per node (from environment/constants)
    #[serde(skip)]
    pub target_words_per_node: usize,
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// LM Studio URL (from environment/constants)
    #[serde(skip)]
    pub url: String,
    
    /// LLM model name (from environment/constants)
    #[serde(skip)]
    pub model_name: String,
    
    /// Maximum tokens per node (from environment/constants)
    #[serde(skip)]
    pub max_tokens_per_node: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "story-generator".to_string(),
            version: "0.1.0".to_string(),
            description: "TaleTrail Story Generator MCP Server".to_string(),
        }
    }
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: NATS_URL.clone(),
            tls_ca_cert: NATS_TLS_CA_CERT_PATH.clone(),
            tls_client_cert: NATS_TLS_CLIENT_CERT_PATH.clone(),
            tls_client_key: NATS_TLS_CLIENT_KEY_PATH.clone(),
        }
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            timeout_secs: *GENERATION_TIMEOUT_SECS,
            batch_size_min: *BATCH_SIZE_MIN,
            batch_size_max: *BATCH_SIZE_MAX,
            target_words_per_node: *TARGET_WORDS_PER_NODE,
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            url: LM_STUDIO_URL.clone(),
            model_name: LM_STUDIO_MODEL_NAME.clone(),
            max_tokens_per_node: *MAX_TOKENS_PER_NODE,
        }
    }
}

impl Default for StoryGeneratorConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            nats: NatsConfig::default(),
            generation: GenerationConfig::default(),
            llm: LlmConfig::default(),
        }
    }
}

impl StoryGeneratorConfig {
    /// Load configuration from config.toml and environment variables
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        let mut config: Self = Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("STORY_GENERATOR_"))
            .extract()
            .unwrap_or_default();
        
        // Populate runtime values from constants (following configuration inheritance pattern)
        config.nats.url = NATS_URL.clone();
        config.nats.tls_ca_cert = NATS_TLS_CA_CERT_PATH.clone();
        config.nats.tls_client_cert = NATS_TLS_CLIENT_CERT_PATH.clone();
        config.nats.tls_client_key = NATS_TLS_CLIENT_KEY_PATH.clone();
        
        config.generation.timeout_secs = *GENERATION_TIMEOUT_SECS;
        config.generation.batch_size_min = *BATCH_SIZE_MIN;
        config.generation.batch_size_max = *BATCH_SIZE_MAX;
        config.generation.target_words_per_node = *TARGET_WORDS_PER_NODE;
        
        config.llm.url = LM_STUDIO_URL.clone();
        config.llm.model_name = LM_STUDIO_MODEL_NAME.clone();
        config.llm.max_tokens_per_node = *MAX_TOKENS_PER_NODE;
        
        Ok(config)
    }
}
