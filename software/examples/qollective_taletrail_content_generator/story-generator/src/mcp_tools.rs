//! MCP Tool Definitions for Story Generator Service
//!
//! This module defines the MCP tools exposed by the Story Generator service
//! using rmcp 0.8.0 types and schemars for JSON Schema generation.
//!
//! # Tools
//!
//! - `generate_structure`: Creates DAG structure with convergence points
//! - `generate_nodes`: Generates narrative content for batches of nodes (stub for task 2.2)
//! - `validate_paths`: Validates DAG path connectivity and reachability

use rmcp::model::Tool;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use shared_types::{ContentNode, GenerationRequest, DAG};
use std::sync::Arc;

// ============================================================================
// Tool Parameter Structures
// ============================================================================

/// Parameters for generating DAG structure with convergence points
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateStructureParams {
    /// Complete generation request containing node count, theme, age group, etc.
    pub generation_request: GenerationRequest,
}

/// Parameters for generating content for a batch of nodes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateNodesParams {
    /// DAG structure containing nodes to generate content for
    pub dag: DAG,
    /// List of node IDs to generate content for in this batch
    pub node_ids: Vec<String>,
    /// Generation request containing theme, age group, language, etc.
    pub generation_request: GenerationRequest,
}

/// Parameters for validating DAG path connectivity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidatePathsParams {
    /// Complete DAG structure to validate
    pub dag: DAG,
}

// ============================================================================
// Response Structures
// ============================================================================

/// Response from generate_structure tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateStructureResponse {
    /// Generated DAG structure with nodes and edges (no content yet)
    pub dag: DAG,
    /// Number of convergence points calculated
    pub convergence_point_count: usize,
    /// Total node count
    pub node_count: usize,
    /// Total edge count
    pub edge_count: usize,
}

/// Response from generate_nodes tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateNodesResponse {
    /// Content nodes with generated narrative content
    pub nodes: Vec<ContentNode>,
    /// Number of nodes successfully generated
    pub generated_count: usize,
}

/// Response from validate_paths tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidatePathsResponse {
    /// Overall validation result
    pub is_valid: bool,
    /// Detailed validation messages
    pub validation_messages: Vec<String>,
    /// Number of nodes validated
    pub node_count: usize,
    /// Number of edges validated
    pub edge_count: usize,
}

// ============================================================================
// Tool Creation Functions
// ============================================================================

/// Create the "generate_structure" tool
///
/// This tool generates a DAG structure with convergence points for branching narratives.
/// It creates the skeleton structure (nodes and edges) without content, which will be
/// filled in by the generate_nodes tool.
#[allow(dead_code)]
pub fn create_generate_structure_tool() -> Tool {
    let schema = schema_for!(GenerateStructureParams);
    let schema_value =
        serde_json::to_value(schema).expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "generate_structure".into(),
        description: Some(
            "Create DAG structure with convergence points for branching narrative. \
             Generates the skeleton structure (nodes and edges) based on the generation request, \
             calculating optimal convergence points for story flow. Returns DAG with empty content \
             nodes that will be filled by generate_nodes tool."
                .into(),
        ),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Create the "generate_nodes" tool
///
/// This tool generates narrative content for a batch of nodes in the DAG.
/// For task 2.2, this is a stub that returns nodes with empty content.
/// Task 2.3 will implement actual LLM content generation via rig-core.
#[allow(dead_code)]
pub fn create_generate_nodes_tool() -> Tool {
    let schema = schema_for!(GenerateNodesParams);
    let schema_value =
        serde_json::to_value(schema).expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "generate_nodes".into(),
        description: Some(
            "Generate narrative content for a batch of nodes (STUB: returns empty content for now). \
             Takes a DAG structure and a list of node IDs to generate content for. \
             Returns ContentNode objects with generated narrative text, choices, and educational content. \
             Note: This is currently a stub implementation; task 2.3 will add LLM content generation."
                .into(),
        ),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Create the "validate_paths" tool
///
/// This tool validates DAG path connectivity and reachability.
/// Ensures all nodes are connected, reachable from start, and can reach the end.
#[allow(dead_code)]
pub fn create_validate_paths_tool() -> Tool {
    let schema = schema_for!(ValidatePathsParams);
    let schema_value =
        serde_json::to_value(schema).expect("Failed to serialize schema to JSON");

    let input_schema = if let serde_json::Value::Object(map) = schema_value {
        Arc::new(map)
    } else {
        panic!("Schema must be an object");
    };

    Tool {
        name: "validate_paths".into(),
        description: Some(
            "Validate DAG path connectivity and reachability. \
             Performs comprehensive validation including: \
             - All nodes except start have incoming edges \
             - All nodes except end have outgoing edges \
             - All nodes are reachable from start node \
             - All nodes can reach the end node \
             Returns detailed validation results with specific error messages if validation fails."
                .into(),
        ),
        input_schema,
        output_schema: None,
        annotations: None,
        icons: None,
        title: None,
    }
}

/// Get all tools provided by the Story Generator service
///
/// Returns a vector of all 3 tool definitions for registration
/// with the MCP server.
#[allow(dead_code)]
pub fn get_all_tools() -> Vec<Tool> {
    vec![
        create_generate_structure_tool(),
        create_generate_nodes_tool(),
        create_validate_paths_tool(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tools_have_names() {
        let tools = get_all_tools();
        assert_eq!(tools.len(), 3);

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(names.contains(&"generate_structure"));
        assert!(names.contains(&"generate_nodes"));
        assert!(names.contains(&"validate_paths"));
    }

    #[test]
    fn test_all_tools_have_descriptions() {
        let tools = get_all_tools();
        for tool in tools {
            assert!(
                tool.description.is_some(),
                "Tool {} missing description",
                tool.name
            );
            assert!(
                !tool.description.as_ref().unwrap().is_empty(),
                "Tool {} has empty description",
                tool.name
            );
        }
    }

    #[test]
    fn test_all_tools_have_input_schemas() {
        let tools = get_all_tools();
        for tool in tools {
            assert!(
                !tool.input_schema.is_empty(),
                "Tool {} missing input schema",
                tool.name
            );
        }
    }

    #[test]
    fn test_generate_structure_tool_schema() {
        let tool = create_generate_structure_tool();
        assert_eq!(tool.name, "generate_structure");
        assert!(tool.description.is_some());
        assert!(!tool.input_schema.is_empty());
    }

    #[test]
    fn test_generate_nodes_tool_schema() {
        let tool = create_generate_nodes_tool();
        assert_eq!(tool.name, "generate_nodes");
        assert!(tool.description.is_some());
        assert!(tool
            .description
            .unwrap()
            .contains("STUB: returns empty content"));
    }

    #[test]
    fn test_validate_paths_tool_schema() {
        let tool = create_validate_paths_tool();
        assert_eq!(tool.name, "validate_paths");
        assert!(tool.description.is_some());
        assert!(!tool.input_schema.is_empty());
    }
}
