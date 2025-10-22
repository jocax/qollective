//! Convergence pattern algorithm tests
//!
//! These tests verify the new convergence point calculation algorithms based on
//! ConvergencePattern enum and DagStructureConfig. They test all five patterns:
//! - SingleConvergence: One convergence point at specified ratio
//! - MultipleConvergence: Multiple convergence points at intervals
//! - EndOnly: Convergence only at the end (high ratio ~0.9)
//! - PureBranching: No convergence points (fully branching narrative)
//! - ParallelPaths: Parallel story threads with minimal convergence

use shared_types::{ConvergencePattern, DagStructureConfig};
use story_generator::structure::{calculate_convergence_points, ConvergenceConfig};

// ============================================================================
// SINGLE CONVERGENCE PATTERN TESTS
// ============================================================================

#[test]
fn test_single_convergence_12_nodes_ratio_50() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::SingleConvergence,
        ratio: Some(0.5),
    };

    let result = calculate_convergence_points(12, &config);

    // With 12 nodes and ratio 0.5, convergence should be at node 6
    assert_eq!(result.len(), 1, "SingleConvergence should produce exactly one convergence point");
    assert_eq!(result[0], 6, "Convergence should be at node 6 (50% of 12)");
}

#[test]
fn test_single_convergence_16_nodes_ratio_50() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::SingleConvergence,
        ratio: Some(0.5),
    };

    let result = calculate_convergence_points(16, &config);

    // With 16 nodes and ratio 0.5, convergence should be at node 8
    assert_eq!(result.len(), 1, "SingleConvergence should produce exactly one convergence point");
    assert_eq!(result[0], 8, "Convergence should be at node 8 (50% of 16)");
}

#[test]
fn test_single_convergence_20_nodes_ratio_33() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::SingleConvergence,
        ratio: Some(0.33),
    };

    let result = calculate_convergence_points(20, &config);

    // With 20 nodes and ratio 0.33, convergence should be at node 6 or 7 (0.33 * 20 = 6.6)
    assert_eq!(result.len(), 1, "SingleConvergence should produce exactly one convergence point");
    assert!(
        result[0] == 6 || result[0] == 7,
        "Convergence should be at node 6 or 7 (33% of 20), got {}",
        result[0]
    );
}

#[test]
fn test_single_convergence_24_nodes_ratio_75() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::SingleConvergence,
        ratio: Some(0.75),
    };

    let result = calculate_convergence_points(24, &config);

    // With 24 nodes and ratio 0.75, convergence should be at node 18
    assert_eq!(result.len(), 1, "SingleConvergence should produce exactly one convergence point");
    assert_eq!(result[0], 18, "Convergence should be at node 18 (75% of 24)");
}

// ============================================================================
// MULTIPLE CONVERGENCE PATTERN TESTS
// ============================================================================

#[test]
fn test_multiple_convergence_16_nodes_ratio_33() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::MultipleConvergence,
        ratio: Some(0.33),
    };

    let result = calculate_convergence_points(16, &config);

    // With 16 nodes and ratio 0.33, expect convergence at ~33% intervals
    // First convergence: 16 * 0.33 ≈ 5
    // Second convergence: 16 * 0.66 ≈ 10-11
    assert_eq!(result.len(), 2, "MultipleConvergence with ratio 0.33 should produce 2 convergence points");

    // Allow tolerance of ±1 for rounding
    assert!(
        (result[0] as i32 - 5).abs() <= 1,
        "First convergence should be around node 5, got {}",
        result[0]
    );
    assert!(
        (result[1] as i32 - 10).abs() <= 1,
        "Second convergence should be around node 10, got {}",
        result[1]
    );

    // Verify convergence points are in ascending order
    assert!(result[0] < result[1], "Convergence points should be in ascending order");
}

