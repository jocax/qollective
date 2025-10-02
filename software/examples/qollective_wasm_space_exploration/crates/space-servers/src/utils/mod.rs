// ABOUTME: Space exploration server utilities module
// ABOUTME: Common utilities for TLS configuration and server setup

use tracing::info;

/// Initialize logging for space servers
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("space_servers=info,qollective=debug")
        .init();
    
    info!("🛰️ Space servers logging initialized");
}