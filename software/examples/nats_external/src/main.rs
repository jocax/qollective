//! NATS External Example
//!
//! Demonstrates the new shared connection pattern using `NatsServer::from_client()`
//! and `server.client()` for multi-layer architectures (TaleTrails pattern).

use futures_util::StreamExt;
use qollective::envelope::{Envelope, Meta};
use qollective::error::Result;
use qollective::server::nats::NatsServer;
use qollective::server::EnvelopeHandler;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Request payload for MCP layer
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EchoRequest {
    message: String,
}

/// Response payload for MCP layer
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EchoResponse {
    echo: String,
}

/// Handler that echoes back the message
#[derive(Clone)]
struct EchoHandler;

impl EnvelopeHandler<EchoRequest, EchoResponse> for EchoHandler {
    async fn handle(&self, envelope: Envelope<EchoRequest>) -> Result<Envelope<EchoResponse>> {
        tracing::info!("[MCP Layer] Handler received: {}", envelope.payload.message);

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
                .add_directive("nats_external=info".parse().unwrap())
        )
        .init();

    // Get NATS URL from environment or use default
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:10222".to_string());

    tracing::info!("=== NATS External (Shared Connection) Example ===");
    tracing::info!("Connecting to NATS at {}", nats_url);

    // === SINGLE SHARED CONNECTION ===
    tracing::info!("Creating shared NATS connection...");
    let client = async_nats::connect(&nats_url).await
        .map_err(|e| qollective::error::QollectiveError::nats_connection(e.to_string()))?;

    // === SERVER FROM EXISTING CLIENT ===
    tracing::info!("Creating NatsServer from existing client...");
    let mut server = NatsServer::from_client(client.clone(), None).await?;

    // Register handler for MCP layer
    let mcp_subject = "example.echo";
    server.handle::<EchoRequest, EchoResponse, _>(mcp_subject, EchoHandler).await?;

    // Start server (runs in background)
    server.start().await?;
    tracing::info!("Server listening on subject: {}", mcp_subject);

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // === APPLICATION LAYER: Direct NATS using server.client() ===
    let app_log_subject = "app.logs.service";
    let app_events_subject = "app.events.status";

    // Subscribe to app logs using the original client
    let mut log_subscriber = client.subscribe(app_log_subject.to_string()).await
        .map_err(|e| qollective::error::QollectiveError::nats_message(e.to_string()))?;

    // Publish app log using server.client() - DEMONSTRATES THE NEW PATTERN
    tracing::info!("[App Layer] Publishing log via server.client()...");
    server.client()
        .publish(app_log_subject.to_string(), "Service started successfully".into())
        .await
        .map_err(|e| qollective::error::QollectiveError::nats_message(e.to_string()))?;

    // Publish status event using server.client()
    tracing::info!("[App Layer] Publishing status event via server.client()...");
    server.client()
        .publish(app_events_subject.to_string(), r#"{"status":"ready"}"#.into())
        .await
        .map_err(|e| qollective::error::QollectiveError::nats_message(e.to_string()))?;

    // Receive the log message
    if let Ok(Some(msg)) = tokio::time::timeout(
        Duration::from_secs(2),
        async { log_subscriber.next().await }
    ).await {
        tracing::info!("[App Layer] Received log: {}", String::from_utf8_lossy(&msg.payload));
    }

    // === MCP LAYER: Envelope request via same connection ===
    tracing::info!("[MCP Layer] Sending envelope request via same connection...");

    // Use the shared client for request-reply
    let request_payload = serde_json::to_vec(&Envelope::new(
        Meta::default(),
        EchoRequest {
            message: "Hello from shared connection!".to_string(),
        },
    )).map_err(|e| qollective::error::QollectiveError::serialization(e.to_string()))?;

    let response = client
        .request(mcp_subject.to_string(), request_payload.into())
        .await
        .map_err(|e| qollective::error::QollectiveError::nats_message(e.to_string()))?;

    let response_envelope: Envelope<EchoResponse> = serde_json::from_slice(&response.payload)
        .map_err(|e| qollective::error::QollectiveError::deserialization(e.to_string()))?;

    tracing::info!("[MCP Layer] Received response: {}", response_envelope.payload.echo);

    // === SUMMARY ===
    tracing::info!("=== Summary ===");
    tracing::info!("Connection count: 1 (vs 2 with separate connections)");
    tracing::info!("Both MCP envelope requests and raw app messaging used the SAME connection");

    // Cleanup
    server.shutdown().await?;

    Ok(())
}
