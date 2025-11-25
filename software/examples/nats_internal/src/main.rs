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

/// Request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EchoRequest {
    message: String,
}

/// Response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EchoResponse {
    echo: String,
}

/// Handler that echoes back the message
#[derive(Clone)]
struct EchoHandler;

impl EnvelopeHandler<EchoRequest, EchoResponse> for EchoHandler {
    async fn handle(&self, envelope: Envelope<EchoRequest>) -> Result<Envelope<EchoResponse>> {
        tracing::info!("Handler received: {}", envelope.payload.message);

        Ok(Envelope {
            meta: Meta::preserve_for_response(Some(&envelope.meta)),
            payload: EchoResponse {
                echo: format!("Echo: {}", envelope.payload.message),
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
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());

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
    tracing::info!("Sending envelope request...");

    let request = Envelope::new(
        Meta::default(),
        EchoRequest {
            message: "Hello from internal client!".to_string(),
        },
    );

    let response: Envelope<EchoResponse> = client
        .send_envelope(subject, request)
        .await?;

    tracing::info!("Received response: {}", response.payload.echo);

    // Cleanup
    server.shutdown().await?;

    Ok(())
}
