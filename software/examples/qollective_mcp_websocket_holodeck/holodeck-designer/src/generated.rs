// ABOUTME: Generated types from JSON schemas - manually created for now
// ABOUTME: Will be auto-generated when typify schema issues are resolved

// Include the build.rs generated file (currently empty)
include!(concat!(env!("OUT_DIR"), "/generated_types.rs"));

// NOTE: LLM response types are now defined in shared-types/src/generated.rs 
// with proper schemars_v08 compatibility for rig-core integration.
// Use shared_types::{LlmStoryResponse, LlmScene, LlmStoryGraph, LlmGraphNode, LlmNodeConnection} instead.

// Re-export common types for convenience
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};
pub use std::collections::HashMap;

// Re-export LLM types from shared-types
pub use shared_types::{LlmStoryResponse, LlmScene, LlmStoryGraph, LlmGraphNode, LlmNodeConnection};