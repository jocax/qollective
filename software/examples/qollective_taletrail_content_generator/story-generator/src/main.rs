//! Story Generator MCP Server
//!
//! TLS-enabled NATS-based MCP server for generating DAG structures and narrative content.

use anyhow::Result;
use async_nats::ConnectOptions;
use futures::StreamExt;
use rustls::ClientConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use shared_types::constants::*;
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
    let _config = StoryGeneratorConfig::load()?;
    info!("📋 Configuration loaded successfully");
    info!("   NATS URL: {}", *NATS_URL);
    info!("   Queue Group: {}", STORY_GENERATOR_GROUP);
    info!("   Subject: {}", MCP_STORY_GENERATE);

    // Build TLS configuration
    info!("🔐 Configuring TLS...");
    let tls_config = build_tls_config()?;

    // Connect to NATS with TLS
    info!("🔌 Connecting to NATS at {} (TLS enabled)...", *NATS_URL);
    let client = ConnectOptions::new()
        .name("story-generator-mcp-server")
        .tls_client_config(tls_config)
        .connect(&*NATS_URL)
        .await
        .map_err(|e| {
            error!("Failed to connect to NATS: {}", e);
            anyhow::anyhow!("NATS connection failed: {}", e)
        })?;

    info!("✅ Connected to NATS successfully");

    // Subscribe to story generation subject with queue group
    let subscriber = client
        .queue_subscribe(MCP_STORY_GENERATE.to_string(), STORY_GENERATOR_GROUP.to_string())
        .await
        .map_err(|e| {
            error!("Failed to subscribe to {}: {}", MCP_STORY_GENERATE, e);
            anyhow::anyhow!("NATS subscription failed: {}", e)
        })?;

    info!("👂 Story Generator MCP Server ready on {} (TLS enabled)", MCP_STORY_GENERATE);
    info!("📦 Queue Group: {}", STORY_GENERATOR_GROUP);
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
fn build_tls_config() -> Result<ClientConfig> {
    // Load CA certificate
    let ca_cert_file = File::open(&*NATS_TLS_CA_CERT_PATH)
        .map_err(|e| anyhow::anyhow!("Failed to open CA cert at {}: {}", *NATS_TLS_CA_CERT_PATH, e))?;
    let mut ca_cert_reader = BufReader::new(ca_cert_file);
    let ca_certs = certs(&mut ca_cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Failed to parse CA cert: {}", e))?;

    // Load client certificate
    let client_cert_file = File::open(&*NATS_TLS_CLIENT_CERT_PATH)
        .map_err(|e| anyhow::anyhow!("Failed to open client cert at {}: {}", *NATS_TLS_CLIENT_CERT_PATH, e))?;
    let mut client_cert_reader = BufReader::new(client_cert_file);
    let client_certs = certs(&mut client_cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Failed to parse client cert: {}", e))?;

    // Load client private key
    let client_key_file = File::open(&*NATS_TLS_CLIENT_KEY_PATH)
        .map_err(|e| anyhow::anyhow!("Failed to open client key at {}: {}", *NATS_TLS_CLIENT_KEY_PATH, e))?;
    let mut client_key_reader = BufReader::new(client_key_file);
    let mut client_keys = pkcs8_private_keys(&mut client_key_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Failed to parse client key: {}", e))?;

    if client_keys.is_empty() {
        return Err(anyhow::anyhow!("No private keys found in {}", *NATS_TLS_CLIENT_KEY_PATH));
    }

    // Build root certificate store
    let mut root_cert_store = rustls::RootCertStore::empty();
    for cert in ca_certs {
        root_cert_store
            .add(cert)
            .map_err(|e| anyhow::anyhow!("Failed to add CA cert to root store: {}", e))?;
    }

    // Build TLS configuration
    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(client_certs, client_keys.remove(0).into())
        .map_err(|e| anyhow::anyhow!("Failed to build TLS config: {}", e))?;

    Ok(config)
}

/// Listen for incoming NATS messages (stub implementation)
async fn listen_for_messages(mut subscriber: async_nats::Subscriber) {
    while let Some(message) = subscriber.next().await {
        info!("📨 Received message on {}", message.subject);
        info!("   Payload size: {} bytes", message.payload.len());
        
        // Stub: Just acknowledge receipt for now
        // TODO: Implement actual MCP request handling in Phase 2
        
        if let Some(reply) = message.reply {
            info!("   Reply subject: {}", reply);
            // We would send response here in actual implementation
        }
    }
}
