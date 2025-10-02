// ABOUTME: Space exploration servers main binary for starting all servers
// ABOUTME: Runs REST, WebSocket, and MCP servers concurrently for space exploration demo

use space_servers::{
    utils::init_logging, 
    handlers::{
        rest::create_space_rest_server, 
        websocket::create_space_websocket_server,
        mcp::create_space_mcp_server
    }
};
use tracing::{info, error};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    
    info!("🚀 Starting Space Exploration Demo Servers...");
    
    // Start REST server
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
    
    // Start WebSocket server  
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
    
    // Start MCP server (WebSocket-based)
    let mut mcp_server = match create_space_mcp_server().await {
        Ok(server) => {
            info!("✅ MCP server created successfully");
            server
        }
        Err(e) => {
            error!("❌ Failed to create MCP server: {}", e);
            return Err(e.into());
        }
    };
    
    // Start servers in background
    let rest_handle = tokio::spawn(async move {
        if let Err(e) = rest_server.start().await {
            error!("❌ REST server failed: {}", e);
        }
    });
    
    let websocket_handle = tokio::spawn(async move {
        if let Err(e) = websocket_server.start().await {
            error!("❌ WebSocket server failed: {}", e);
        }
    });
    
    let mcp_handle = tokio::spawn(async move {
        if let Err(e) = mcp_server.start().await {
            error!("❌ MCP server failed: {}", e);
        }
    });
    
    info!("🎯 All servers started successfully!");
    info!("📡 REST API available at: http://127.0.0.1:8443");
    info!("🛰️ WebSocket available at: ws://127.0.0.1:8444");
    info!("🛠️ MCP Tools available at: ws://127.0.0.1:8445/mcp");
    info!("🔧 Press Ctrl+C to stop all servers");
    
    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("🛑 Shutdown signal received, stopping servers...");
    
    // Stop servers
    rest_handle.abort();
    websocket_handle.abort();
    mcp_handle.abort();
    
    info!("✅ All servers stopped successfully");
    Ok(())
}