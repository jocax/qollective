//! Constraint Enforcer MCP Server (Stub with TLS NATS Connection)

use shared_types::*;
use tracing::info;
use std::fs;

mod config;
mod server;

use config::ConstraintEnforcerConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = ConstraintEnforcerConfig::load()?;

    info!("=== Constraint Enforcer MCP Server Starting ===");
    info!("Configuration:");
    info!("  Service: {} v{}", config.service.name, config.service.version);
    info!("  NATS URL: {}", config.nats.url);
    info!("  NATS Subject: {}", config.nats.subject);
    info!("  NATS Queue Group: {}", config.nats.queue_group);
    info!("  Vocabulary Check: {}", config.constraints.vocabulary_check_enabled);
    info!("  Theme Consistency: {}", config.constraints.theme_consistency_enabled);
    info!("  Required Elements: {}", config.constraints.required_elements_check_enabled);
    info!("  Vocabulary Levels: {:?}", config.constraints.vocabulary_levels);
    info!("");

    // Load TLS certificates
    info!("Loading TLS certificates...");
    let ca_cert = load_cert(&config.nats.tls.ca_cert)?;
    let client_cert = load_cert(&config.nats.tls.client_cert)?;
    let client_key = load_key(&config.nats.tls.client_key)?;

    info!("✅ TLS certificates loaded successfully");

    // Build TLS configuration
    let mut root_cert_store = rustls::RootCertStore::empty();
    root_cert_store.add(ca_cert).map_err(|e| {
        TaleTrailError::TlsCertificateError(format!("Failed to add CA cert: {:?}", e))
    })?;

    let client_auth_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(vec![client_cert], client_key)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to build TLS config: {}", e)))?;

    info!("✅ TLS configuration built successfully");

    // Connect to NATS with TLS
    info!("Connecting to NATS with TLS...");
    let _nats_client = async_nats::ConnectOptions::new()
        .tls_client_config(client_auth_config)
        .connect(&config.nats.url)
        .await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to connect to NATS: {}", e)))?;

    info!("✅ Connected to NATS with TLS");
    info!("");
    info!("Constraint Enforcer MCP Server ready on {} (TLS enabled)", config.nats.subject);
    info!("Listening for shutdown signal (Ctrl+C)...");

    // Wait for shutdown signal
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| TaleTrailError::ConfigError(format!("Failed to listen for Ctrl+C: {}", e)))?;

    info!("Received Ctrl+C, shutting down gracefully...");
    Ok(())
}

/// Load a certificate from PEM file
fn load_cert(path: &str) -> Result<rustls::pki_types::CertificateDer<'static>> {
    let cert_data = fs::read(path)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to read cert {}: {}", path, e)))?;

    let mut cursor = std::io::Cursor::new(cert_data);
    let certs = rustls_pemfile::certs(&mut cursor)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to parse cert: {}", e)))?;

    certs.into_iter().next()
        .ok_or_else(|| TaleTrailError::TlsCertificateError(format!("No certificate found in {}", path)))
}

/// Load a private key from PEM file
fn load_key(path: &str) -> Result<rustls::pki_types::PrivateKeyDer<'static>> {
    let key_data = fs::read(path)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to read key {}: {}", path, e)))?;

    let mut cursor = std::io::Cursor::new(key_data);
    rustls_pemfile::private_key(&mut cursor)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to parse key: {}", e)))?
        .ok_or_else(|| TaleTrailError::TlsCertificateError(format!("No private key found in {}", path)))
}