#[test]
fn test_multiple_convergence_24_nodes_ratio_25() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::MultipleConvergence,
        ratio: Some(0.25),
    };

    let result = calculate_convergence_points(24, &config);

    // With 24 nodes and ratio 0.25, expect convergence at 25%, 50%, 75%
    // Convergence at: 6, 12, 18
    assert_eq!(result.len(), 3, "MultipleConvergence with ratio 0.25 should produce 3 convergence points");

    assert_eq!(result[0], 6, "First convergence at 25%");
    assert_eq!(result[1], 12, "Second convergence at 50%");
    assert_eq!(result[2], 18, "Third convergence at 75%");
}

#[test]
fn test_multiple_convergence_32_nodes_ratio_20() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::MultipleConvergence,
        ratio: Some(0.20),
    };

    let result = calculate_convergence_points(32, &config);

    // With 32 nodes and ratio 0.20, expect convergence at 20%, 40%, 60%, 80%
    // Convergence at: 6-7, 12-13, 19-20, 25-26
    assert_eq!(result.len(), 4, "MultipleConvergence with ratio 0.20 should produce 4 convergence points");

    // Verify all points are within valid range
    for (i, &point) in result.iter().enumerate() {
        assert!(
            point > 0 && point < 32,
            "Convergence point {} at index {} should be between 0 and 32",
            point,
            i
        );
    }

    // Verify ascending order
    for i in 0..result.len() - 1 {
        assert!(
            result[i] < result[i + 1],
            "Convergence points should be in ascending order"
        );
    }
}

// ============================================================================
// END ONLY PATTERN TESTS
// ============================================================================

#[test]
fn test_end_only_24_nodes_ratio_90() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::EndOnly,
        ratio: Some(0.9),
    };

    let result = calculate_convergence_points(24, &config);

    // With 24 nodes and ratio 0.9, convergence should be at node 21 or 22
    assert_eq!(result.len(), 1, "EndOnly should produce exactly one convergence point");

    let expected = (24.0_f64 * 0.9).round() as usize;
    assert!(
        (result[0] as i32 - expected as i32).abs() <= 1,
        "Convergence should be around node {} (90% of 24), got {}",
        expected,
        result[0]
    );

    // Ensure convergence is near the end (>80% of nodes)
    assert!(
        result[0] > (24 * 80 / 100),
        "EndOnly convergence should be in final 20% of nodes"
    );
}

#[test]
fn test_end_only_20_nodes_ratio_85() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::EndOnly,
        ratio: Some(0.85),
    };

    let result = calculate_convergence_points(20, &config);

    // With 20 nodes and ratio 0.85, convergence should be at node 17
    assert_eq!(result.len(), 1, "EndOnly should produce exactly one convergence point");
    assert_eq!(result[0], 17, "Convergence should be at node 17 (85% of 20)");
}

#[test]
fn test_end_only_16_nodes_ratio_95() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::EndOnly,
        ratio: Some(0.95),
    };

    let result = calculate_convergence_points(16, &config);

    // With 16 nodes (indices 0-15) and ratio 0.95, raw calculation is 15.2 → 15
    // But EndOnly clamps to [1, node_count-2] = [1, 14] to ensure not AT the final node
    assert_eq!(result.len(), 1, "EndOnly should produce exactly one convergence point");
    assert_eq!(result[0], 14, "Convergence should be at node 14 (clamped from 95% to avoid final node)");

    // Must not be the final node (index 15)
    assert!(
        result[0] < 15,
        "EndOnly convergence must not be at the final node"
    );
}

// ============================================================================
// PURE BRANCHING PATTERN TESTS
// ============================================================================

#[test]
fn test_pure_branching_16_nodes() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::PureBranching,
        ratio: None, // Ratio not used for PureBranching
    };

    let result = calculate_convergence_points(16, &config);

    // PureBranching should have NO convergence points
    assert_eq!(
        result.len(),
        0,
        "PureBranching should have no convergence points"
    );
}

#[test]
fn test_pure_branching_24_nodes() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::PureBranching,
        ratio: None,
    };

    let result = calculate_convergence_points(24, &config);

    assert_eq!(
        result.len(),
        0,
        "PureBranching should have no convergence points regardless of node count"
    );
}

