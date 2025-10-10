//! Integration tests for MCP tools
//!
//! These tests verify that MCP tools are correctly defined and handlers
//! work as expected with realistic inputs.

use shared_types::{constants::DEFAULT_NODE_COUNT, AgeGroup, Language};
use story_generator::{
    create_generate_nodes_tool, create_generate_structure_tool, create_validate_paths_tool,
    get_all_tools, handle_generate_nodes, handle_generate_structure, handle_validate_paths,
    GenerateNodesParams, GenerateStructureParams, ValidatePathsParams,
};

// ============================================================================
// Tool Definition Tests
// ============================================================================

#[test]
fn test_tool_definitions_complete() {
    let tools = get_all_tools();
    assert_eq!(tools.len(), 3, "Should have exactly 3 tools");

    // Verify all expected tools are present
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    assert!(tool_names.contains(&"generate_structure".to_string()));
    assert!(tool_names.contains(&"generate_nodes".to_string()));
    assert!(tool_names.contains(&"validate_paths".to_string()));
}

#[test]
fn test_generate_structure_tool_definition() {
    let tool = create_generate_structure_tool();

    assert_eq!(tool.name, "generate_structure");
    assert!(tool.description.is_some());
    assert!(!tool.input_schema.is_empty());

    let description = tool.description.unwrap();
    assert!(description.contains("DAG structure"));
    assert!(description.contains("convergence points"));
}

#[test]
fn test_generate_nodes_tool_definition() {
    let tool = create_generate_nodes_tool();

    assert_eq!(tool.name, "generate_nodes");
    assert!(tool.description.is_some());
    assert!(!tool.input_schema.is_empty());

    let description = tool.description.unwrap();
    assert!(description.contains("STUB"));
    assert!(description.contains("narrative content"));
}

#[test]
fn test_validate_paths_tool_definition() {
    let tool = create_validate_paths_tool();

    assert_eq!(tool.name, "validate_paths");
    assert!(tool.description.is_some());
    assert!(!tool.input_schema.is_empty());

    let description = tool.description.unwrap();
    assert!(description.contains("connectivity"));
    assert!(description.contains("reachability"));
}

#[test]
fn test_tool_schemas_are_valid_json() {
    let tools = get_all_tools();

    for tool in tools {
        // Verify that input schemas can be serialized to JSON
        let schema_json = serde_json::to_value(tool.input_schema.as_ref());
        assert!(
            schema_json.is_ok(),
            "Tool {} schema should serialize to JSON",
            tool.name
        );
    }
}

// ============================================================================
// Tool Handler Tests
// ============================================================================

fn create_test_generation_request() -> shared_types::GenerationRequest {
    shared_types::GenerationRequest {
        theme: "Space Adventure".to_string(),
        age_group: AgeGroup::_9To11,
        language: Language::En,
        tenant_id: 1,
        node_count: Some(DEFAULT_NODE_COUNT as i64),
        vocabulary_level: None,
        educational_goals: Some(vec!["Learn about space".to_string()]),
        required_elements: Some(vec!["rocket".to_string()]),
        tags: Some(vec!["science".to_string()]),
        author_id: Some(Some(123)),
        prompt_packages: None,
    }
}

#[test]
fn test_handle_generate_structure_with_valid_input() {
    let params = GenerateStructureParams {
        generation_request: create_test_generation_request(),
    };

    let result = handle_generate_structure(params);
    assert!(result.is_ok(), "Should successfully generate structure");

    let response = result.unwrap();
    assert_eq!(
        response.node_count, DEFAULT_NODE_COUNT,
        "Should create correct number of nodes"
    );
    assert!(
        response.edge_count > 0,
        "Should create at least some edges"
    );
    assert!(
        response.convergence_point_count > 0,
        "Should have convergence points"
    );
    assert_eq!(response.dag.nodes.len(), DEFAULT_NODE_COUNT);
    assert_eq!(response.dag.start_node_id, "0");
}

#[test]
fn test_handle_generate_structure_with_custom_node_count() {
    let mut request = create_test_generation_request();
    request.node_count = Some(10);

    let params = GenerateStructureParams {
        generation_request: request,
    };

    let result = handle_generate_structure(params);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.node_count, 10, "Should respect custom node count");
}

