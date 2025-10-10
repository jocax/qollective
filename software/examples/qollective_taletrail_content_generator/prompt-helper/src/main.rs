//! Prompt Helper MCP Server with Qollective Infrastructure
//!
//! This server provides Model Context Protocol (MCP) tools for generating
//! prompts in the TaleTrail content generation pipeline using Qollective's
//! envelope-first architecture and NatsServer infrastructure.
//!
//! # Architecture
//!
//! - **Envelope-First**: All requests/responses wrapped in `Envelope<McpData>`
//! - **NATS Infrastructure**: Uses `qollective::server::nats::NatsServer`
//! - **Queue Groups**: Automatic load balancing across multiple instances
//! - **TLS Security**: mTLS authentication with NATS server
//! - **Tenant Isolation**: Tenant ID tracked in envelope metadata
//! - **Distributed Tracing**: Request and trace IDs propagated
//!
//! # Message Flow
//!
//! 1. NatsServer receives NATS message on subject
//! 2. Auto-decodes to `Envelope<McpData>`
//! 3. Calls `PromptHelperHandler::handle(envelope)`
//! 4. Handler extracts `CallToolRequest` from envelope
//! 5. Routes to tool handler (generate_story_prompts, etc.)
//! 6. Wraps `CallToolResult` in response envelope
//! 7. NatsServer auto-encodes and publishes to reply subject
//!
//! # Configuration
//!
//! Loaded from `config.toml` with environment variable overrides:
//! - NATS connection (URL, subject, queue group)
//! - TLS certificates (CA, client cert, client key)
//! - LLM service (base URL, model)
//! - Prompt generation (languages, models, educational config)

use qollective::server::nats::NatsServer;
use qollective::config::nats::{NatsConfig, NatsConnectionConfig};
use qollective::config::tls::TlsConfig as QollectiveTlsConfig;
use tracing::info;

mod config;
mod tool_handlers;
mod llm;
mod mcp_tools;
mod server;
mod templates;
mod envelope_handlers;

use config::PromptHelperConfig;
use llm::SharedLlmService;
use envelope_handlers::PromptHelperHandler;
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
    let app_config = PromptHelperConfig::load()?;

    info!("=== Prompt Helper MCP Server Starting ===");
    info!("Configuration:");
    info!("  NATS URL: {}", app_config.nats.url);
    info!("  NATS Subject: {}", app_config.nats.subject);
    info!("  NATS Queue Group: {}", app_config.nats.queue_group);
    info!("  LLM Provider: {:?}", app_config.llm.provider.provider_type);
    info!("  LLM URL: {}", app_config.llm.provider.url);
    info!("  LLM Default Model: {}", app_config.llm.provider.default_model);
    info!("  Supported Languages: {:?}", app_config.prompt.supported_languages);
    info!("");

    // Create LLM service
    let llm_service = SharedLlmService::new(app_config.llm.clone())?;
    info!("✅ Created LLM service with provider: {:?}", app_config.llm.provider.provider_type);

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
    let handler = PromptHelperHandler::new(app_config.clone(), llm_service);
    info!("✅ Created PromptHelperHandler with envelope support");

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
    info!("Prompt Helper MCP Server is running. Press Ctrl+C to shutdown.");
    info!("Architecture: Envelope-first with NatsServer infrastructure");
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
