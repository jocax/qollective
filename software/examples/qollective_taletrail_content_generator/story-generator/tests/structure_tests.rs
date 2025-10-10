//! Integration tests for story structure planning
//!
//! These tests verify the DAG structure generation, convergence point calculation,
//! and validation logic for the story-generator service.

use shared_types::constants::{CONVERGENCE_POINT_RATIO, DEFAULT_NODE_COUNT};
use shared_types::extensions::dag::DagExt;
use shared_types::{ContentNode, DAG, Edge};
use story_generator::structure::{
    calculate_convergence_points, generate_dag_structure, validate_path_connectivity,
    validate_reachability,
};

// ============================================================================
// CONVERGENCE POINT CALCULATION TESTS
// ============================================================================

#[test]
fn test_calculate_convergence_points_default_count() {
    // With DEFAULT_NODE_COUNT (16) and CONVERGENCE_POINT_RATIO (0.5)
    // We expect convergence at ~50% intervals: ~8
    let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);

    // Should have at least one convergence point
    assert!(!convergence_points.is_empty());

    // All convergence points should be within valid range
    for &point in &convergence_points {
        assert!(point > 0, "Convergence point should be after start");
        assert!(
            point < DEFAULT_NODE_COUNT,
            "Convergence point should be before end"
        );
    }
}

#[test]
fn test_calculate_convergence_points_small_count() {
    // Test with a small node count (8 nodes)
    let node_count = 8;
    let convergence_points = calculate_convergence_points(node_count);

    // Should have convergence points
    assert!(!convergence_points.is_empty());

    // With ratio 0.5, expect convergence around position 4
    for &point in &convergence_points {
        assert!(point > 0);
        assert!(point < node_count);
    }
}

#[test]
fn test_calculate_convergence_points_large_count() {
    // Test with a large node count (32 nodes)
    let node_count = 32;
    let convergence_points = calculate_convergence_points(node_count);

    // Should have multiple convergence points
    assert!(!convergence_points.is_empty());

    // With ratio 0.5, expect convergence around positions 16
    for &point in &convergence_points {
        assert!(point > 0);
        assert!(point < node_count);
    }
}

#[test]
fn test_calculate_convergence_points_minimum_valid() {
    // Minimum node count for meaningful convergence (4 nodes: start, choice, converge, end)
    let node_count = 4;
    let convergence_points = calculate_convergence_points(node_count);

    // Should still have convergence point
    assert!(!convergence_points.is_empty());
}

#[test]
fn test_calculate_convergence_points_ratio_based() {
    // Verify the ratio calculation is working correctly
    let node_count = 20;
    let convergence_points = calculate_convergence_points(node_count);

    // With CONVERGENCE_POINT_RATIO = 0.5, expect convergence around position 10
    // Allow some tolerance for rounding
    let expected_position = (node_count as f64 * CONVERGENCE_POINT_RATIO) as usize;
    let has_point_near_expected = convergence_points
        .iter()
        .any(|&p| (p as i32 - expected_position as i32).abs() <= 2);

    assert!(
        has_point_near_expected,
        "Should have convergence point near expected position based on ratio"
    );
}

// ============================================================================
// DAG STRUCTURE GENERATION TESTS
// ============================================================================

#[test]
fn test_generate_dag_structure_default() {
    let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
    let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points)
        .expect("Should generate DAG successfully");

    // Verify node count
    assert_eq!(
        dag.nodes.len(),
        DEFAULT_NODE_COUNT,
        "Should have correct number of nodes"
    );

    // Verify start node exists
    assert!(
        dag.nodes.contains_key(&dag.start_node_id),
        "Start node should exist"
    );

    // Verify edges exist
    assert!(!dag.edges.is_empty(), "Should have edges connecting nodes");

    // Verify convergence points are set
    assert!(
        !dag.convergence_points.is_empty(),
        "Should have convergence points"
    );
}

#[test]
fn test_generate_dag_structure_small() {
    let node_count = 8;
    let convergence_points = calculate_convergence_points(node_count);
    let dag =
        generate_dag_structure(node_count, convergence_points).expect("Should generate DAG");

    assert_eq!(dag.nodes.len(), node_count);
    assert!(dag.nodes.contains_key(&dag.start_node_id));
}

