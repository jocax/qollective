// ABOUTME: Generated types from JSON schemas - includes LLM response types for pipeline usage
// ABOUTME: Contains types used for structured LLM responses and rig-core pipeline integration

use serde::{Deserialize, Serialize};
// Temporarily disable JsonSchema to resolve version conflicts
// TODO: Implement proper schema generation once version conflicts are resolved

pub use crate::storytemplate::*;
pub use crate::holodeck::*;
pub use crate::characters::*;

/// LLM story generation response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStoryResponse {
    /// Main narrative description of the story
    pub story_content: String,
    /// List of scenes in the story
    pub scenes: Vec<LlmScene>,
    /// Story graph showing scene connections and flow
    pub story_graph: LlmStoryGraph,
}

/// Scene template from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmScene {
    /// Unique scene identifier
    pub id: String,
    /// Scene title
    pub name: String,
    /// Detailed scene description
    pub description: String,
    /// Environment type (e.g., starship_bridge, alien_planet, space_station)
    pub environment_type: String,
    /// Characters required for this scene
    pub required_characters: Vec<String>,
    /// Characters optional for this scene
    pub optional_characters: Vec<String>,
}

/// Story graph structure from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStoryGraph {
    /// List of graph nodes representing scenes
    pub nodes: Vec<LlmGraphNode>,
    /// ID of the starting node
    pub root_node_id: String,
    /// IDs of possible ending nodes
    pub ending_node_ids: Vec<String>,
}

/// Graph node from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmGraphNode {
    /// Unique node identifier
    pub id: String,
    /// Scene ID this node represents
    pub scene_id: String,
    /// Connections to other nodes
    pub connections: Vec<LlmNodeConnection>,
    /// Whether this node is a story checkpoint
    pub is_checkpoint: bool,
}

/// Node connection from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmNodeConnection {
    /// ID of the target node
    pub target_node_id: String,
    /// Condition for this connection
    pub condition: String,
    /// Description of this story path
    pub description: String,
}

// Re-export the core types for convenience
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};
pub use std::collections::HashMap;