//! Orchestrator configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use figment::{Figment, providers::{Env, Format, Serialized, Toml}};

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
    pub llm: LlmConfig,
    pub pipeline: PipelineConfig,
    pub batch: BatchConfig,
    pub dag: DagConfig,
    pub negotiation: NegotiationConfig,
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
pub struct LlmConfig {
    pub url: String,
    pub model_name: String,
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
    pub convergence_point_ratio: f32,
    pub max_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationConfig {
    pub max_rounds: u32,
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
            subject: "mcp.orchestrator.request".to_string(),
            queue_group: "orchestrator".to_string(),
            tls: TlsConfig::default(),
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:1234/v1".to_string(),
            model_name: "local-model".to_string(),
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
            convergence_point_ratio: 0.25,
            max_depth: 10,
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

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            llm: LlmConfig::default(),
            pipeline: PipelineConfig::default(),
            batch: BatchConfig::default(),
            dag: DagConfig::default(),
            negotiation: NegotiationConfig::default(),
        }
    }
}

impl OrchestratorConfig {
    /// Load configuration using Figment merge strategy
    /// Priority (lowest to highest): Defaults → config.toml → Environment variables
    pub fn load() -> Result<Self> {
        let config: Self = Figment::new()
            // Layer 1: Hardcoded defaults (fallback)
            .merge(Serialized::defaults(Self::default()))

            // Layer 2: config.toml file (overrides defaults)
            .merge(Toml::file("orchestrator/config.toml"))

            // Layer 3: Environment variables (highest priority)
            .merge(Env::prefixed("ORCHESTRATOR_"))

            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        Ok(config)
    }
}
