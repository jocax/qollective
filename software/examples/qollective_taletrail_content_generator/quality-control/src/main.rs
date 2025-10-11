//! Quality Control MCP Server with Qollective Infrastructure
//!
//! This server provides Model Context Protocol (MCP) tools for validating
//! narrative content against age appropriateness, safety, and educational rubrics
//! using Qollective's envelope-first architecture and NatsServer infrastructure.
//!
//! # Architecture
//!
//! - **Envelope-First**: All requests/responses wrapped in `Envelope<McpData>`
//! - **NATS Infrastructure**: Uses `qollective::server::nats::NatsServer`
//! - **Queue Groups**: Automatic load balancing across multiple instances
//! - **TLS Security**: mTLS or NKey authentication with NATS server
//! - **Tenant Isolation**: Tenant ID tracked in envelope metadata
//! - **Distributed Tracing**: Request and trace IDs propagated
//!
//! # Message Flow
//!
//! 1. NatsServer receives NATS message on subject
//! 2. Auto-decodes to `Envelope<McpData>`
//! 3. Calls `QualityControlHandler::handle(envelope)`
//! 4. Handler extracts `CallToolRequest` from envelope
//! 5. Routes to tool handler (validate_content, batch_validate)
//! 6. Wraps `CallToolResult` in response envelope
//! 7. NatsServer auto-encodes and publishes to reply subject
//!
//! # Configuration
//!
//! Loaded from `config.toml` with environment variable overrides:
//! - NATS connection (URL, subject, queue group)
//! - TLS certificates (CA, client cert, client key) or NKey authentication
//! - Validation settings (thresholds, timeouts)
//! - Rubrics (age-specific criteria)
//! - Safety keywords and educational criteria

use qollective::server::nats::NatsServer;
use qollective::config::nats::{NatsConfig, NatsConnectionConfig};
use qollective::config::tls::TlsConfig as QollectiveTlsConfig;
use tracing::info;

use quality_control::config::QualityControlConfig;
use quality_control::QualityControlHandler;
use shared_types::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Load application config
    let app_config = QualityControlConfig::load()
        .map_err(|e| TaleTrailError::ConfigError(format!("Failed to load config: {}", e)))?;

    info!("=== Quality Control MCP Server Starting ===");
    info!("Configuration:");
    info!("  NATS URL: {}", app_config.nats.url);
    info!("  NATS Subject: {}", app_config.nats.subject);
    info!("  NATS Queue Group: {}", app_config.nats.queue_group);
    info!("  Min Quality Score: {}", app_config.validation.min_quality_score);
    info!("  Timeout: {}s", app_config.validation.timeout_secs);
    info!("");

    // Create Qollective NATS config with TLS
    let nats_config = NatsConfig {
        connection: NatsConnectionConfig {
            urls: vec![app_config.nats.url.clone()],
            tls: QollectiveTlsConfig {
                enabled: true,
                ca_cert_path: Some(app_config.nats.tls.ca_cert.clone().into()),
                cert_path: Some(app_config.nats.tls.client_cert.clone().into()),
                key_path: Some(app_config.nats.tls.client_key.clone().into()),
                verification_mode: qollective::config::tls::VerificationMode::CustomCa,
            },
            crypto_provider_strategy: Some(
                qollective::crypto::CryptoProviderStrategy::Skip
            ),
            ..Default::default()
        },
        ..Default::default()
    };

    // Create NATS server using Qollective infrastructure
    let mut nats_server = NatsServer::new(nats_config).await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to create NATS server: {}", e)))?;
    info!("✅ Connected to NATS at {} with TLS", app_config.nats.url);

    // Create handler
    let handler = QualityControlHandler::new(app_config.clone());
    info!("✅ Created QualityControlHandler with envelope support");

    // Subscribe to subject with queue group
    nats_server.subscribe_queue_group(
        &app_config.nats.subject,
        &app_config.nats.queue_group,
        handler,
    ).await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to subscribe: {}", e)))?;
    info!("✅ Subscribed to '{}' with queue group '{}'",
        app_config.nats.subject, app_config.nats.queue_group);
    info!("   Automatic envelope decoding/encoding enabled");

    // Start processing messages
    nats_server.start().await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to start NATS server: {}", e)))?;
    info!("");
    info!("Quality Control MCP Server is running. Press Ctrl+C to shutdown.");
    info!("Architecture: Envelope-first with NatsServer infrastructure");
    info!("Tools available: validate_content, batch_validate");
    info!("");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to listen for Ctrl+C: {}", e)))?;
    info!("Received shutdown signal");

    // Graceful shutdown
    nats_server.shutdown().await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to shutdown: {}", e)))?;
    info!("✅ Shutdown complete");

    Ok(())
}
