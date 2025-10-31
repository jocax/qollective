//! Prompt Helper library with envelope-first architecture
//!
//! This library provides MCP tools for prompt generation using Qollective's
//! envelope-first architecture. The main entry point for the server is
//! `PromptHelperHandler` which implements `EnvelopeHandler<McpData, McpData>`.

pub mod config;
pub mod discovery;
pub mod envelope_handlers;
pub mod execution_logger;
pub mod llm;
pub mod mcp_tools;
pub mod server;
pub mod templates;
pub mod tool_handlers;

// Re-export MCP tool functions for external use
pub use mcp_tools::{
    create_generate_constraint_prompts_tool,
    create_generate_story_prompts_tool,
    create_generate_validation_prompts_tool,
    create_get_model_for_language_tool,
    get_all_tools,
};

// Re-export handler functions for external use
pub use tool_handlers::{
    handle_generate_constraint_prompts,
    handle_generate_story_prompts,
    handle_generate_validation_prompts,
    handle_get_model_for_language,
};

// Re-export LLM service for external use
pub use llm::SharedLlmService;

// Re-export envelope handler for external use
pub use envelope_handlers::PromptHelperHandler;

// Test helpers module - always compiled so unit tests in modules can use it
#[cfg(test)]
pub(crate) mod test_helpers;
