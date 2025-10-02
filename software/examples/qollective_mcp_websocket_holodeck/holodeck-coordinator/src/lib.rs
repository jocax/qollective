// ABOUTME: Library entry point for holodeck-coordinator - holodeck session orchestration and MCP server coordination
// ABOUTME: Provides MCP server implementation for orchestrating all holodeck MCP servers with rmcp integration

pub mod config;
pub mod server;

pub use config::ServiceConfig;
pub use server::HolodeckCoordinatorServer;