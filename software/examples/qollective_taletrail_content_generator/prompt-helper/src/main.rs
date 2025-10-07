//! Prompt Helper MCP Server (Stub with TLS NATS Connection)

use shared_types::*;
use tracing::info;
use std::fs;
use std::io::BufReader;
use rustls_pemfile::{certs, pkcs8_private_keys};

mod config;
mod server;

use config::PromptHelperConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Load configuration
    let config = PromptHelperConfig::load()?;

    info!("=== Prompt Helper MCP Server Starting ===");
    info!("Configuration:");
    info!("  NATS URL: {}", config.nats.url);
    info!("  NATS Subject: {}", config.nats.subject);
    info!("  NATS Queue Group: {}", config.nats.queue_group);
    info!("  Supported Languages: {:?}", config.prompt.supported_languages);
    info!("  Default Model: {}", config.prompt.models.default_model);
    info!("");

    // Build TLS configuration
    info!("Loading TLS certificates...");
    let tls_config = build_tls_config(&config)?;
    info!("✅ TLS configuration built successfully");

    // Connect to NATS with mTLS
    info!("Connecting to NATS with TLS...");
    let _nats_client = async_nats::ConnectOptions::new()
        .name("prompt-helper-mcp-server")
        .tls_client_config(tls_config)
        .connect(&config.nats.url)
        .await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to connect to NATS: {}", e)))?;

    info!("✅ Connected to NATS with TLS");
    info!("");
    info!("Prompt Helper MCP Server ready on {} (TLS enabled)", config.nats.subject);
    info!("Listening for shutdown signal (Ctrl+C)...");

    // Wait for shutdown signal
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| TaleTrailError::ConfigError(format!("Failed to listen for Ctrl+C: {}", e)))?;

    info!("Received Ctrl+C, shutting down gracefully...");
    Ok(())
}

/// Build TLS configuration from certificate files
fn build_tls_config(config: &PromptHelperConfig) -> Result<rustls::ClientConfig> {
    // Load CA certificate
    let ca_cert_file = fs::File::open(&config.nats.tls.ca_cert)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to open CA cert at {}: {}", config.nats.tls.ca_cert, e)))?;
    let mut ca_cert_reader = BufReader::new(ca_cert_file);
    let ca_certs = certs(&mut ca_cert_reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to parse CA cert: {}", e)))?;

    // Load client certificate
    let client_cert_file = fs::File::open(&config.nats.tls.client_cert)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to open client cert at {}: {}", config.nats.tls.client_cert, e)))?;
    let mut client_cert_reader = BufReader::new(client_cert_file);
    let client_certs = certs(&mut client_cert_reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to parse client cert: {}", e)))?;

    // Load client private key
    let client_key_file = fs::File::open(&config.nats.tls.client_key)
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to open client key at {}: {}", config.nats.tls.client_key, e)))?;
    let mut client_key_reader = BufReader::new(client_key_file);
    let mut client_keys = pkcs8_private_keys(&mut client_key_reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to parse client key: {}", e)))?;

    let client_key = client_keys.pop()
        .ok_or_else(|| TaleTrailError::TlsCertificateError("No private key found in client key file".to_string()))?;

    // Build root certificate store
    let mut root_cert_store = rustls::RootCertStore::empty();
    for ca_cert in ca_certs {
        root_cert_store.add(ca_cert)
            .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to add CA cert: {:?}", e)))?;
    }

    // Build TLS configuration with client authentication
    let tls_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(client_certs, client_key.into())
        .map_err(|e| TaleTrailError::TlsCertificateError(format!("Failed to build TLS config: {}", e)))?;

    Ok(tls_config)
}
