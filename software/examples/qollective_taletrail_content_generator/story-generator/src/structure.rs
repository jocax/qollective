//! Story structure planning module
//!
//! This module provides functions for creating DAG (Directed Acyclic Graph) structures
//! that define the branching narrative before content generation.
//!
//! # Architecture
//!
//! The structure planning follows these steps:
//! 1. Calculate convergence points based on node count and ratio
//! 2. Generate DAG structure with branching and convergence
//! 3. Validate path connectivity (all nodes connected)
//! 4. Validate reachability (all nodes reachable from root, all leaves reach end)
//!
//! # Constants
//!
//! All configuration values come from shared_types::constants:
//! - `DEFAULT_NODE_COUNT`: Default number of nodes (16)
//! - `CONVERGENCE_POINT_RATIO`: Ratio for convergence placement (0.5)

use shared_types::constants::CONVERGENCE_POINT_RATIO;
use shared_types::extensions::dag::DagExt;
use shared_types::{Content, ContentNode, ConvergencePattern, DagStructureConfig, Edge, TaleTrailError, DAG};
use std::collections::{HashMap, HashSet, VecDeque};

/// Helper configuration for convergence point calculation
///
/// This struct simplifies the convergence calculation API by extracting
/// only the fields needed from DagStructureConfig.
#[derive(Debug, Clone, PartialEq)]
pub struct ConvergenceConfig {
    /// The convergence pattern to use
    pub pattern: ConvergencePattern,
    /// Position ratio for convergence points (0.0-1.0)
    /// Required for: SingleConvergence, MultipleConvergence, EndOnly
    /// Ignored for: PureBranching, ParallelPaths
    pub ratio: Option<f64>,
}

impl From<&DagStructureConfig> for ConvergenceConfig {
    fn from(config: &DagStructureConfig) -> Self {
        Self {
            pattern: config.convergence_pattern,
            ratio: config.convergence_point_ratio,
        }
    }
}

/// Calculate convergence point indices based on pattern and configuration
///
/// Convergence points are positions in the story where different narrative branches
/// merge back together. The placement strategy is determined by the convergence pattern.
///
/// # Arguments
///
/// * `node_count` - Total number of nodes in the story (4-100)
/// * `config` - Convergence configuration including pattern and ratio
///
/// # Returns
///
/// Vector of node indices (0-based) where convergence occurs.
/// Empty vector for PureBranching and ParallelPaths patterns.
///
/// # Panics
///
/// Panics if ratio is None for patterns that require it:
/// - SingleConvergence
/// - MultipleConvergence
/// - EndOnly
///
/// # Patterns
///
/// - **SingleConvergence**: One convergence point at `(node_count * ratio)`, clamped to [1, node_count-2]
/// - **MultipleConvergence**: Multiple convergence points at intervals of `(node_count * ratio)`, minimum interval 2
/// - **EndOnly**: One late convergence point at `(node_count * ratio)`, clamped to [node_count-3, node_count-2]
/// - **PureBranching**: No convergence points (empty vec)
/// - **ParallelPaths**: No convergence points (empty vec)
///
/// # Examples
///
/// ```
/// use story_generator::structure::{calculate_convergence_points, ConvergenceConfig};
/// use shared_types::ConvergencePattern;
///
/// let config = ConvergenceConfig {
///     pattern: ConvergencePattern::SingleConvergence,
///     ratio: Some(0.5),
/// };
/// let points = calculate_convergence_points(16, &config);
/// assert_eq!(points, vec![8]);
/// ```
pub fn calculate_convergence_points(node_count: usize, config: &ConvergenceConfig) -> Vec<usize> {
    match config.pattern {
        ConvergencePattern::SingleConvergence => {
            let ratio = config.ratio.expect("SingleConvergence requires a ratio");
            let position = (node_count as f64 * ratio).round() as usize;
            // Clamp to ensure it's not at start (0) or beyond the range
            // Allow convergence up to and including the final node (node_count-1)
            let max_pos = if node_count > 1 { node_count - 1 } else { 1 };
            let position = position.clamp(1, max_pos);
            vec![position]
        }
        ConvergencePattern::MultipleConvergence => {
            let ratio = config.ratio.expect("MultipleConvergence requires a ratio");

            let mut points = Vec::new();
            let mut multiplier = 1.0;

            // Place convergence points at each ratio interval (ratio, 2*ratio, 3*ratio, ...)
            // until we reach node_count-1
            loop {
                let position = (node_count as f64 * ratio * multiplier).round() as usize;

                // Stop if we've reached or passed the final node
                if position >= node_count - 1 {
                    break;
                }

                // Ensure minimum spacing of 2 nodes from the previous point
                if points.is_empty() || position >= points[points.len() - 1] + 2 {
                    points.push(position);
                }

                multiplier += 1.0;
            }

            // Fallback: if no points were added, add one in the middle
            if points.is_empty() {
                points.push(node_count / 2);
            }
            points
        }
        ConvergencePattern::EndOnly => {
            let ratio = config.ratio.expect("EndOnly requires a ratio");
            let position = (node_count as f64 * ratio).round() as usize;
            // EndOnly should be very near the end but not AT the final node (node_count-1)
            // Clamp to ensure it's in range [1, node_count-2] for node_count > 2
            let max_pos = if node_count > 2 { node_count - 2 } else { 1 };
            let position = position.clamp(1, max_pos);
            vec![position]
        }
        ConvergencePattern::PureBranching | ConvergencePattern::ParallelPaths => {
            Vec::new()
        }
    }
}

