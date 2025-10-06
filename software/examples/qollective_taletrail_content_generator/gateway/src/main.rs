//! TaleTrail Gateway - Qollective REST Server

use qollective::{
    server::rest::RestServer,
    prelude::UnifiedEnvelopeReceiver,
};
use shared_types::*;
use tracing::info;
use tokio::signal;

mod config;
mod routes;

use config::get_gateway_config;
use routes::HealthHandler;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    info!("=== TaleTrail Gateway Starting ===");
    info!("Using Qollective REST Server with envelope-first architecture");
    info!("");

    // Create REST server with TLS configuration
    let config = get_gateway_config()?;
    info!("Configuration:");
    info!("  Bind Address: {}", config.base.bind_address);
    info!("  Port: {} (HTTPS/TLS)", config.base.port);
    info!("  TLS Enabled: {}", config.tls.is_some());
    if let Some(ref tls) = config.tls {
        info!("  TLS Certificate: ./certs/gateway-cert.pem");
        info!("  TLS Key: ./certs/gateway-key.pem");
    }
    info!("");

    let mut server = RestServer::new(config).await
        .map_err(|e| TaleTrailError::QollectiveError(format!("Failed to create REST server: {}", e)))?;

    info!("📝 Registering REST endpoints...");

    // Register health endpoint
    let health_handler = HealthHandler;
    server.receive_envelope_at("/health", health_handler).await
        .map_err(|e| TaleTrailError::QollectiveError(format!("Failed to register /health: {}", e)))?;

    info!("✅ Registered {} REST endpoint(s)", server.route_count());
    info!("");

    // Display server information
    let protocol = if server.config().tls.is_some() { "https" } else { "http" };
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                🎯 TALETRAIL GATEWAY (QOLLECTIVE + TLS)  🎯                  ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  🔒 Endpoint: {}://{}:{}                                    ║",
             protocol,
             server.config().base.bind_address,
             server.config().base.port);
    println!("║                                                                              ║");
    println!("║  Available Endpoints:                                                        ║");
    println!("║    POST /health            - Health check (envelope-wrapped)                 ║");
    println!("║                                                                              ║");
    println!("║  🔧 Architecture: Envelope-first with UnifiedEnvelope wrapping               ║");
    println!("║  📦 All responses include metadata (trace_id, timestamp, etc.)               ║");
    println!("║  🔒 TLS: Per-service certificates (CN=taletrail-gateway)                     ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    info!("🔧 Press Ctrl+C to stop server");
    info!("");

    // Start server and wait for shutdown
    tokio::select! {
        result = server.start() => {
            if let Err(e) = result {
                return Err(TaleTrailError::NetworkError(format!("Server failed: {}", e)));
            }
        }
        _ = signal::ctrl_c() => {
            info!("🛑 Shutdown signal received, stopping server...");
        }
    }

    info!("✅ Gateway stopped successfully");
    Ok(())
}
