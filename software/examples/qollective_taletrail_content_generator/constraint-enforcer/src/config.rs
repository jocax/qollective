//! Constraint Enforcer configuration

use serde::{Deserialize, Serialize};
use shared_types::*;
use figment::{Figment, providers::{Env, Format, Serialized, Toml}};

/// Constraint Enforcer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintEnforcerConfig {
    pub service: ServiceConfig,
    pub nats: NatsConfig,
    pub constraints: ConstraintsConfig,
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
pub struct ConstraintsConfig {
    pub vocabulary_check_enabled: bool,
    pub theme_consistency_enabled: bool,
    pub required_elements_check_enabled: bool,
    pub vocabulary_levels: Vec<String>,
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
            subject: "mcp.constraint.enforce".to_string(),
            queue_group: "constraint-enforcer".to_string(),
            tls: TlsConfig::default(),
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
        }
    }
}

impl Default for ConstraintEnforcerConfig {
    fn default() -> Self {
        Self {
            service: ServiceConfig::default(),
            nats: NatsConfig::default(),
            constraints: ConstraintsConfig::default(),
        }
    }
}

impl ConstraintEnforcerConfig {
    /// Load configuration using Figment merge strategy
    /// Priority (lowest to highest): Defaults → config.toml → Environment variables
    pub fn load() -> Result<Self> {
        let config: Self = Figment::new()
            // Layer 1: Hardcoded defaults (fallback)
            .merge(Serialized::defaults(Self::default()))

            // Layer 2: config.toml file (overrides defaults)
            .merge(Toml::file("constraint-enforcer/config.toml"))

            // Layer 3: Environment variables (highest priority)
            .merge(Env::prefixed("CONSTRAINT_ENFORCER_"))

            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        Ok(config)
    }
}
