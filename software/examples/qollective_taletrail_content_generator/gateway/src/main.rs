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
mod nats_client;
mod orchestrator_client;

use config::{get_gateway_config, GatewayConfig};
use routes::HealthHandler;
use nats_client::connect_nats_with_tls;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize crypto provider ONCE for both HTTPS server and NATS client TLS
    // This must be called before any TLS operations (REST server or NATS client)
    qollective::ensure_crypto_provider()
        .map_err(|e| TaleTrailError::ConfigError(format!("Failed to init crypto provider: {}", e)))?;

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    info!("=== TaleTrail Gateway Starting ===");
    info!("Using Qollective REST Server with envelope-first architecture");
    info!("");

    // Load full gateway configuration (HTTP + NATS)
    let config = GatewayConfig::load()?;

    info!("Configuration:");
    info!("  Gateway: {} v{}", config.gateway.name, config.gateway.version);
    info!("");

    info!("HTTP Server Configuration:");
    info!("  Bind Address: {}", config.http.bind_address);
    info!("  Port: {} (HTTPS/TLS)", config.http.port);
    info!("  TLS Enabled: {}", config.http.tls.enabled);
    info!("  TLS Certificate: {}", config.http.tls.cert);
    info!("  TLS Key: {}", config.http.tls.key);
    info!("");

    info!("NATS Client Configuration:");
    info!("  NATS URL: {}", config.nats.url);
    info!("  Orchestrator Subject: {}", config.nats.subjects.orchestrator);
    info!("  TLS CA Cert: {}", config.nats.tls.ca_cert);
    info!("  TLS Client Cert: {}", config.nats.tls.client_cert);
    info!("  TLS Client Key: {}", config.nats.tls.client_key);
    info!("");

    // Connect to NATS with TLS
    info!("Connecting to NATS with TLS...");
    let _nats_client = connect_nats_with_tls(
        &config.nats.url,
        &config.nats.tls.ca_cert,
        &config.nats.tls.client_cert,
        &config.nats.tls.client_key,
    ).await?;
    info!("✅ Connected to NATS with TLS");
    info!("");

    // Create REST server with TLS configuration
    let rest_config = get_gateway_config(&config)?;
    info!("Starting HTTP/TLS server...");
    info!("");

    let mut server = RestServer::new(rest_config).await
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
    println!("║          🎯 TALETRAIL GATEWAY (DUAL TLS: HTTPS + NATS)  🎯                  ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  🔒 HTTPS Endpoint: {}://{}:{}                              ║",
             protocol,
             server.config().base.bind_address,
             server.config().base.port);
    println!("║  🔒 NATS Client: {} (TLS enabled)                  ║", config.nats.url);
    println!("║                                                                              ║");
    println!("║  Available Endpoints:                                                        ║");
    println!("║    POST /health            - Health check (envelope-wrapped)                 ║");
    println!("║                                                                              ║");
    println!("║  🔧 Architecture: Envelope-first with UnifiedEnvelope wrapping               ║");
    println!("║  📦 All responses include metadata (trace_id, timestamp, etc.)               ║");
    println!("║  🔒 TLS Server: gateway-cert.pem (HTTPS)                                     ║");
    println!("║  🔒 TLS Client: client-cert.pem (NATS)                                       ║");
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
