// ABOUTME: Test server for cross-language integration testing
// ABOUTME: Provides REST endpoints that accept and return Qollective envelopes for testing purposes

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use qollective::envelope::{meta::Meta, Envelope, ExtensionsMeta};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Clone)]
struct TestServerState {
    pub server_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("TEST_PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()?;

    let test_mode = env::var("TEST_MODE").unwrap_or_else(|_| "integration".to_string());

    if test_mode != "integration" {
        println!(
            "Starting Rust test server on port {} in {} mode",
            port, test_mode
        );
    }

    let state = TestServerState {
        server_id: format!("rust-test-server-{}", Uuid::now_v7()),
    };

    // Create router with test endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/test/echo", post(echo_envelope))
        .route("/test/process", post(process_envelope))
        .with_state(state);

    // Start server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    if test_mode != "integration" {
        println!("Rust test server listening on {}", listener.local_addr()?);
    }

    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_check(State(state): State<TestServerState>) -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "server_id": state.server_id,
        "language": "rust",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Echo endpoint that returns the received envelope with added metadata
async fn echo_envelope(
    State(state): State<TestServerState>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // println!("Received request: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    // Try to parse as envelope, but also handle raw data
    let mut response_envelope =
        if let Ok(envelope) = serde_json::from_value::<Envelope<Value>>(request.clone()) {
            // It's already an envelope, add our processing metadata
            let mut new_meta = envelope.meta.clone();
            new_meta.extensions = Some({
                let mut sections = envelope
                    .meta
                    .extensions
                    .map(|ext| ext.sections)
                    .unwrap_or_else(HashMap::new);
                sections.insert(
                    "processed_by".to_string(),
                    serde_json::json!({
                        "server_id": state.server_id,
                        "language": "rust",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }),
                );
                ExtensionsMeta { sections }
            });

            Envelope::new(new_meta, envelope.payload)
        } else {
            // Raw data - wrap in envelope
            let meta = Meta {
                timestamp: Some(chrono::Utc::now()),
                request_id: Some(Uuid::now_v7()),
                version: Some("1.0.0".to_string()),
                tenant: Some("test-tenant".to_string()),
                extensions: Some(ExtensionsMeta {
                    sections: {
                        let mut sections = HashMap::new();
                        sections.insert(
                            "processed_by".to_string(),
                            serde_json::json!({
                                "server_id": state.server_id,
                                "language": "rust",
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }),
                        );
                        sections
                    },
                }),
                ..Default::default()
            };

            Envelope::new(meta, request)
        };

    // Add echo-specific metadata
    if let Some(ref mut extensions) = response_envelope.meta.extensions {
        extensions
            .sections
            .insert("operation".to_string(), serde_json::json!("echo"));
    }

    let response_json =
        serde_json::to_value(&response_envelope).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // println!("Sending response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());

    Ok(Json(response_json))
}

/// Process endpoint that performs some transformation on the data
async fn process_envelope(
    State(state): State<TestServerState>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // println!("Processing request: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    // Parse envelope or create one from raw data
    let (original_meta, data) =
        if let Ok(envelope) = serde_json::from_value::<Envelope<Value>>(request.clone()) {
            (Some(envelope.meta), envelope.payload)
        } else {
            (None, request)
        };

    // Create processed data
    let processed_data = serde_json::json!({
        "original": data,
        "processing_result": {
            "processed_at": chrono::Utc::now().to_rfc3339(),
            "processed_by": format!("{} (Rust)", state.server_id),
            "operation": "process",
            "transformations": [
                "envelope_wrapping",
                "metadata_enrichment",
                "context_propagation"
            ]
        }
    });

    // Create response meta, preserving original context where possible
    let mut response_meta = original_meta.unwrap_or_else(|| Meta {
        timestamp: Some(chrono::Utc::now()),
        request_id: Some(Uuid::now_v7()),
        version: Some("1.0.0".to_string()),
        tenant: Some("test-tenant".to_string()),
        ..Default::default()
    });

    // Add processing metadata
    response_meta.extensions = Some({
        let mut sections = response_meta
            .extensions
            .map(|ext| ext.sections)
            .unwrap_or_else(HashMap::new);
        sections.insert(
            "processed_by".to_string(),
            serde_json::json!({
                "server_id": state.server_id,
                "language": "rust",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        );
        sections.insert("operation".to_string(), serde_json::json!("process"));
        ExtensionsMeta { sections }
    });

    // Update timestamp to reflect processing time
    response_meta.timestamp = Some(chrono::Utc::now());

    let response_envelope = Envelope::new(response_meta, processed_data);

    let response_json =
        serde_json::to_value(&response_envelope).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // println!("Sending processed response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());

    Ok(Json(response_json))
}
