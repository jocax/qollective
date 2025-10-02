// ABOUTME: Standalone WebSocket server binary for space exploration demo
// ABOUTME: Runs only the WebSocket server for real-time telemetry streaming

use space_servers::{utils::init_logging, handlers::websocket::create_space_websocket_server};
use tracing::{info, error};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    
    info!("🛰️ Starting Space Exploration WebSocket Server...");
    
    // Create and start WebSocket server
    let mut websocket_server = match create_space_websocket_server().await {
        Ok(server) => {
            info!("✅ WebSocket server created successfully");
            server
        }
        Err(e) => {
            error!("❌ Failed to create WebSocket server: {}", e);
            return Err(e.into());
        }
    };
    
    info!("🎯 WebSocket server started successfully!");
    info!("📡 WebSocket available at: ws://127.0.0.1:8444");
    info!("🔧 Press Ctrl+C to stop server");
    
    // Start server and wait for shutdown
    tokio::select! {
        result = websocket_server.start() => {
            if let Err(e) = result {
                error!("❌ WebSocket server failed: {}", e);
                return Err(e.into());
            }
        }
        _ = signal::ctrl_c() => {
            info!("🛑 Shutdown signal received, stopping server...");
        }
    }
    
    info!("✅ WebSocket server stopped successfully");
    Ok(())
}