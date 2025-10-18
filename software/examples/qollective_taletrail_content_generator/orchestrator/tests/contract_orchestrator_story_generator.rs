//! Contract Tests: Orchestrator → Story Generator
//!
//! These tests validate that parameter schemas match between
//! orchestrator calls and story-generator handlers.

use shared_types::contract_tests::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared_types::{ContentNode, GenerationRequest, DAG, AgeGroup};

// ============================================================================
// Story Generator Parameter Types (mirrored from story-generator crate)
// ============================================================================

/// Parameters for generating DAG structure with convergence points
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SgGenerateStructureParams {
    /// Complete generation request containing node count, theme, age group, etc.
    pub generation_request: GenerationRequest,
}

/// Response from generate_structure tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SgGenerateStructureResponse {
    /// Generated DAG structure with nodes and edges (no content yet)
    pub dag: DAG,
    /// Number of convergence points calculated
    pub convergence_point_count: usize,
    /// Total node count
    pub node_count: usize,
    /// Total edge count
    pub edge_count: usize,
}

/// Parameters for generating content for a batch of nodes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SgGenerateNodesParams {
    /// DAG structure containing nodes to generate content for
    pub dag: DAG,
    /// List of node IDs to generate content for in this batch
    pub node_ids: Vec<String>,
    /// Generation request containing theme, age group, language, etc.
    pub generation_request: GenerationRequest,
}

/// Response from generate_nodes tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SgGenerateNodesResponse {
    /// Content nodes with generated narrative content
    pub nodes: Vec<ContentNode>,
    /// Number of nodes successfully generated
    pub generated_count: usize,
}

// ============================================================================
// Contract Tests
// ============================================================================

#[test]
fn test_generate_structure_params_schema() {
    validate_tool_contract::<SgGenerateStructureParams>()
        .expect("SgGenerateStructureParams schema should be valid");
}

#[test]
fn test_generate_structure_response_schema() {
    validate_tool_contract::<SgGenerateStructureResponse>()
        .expect("SgGenerateStructureResponse schema should be valid");
}

#[test]
fn test_generate_nodes_params_schema() {
    validate_tool_contract::<SgGenerateNodesParams>()
        .expect("SgGenerateNodesParams schema should be valid");
}

#[test]
fn test_generate_nodes_response_schema() {
    validate_tool_contract::<SgGenerateNodesResponse>()
        .expect("SgGenerateNodesResponse schema should be valid");
}

#[test]
fn test_generate_structure_params_roundtrip() {
    let generation_request = GenerationRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        node_count: 15,
        theme: Some("Underwater Adventure".to_string()),
        age_group: AgeGroup::_6To8,
        language: "en".to_string(),
        convergence_points: Some(3),
        prompt_packages: None,
        tenant_id: Some("test-tenant".to_string()),
        user_id: Some("test-user".to_string()),
    };

    let params = SgGenerateStructureParams {
        generation_request,
    };

    test_roundtrip_serialization(params)
        .expect("GenerateStructureParams roundtrip should succeed");
}

#[test]
fn test_generate_structure_response_roundtrip() {
    let dag = DAG {
        nodes: vec![
            ContentNode {
                id: "start".to_string(),
                depth: 0,
                content: None,
                choices: vec![],
                educational_content: None,
            },
            ContentNode {
                id: "node-1".to_string(),
                depth: 1,
                content: None,
                choices: vec![],
                educational_content: None,
            },
        ],
        edges: vec![
            ("start".to_string(), "node-1".to_string()),
        ],
    };

    let response = SgGenerateStructureResponse {
        dag,
        convergence_point_count: 2,
        node_count: 2,
        edge_count: 1,
    };

    test_roundtrip_serialization(response)
        .expect("GenerateStructureResponse roundtrip should succeed");
}

#[test]
fn test_generate_nodes_params_roundtrip() {
    let dag = DAG {
        nodes: vec![
            ContentNode {
                id: "start".to_string(),
                depth: 0,
                content: Some("Start of adventure".to_string()),
                choices: vec!["Go north".to_string(), "Go south".to_string()],
                educational_content: Some("Learning about directions".to_string()),
            },
        ],
        edges: vec![],
    };

    let generation_request = GenerationRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        node_count: 10,
        theme: Some("Forest Adventure".to_string()),
        age_group: AgeGroup::_9To11,
        language: "en".to_string(),
        convergence_points: None,
        prompt_packages: None,
        tenant_id: Some("test-tenant".to_string()),
        user_id: Some("test-user".to_string()),
    };

    let params = SgGenerateNodesParams {
        dag,
        node_ids: vec!["start".to_string()],
        generation_request,
    };

    test_roundtrip_serialization(params)
        .expect("GenerateNodesParams roundtrip should succeed");
}

#[test]
fn test_generate_nodes_response_roundtrip() {
    let nodes = vec![
        ContentNode {
            id: "node-1".to_string(),
            depth: 1,
            content: Some("You discover a hidden cave.".to_string()),
            choices: vec![
                "Enter the cave".to_string(),
                "Continue on the path".to_string(),
            ],
            educational_content: Some("Learning about exploration and choices".to_string()),
        },
        ContentNode {
            id: "node-2".to_string(),
            depth: 2,
            content: Some("The cave is dark and mysterious.".to_string()),
            choices: vec![
                "Light a torch".to_string(),
                "Turn back".to_string(),
            ],
            educational_content: Some("Understanding problem-solving".to_string()),
        },
    ];

    let response = SgGenerateNodesResponse {
        nodes,
        generated_count: 2,
    };

    test_roundtrip_serialization(response)
        .expect("GenerateNodesResponse roundtrip should succeed");
}

#[test]
fn test_generate_structure_with_minimal_request() {
    let generation_request = GenerationRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        node_count: 5,
        theme: None, // No theme
        age_group: AgeGroup::_15To17,
        language: "de".to_string(),
        convergence_points: None,
        prompt_packages: None,
        tenant_id: None,
        user_id: None,
    };

    let params = SgGenerateStructureParams {
        generation_request,
    };

    test_roundtrip_serialization(params)
        .expect("GenerateStructureParams with minimal fields should roundtrip");
}

#[test]
fn test_generate_nodes_with_empty_batch() {
    let dag = DAG {
        nodes: vec![],
        edges: vec![],
    };

    let generation_request = GenerationRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        node_count: 0,
        theme: Some("Test".to_string()),
        age_group: AgeGroup::Plus18,
        language: "en".to_string(),
        convergence_points: None,
        prompt_packages: None,
        tenant_id: Some("test-tenant".to_string()),
        user_id: Some("test-user".to_string()),
    };

    let params = SgGenerateNodesParams {
        dag,
        node_ids: vec![], // Empty batch
        generation_request,
    };

    test_roundtrip_serialization(params)
        .expect("GenerateNodesParams with empty batch should roundtrip");
}