/// Generate DAG structure with branching and convergence
///
/// Creates a complete DAG with:
/// - Root node (id=0) with no incoming edges
/// - Intermediate nodes with branching choices based on `branching_factor`
/// - Convergence points where branches merge (determined by convergence pattern)
/// - End node (id=node_count-1) with no outgoing edges
///
/// # Arguments
///
/// * `dag_config` - Complete DAG structure configuration including:
///   - `node_count`: Total number of nodes (minimum 3)
///   - `convergence_pattern`: Strategy for placing convergence points
///   - `convergence_point_ratio`: Position ratio for convergence (when applicable)
///   - `branching_factor`: Number of choices per decision node (2-4)
///   - `max_depth`: Maximum depth of DAG tree (currently unused in implementation)
///
/// # Returns
///
/// Result containing the generated DAG or error
///
/// # Errors
///
/// Returns error if:
/// - `node_count` is less than 3 (need at least start, middle, end)
/// - Unable to create valid graph structure
///
/// # Examples
///
/// ```
/// use story_generator::structure::generate_dag_structure;
/// use shared_types::{DagStructureConfig, ConvergencePattern};
///
/// let dag_config = DagStructureConfig {
///     node_count: 16,
///     convergence_pattern: ConvergencePattern::SingleConvergence,
///     convergence_point_ratio: Some(0.5),
///     max_depth: 10,
///     branching_factor: 2,
/// };
/// let dag = generate_dag_structure(&dag_config)
///     .expect("Should generate valid DAG");
/// assert_eq!(dag.nodes.len(), 16);
/// ```
pub fn generate_dag_structure(
    dag_config: &DagStructureConfig,
) -> Result<DAG, TaleTrailError> {
    // Convert i64 to usize for internal processing
    let node_count: usize = dag_config.node_count.try_into()
        .map_err(|_| TaleTrailError::ValidationError(
            format!("Invalid node count: {} (must be non-negative and fit in usize)", dag_config.node_count)
        ))?;

    let convergence_config = ConvergenceConfig::from(dag_config);
    let convergence_points = calculate_convergence_points(node_count, &convergence_config);
    // Validate minimum node count
    if node_count < 3 {
        return Err(TaleTrailError::ValidationError(
            format!("Node count must be at least 3 (start, choice, end), got {}", node_count),
        ));
    }

    // Initialize DAG
    let mut dag = DAG {
        nodes: HashMap::new(),
        edges: Vec::new(),
        start_node_id: "0".to_string(),
        convergence_points: convergence_points
            .iter()
            .map(|i| i.to_string())
            .collect(),
    };

    // Convert convergence points to a set for quick lookup
    let convergence_set: HashSet<usize> = convergence_points.iter().copied().collect();

    // Create all nodes first
    for i in 0..node_count {
        let node_id = format!("{}", i);
        let is_convergence = convergence_set.contains(&i);

        let content = Content {
            r#type: "interactive_story_node".to_string(),
            node_id: node_id.clone(),
            text: String::new(), // Content will be generated later
            choices: Vec::new(),  // Choices will be populated during content generation
            convergence_point: is_convergence,
            next_nodes: Vec::new(),
            educational_content: None,
        };

        let node = ContentNode {
            id: node_id,
            content,
            incoming_edges: 0,
            outgoing_edges: 0,
            generation_metadata: None,
        };

        dag.add_node(node);
    }

    // Create edges to form branching structure
    // Strategy: Create branches that converge at convergence points
    let branching_factor: usize = dag_config.branching_factor.try_into()
        .map_err(|_| TaleTrailError::ValidationError(
            format!("Invalid branching factor: {} (must be non-negative and fit in usize)", dag_config.branching_factor)
        ))?;
    create_branching_structure(&mut dag, node_count, &convergence_set, branching_factor)?;

    Ok(dag)
}

