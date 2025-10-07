//! Gateway configuration using Qollective REST server patterns with TLS

use qollective::{
    server::rest::RestServerConfig,
    server::common::ServerConfig as QollectiveServerConfig,
    config::tls::{TlsConfigBuilder, VerificationMode},
};
use shared_types::*;
use serde::{Deserialize, Serialize};
use figment::providers::Format;

/// Get Gateway REST server configuration with TLS enabled from loaded config
pub fn get_gateway_config(config: &GatewayConfig) -> Result<RestServerConfig> {
    // Build TLS configuration for HTTPS server
    let tls_config = TlsConfigBuilder::new()
        .enabled(config.http.tls.enabled)
        .cert_path(&config.http.tls.cert)
        .key_path(&config.http.tls.key)
        .ca_cert_path(&config.http.tls.ca_cert)
        .verification_mode(VerificationMode::SystemCa)
        .build()
        .map_err(|e| TaleTrailError::ConfigError(format!("Failed to build TLS config: {}", e)))?;

    Ok(RestServerConfig {
        base: QollectiveServerConfig {
            bind_address: config.http.bind_address.clone(),
            port: config.http.port,
            ..Default::default()
        },
        tls: Some(tls_config),
        ..Default::default()
    })
}

// ============================================================================
// Full Gateway Configuration (HTTP + NATS)
// ============================================================================

/// Complete gateway configuration including HTTP server and NATS client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub http: HttpConfig,
    pub nats: NatsClientConfig,
    pub gateway: ServiceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub bind_address: String,
    pub port: u16,
    pub tls: HttpTlsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTlsConfig {
    pub enabled: bool,
    pub cert: String,
    pub key: String,
    pub ca_cert: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsClientConfig {
    pub url: String,
    pub subjects: NatsSubjects,
    pub tls: NatsTlsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsSubjects {
    pub orchestrator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsTlsConfig {
    pub ca_cert: String,
    pub client_cert: String,
    pub client_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
}

impl Default for HttpTlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cert: "./certs/gateway-cert.pem".to_string(),
            key: "./certs/gateway-key.pem".to_string(),
            ca_cert: "./certs/ca.pem".to_string(),
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8443,
            tls: HttpTlsConfig::default(),
        }
    }
}

impl Default for NatsTlsConfig {
    fn default() -> Self {
        Self {
            ca_cert: "./certs/ca.pem".to_string(),
            client_cert: "./certs/client-cert.pem".to_string(),
            client_key: "./certs/client-key.pem".to_string(),
        }
    }
}

impl Default for NatsSubjects {
    fn default() -> Self {
        Self {
            orchestrator: "mcp.orchestrator.request".to_string(),
        }
    }
}

impl Default for NatsClientConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:5222".to_string(),
            subjects: NatsSubjects::default(),
            tls: NatsTlsConfig::default(),
        }
    }
}

impl Default for ServiceInfo {
    fn default() -> Self {
        Self {
            name: "taletrail-gateway".to_string(),
            version: "0.1.0".to_string(),
        }
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            http: HttpConfig::default(),
            nats: NatsClientConfig::default(),
            gateway: ServiceInfo::default(),
        }
    }
}

impl GatewayConfig {
    /// Load configuration using Figment merge strategy
    /// Priority (lowest to highest): Defaults → config.toml → Environment variables
    pub fn load() -> Result<Self> {
        let config: Self = figment::Figment::new()
            // Layer 1: Hardcoded defaults (fallback)
            .merge(figment::providers::Serialized::defaults(Self::default()))

            // Layer 2: config.toml file (overrides defaults)
            .merge(figment::providers::Toml::file("gateway/config.toml"))

            // Layer 3: Environment variables (highest priority)
            .merge(figment::providers::Env::prefixed("GATEWAY_"))

            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Config error: {}", e)))?;

        Ok(config)
    }
}
