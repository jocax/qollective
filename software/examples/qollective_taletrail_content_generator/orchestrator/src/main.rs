//! TaleTrail Orchestrator Service
//!
//! Main service entry point that listens for generation requests via NATS
//! and orchestrates the complete content generation pipeline.

use futures::stream::StreamExt;
use orchestrator::{Orchestrator, OrchestratorConfig};
use shared_types::*;
use tracing::{error, info, warn};

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

    // Connect to NATS with NKey authentication
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

    info!("Connecting to NATS at {} with NKey authentication and TLS...", config.nats.url);
    let nats_client = connect_options
        .connect(&config.nats.url)
        .await
        .map_err(|e| TaleTrailError::NatsError(format!("Failed to connect to NATS: {}", e)))?;

    info!("✅ Connected to NATS with NKey authentication");

    // Create orchestrator
    let orchestrator = std::sync::Arc::new(Orchestrator::new(
        std::sync::Arc::new(nats_client.clone()),
        config.clone(),
    ));

    // Subscribe to orchestrator requests
    let mut subscriber = nats_client
        .subscribe(config.nats.subject.clone())
        .await
        .map_err(|e| TaleTrailError::NatsError(e.to_string()))?;

    info!("✅ Listening on {}", config.nats.subject);
    info!("Orchestrator ready - waiting for requests...");

    // Handle requests
    loop {
        tokio::select! {
            Some(message) = subscriber.next() => {
                let orch = orchestrator.clone();
                let nats = nats_client.clone();

                tokio::spawn(async move {
                    // Deserialize request
                    let request: GenerationRequest = match serde_json::from_slice(&message.payload) {
                        Ok(req) => req,
                        Err(e) => {
                            error!("Failed to deserialize request: {}", e);
                            return;
                        }
                    };

                    info!("Received generation request for theme: {}", request.theme);

                    // Execute orchestration
                    match orch.orchestrate_generation(request).await {
                        Ok(response) => {
                            // Send response
                            if let Some(reply) = message.reply {
                                let response_bytes = match serde_json::to_vec(&response) {
                                    Ok(bytes) => bytes,
                                    Err(e) => {
                                        error!("Failed to serialize response: {}", e);
                                        return;
                                    }
                                };

                                if let Err(e) = nats.publish(reply, response_bytes.into()).await {
                                    error!("Failed to send response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Orchestration failed: {}", e);
                        }
                    }
                });
            }
            _ = tokio::signal::ctrl_c() => {
                warn!("Received Ctrl+C, shutting down gracefully...");
                break;
            }
        }
    }

    Ok(())
}
