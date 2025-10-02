// ABOUTME: Library entry point for holodeck-character - Star Trek character AI and dialogue
// ABOUTME: Provides MCP server implementation for authentic character interactions with rmcp integration

pub mod server;
pub mod config;

pub use server::HolodeckCharacterServer;
pub use config::ServiceConfig;