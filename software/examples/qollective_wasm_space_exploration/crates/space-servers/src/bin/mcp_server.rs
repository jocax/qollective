// ABOUTME: Standalone MCP server binary for space exploration demo
// ABOUTME: Runs only the MCP server for space tool execution with TLS

use space_servers::{utils::init_logging, handlers::mcp::create_space_mcp_server};
use tracing::{info, error};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    
    info!("🛠️ Starting Space Exploration MCP Server...");
    
    // Create and start MCP server
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
    
    info!("🎯 MCP server started successfully!");
    info!("🛠️ MCP Tools available at: ws://127.0.0.1:8445/mcp");
    info!("🔧 Press Ctrl+C to stop server");
    
    // Start server and wait for shutdown
    tokio::select! {
        result = mcp_server.start() => {
            if let Err(e) = result {
                error!("❌ MCP server failed: {}", e);
                return Err(e.into());
            }
        }
        _ = signal::ctrl_c() => {
            info!("🛑 Shutdown signal received, stopping server...");
        }
    }
    
    info!("✅ MCP server stopped successfully");
    Ok(())
}