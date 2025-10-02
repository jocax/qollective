// ABOUTME: Standalone REST server binary for space exploration demo
// ABOUTME: Runs only the REST server for mission data and spacecraft status

use space_servers::{utils::init_logging, handlers::rest::create_space_rest_server};
use tracing::{info, error};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    
    info!("🚀 Starting Space Exploration REST Server...");
    
    // Create and start REST server
    let mut rest_server = match create_space_rest_server().await {
        Ok(server) => {
            info!("✅ REST server created successfully");
            server
        }
        Err(e) => {
            error!("❌ Failed to create REST server: {}", e);
            return Err(e.into());
        }
    };
    
    info!("🎯 REST server started successfully!");
    info!("📡 REST API available at: http://127.0.0.1:8443");
    info!("🔧 Press Ctrl+C to stop server");
    
    // Start server and wait for shutdown
    tokio::select! {
        result = rest_server.start() => {
            if let Err(e) = result {
                error!("❌ REST server failed: {}", e);
                return Err(e.into());
            }
        }
        _ = signal::ctrl_c() => {
            info!("🛑 Shutdown signal received, stopping server...");
        }
    }
    
    info!("✅ REST server stopped successfully");
    Ok(())
}