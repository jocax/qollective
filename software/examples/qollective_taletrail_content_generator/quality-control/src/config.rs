//! Quality Control configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use figment::{Figment, providers::{Env, Format, Serialized, Toml}};

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

impl Default for QualityControlConfig {
    fn default() -> Self {
        Self {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            validation: ValidationConfig::default(),
        }
    }
}

impl QualityControlConfig {
    /// Load configuration using Figment merge strategy
    /// Priority (lowest to highest): Defaults → config.toml → Environment variables
    pub fn load() -> Result<Self> {
        let config: Self = Figment::new()
            // Layer 1: Hardcoded defaults (fallback)
            .merge(Serialized::defaults(Self::default()))

            // Layer 2: config.toml file (overrides defaults)
            .merge(Toml::file("quality-control/config.toml"))

            // Layer 3: Environment variables (highest priority)
            .merge(Env::prefixed("QUALITY_CONTROL_"))

            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        Ok(config)
    }
}