#[test]
fn test_handle_generate_structure_with_no_node_count() {
    let mut request = create_test_generation_request();
    request.node_count = None;

    let params = GenerateStructureParams {
        generation_request: request,
    };

    let result = handle_generate_structure(params);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(
        response.node_count, DEFAULT_NODE_COUNT,
        "Should use default node count when not specified"
    );
}

// NOTE: Tests for handle_generate_nodes are disabled because the function is now async
// and requires an LLM client. See tests/generation_tests.rs for comprehensive tests.

/*
#[test]
fn test_handle_generate_nodes_stub_behavior() {
    // First generate structure
    let structure_params = GenerateStructureParams {
        generation_request: create_test_generation_request(),
    };
    let structure_response = handle_generate_structure(structure_params).unwrap();

    // Select first 3 nodes
    let node_ids: Vec<String> = structure_response
        .dag
        .nodes
        .keys()
        .take(3)
        .cloned()
        .collect();

    let params = GenerateNodesParams {
        dag: structure_response.dag,
        node_ids: node_ids.clone(),
        generation_request: create_test_generation_request(),
    };

    let result = handle_generate_nodes(params);
    assert!(result.is_ok(), "Stub handler should succeed");

    let response = result.unwrap();
    assert_eq!(
        response.generated_count,
        node_ids.len(),
        "Should generate correct number of nodes"
    );
    assert_eq!(response.nodes.len(), node_ids.len());

    // Verify stub behavior: all content should be empty
    for node in &response.nodes {
        assert!(
            node.content.text.is_empty(),
            "Stub should return empty text for node {}",
            node.id
        );
        assert!(
            node.content.choices.is_empty(),
            "Stub should return empty choices for node {}",
            node.id
        );
        assert!(
            node.content.educational_content.is_none(),
            "Stub should return no educational content for node {}",
            node.id
        );
    }
}

#[test]
fn test_handle_generate_nodes_preserves_dag_structure() {
    // Generate structure
    let structure_params = GenerateStructureParams {
        generation_request: create_test_generation_request(),
    };
    let structure_response = handle_generate_structure(structure_params).unwrap();

    // Get some node IDs
    let node_ids: Vec<String> = structure_response
        .dag
        .nodes
        .keys()
        .take(5)
        .cloned()
        .collect();

    let params = GenerateNodesParams {
        dag: structure_response.dag.clone(),
        node_ids: node_ids.clone(),
        generation_request: create_test_generation_request(),
    };

    let result = handle_generate_nodes(params).unwrap();

    // Verify that edge counts are preserved from original DAG
    for node in result.nodes {
        let original_node = structure_response.dag.nodes.get(&node.id).unwrap();
        assert_eq!(
            node.incoming_edges, original_node.incoming_edges,
            "Should preserve incoming edge count"
        );
        assert_eq!(
            node.outgoing_edges, original_node.outgoing_edges,
            "Should preserve outgoing edge count"
        );
        assert_eq!(
            node.content.convergence_point, original_node.content.convergence_point,
            "Should preserve convergence point status"
        );
    }
}

#[test]
fn test_handle_generate_nodes_with_invalid_node_id() {
    // Generate structure
    let structure_params = GenerateStructureParams {
        generation_request: create_test_generation_request(),
    };
    let structure_response = handle_generate_structure(structure_params).unwrap();

    // Try to generate with invalid node ID
    let params = GenerateNodesParams {
        dag: structure_response.dag,
        node_ids: vec!["nonexistent_node".to_string()],
        generation_request: create_test_generation_request(),
    };

    let result = handle_generate_nodes(params);
    assert!(result.is_err(), "Should fail with invalid node ID");
}
*/

#[test]
fn test_handle_validate_paths_with_valid_dag() {
    // Generate valid structure
    let structure_params = GenerateStructureParams {
        generation_request: create_test_generation_request(),
    };
    let structure_response = handle_generate_structure(structure_params).unwrap();

    // Validate it
    let params = ValidatePathsParams {
        dag: structure_response.dag,
    };

    let result = handle_validate_paths(params);
    assert!(result.is_ok(), "Validation should succeed");

    let response = result.unwrap();
    assert!(response.is_valid, "Generated DAG should be valid");
    assert!(
        response.validation_messages.len() >= 3,
        "Should have multiple validation messages"
    );
    assert_eq!(response.node_count, DEFAULT_NODE_COUNT);
    assert!(response.edge_count > 0);

    // Check that success messages are present
    let messages_str = response.validation_messages.join(" ");
    // Should contain validation success indicators
    assert!(
        messages_str.contains("passed") || messages_str.contains("validation checks"),
        "Expected success message but got: {:?}",
        response.validation_messages
    );
}