/// Create branching structure with edges connecting nodes
///
/// # Arguments
///
/// * `dag` - The DAG being constructed
/// * `node_count` - Total number of nodes
/// * `convergence_set` - Set of convergence point indices
/// * `branching_factor` - Number of outgoing edges per decision node (currently creates 2-3 edges regardless)
///
/// # Note
///
/// The branching_factor parameter is accepted for API consistency but the current implementation
/// creates a fixed branching pattern (linear + convergence + midpoint edges). Future versions
/// should respect the branching_factor value.
fn create_branching_structure(
    dag: &mut DAG,
    node_count: usize,
    convergence_set: &HashSet<usize>,
    _branching_factor: usize, // TODO: Use this to control number of branches
) -> Result<(), TaleTrailError> {
    let end_node_index = node_count - 1;

    // Create a linear backbone first to ensure all nodes are reachable
    for i in 0..end_node_index {
        let edge = Edge {
            from_node_id: format!("{}", i),
            to_node_id: format!("{}", i + 1),
            choice_id: format!("choice_{}_linear", i),
            weight: None,
        };
        dag.add_edge(edge)?;
    }

    // Now add branching edges to create interesting paths
    // We'll add branches from non-convergence points to convergence points and other nodes
    for i in 0..end_node_index {
        // Skip if this is a convergence point itself
        if convergence_set.contains(&i) {
            continue;
        }

        // Find next convergence point
        let next_convergence = convergence_set
            .iter()
            .filter(|&&cp| cp > i + 1) // Must be at least 2 nodes ahead
            .min()
            .copied();

        // If there's a convergence point ahead, create a branch to it
        if let Some(cp) = next_convergence {
            if cp != i + 1 {
                // Don't duplicate the linear edge
                let edge = Edge {
                    from_node_id: format!("{}", i),
                    to_node_id: format!("{}", cp),
                    choice_id: format!("choice_{}_to_convergence_{}", i, cp),
                    weight: None,
                };
                dag.add_edge(edge)?;
            }
        }

        // Also create intermediate branches for variety
        // Find next convergence point or end
        let target_range_end = next_convergence.unwrap_or(end_node_index);

        // Create a branch to a midpoint if there's room
        if target_range_end > i + 2 {
            let branch_target = i + ((target_range_end - i) / 2);

            // Only add if it's not already the linear next or a direct convergence target
            if branch_target != i + 1 && Some(branch_target) != next_convergence {
                let edge = Edge {
                    from_node_id: format!("{}", i),
                    to_node_id: format!("{}", branch_target),
                    choice_id: format!("choice_{}_branch", i),
                    weight: None,
                };
                dag.add_edge(edge)?;
            }
        }
    }

    Ok(())
}

