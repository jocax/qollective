//! Prompt Helper library with envelope-first architecture
//!
//! This library provides MCP tools for prompt generation using Qollective's
//! envelope-first architecture. The main entry point for the server is
//! `PromptHelperHandler` which implements `EnvelopeHandler<McpData, McpData>`.

pub mod config;
pub mod handlers;
pub mod llm;
pub mod mcp_tools;
pub mod server;
pub mod templates;
pub mod handler;

// Re-export MCP tool functions for external use
pub use mcp_tools::{
    create_generate_constraint_prompts_tool,
    create_generate_story_prompts_tool,
    create_generate_validation_prompts_tool,
    create_get_model_for_language_tool,
    get_all_tools,
};

// Re-export handler functions for external use
pub use handlers::{
    handle_generate_constraint_prompts,
    handle_generate_story_prompts,
    handle_generate_validation_prompts,
    handle_get_model_for_language,
};

// Re-export LLM service for external use
pub use llm::RigLlmService;

// Re-export envelope handler for external use
pub use handler::PromptHelperHandler;
