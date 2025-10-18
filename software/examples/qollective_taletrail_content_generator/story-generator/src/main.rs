//! Story Generator MCP Server with Qollective Infrastructure
//!
//! This server provides Model Context Protocol (MCP) tools for generating
//! branching narrative DAG structures and content using Qollective's
//! envelope-first architecture and NatsServer infrastructure.
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
//! 3. Calls `StoryGeneratorHandler::handle(envelope)`
//! 4. Handler extracts `CallToolRequest` from envelope
//! 5. Routes to tool handler (generate_structure, generate_nodes, validate_paths)
//! 6. Wraps `CallToolResult` in response envelope
//! 7. NatsServer auto-encodes and publishes to reply subject
//!
//! # Configuration
//!
//! Loaded from `config.toml` with environment variable overrides:
//! - NATS connection (URL, subject, queue group)
//! - TLS certificates (CA, client cert, client key) or NKey authentication
//! - LLM service (base URL, model)
//! - Generation settings (timeout, batch sizes, node counts)

use qollective::server::nats::NatsServer;
use qollective::config::nats::{NatsConfig, NatsConnectionConfig};
use qollective::config::tls::TlsConfig as QollectiveTlsConfig;
use tracing::info;

mod config;
mod discovery;
mod envelope_handlers;
mod tool_handlers;
mod llm;
mod mcp_tools;
mod prompts;
mod server;
mod structure;

use config::StoryGeneratorConfig;
use llm::StoryLlmClient;
use envelope_handlers::StoryGeneratorHandler;
use discovery::{DiscoveryHandler, HealthHandler};
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
    let app_config = StoryGeneratorConfig::load()
        .map_err(|e| TaleTrailError::ConfigError(format!("Failed to load config: {}", e)))?;

    info!("=== Story Generator MCP Server Starting ===");
    info!("Configuration:");
    info!("  NATS URL: {}", app_config.nats.url);
    info!("  NATS Subject: {}", app_config.nats.subject);
    info!("  NATS Queue Group: {}", app_config.nats.queue_group);
    info!("  LLM Provider: {:?}", app_config.llm.provider.provider_type);
    info!("  LLM URL: {}", app_config.llm.provider.url);
    info!("  LLM Default Model: {}", app_config.llm.provider.default_model);
    info!("  Batch Size: {}-{}", app_config.generation.batch_size_min, app_config.generation.batch_size_max);
    info!("");

    // Create LLM client
    let llm_client = StoryLlmClient::new(app_config.llm.clone())?;
    info!("✅ Created LLM client with provider: {:?}", app_config.llm.provider.provider_type);

    // Create Qollective NATS config with NKey authentication
    let nats_config = NatsConfig {
        connection: NatsConnectionConfig {
            urls: vec![app_config.nats.url.clone()],
            // Use Qollective's native NKey support with priority: nkey_seed > nkey_file
            nkey_file: if app_config.nats.auth.nkey_seed.is_some() {
                None  // If seed is set, ignore file
            } else {
                app_config.nats.auth.nkey_file.as_ref().map(|p| p.into())
            },
            nkey_seed: app_config.nats.auth.nkey_seed.clone(),
            tls: QollectiveTlsConfig {
                enabled: true,
                ca_cert_path: Some(app_config.nats.tls.ca_cert.clone().into()),
                cert_path: None, // No client cert needed with NKey
                key_path: None,  // No client key needed with NKey
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
    let handler = StoryGeneratorHandler::new(app_config.clone(), llm_client);
    info!("✅ Created StoryGeneratorHandler with envelope support");

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

    // Subscribe to discovery endpoint
    let discovery_subject = format!("{}.{}", MCP_DISCOVERY_LIST_TOOLS, "story-generator");
    let discovery_handler = DiscoveryHandler::new();
    nats_server.subscribe_queue_group(
        &discovery_subject,
        &app_config.nats.queue_group,
        discovery_handler,
    ).await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to subscribe to discovery: {}", e)))?;
    info!("✅ Subscribed to discovery endpoint: '{}'", discovery_subject);

    // Subscribe to health endpoint
    let health_subject = format!("{}.{}", MCP_DISCOVERY_HEALTH, "story-generator");
    let health_handler = HealthHandler::new();
    nats_server.subscribe_queue_group(
        &health_subject,
        &app_config.nats.queue_group,
        health_handler,
    ).await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to subscribe to health: {}", e)))?;
    info!("✅ Subscribed to health endpoint: '{}'", health_subject);

    // Start processing messages
    nats_server.start().await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to start NATS server: {}", e)))?;
    info!("");
    info!("Story Generator MCP Server is running. Press Ctrl+C to shutdown.");
    info!("Architecture: Envelope-first with NatsServer infrastructure");
    info!("Tools available: generate_structure, generate_nodes, validate_paths");
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