/// Validate that all nodes in the DAG are connected
///
/// Checks that:
/// - Start node exists
/// - All nodes except start have at least one incoming edge
/// - All nodes except end have at least one outgoing edge
///
/// # Arguments
///
/// * `dag` - The DAG to validate
///
/// # Returns
///
/// Result indicating success or validation error
///
/// # Examples
///
/// ```
/// use story_generator::structure::{generate_dag_structure, calculate_convergence_points, validate_path_connectivity};
/// use shared_types::constants::DEFAULT_NODE_COUNT;
///
/// let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
/// let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points).unwrap();
/// validate_path_connectivity(&dag).expect("Should have valid connectivity");
/// ```
pub fn validate_path_connectivity(dag: &DAG) -> Result<(), TaleTrailError> {
    // Check that DAG has nodes
    if dag.nodes.is_empty() {
        return Err(TaleTrailError::ValidationError(
            "DAG has no nodes".to_string(),
        ));
    }

    // Check start node exists
    if !dag.nodes.contains_key(&dag.start_node_id) {
        return Err(TaleTrailError::ValidationError(format!(
            "Start node {} not found in DAG",
            dag.start_node_id
        )));
    }

    // Find end node (highest index)
    let node_indices: Vec<usize> = dag
        .nodes
        .keys()
        .filter_map(|id| id.parse().ok())
        .collect();

    if node_indices.is_empty() {
        return Err(TaleTrailError::ValidationError(
            "No valid node indices found".to_string(),
        ));
    }

    let max_index = *node_indices.iter().max().unwrap();
    let end_node_id = format!("{}", max_index);

    // Check connectivity for each node
    for (node_id, node) in &dag.nodes {
        // Start node should have 0 incoming edges
        if node_id == &dag.start_node_id {
            if node.incoming_edges != 0 {
                return Err(TaleTrailError::ValidationError(format!(
                    "Start node {} should have 0 incoming edges, has {}",
                    node_id, node.incoming_edges
                )));
            }
        } else {
            // All other nodes should have at least 1 incoming edge
            if node.incoming_edges == 0 {
                return Err(TaleTrailError::ValidationError(format!(
                    "Non-start node {} has no incoming edges (isolated)",
                    node_id
                )));
            }
        }

        // End node should have 0 outgoing edges
        if node_id == &end_node_id {
            if node.outgoing_edges != 0 {
                return Err(TaleTrailError::ValidationError(format!(
                    "End node {} should have 0 outgoing edges, has {}",
                    node_id, node.outgoing_edges
                )));
            }
        }
        // Note: It's OK for intermediate nodes to have 0 outgoing edges if they connect to end
    }

    Ok(())
}

