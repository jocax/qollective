//! DAG structure types (internal generation)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::models::Content;

/// Directed Acyclic Graph structure (internal representation during generation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAG {
    /// Map of node_id to ContentNode
    pub nodes: HashMap<Uuid, ContentNode>,

    /// Directed edges between nodes
    pub edges: Vec<Edge>,

    /// Entry point node ID
    pub start_node_id: Uuid,

    /// Story merge points
    pub convergence_points: Vec<Uuid>,
}

/// Content node with generation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentNode {
    /// Unique node identifier
    pub id: Uuid,

    /// Content structure
    pub content: Content,

    /// Count of incoming edges
    pub incoming_edges: usize,

    /// Count of outgoing edges
    pub outgoing_edges: usize,

    /// Generation-specific metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_metadata: Option<serde_json::Value>,
}

/// Directed edge between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source node ID
    pub from_node_id: Uuid,

    /// Target node ID
    pub to_node_id: Uuid,

    /// Choice that leads to this edge
    pub choice_id: Uuid,

    /// Optional edge weight for pathfinding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,
}
