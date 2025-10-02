// ABOUTME: MCP server library for AI-powered holodeck story generation  
// ABOUTME: Public API and types for story designer functionality with rmcp-macros integration

pub mod server;
pub mod config;
pub mod generated;
pub mod pipeline;
pub mod compatibility_types;

pub use server::HolodeckDesignerServer;