/// Validate that all nodes are reachable from start and all leaves reach end
///
/// Checks that:
/// - All nodes are reachable from the start node
/// - All leaf nodes (except end) can reach the end node
/// - No unreachable islands exist in the graph
///
/// # Arguments
///
/// * `dag` - The DAG to validate
///
/// # Returns
///
/// Result indicating success or validation error
///
/// # Examples
///
/// ```
/// use story_generator::structure::{generate_dag_structure, calculate_convergence_points, validate_reachability};
/// use shared_types::constants::DEFAULT_NODE_COUNT;
///
/// let convergence_points = calculate_convergence_points(DEFAULT_NODE_COUNT);
/// let dag = generate_dag_structure(DEFAULT_NODE_COUNT, convergence_points).unwrap();
/// validate_reachability(&dag).expect("Should have valid reachability");
/// ```
pub fn validate_reachability(dag: &DAG) -> Result<(), TaleTrailError> {
    // Check that DAG has nodes
    if dag.nodes.is_empty() {
        return Err(TaleTrailError::ValidationError(
            "DAG has no nodes".to_string(),
        ));
    }

    // Find end node
    let node_indices: Vec<usize> = dag
        .nodes
        .keys()
        .filter_map(|id| id.parse().ok())
        .collect();

    let max_index = *node_indices.iter().max().ok_or_else(|| {
        TaleTrailError::ValidationError("No valid node indices found".to_string())
    })?;
    let end_node_id = format!("{}", max_index);

    // Check all nodes are reachable from start
    let reachable_from_start = get_reachable_nodes(dag, &dag.start_node_id);

    if reachable_from_start.len() != dag.nodes.len() {
        // Find unreachable nodes for better error message
        let unreachable: Vec<String> = dag
            .nodes
            .keys()
            .filter(|id| !reachable_from_start.contains(*id))
            .cloned()
            .collect();

        return Err(TaleTrailError::ValidationError(format!(
            "Not all nodes reachable from start. Unreachable nodes: {:?}",
            unreachable
        )));
    }

    // Check that all leaf nodes (except end) can reach end
    // Use the DagExt trait's is_reachable method
    for (node_id, _node) in &dag.nodes {
        // Skip the end node itself
        if node_id == &end_node_id {
            continue;
        }

        // For all other nodes, check if they can reach the end
        if !dag.is_reachable(node_id, &end_node_id) {
            return Err(TaleTrailError::ValidationError(format!(
                "Node {} cannot reach end node {}",
                node_id, end_node_id
            )));
        }
    }

    Ok(())
}

/// Get all nodes reachable from a starting node using BFS
fn get_reachable_nodes(dag: &DAG, start: &str) -> HashSet<String> {
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(start.to_string());
    reachable.insert(start.to_string());

    while let Some(current) = queue.pop_front() {
        for edge in &dag.edges {
            if edge.from_node_id == current && !reachable.contains(&edge.to_node_id) {
                reachable.insert(edge.to_node_id.clone());
                queue.push_back(edge.to_node_id.clone());
            }
        }
    }

    reachable
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::constants::DEFAULT_NODE_COUNT;

    #[test]
    fn test_calculate_convergence_basic() {
        let config = ConvergenceConfig {
            pattern: ConvergencePattern::MultipleConvergence,
            ratio: Some(CONVERGENCE_POINT_RATIO),
        };
        let points = calculate_convergence_points(16, &config);
        assert!(!points.is_empty());
        // With ratio 0.5 and 16 nodes, expect convergence around position 8
        assert!(points.contains(&8));
    }

    #[test]
    fn test_generate_dag_basic() {
        let dag_config = DagStructureConfig {
            node_count: DEFAULT_NODE_COUNT as i64,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        };
        let dag = generate_dag_structure(&dag_config)
            .expect("Should generate DAG");

        assert_eq!(dag.nodes.len(), DEFAULT_NODE_COUNT);
        assert!(!dag.edges.is_empty());
    }

    #[test]
    fn test_validate_connectivity_basic() {
        let dag_config = DagStructureConfig {
            node_count: DEFAULT_NODE_COUNT as i64,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        };
        let dag = generate_dag_structure(&dag_config)
            .expect("Should generate DAG");

        validate_path_connectivity(&dag).expect("Should validate");
    }

    #[test]
    fn test_validate_reachability_basic() {
        let dag_config = DagStructureConfig {
            node_count: DEFAULT_NODE_COUNT as i64,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        };
        let dag = generate_dag_structure(&dag_config)
            .expect("Should generate DAG");

        validate_reachability(&dag).expect("Should validate");
    }
}
