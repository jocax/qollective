// ABOUTME: Library entry point for holodeck-environment - 3D environment simulation and management
// ABOUTME: Provides MCP server implementation for holodeck environment creation with rmcp integration

pub mod config;
pub mod server;
pub use server::HolodeckEnvironmentServer;