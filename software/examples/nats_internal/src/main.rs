//! NATS Internal Example
//!
//! Demonstrates the standard Qollective NATS pattern using internal
//! client/server with envelope-based requests.

use qollective::config::nats::NatsConfig;
use qollective::envelope::{Envelope, Meta};
use qollective::error::Result;
use qollective::server::nats::NatsServer;
use qollective::server::EnvelopeHandler;
use qollective::client::nats::NatsClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Request payload with multiple fields to demonstrate envelope structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EchoRequest {
    message: String,
    sender_id: String,
}

/// Response payload with multiple fields
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EchoResponse {
    echo: String,
    processed_at: String,
}

/// Handler that echoes back the message
#[derive(Clone)]
struct EchoHandler;

impl EnvelopeHandler<EchoRequest, EchoResponse> for EchoHandler {
    async fn handle(&self, envelope: Envelope<EchoRequest>) -> Result<Envelope<EchoResponse>> {
        tracing::info!("=== Server Handler Received Envelope ===");
        tracing::info!("Envelope (pretty):\n{}",
            serde_json::to_string_pretty(&envelope).unwrap_or_default());

        Ok(Envelope {
            meta: Meta::preserve_for_response(Some(&envelope.meta)),
            payload: EchoResponse {
                echo: format!("Echo: {}", envelope.payload.message),
                processed_at: chrono::Utc::now().to_rfc3339(),
            },
            error: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("nats_internal=info".parse().unwrap())
        )
        .init();

    // Get NATS URL from environment or use default
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:10222".to_string());

    tracing::info!("Connecting to NATS at {}", nats_url);

    // Create config with custom URL
    let config = NatsConfig::builder()
        .with_urls(vec![nats_url])
        .build()
        .expect("Valid config");

    // === SERVER SETUP ===
    tracing::info!("Starting NATS server...");
    let mut server = NatsServer::new(config.clone()).await?;

    // Register handler for echo subject
    let subject = "example.echo";
    server.handle::<EchoRequest, EchoResponse, _>(subject, EchoHandler).await?;

    // Start server (runs in background)
    server.start().await?;
    tracing::info!("Server listening on subject: {}", subject);

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // === CLIENT SETUP ===
    tracing::info!("Creating NATS client...");
    let client = NatsClient::new(config.into()).await?;

    // === SEND REQUEST ===
    tracing::info!("=== Sending Envelope Request ===");

    // Create metadata with request tracking
    let mut meta = Meta::default();
    meta.request_id = Some(uuid::Uuid::new_v4());
    meta.timestamp = Some(chrono::Utc::now());

    let request = Envelope::new(
        meta,
        EchoRequest {
            message: "Hello from internal client!".to_string(),
            sender_id: "example-client-001".to_string(),
        },
    );

    tracing::info!("Request Envelope (pretty):\n{}",
        serde_json::to_string_pretty(&request).unwrap_or_default());

    let response: Envelope<EchoResponse> = client
        .send_envelope(subject, request)
        .await?;

    tracing::info!("=== Received Response Envelope ===");
    tracing::info!("Response Envelope (pretty):\n{}",
        serde_json::to_string_pretty(&response).unwrap_or_default());

    // Cleanup
    server.shutdown().await?;

    Ok(())
}
