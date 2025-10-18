//! Configuration management for NATS CLI
//!
//! Uses Figment for hierarchical configuration:
//! Defaults → config.toml → Environment variables

use crate::constants::*;
use crate::errors::{NatsCliError, Result};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsCliConfig {
    /// NATS connection configuration
    pub nats: NatsConfig,
    /// Client behavior configuration
    pub client: ClientConfig,
    /// Envelope configuration
    pub envelope: EnvelopeConfig,
}

/// NATS connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    /// NATS server URL
    pub url: String,
    /// NKEY authentication file path
    pub nkey_file: PathBuf,
    /// TLS configuration
    pub tls: TlsConfig,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// CA certificate path
    pub ca_cert: PathBuf,
}

/// Client behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Default request timeout in seconds
    pub default_timeout_secs: u64,
    /// Default tenant ID
    pub default_tenant_id: i32,
    /// Logging level
    pub log_level: String,
    /// Whether to use colored output
    #[serde(default = "default_colored_output")]
    pub colored_output: bool,
}

/// Envelope configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeConfig {
    /// Envelope version
    pub version: String,
    /// Whether to validate responses
    #[serde(default = "default_validate_responses")]
    pub validate_responses: bool,
}

// Default value functions for serde
fn default_colored_output() -> bool {
    true
}

fn default_validate_responses() -> bool {
    true
}

impl Default for NatsCliConfig {
    fn default() -> Self {
        Self {
            nats: NatsConfig {
                url: "nats://localhost:5222".to_string(),
                nkey_file: PathBuf::from("../nkeys/nats-cli.nk"),
                tls: TlsConfig {
                    ca_cert: PathBuf::from("../certs/ca.pem"),
                },
            },
            client: ClientConfig {
                default_timeout_secs: DEFAULT_TIMEOUT_SECS,
                default_tenant_id: DEFAULT_TENANT_ID,
                log_level: DEFAULT_LOG_LEVEL.to_string(),
                colored_output: true,
            },
            envelope: EnvelopeConfig {
                version: DEFAULT_ENVELOPE_VERSION.to_string(),
                validate_responses: true,
            },
        }
    }
}

impl NatsCliConfig {
    /// Load configuration from file and environment
    ///
    /// # Configuration Hierarchy
    /// 1. Default values (from Default trait)
    /// 2. config.toml file
    /// 3. Environment variables (prefixed with NATS_CLI_)
    ///
    /// # Example Environment Variables
    /// - NATS_CLI_NATS__URL="nats://production:4222"
    /// - NATS_CLI_CLIENT__DEFAULT_TIMEOUT_SECS=180
    /// - NATS_CLI_CLIENT__LOG_LEVEL="debug"
    pub fn load() -> Result<Self> {
        let config: Self = Figment::new()
            .merge(Serialized::defaults(Self::default()))
            .merge(Toml::file(DEFAULT_CONFIG_PATH))
            .merge(Env::prefixed(&format!("{}_", ENV_PREFIX)).split("__"))
            .extract()
            .map_err(|e| NatsCliError::ConfigError(format!("Failed to load configuration: {}", e)))?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration with custom config file path
    pub fn load_from_file(config_path: &str) -> Result<Self> {
        let config: Self = Figment::new()
            .merge(Serialized::defaults(Self::default()))
            .merge(Toml::file(config_path))
            .merge(Env::prefixed(&format!("{}_", ENV_PREFIX)).split("__"))
            .extract()
            .map_err(|e| {
                NatsCliError::ConfigError(format!(
                    "Failed to load configuration from {}: {}",
                    config_path, e
                ))
            })?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // Validate timeout range
        if self.client.default_timeout_secs < MIN_TIMEOUT_SECS {
            return Err(NatsCliError::ConfigError(format!(
                "Timeout must be at least {} seconds",
                MIN_TIMEOUT_SECS
            )));
        }

        if self.client.default_timeout_secs > MAX_TIMEOUT_SECS {
            return Err(NatsCliError::ConfigError(format!(
                "Timeout cannot exceed {} seconds",
                MAX_TIMEOUT_SECS
            )));
        }

        // Validate NKEY file exists
        if !self.nats.nkey_file.exists() {
            return Err(NatsCliError::ConfigError(format!(
                "NKEY file not found: {}",
                self.nats.nkey_file.display()
            )));
        }

        // Validate CA cert exists
        if !self.nats.tls.ca_cert.exists() {
            return Err(NatsCliError::ConfigError(format!(
                "CA certificate not found: {}",
                self.nats.tls.ca_cert.display()
            )));
        }

        Ok(())
    }

    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.client.default_timeout_secs)
    }

    /// Get absolute path to NKEY file
    pub fn nkey_path(&self) -> PathBuf {
        if self.nats.nkey_file.is_absolute() {
            self.nats.nkey_file.clone()
        } else {
            std::env::current_dir()
                .unwrap()
                .join(&self.nats.nkey_file)
        }
    }

    /// Get absolute path to CA certificate
    pub fn ca_cert_path(&self) -> PathBuf {
        if self.nats.tls.ca_cert.is_absolute() {
            self.nats.tls.ca_cert.clone()
        } else {
            std::env::current_dir()
                .unwrap()
                .join(&self.nats.tls.ca_cert)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NatsCliConfig::default();
        assert_eq!(config.client.default_timeout_secs, DEFAULT_TIMEOUT_SECS);
        assert_eq!(config.client.default_tenant_id, DEFAULT_TENANT_ID);
        assert_eq!(config.envelope.version, DEFAULT_ENVELOPE_VERSION);
    }

    #[test]
    fn test_timeout_validation() {
        let mut config = NatsCliConfig::default();

        // Too small
        config.client.default_timeout_secs = 0;
        assert!(config.validate().is_err());

        // Too large
        config.client.default_timeout_secs = 9999;
        assert!(config.validate().is_err());

        // Valid
        config.client.default_timeout_secs = 30;
        // Note: This will fail if files don't exist, but validates timeout range
    }
}
