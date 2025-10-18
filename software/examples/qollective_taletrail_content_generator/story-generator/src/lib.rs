//! Story Generator Library
//!
//! Core functionality for the Story Generator MCP Server.

pub mod config;
pub mod discovery;
pub mod envelope_handlers;
pub mod tool_handlers;
pub mod llm;
pub mod mcp_tools;
pub mod prompts;
pub mod server;
pub mod structure;

// Re-export key types
pub use config::StoryGeneratorConfig;
pub use server::StoryGeneratorServer;
pub use envelope_handlers::StoryGeneratorHandler;

// Re-export LLM functionality
pub use llm::{generate_node_content, generate_nodes_batch, StoryLlmClient};
pub use prompts::PromptTemplates;

// Re-export MCP tools and handlers
pub use tool_handlers::{handle_generate_nodes, handle_generate_structure, handle_validate_paths};
pub use mcp_tools::{
    create_generate_nodes_tool, create_generate_structure_tool, create_validate_paths_tool,
    get_all_tools, GenerateNodesParams, GenerateNodesResponse, GenerateStructureParams,
    GenerateStructureResponse, ValidatePathsParams, ValidatePathsResponse,
};