#[test]
fn test_pure_branching_ignores_ratio() {
    // Even if ratio is provided, it should be ignored
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::PureBranching,
        ratio: Some(0.5), // This should be ignored
    };

    let result = calculate_convergence_points(20, &config);

    assert_eq!(
        result.len(),
        0,
        "PureBranching should ignore ratio and have no convergence points"
    );
}

// ============================================================================
// PARALLEL PATHS PATTERN TESTS
// ============================================================================

#[test]
fn test_parallel_paths_16_nodes() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::ParallelPaths,
        ratio: None, // Ratio not used for ParallelPaths
    };

    let result = calculate_convergence_points(16, &config);

    // ParallelPaths should have minimal convergence (0-1 points)
    assert!(
        result.len() <= 1,
        "ParallelPaths should have at most 1 convergence point, got {}",
        result.len()
    );

    // If there is a convergence point, it should be near the end
    if !result.is_empty() {
        assert!(
            result[0] > (16 * 70 / 100),
            "ParallelPaths convergence should be in final 30% of nodes"
        );
    }
}

#[test]
fn test_parallel_paths_24_nodes() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::ParallelPaths,
        ratio: None,
    };

    let result = calculate_convergence_points(24, &config);

    assert!(
        result.len() <= 1,
        "ParallelPaths should have at most 1 convergence point"
    );
}

#[test]
fn test_parallel_paths_ignores_ratio() {
    // Even if ratio is provided, it should be ignored
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::ParallelPaths,
        ratio: Some(0.5), // This should be ignored
    };

    let result = calculate_convergence_points(32, &config);

    assert!(
        result.len() <= 1,
        "ParallelPaths should ignore ratio and have at most 1 convergence point"
    );
}

// ============================================================================
// EDGE CASE TESTS - MINIMUM NODE COUNT
// ============================================================================

#[test]
fn test_minimum_node_count_single_convergence() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::SingleConvergence,
        ratio: Some(0.5),
    };

    let result = calculate_convergence_points(4, &config);

    // With 4 nodes (minimum), convergence should be at node 2
    assert_eq!(result.len(), 1, "SingleConvergence should produce one point even at minimum");
    assert_eq!(result[0], 2, "Convergence at 50% of 4 nodes");
}

#[test]
fn test_minimum_node_count_multiple_convergence() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::MultipleConvergence,
        ratio: Some(0.33),
    };

    let result = calculate_convergence_points(4, &config);

    // With 4 nodes, even MultipleConvergence might only have 1 point
    assert!(
        !result.is_empty(),
        "Should have at least one convergence point"
    );
    assert!(
        result.len() <= 2,
        "With only 4 nodes, should have at most 2 convergence points"
    );
}

#[test]
fn test_minimum_node_count_end_only() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::EndOnly,
        ratio: Some(0.75),
    };

    let result = calculate_convergence_points(4, &config);

    // With 4 nodes (indices 0-3) and ratio 0.75, raw calculation is 3
    // But EndOnly clamps to [1, node_count-2] = [1, 2] to ensure not AT the final node
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 2, "EndOnly convergence at node 2 (clamped from 75% to avoid final node)");
}

#[test]
fn test_minimum_node_count_pure_branching() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::PureBranching,
        ratio: None,
    };

    let result = calculate_convergence_points(4, &config);

    assert_eq!(result.len(), 0, "PureBranching has no convergence even at minimum");
}

// ============================================================================
// EDGE CASE TESTS - MAXIMUM NODE COUNT
// ============================================================================

#[test]
fn test_maximum_node_count_single_convergence() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::SingleConvergence,
        ratio: Some(0.5),
    };

    let result = calculate_convergence_points(100, &config);

    // With 100 nodes and ratio 0.5, convergence at node 50
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 50, "Convergence at 50% of 100 nodes");
}

#[test]
fn test_maximum_node_count_multiple_convergence() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::MultipleConvergence,
        ratio: Some(0.2),
    };

    let result = calculate_convergence_points(100, &config);

    // With 100 nodes and ratio 0.2, expect convergence at 20%, 40%, 60%, 80%
    // Nodes: 20, 40, 60, 80
    assert_eq!(result.len(), 4);
    assert_eq!(result[0], 20);
    assert_eq!(result[1], 40);
    assert_eq!(result[2], 60);
    assert_eq!(result[3], 80);
}

