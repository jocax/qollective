//! Gateway configuration using Qollective REST server patterns with TLS

use qollective::{
    server::rest::RestServerConfig,
    server::common::ServerConfig as QollectiveServerConfig,
    config::tls::{TlsConfigBuilder, VerificationMode},
};
use shared_types::*;
use serde::{Deserialize, Serialize};
use figment::providers::Format;

/// Get Gateway REST server configuration with TLS enabled
pub fn get_gateway_config() -> Result<RestServerConfig> {
    // Build TLS configuration for HTTPS server
    let tls_config = TlsConfigBuilder::new()
        .enabled(true)
        .cert_path("./certs/gateway-cert.pem")  // Gateway-specific certificate
        .key_path("./certs/gateway-key.pem")    // Gateway-specific private key
        .ca_cert_path("./certs/ca.pem")         // Shared CA for verification
        .verification_mode(VerificationMode::SystemCa)
        .build()
        .map_err(|e| TaleTrailError::ConfigError(format!("Failed to build TLS config: {}", e)))?;

    Ok(RestServerConfig {
        base: QollectiveServerConfig {
            bind_address: std::env::var("GATEWAY_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("GATEWAY_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(*GATEWAY_PORT),
            ..Default::default()
        },
        tls: Some(tls_config),  // Enable TLS for HTTPS
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

impl GatewayConfig {
    /// Load configuration from config.toml with environment variable overrides
    pub fn load() -> Result<Self> {
        let config: Self = figment::Figment::new()
            .merge(figment::providers::Toml::file("gateway/config.toml"))
            .merge(figment::providers::Env::prefixed("GATEWAY_"))
            .extract()
            .map_err(|e| TaleTrailError::ConfigError(format!("Failed to load gateway config: {}", e)))?;

        Ok(config)
    }
}
