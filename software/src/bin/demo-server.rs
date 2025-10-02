// ABOUTME: Interactive demo server for cross-language integration testing
// ABOUTME: Features beautiful colored logging and envelope visualization for demonstration purposes

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use colored::*;
use qollective::envelope::{meta::Meta, Envelope, ExtensionsMeta};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Clone)]
struct DemoServerState {
    pub server_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let port = env::var("QOLLECTIVE_PORT")
        .or_else(|_| env::var("TEST_PORT"))
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()?;

    let server_id = format!("rust-demo-server-{}", &Uuid::now_v7().to_string()[..8]);
    let start_time = chrono::Utc::now();

    print_startup_banner(&server_id, port);

    let state = DemoServerState {
        server_id: server_id.clone(),
        start_time,
    };

    // Create router with demo endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/test/echo", post(echo_envelope))
        .route("/test/process", post(process_envelope))
        .with_state(state);

    // Start server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    println!(
        "{}",
        "🚀 Server ready for cross-language testing!"
            .bright_green()
            .bold()
    );
    println!(
        "{}",
        format!(
            "   Listening on: {}",
            format!("http://0.0.0.0:{}", port).bright_cyan()
        )
        .dimmed()
    );
    println!("{}", "   Endpoints:".dimmed());
    println!("{}", "     • GET  /health        - Health check".dimmed());
    println!(
        "{}",
        "     • POST /test/echo     - Echo with metadata".dimmed()
    );
    println!(
        "{}",
        "     • POST /test/process  - Process and transform".dimmed()
    );
    println!();

    axum::serve(listener, app).await?;

    Ok(())
}

fn print_startup_banner(server_id: &str, port: u16) {
    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗".bright_blue()
    );
    println!(
        "{}",
        "║              🦀 QOLLECTIVE RUST DEMO SERVER 🦀               ║"
            .bright_blue()
            .bold()
    );
    println!(
        "{}",
        "╠══════════════════════════════════════════════════════════════╣".bright_blue()
    );
    println!(
        "{}",
        format!("║ Server ID: {:<46} ║", server_id).bright_blue()
    );
    println!("{}", format!("║ Port:      {:<46} ║", port).bright_blue());
    println!(
        "{}",
        format!(
            "║ Time:      {:<46} ║",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
        .bright_blue()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝".bright_blue()
    );
    println!();
}

fn print_request_header(endpoint: &str, method: &str) {
    println!(
        "{}",
        format!(
            "┌─ {} {} {}",
            "📥".bright_yellow(),
            method.bright_white().bold(),
            endpoint.bright_cyan()
        )
        .on_black()
    );
}

fn print_envelope(envelope: &Value, label: &str, is_request: bool) {
    let arrow = if is_request { "📥" } else { "📤" };
    let color = if is_request {
        Color::Yellow
    } else {
        Color::Green
    };

    println!(
        "{}",
        format!("│ {} {} Envelope:", arrow, label)
            .color(color)
            .bold()
    );

    // Pretty print the JSON with indentation
    let pretty_json = serde_json::to_string_pretty(envelope).unwrap_or_default();
    for line in pretty_json.lines() {
        if line.trim().starts_with("\"meta\"") {
            println!("{}", format!("│   {}", line).bright_magenta());
        } else if line.trim().starts_with("\"data\"") {
            println!("{}", format!("│   {}", line).bright_cyan());
        } else if line.trim().starts_with("\"extensions\"") {
            println!("{}", format!("│   {}", line).bright_yellow());
        } else if line.trim().starts_with("\"timestamp\"")
            || line.trim().starts_with("\"request_id\"")
        {
            println!("{}", format!("│   {}", line).bright_white());
        } else {
            println!("{}", format!("│   {}", line).white());
        }
    }
}

fn print_request_footer(duration_ms: u128) {
    println!(
        "{}",
        format!("└─ ⏱️  Processed in {}ms", duration_ms).bright_green()
    );
    println!();
}