#[test]
fn test_handle_validate_paths_with_empty_dag() {
    use shared_types::DAG;
    use std::collections::HashMap;

    let empty_dag = DAG {
        nodes: HashMap::new(),
        edges: Vec::new(),
        start_node_id: "0".to_string(),
        convergence_points: Vec::new(),
    };

    let params = ValidatePathsParams { dag: empty_dag };

    let result = handle_validate_paths(params);
    assert!(result.is_ok(), "Validation handler should complete");

    let response = result.unwrap();
    assert!(
        !response.is_valid,
        "Empty DAG should fail validation"
    );
    assert!(
        !response.validation_messages.is_empty(),
        "Should have error messages"
    );
}

// ============================================================================
// Integration Tests (Tool Workflow)
// ============================================================================

#[test]
fn test_full_workflow_generate_structure_to_validate() {
    // Step 1: Generate structure
    let structure_params = GenerateStructureParams {
        generation_request: create_test_generation_request(),
    };
    let structure_response = handle_generate_structure(structure_params).unwrap();

    assert_eq!(structure_response.node_count, DEFAULT_NODE_COUNT);

    // Step 2: Validate the generated structure
    let validate_params = ValidatePathsParams {
        dag: structure_response.dag.clone(),
    };
    let validate_response = handle_validate_paths(validate_params).unwrap();

    assert!(
        validate_response.is_valid,
        "Freshly generated structure should be valid"
    );

    // NOTE: Content generation step removed (handle_generate_nodes is now async)
    // See tests/generation_tests.rs for comprehensive content generation tests
}

#[test]
fn test_full_workflow_with_different_age_groups() {
    for age_group in [
        AgeGroup::_6To8,
        AgeGroup::_9To11,
        AgeGroup::_12To14,
        AgeGroup::_15To17,
        AgeGroup::Plus18,
    ] {
        let mut request = create_test_generation_request();
        request.age_group = age_group;

        let params = GenerateStructureParams {
            generation_request: request,
        };

        let result = handle_generate_structure(params);
        assert!(
            result.is_ok(),
            "Should generate structure for age group {:?}",
            age_group
        );

        let response = result.unwrap();
        assert_eq!(response.node_count, DEFAULT_NODE_COUNT);
    }
}

#[test]
fn test_full_workflow_with_different_languages() {
    for language in [Language::En, Language::De] {
        let mut request = create_test_generation_request();
        request.language = language;

        let params = GenerateStructureParams {
            generation_request: request,
        };

        let result = handle_generate_structure(params);
        assert!(
            result.is_ok(),
            "Should generate structure for language {:?}",
            language
        );

        let response = result.unwrap();
        assert_eq!(response.node_count, DEFAULT_NODE_COUNT);
    }
}

#[test]
fn test_error_handling_for_invalid_inputs() {
    // Test with node count too small (less than 3)
    let mut request = create_test_generation_request();
    request.node_count = Some(2);

    let params = GenerateStructureParams {
        generation_request: request,
    };

    let result = handle_generate_structure(params);
    assert!(
        result.is_err(),
        "Should fail with node count less than 3"
    );
}

#[test]
fn test_convergence_points_in_generated_structure() {
    let params = GenerateStructureParams {
        generation_request: create_test_generation_request(),
    };
    let response = handle_generate_structure(params).unwrap();

    // Verify that convergence points are marked in the DAG
    let convergence_node_ids: Vec<&String> = response
        .dag
        .nodes
        .values()
        .filter(|n| n.content.convergence_point)
        .map(|n| &n.id)
        .collect();

    assert!(
        !convergence_node_ids.is_empty(),
        "Should have at least one convergence point"
    );

    // Verify that convergence point IDs match the DAG's convergence_points list
    for cp_id in &response.dag.convergence_points {
        assert!(
            response.dag.nodes.contains_key(cp_id),
            "Convergence point {} should exist in nodes",
            cp_id
        );

        let node = response.dag.nodes.get(cp_id).unwrap();
        assert!(
            node.content.convergence_point,
            "Node {} should be marked as convergence point",
            cp_id
        );
    }
}
