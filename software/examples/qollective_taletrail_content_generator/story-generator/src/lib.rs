//! Story Generator Library
//!
//! Core functionality for the Story Generator MCP Server.

pub mod config;
pub mod server;

// Re-export key types
pub use config::StoryGeneratorConfig;
pub use server::StoryGeneratorServer;