/// Health check endpoint
async fn health_check(State(state): State<DemoServerState>) -> Json<Value> {
    let start = std::time::Instant::now();
    print_request_header("/health", "GET");

    let uptime_seconds = chrono::Utc::now()
        .signed_duration_since(state.start_time)
        .num_seconds();

    let response = serde_json::json!({
        "status": "healthy",
        "server_id": state.server_id,
        "language": "rust",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime_seconds": uptime_seconds,
        "version": "1.0.0"
    });

    println!(
        "{}",
        format!("│ 🔍 Health Check Response:").bright_green().bold()
    );
    println!(
        "{}",
        format!("│   Status: {}", "✅ HEALTHY".bright_green().bold())
    );
    println!(
        "{}",
        format!("│   Uptime: {}s", uptime_seconds).bright_white()
    );

    print_request_footer(start.elapsed().as_millis());
    Json(response)
}

/// Echo endpoint that returns the received envelope with added metadata
async fn echo_envelope(
    State(state): State<DemoServerState>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let start = std::time::Instant::now();
    print_request_header("/test/echo", "POST");

    print_envelope(&request, "Incoming", true);

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
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "operation": "echo"
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
                tenant: Some("demo-tenant".to_string()),
                extensions: Some(ExtensionsMeta {
                    sections: {
                        let mut sections = HashMap::new();
                        sections.insert(
                            "processed_by".to_string(),
                            serde_json::json!({
                                "server_id": state.server_id,
                                "language": "rust",
                                "timestamp": chrono::Utc::now().to_rfc3339(),
                                "operation": "echo"
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
        extensions.sections.insert(
            "echo_info".to_string(),
            serde_json::json!({
                "original_preserved": true,
                "metadata_enhanced": true
            }),
        );
    }

    let response_json =
        serde_json::to_value(&response_envelope).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    print_envelope(&response_json, "Outgoing", false);
    print_request_footer(start.elapsed().as_millis());

    Ok(Json(response_json))
}

/// Process endpoint that performs some transformation on the data
async fn process_envelope(
    State(state): State<DemoServerState>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let start = std::time::Instant::now();
    print_request_header("/test/process", "POST");

    print_envelope(&request, "Incoming", true);

    // Parse envelope or create one from raw data
    let (original_meta, data) =
        if let Ok(envelope) = serde_json::from_value::<Envelope<Value>>(request.clone()) {
            (Some(envelope.meta), envelope.payload)
        } else {
            (None, request)
        };

    // Create processed data with transformation
    let processed_data = serde_json::json!({
        "original": data,
        "processing_result": {
            "processed_at": chrono::Utc::now().to_rfc3339(),
            "processed_by": format!("{} (Rust Demo Server)", state.server_id),
            "operation": "process",
            "transformations": [
                "envelope_wrapping",
                "metadata_enrichment",
                "context_propagation",
                "data_transformation"
            ],
            "processing_stats": {
                "data_size_bytes": serde_json::to_string(&data).unwrap_or_default().len(),
                "processing_time_ms": start.elapsed().as_millis()
            }
        }
    });

    // Create response meta, preserving original context where possible
    let mut response_meta = original_meta.unwrap_or_else(|| Meta {
        timestamp: Some(chrono::Utc::now()),
        request_id: Some(Uuid::now_v7()),
        version: Some("1.0.0".to_string()),
        tenant: Some("demo-tenant".to_string()),
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
        sections.insert(
            "processing_info".to_string(),
            serde_json::json!({
                "transformation_applied": true,
                "original_data_preserved": true,
                "enhanced_with_stats": true
            }),
        );
        ExtensionsMeta { sections }
    });

    // Update timestamp to reflect processing time
    response_meta.timestamp = Some(chrono::Utc::now());

    let response_envelope = Envelope::new(response_meta, processed_data);

    let response_json =
        serde_json::to_value(&response_envelope).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    print_envelope(&response_json, "Outgoing", false);
    print_request_footer(start.elapsed().as_millis());

    Ok(Json(response_json))
}