#[test]
fn test_generate_dag_structure_large() {
    let node_count = 24;
    let convergence_points = calculate_convergence_points(node_count);
    let dag =
        generate_dag_structure(node_count, convergence_points).expect("Should generate DAG");

    assert_eq!(dag.nodes.len(), node_count);
    assert!(dag.nodes.contains_key(&dag.start_node_id));
}

#[test]
fn test_generate_dag_has_root_node() {
    let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
    let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points)
        .expect("Should generate DAG");

    // Root node should have ID "0"
    let root_node = dag.nodes.get("0").expect("Root node with ID '0' should exist");

    // Root node should have no incoming edges
    assert_eq!(root_node.incoming_edges, 0, "Root node should have no incoming edges");

    // Root node should have outgoing edges (branching choices)
    assert!(root_node.outgoing_edges > 0, "Root node should have outgoing edges");
}

#[test]
fn test_generate_dag_has_end_node() {
    let node_count = DEFAULT_NODE_COUNT;
    let convergence_points = calculate_convergence_points(node_count);
    let dag = generate_dag_structure(node_count, convergence_points).expect("Should generate DAG");

    // End node should have ID equal to (node_count - 1)
    let end_node_id = format!("{}", node_count - 1);
    let end_node = dag
        .nodes
        .get(&end_node_id)
        .expect("End node should exist");

    // End node should have no outgoing edges
    assert_eq!(end_node.outgoing_edges, 0, "End node should have no outgoing edges");

    // End node should have incoming edges
    assert!(end_node.incoming_edges > 0, "End node should have incoming edges");
}

#[test]
fn test_generate_dag_branching_structure() {
    let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
    let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points)
        .expect("Should generate DAG");

    // Count nodes with multiple outgoing edges (branching points)
    let branching_nodes = dag
        .nodes
        .values()
        .filter(|node| node.outgoing_edges >= 2)
        .count();

    // Should have at least some branching nodes
    assert!(branching_nodes > 0, "Should have nodes with branching choices");

    // Count nodes with multiple incoming edges (convergence points)
    let converging_nodes = dag
        .nodes
        .values()
        .filter(|node| node.incoming_edges >= 2)
        .count();

    // Should have convergence points
    assert!(converging_nodes > 0, "Should have convergence points");
}

#[test]
fn test_generate_dag_edges_connect_valid_nodes() {
    let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
    let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points)
        .expect("Should generate DAG");

    // Every edge should connect existing nodes
    for edge in &dag.edges {
        assert!(
            dag.nodes.contains_key(&edge.from_node_id),
            "Edge source node should exist: {}",
            edge.from_node_id
        );
        assert!(
            dag.nodes.contains_key(&edge.to_node_id),
            "Edge target node should exist: {}",
            edge.to_node_id
        );
    }
}

#[test]
fn test_generate_dag_convergence_points_marked() {
    let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
    let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points.clone())
        .expect("Should generate DAG");

    // All specified convergence points should be in the DAG
    for cp_index in &convergence_points {
        let cp_id = format!("{}", cp_index);
        assert!(
            dag.convergence_points.contains(&cp_id),
            "Convergence point {} should be marked in DAG",
            cp_id
        );
    }
}

#[test]
fn test_generate_dag_min_node_count() {
    // Test with minimum viable node count
    let node_count = 4; // start, choice, converge, end
    let convergence_points = calculate_convergence_points(node_count);
    let dag =
        generate_dag_structure(node_count, convergence_points).expect("Should generate DAG");

    assert_eq!(dag.nodes.len(), node_count);
    assert!(!dag.edges.is_empty());
}

#[test]
fn test_generate_dag_invalid_node_count_zero() {
    let convergence_points = vec![];
    let result = generate_dag_structure(0, convergence_points);
    assert!(result.is_err(), "Should error with zero nodes");
    assert!(
        result.unwrap_err().to_string().contains("Node count must be at least"),
        "Error should mention minimum node count"
    );
}

