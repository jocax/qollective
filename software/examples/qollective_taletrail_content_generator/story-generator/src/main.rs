//! Story Generator MCP Server
//!
//! TLS-enabled NATS-based MCP server for generating DAG structures and narrative content.

use anyhow::Result;
use async_nats::ConnectOptions;
use futures::StreamExt;
use rustls::ClientConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use tokio::signal;
use tracing::{info, warn, error};
use tracing_subscriber::EnvFilter;

mod config;
mod server;

use config::StoryGeneratorConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    info!("🚀 TaleTrail Story Generator MCP Server starting...");

    // Load configuration
    let config = StoryGeneratorConfig::load()?;
    info!("📋 Configuration loaded successfully");
    info!("   NATS URL: {}", config.nats.url);
    info!("   Queue Group: {}", config.nats.queue_group);
    info!("   Subject: {}", config.nats.subject);

    // Build TLS configuration
    info!("🔐 Configuring TLS...");
    let tls_config = build_tls_config(&config)?;

    // Connect to NATS with TLS
    info!("🔌 Connecting to NATS at {} (TLS enabled)...", config.nats.url);
    let client = ConnectOptions::new()
        .name("story-generator-mcp-server")
        .tls_client_config(tls_config)
        .connect(&config.nats.url)
        .await
        .map_err(|e| {
            error!("Failed to connect to NATS: {}", e);
            anyhow::anyhow!("NATS connection failed: {}", e)
        })?;

    info!("✅ Connected to NATS successfully");

    // Subscribe to story generation subject with queue group
    let subscriber = client
        .queue_subscribe(config.nats.subject.clone(), config.nats.queue_group.clone())
        .await
        .map_err(|e| {
            error!("Failed to subscribe to subject: {}", e);
            anyhow::anyhow!("NATS subscription failed: {}", e)
        })?;

    info!("👂 Story Generator MCP Server ready on {} (TLS enabled)", config.nats.subject);
    info!("⏳ Waiting for generation requests...");

    // Set up graceful shutdown
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        warn!("🛑 Shutdown signal received");
    };

    // Listen for messages (stub - just acknowledge for now)
    tokio::select! {
        _ = listen_for_messages(subscriber) => {
            error!("Message listener terminated unexpectedly");
        }
        _ = shutdown_signal => {
            info!("📴 Shutting down gracefully...");
        }
    }

    info!("👋 Story Generator MCP Server stopped");
    Ok(())
}

/// Build TLS configuration from certificate files
fn build_tls_config(config: &StoryGeneratorConfig) -> Result<ClientConfig> {
    // Load CA certificate
    let ca_cert_file = File::open(&config.nats.tls.ca_cert)
        .map_err(|e| anyhow::anyhow!("Failed to open CA cert at {}: {}", config.nats.tls.ca_cert, e))?;
    let mut ca_cert_reader = BufReader::new(ca_cert_file);
    let ca_certs = certs(&mut ca_cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Failed to parse CA cert: {}", e))?;

    // Load client certificate
    let client_cert_file = File::open(&config.nats.tls.client_cert)
        .map_err(|e| anyhow::anyhow!("Failed to open client cert at {}: {}", config.nats.tls.client_cert, e))?;
    let mut client_cert_reader = BufReader::new(client_cert_file);
    let client_certs = certs(&mut client_cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Failed to parse client cert: {}", e))?;

    // Load client private key
    let client_key_file = File::open(&config.nats.tls.client_key)
        .map_err(|e| anyhow::anyhow!("Failed to open client key at {}: {}", config.nats.tls.client_key, e))?;
    let mut client_key_reader = BufReader::new(client_key_file);
    let mut client_keys = pkcs8_private_keys(&mut client_key_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Failed to parse client key: {}", e))?;

    let client_key = client_keys.pop()
        .ok_or_else(|| anyhow::anyhow!("No private key found in client key file"))?;

    // Build root certificate store
    let mut root_cert_store = rustls::RootCertStore::empty();
    for ca_cert in ca_certs {
        root_cert_store.add(ca_cert)
            .map_err(|e| anyhow::anyhow!("Failed to add CA cert: {:?}", e))?;
    }

    // Build TLS configuration with client authentication
    let tls_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(client_certs, client_key.into())
        .map_err(|e| anyhow::anyhow!("Failed to build TLS config: {}", e))?;

    Ok(tls_config)
}

/// Listen for incoming NATS messages (stub implementation)
async fn listen_for_messages(mut subscriber: async_nats::Subscriber) -> Result<()> {
    while let Some(message) = subscriber.next().await {
        info!("📨 Received message on subject: {}", message.subject);

        // Stub: Just log the message for Phase 0
        if let Ok(payload) = std::str::from_utf8(&message.payload) {
            info!("   Payload preview: {}...", &payload[..payload.len().min(100)]);
        }

        // In Phase 2, this will:
        // 1. Deserialize MCP request from envelope
        // 2. Execute appropriate tool (generate_structure, generate_nodes, validate_paths)
        // 3. Serialize response and send back
    }

    Ok(())
}
