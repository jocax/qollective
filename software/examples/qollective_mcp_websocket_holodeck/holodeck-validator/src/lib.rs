// ABOUTME: Library entry point for holodeck-validator - content validation and safety monitoring
// ABOUTME: Provides MCP server implementation for story content validation with rmcp integration

pub mod config;
pub mod server;
pub use server::HolodeckValidatorServer;