#[test]
fn test_generate_dag_invalid_node_count_one() {
    let convergence_points = vec![];
    let result = generate_dag_structure(1, convergence_points);
    assert!(result.is_err(), "Should error with one node");
    assert!(
        result.unwrap_err().to_string().contains("Node count must be at least"),
        "Error should mention minimum node count"
    );
}

#[test]
fn test_generate_dag_invalid_node_count_two() {
    let convergence_points = vec![];
    let result = generate_dag_structure(2, convergence_points);
    assert!(result.is_err(), "Should error with two nodes");
    assert!(
        result.unwrap_err().to_string().contains("Node count must be at least"),
        "Error should mention minimum node count"
    );
}

// ============================================================================
// PATH CONNECTIVITY VALIDATION TESTS
// ============================================================================

#[test]
fn test_validate_path_connectivity_valid_dag() {
    let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
    let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points)
        .expect("Should generate DAG");

    // Should validate successfully
    validate_path_connectivity(&dag).expect("Generated DAG should have valid connectivity");
}

#[test]
fn test_validate_path_connectivity_detects_isolated_node() {
    // Create a DAG with an isolated node
    let mut dag = DAG {
        nodes: std::collections::HashMap::new(),
        edges: vec![],
        start_node_id: "0".to_string(),
        convergence_points: vec![],
    };

    // Add nodes
    let node0 = create_test_node("0");
    let node1 = create_test_node("1");
    let node2 = create_test_node("2"); // This will be isolated

    dag.add_node(node0);
    dag.add_node(node1);
    dag.add_node(node2);

    // Add edge only between 0 and 1 (2 is isolated)
    let edge = Edge {
        from_node_id: "0".to_string(),
        to_node_id: "1".to_string(),
        choice_id: "choice_0".to_string(),
        weight: None,
    };
    dag.add_edge(edge).expect("Should add edge");

    // Should fail validation due to isolated node
    let result = validate_path_connectivity(&dag);
    assert!(result.is_err(), "Should detect isolated node");
}

#[test]
fn test_validate_path_connectivity_empty_dag() {
    let dag = DAG {
        nodes: std::collections::HashMap::new(),
        edges: vec![],
        start_node_id: "0".to_string(),
        convergence_points: vec![],
    };

    // Should fail - no nodes
    let result = validate_path_connectivity(&dag);
    assert!(result.is_err(), "Empty DAG should fail validation");
}

// ============================================================================
// REACHABILITY VALIDATION TESTS
// ============================================================================

#[test]
fn test_validate_reachability_valid_dag() {
    let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
    let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points)
        .expect("Should generate DAG");

    // Should validate successfully
    validate_reachability(&dag).expect("Generated DAG should have valid reachability");
}

#[test]
fn test_validate_reachability_all_nodes_from_root() {
    let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
    let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points)
        .expect("Should generate DAG");

    // Manually verify all nodes are reachable from root
    for (node_id, _) in &dag.nodes {
        assert!(
            dag.is_reachable(&dag.start_node_id, node_id),
            "Node {} should be reachable from root",
            node_id
        );
    }
}

#[test]
fn test_validate_reachability_all_leaves_reach_end() {
    let node_count = DEFAULT_NODE_COUNT;
    let convergence_points = calculate_convergence_points(node_count);
    let dag = generate_dag_structure(node_count, convergence_points).expect("Should generate DAG");

    let end_node_id = format!("{}", node_count - 1);

    // Find all leaf nodes (nodes with no outgoing edges except the end node itself)
    let leaf_nodes: Vec<_> = dag
        .nodes
        .iter()
        .filter(|(id, node)| node.outgoing_edges == 0 && *id != &end_node_id)
        .collect();

    // All leaf nodes should be able to reach the end node
    for (leaf_id, _) in leaf_nodes {
        assert!(
            dag.is_reachable(leaf_id, &end_node_id),
            "Leaf node {} should be able to reach end node",
            leaf_id
        );
    }
}

