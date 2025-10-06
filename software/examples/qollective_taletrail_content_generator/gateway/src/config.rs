//! Gateway configuration using Qollective REST server patterns with TLS

use qollective::{
    server::rest::RestServerConfig,
    server::common::ServerConfig as QollectiveServerConfig,
    config::tls::{TlsConfig, TlsConfigBuilder, VerificationMode},
};
use shared_types::*;

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
