// ABOUTME: Tauri V2 desktop application library exports for holodeck interface
// ABOUTME: Core functionality and utilities for Star Trek TNG themed holodeck client

pub mod commands;
pub mod websocket_client;
pub mod mcp_commands;

// Re-export commonly used types
pub use commands::{AppState, CoordinatorConnection, SessionData};
pub use websocket_client::{HolodeckWebSocketClient, WebSocketMessage};
pub use mcp_commands::McpClientState;