#[test]
fn test_validate_reachability_detects_unreachable_node() {
    // Create a DAG with an unreachable node
    let mut dag = DAG {
        nodes: std::collections::HashMap::new(),
        edges: vec![],
        start_node_id: "0".to_string(),
        convergence_points: vec![],
    };

    // Add nodes
    let node0 = create_test_node("0");
    let node1 = create_test_node("1");
    let node2 = create_test_node("2");
    let node3 = create_test_node("3"); // This will be unreachable from start

    dag.add_node(node0);
    dag.add_node(node1);
    dag.add_node(node2);
    dag.add_node(node3);

    // Create a path from 0 to 1 to 2
    let edge1 = Edge {
        from_node_id: "0".to_string(),
        to_node_id: "1".to_string(),
        choice_id: "choice_0".to_string(),
        weight: None,
    };
    let edge2 = Edge {
        from_node_id: "1".to_string(),
        to_node_id: "2".to_string(),
        choice_id: "choice_1".to_string(),
        weight: None,
    };

    dag.add_edge(edge1).expect("Should add edge");
    dag.add_edge(edge2).expect("Should add edge");

    // Node 3 is not connected - should fail validation
    let result = validate_reachability(&dag);
    assert!(result.is_err(), "Should detect unreachable node");
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_full_structure_generation_pipeline() {
    // Test the complete pipeline: calculate -> generate -> validate
    let node_count = DEFAULT_NODE_COUNT;

    // Step 1: Calculate convergence points
    let convergence_points = calculate_convergence_points(node_count);
    assert!(!convergence_points.is_empty());

    // Step 2: Generate DAG structure
    let dag =
        generate_dag_structure(node_count, convergence_points).expect("Should generate DAG");

    // Step 3: Validate connectivity
    validate_path_connectivity(&dag).expect("Should have valid connectivity");

    // Step 4: Validate reachability
    validate_reachability(&dag).expect("Should have valid reachability");

    // Step 5: Use DAG extension trait for additional validation
    dag.validate_structure().expect("DAG structure should be valid");
}

#[test]
fn test_structure_generation_with_different_sizes() {
    // Test structure generation with various node counts
    let test_sizes = vec![4, 8, 12, 16, 20, 24, 32];

    for node_count in test_sizes {
        let convergence_points = calculate_convergence_points(node_count);
        let dag =
            generate_dag_structure(node_count, convergence_points).expect("Should generate DAG");

        assert_eq!(dag.nodes.len(), node_count);
        validate_path_connectivity(&dag).expect("Should have valid connectivity");
        validate_reachability(&dag).expect("Should have valid reachability");
    }
}

#[test]
fn test_dag_has_multiple_paths() {
    let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
    let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points)
        .expect("Should generate DAG");

    let end_node_id = format!("{}", DEFAULT_NODE_COUNT - 1);

    // Find paths from start to end
    let paths = dag.find_paths(&dag.start_node_id, &end_node_id);

    // Should have multiple paths (branching narrative)
    assert!(
        paths.len() > 1,
        "Should have multiple paths from start to end for branching narrative"
    );
}

#[test]
fn test_convergence_points_have_multiple_incoming() {
    let convergence_points_indices = calculate_convergence_points(DEFAULT_NODE_COUNT);
    let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points_indices.clone())
        .expect("Should generate DAG");

    // Verify that at least one convergence point has multiple incoming edges
    // (Some convergence points might only have 1 incoming edge depending on graph structure)
    let points_with_multiple_incoming = convergence_points_indices
        .iter()
        .filter_map(|&cp_index| {
            let cp_id = format!("{}", cp_index);
            dag.nodes.get(&cp_id)
        })
        .filter(|node| node.incoming_edges >= 2)
        .count();

    assert!(
        points_with_multiple_incoming > 0,
        "At least one convergence point should have multiple incoming edges"
    );
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a test ContentNode with minimal data
fn create_test_node(id: &str) -> ContentNode {
    use shared_types::Content;

    let content = Content {
        r#type: "interactive_story_node".to_string(),
        node_id: id.to_string(),
        text: String::new(),
        choices: vec![],
        convergence_point: false,
        next_nodes: vec![],
        educational_content: None,
    };

    ContentNode {
        id: id.to_string(),
        content,
        incoming_edges: 0,
        outgoing_edges: 0,
        generation_metadata: None,
    }
}
