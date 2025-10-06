//! Quality Control configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use figment::providers::Format;

/// Quality Control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityControlConfig {
    pub nats: NatsConfig,
    pub server: ServerConfig,
    pub validation: ValidationConfig,
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
pub struct ServerConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub min_quality_score: f32,
}

impl QualityControlConfig {
    pub fn load() -> Result<Self> {
        let config: Self = figment::Figment::new()
            .merge(figment::providers::Toml::file("config.toml"))
            .merge(figment::providers::Env::prefixed("QUALITY_"))
            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Failed to load config: {}", e)))?;

        Ok(config)
    }
}
