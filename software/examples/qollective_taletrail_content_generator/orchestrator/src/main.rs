//! TaleTrail Orchestrator Service
//!
//! Main service entry point that listens for generation requests via NATS
//! and orchestrates the complete content generation pipeline.

use orchestrator::{Orchestrator, OrchestratorConfig, OrchestratorHandler};
use shared_types::*;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config = OrchestratorConfig::load()?;

    info!("=== TaleTrail Orchestrator Starting ===");
    info!("Configuration:");
    info!("  NATS URL: {}", config.nats.url);
    info!("  NATS Subject: {}", config.nats.subject);
    info!("  LLM Provider: {}", config.llm.provider.provider_type);
    info!("  LLM URL: {}", config.llm.provider.url);
    info!("  LLM Model: {}", config.llm.provider.default_model);
    info!(
        "  Generation timeout: {}s",
        config.pipeline.generation_timeout_secs
    );
    info!(
        "  Validation timeout: {}s",
        config.pipeline.validation_timeout_secs
    );
    info!(
        "  Retry max attempts: {}",
        config.pipeline.retry_max_attempts
    );
    info!(
        "  Batch size: {}-{}",
        config.batch.size_min, config.batch.size_max
    );
    info!("  Concurrent batches: {}", config.batch.concurrent_batches);
    info!("  Default node count: {}", config.dag.default_node_count);
    info!(
        "  Convergence ratio: {}",
        config.dag.convergence_point_ratio
    );
    info!(
        "  Max negotiation rounds: {}",
        config.negotiation.max_rounds
    );
    info!("");

    // Initialize rustls crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Connect to NATS with NKey authentication (needed for both server and orchestrator)
    let mut connect_options = async_nats::ConnectOptions::new();

    // Configure NKey authentication
    if let Some(nkey_seed) = &config.nats.auth.nkey_seed {
        info!("Using NKey authentication from seed");
        connect_options = connect_options.nkey(nkey_seed.clone());
    } else if let Some(nkey_file) = &config.nats.auth.nkey_file {
        info!("Using NKey authentication from file: {}", nkey_file);
        let nkey_seed = std::fs::read_to_string(nkey_file)
            .map_err(|e| TaleTrailError::ConfigError(format!("Failed to read NKey file {}: {}", nkey_file, e)))?;
        connect_options = connect_options.nkey(nkey_seed.trim().to_string());
    }

    // Configure TLS
    info!("Configuring TLS with CA cert: {}", config.nats.tls.ca_cert);
    let ca_cert = std::fs::read(&config.nats.tls.ca_cert)
        .map_err(|e| TaleTrailError::ConfigError(format!("Failed to read CA cert: {}", e)))?;

    let root_cert_store = {
        let mut store = rustls::RootCertStore::empty();
        let certs: Vec<_> = rustls_pemfile::certs(&mut ca_cert.as_slice())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| TaleTrailError::ConfigError(format!("Failed to parse CA cert: {}", e)))?;
        for cert in certs {
            store.add(cert)
                .map_err(|e| TaleTrailError::ConfigError(format!("Failed to add CA cert to store: {}", e)))?;
        }
        store
    };

    let tls_client = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    connect_options = connect_options.tls_client_config(tls_client);

    // Configure request timeout for long-running MCP operations
    connect_options = connect_options.request_timeout(Some(
        std::time::Duration::from_secs(config.pipeline.generation_timeout_secs)
    ));

    info!("Connecting to NATS at {} with NKey authentication and TLS...", config.nats.url);
    let nats_client = connect_options
        .connect(&config.nats.url)
        .await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to connect to NATS: {}", e)))?;

    info!("✅ Connected to NATS with NKey authentication and TLS");

    // Create orchestrator with NATS client (orchestrator needs to call other MCP services)
    let orchestrator = std::sync::Arc::new(Orchestrator::new(
        std::sync::Arc::new(nats_client.clone()),
        config.clone(),
    ));
    info!("✅ Created Orchestrator with envelope support");

    // Create handler
    let handler = OrchestratorHandler::new(orchestrator);
    info!("✅ Created OrchestratorHandler with envelope support");

    // Subscribe to orchestrator requests using queue group
    let mut subscriber = nats_client
        .queue_subscribe(config.nats.subject.clone(), config.nats.queue_group.clone())
        .await
        .map_err(|e| TaleTrailError::NatsError(e.to_string()))?;

    info!("✅ Subscribed to '{}' with queue group '{}'",
        config.nats.subject, config.nats.queue_group);
    info!("   Automatic envelope decoding/encoding enabled");
    info!("");
    info!("Orchestrator MCP Server is running. Press Ctrl+C to shutdown.");
    info!("Architecture: Envelope-first with queue group load balancing");
    info!("Tool available: orchestrate_generation");
    info!("");

    // Handle requests using envelope-first pattern
    use qollective::server::EnvelopeHandler;
    use qollective::envelope::nats_codec::NatsEnvelopeCodec;
    use qollective::types::mcp::McpData;
    use qollective::envelope::Envelope;
    use futures::stream::StreamExt;

    loop {
        tokio::select! {
            Some(message) = subscriber.next() => {
                let handler_clone = handler.clone();
                let nats = nats_client.clone();

                tokio::spawn(async move {
                    // Decode envelope from NATS message
                    let envelope: Envelope<McpData> = match NatsEnvelopeCodec::decode(&message.payload) {
                        Ok(env) => env,
                        Err(e) => {
                            tracing::error!("Failed to decode envelope: {}", e);
                            return;
                        }
                    };

                    // Process with handler
                    match handler_clone.handle(envelope).await {
                        Ok(response_envelope) => {
                            // Encode response envelope
                            match NatsEnvelopeCodec::encode(&response_envelope) {
                                Ok(response_bytes) => {
                                    // Send response if reply subject exists
                                    if let Some(reply) = message.reply {
                                        if let Err(e) = nats.publish(reply, response_bytes.into()).await {
                                            tracing::error!("Failed to send response: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to encode response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Handler error: {}", e);
                        }
                    }
                });
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received shutdown signal");
                break;
            }
        }
    }

    info!("✅ Shutdown complete");

    Ok(())
}