#[test]
fn test_maximum_node_count_end_only() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::EndOnly,
        ratio: Some(0.9),
    };

    let result = calculate_convergence_points(100, &config);

    // With 100 nodes and ratio 0.9, convergence at node 90
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 90, "Convergence at 90% of 100 nodes");
}

#[test]
fn test_maximum_node_count_pure_branching() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::PureBranching,
        ratio: None,
    };

    let result = calculate_convergence_points(100, &config);

    assert_eq!(result.len(), 0, "PureBranching has no convergence even at maximum");
}

// ============================================================================
// DAGSTRUCTURECONFIG CONVERSION TESTS
// ============================================================================

#[test]
fn test_from_dag_structure_config_single_convergence() {
    let dag_config = DagStructureConfig {
        node_count: 16,
        convergence_pattern: ConvergencePattern::SingleConvergence,
        convergence_point_ratio: Some(0.5),
        max_depth: 5,
        branching_factor: 2,
    };

    let conv_config = ConvergenceConfig::from(&dag_config);

    assert_eq!(conv_config.pattern, ConvergencePattern::SingleConvergence);
    assert_eq!(conv_config.ratio, Some(0.5));

    let result = calculate_convergence_points(dag_config.node_count, &conv_config);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 8);
}

#[test]
fn test_from_dag_structure_config_pure_branching() {
    let dag_config = DagStructureConfig {
        node_count: 20,
        convergence_pattern: ConvergencePattern::PureBranching,
        convergence_point_ratio: None,
        max_depth: 6,
        branching_factor: 3,
    };

    let conv_config = ConvergenceConfig::from(&dag_config);

    assert_eq!(conv_config.pattern, ConvergencePattern::PureBranching);
    assert_eq!(conv_config.ratio, None);

    let result = calculate_convergence_points(dag_config.node_count, &conv_config);
    assert_eq!(result.len(), 0);
}

// ============================================================================
// VALIDATION TESTS - ERROR HANDLING
// ============================================================================

#[test]
#[should_panic(expected = "SingleConvergence requires a ratio")]
fn test_single_convergence_missing_ratio_panics() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::SingleConvergence,
        ratio: None, // Missing ratio should panic
    };

    calculate_convergence_points(16, &config);
}

#[test]
#[should_panic(expected = "MultipleConvergence requires a ratio")]
fn test_multiple_convergence_missing_ratio_panics() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::MultipleConvergence,
        ratio: None, // Missing ratio should panic
    };

    calculate_convergence_points(16, &config);
}

#[test]
#[should_panic(expected = "EndOnly requires a ratio")]
fn test_end_only_missing_ratio_panics() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::EndOnly,
        ratio: None, // Missing ratio should panic
    };

    calculate_convergence_points(16, &config);
}

// ============================================================================
// BOUNDARY TESTS - RATIO VALUES
// ============================================================================

#[test]
fn test_ratio_near_zero() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::SingleConvergence,
        ratio: Some(0.1), // Very early convergence
    };

    let result = calculate_convergence_points(20, &config);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 2, "10% of 20 nodes");
}

#[test]
fn test_ratio_near_one() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::SingleConvergence,
        ratio: Some(0.95), // Very late convergence
    };

    let result = calculate_convergence_points(20, &config);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 19, "95% of 20 nodes");

    // Must not equal the final node
    assert!(result[0] < 20, "Convergence must be before final node");
}

#[test]
fn test_ratio_exactly_half() {
    let config = ConvergenceConfig {
        pattern: ConvergencePattern::SingleConvergence,
        ratio: Some(0.5),
    };

    // Test with both even and odd node counts
    let result_even = calculate_convergence_points(20, &config);
    assert_eq!(result_even[0], 10, "50% of 20 nodes");

    let result_odd = calculate_convergence_points(21, &config);
    // With 21 nodes, 50% = 10.5, should round to 10 or 11
    assert!(
        result_odd[0] == 10 || result_odd[0] == 11,
        "50% of 21 nodes should round to 10 or 11"
    );
}
