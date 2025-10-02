// ABOUTME: WebSocket/REST server library for holodeck story data and history management
// ABOUTME: Public API and types for storybook server functionality with full MCP integration

pub mod config;
pub mod server;

pub use config::ServiceConfig;
pub use server::HolodeckStorybookServer;