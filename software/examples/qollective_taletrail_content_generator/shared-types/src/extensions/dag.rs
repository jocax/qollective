//! DAG (Directed Acyclic Graph) business logic extensions

use crate::{ContentNode, DAG, Edge};
use crate::errors::{TaleTrailError, Result};
use std::collections::{HashSet, VecDeque};

/// Extension trait for DAG business logic
pub trait DagExt {
    fn add_node(&mut self, node: ContentNode);
    fn add_edge(&mut self, edge: Edge) -> Result<()>;
    fn validate_structure(&self) -> Result<()>;
    fn find_paths(&self, start: &str, end: &str) -> Vec<Vec<String>>;
    fn detect_convergence_points(&self) -> Vec<String>;
    fn is_reachable(&self, from: &str, to: &str) -> bool;
}

impl DagExt for DAG {
    /// Add a node to the DAG
    fn add_node(&mut self, node: ContentNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Add an edge to the DAG and update node edge counts
    fn add_edge(&mut self, edge: Edge) -> Result<()> {
        // Validate that both nodes exist
        if !self.nodes.contains_key(&edge.from_node_id) {
            return Err(TaleTrailError::ValidationError(
                format!("Source node {} not found in DAG", edge.from_node_id)
            ));
        }
        if !self.nodes.contains_key(&edge.to_node_id) {
            return Err(TaleTrailError::ValidationError(
                format!("Target node {} not found in DAG", edge.to_node_id)
            ));
        }

        // Update edge counts
        if let Some(from_node) = self.nodes.get_mut(&edge.from_node_id) {
            from_node.outgoing_edges += 1;
        }
        if let Some(to_node) = self.nodes.get_mut(&edge.to_node_id) {
            to_node.incoming_edges += 1;
        }

        self.edges.push(edge);
        Ok(())
    }

    /// Validate the DAG structure
    fn validate_structure(&self) -> Result<()> {
        // Check start node exists
        if !self.nodes.contains_key(&self.start_node_id) {
            return Err(TaleTrailError::ValidationError(
                "Start node not found in DAG".to_string()
            ));
        }

        // Check for cycles using DFS
        if has_cycle(self) {
            return Err(TaleTrailError::ValidationError(
                "DAG contains cycles".to_string()
            ));
        }

        // Verify all convergence points exist
        for cp in &self.convergence_points {
            if !self.nodes.contains_key(cp) {
                return Err(TaleTrailError::ValidationError(
                    format!("Convergence point {} not found in DAG", cp)
                ));
            }
        }

        // Verify all nodes are reachable from start
        let reachable = get_reachable_nodes(self, &self.start_node_id);
        if reachable.len() != self.nodes.len() {
            return Err(TaleTrailError::ValidationError(
                format!("Not all nodes are reachable from start node. Reachable: {}, Total: {}",
                    reachable.len(), self.nodes.len())
            ));
        }

        Ok(())
    }

    /// Find all paths from start to end node
    fn find_paths(&self, start: &str, end: &str) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut current_path = vec![start.to_string()];
        let mut visited = HashSet::new();

        dfs_paths(self, start, end, &mut current_path, &mut visited, &mut paths);
        paths
    }

    /// Detect convergence points (nodes with 2+ incoming edges)
    fn detect_convergence_points(&self) -> Vec<String> {
        self.nodes
            .values()
            .filter(|node| node.incoming_edges >= 2)
            .map(|node| node.id.clone())
            .collect()
    }

    /// Check if a target node is reachable from a source node
    fn is_reachable(&self, from: &str, to: &str) -> bool {
        let reachable = get_reachable_nodes(self, from);
        reachable.contains(to)
    }
}

// Private helper functions (not methods) to avoid orphan rule issues
fn has_cycle(dag: &DAG) -> bool {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for node_id in dag.nodes.keys() {
        if !visited.contains(node_id.as_str()) {
            if has_cycle_dfs(dag, node_id, &mut visited, &mut rec_stack) {
                return true;
            }
        }
    }
    false
}

fn has_cycle_dfs(
    dag: &DAG,
    node: &str,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> bool {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());

    // Get all outgoing edges from this node
    for edge in &dag.edges {
        if edge.from_node_id == node {
            let neighbor = &edge.to_node_id;
            if !visited.contains(neighbor.as_str()) {
                if has_cycle_dfs(dag, neighbor, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(neighbor.as_str()) {
                return true;
            }
        }
    }

    rec_stack.remove(node);
    false
}

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

fn dfs_paths(
    dag: &DAG,
    current: &str,
    end: &str,
    current_path: &mut Vec<String>,
    visited: &mut HashSet<String>,
    paths: &mut Vec<Vec<String>>,
) {
    if current == end {
        paths.push(current_path.clone());
        return;
    }

    visited.insert(current.to_string());

    for edge in &dag.edges {
        if edge.from_node_id == current && !visited.contains(&edge.to_node_id) {
            current_path.push(edge.to_node_id.clone());
            dfs_paths(dag, &edge.to_node_id, end, current_path, visited, paths);
            current_path.pop();
        }
    }

    visited.remove(current);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Content;

    #[test]
    fn test_add_node_basic() {
        let start_id = "node_start".to_string();
        let mut dag = DAG {
            nodes: Default::default(),
            edges: vec![],
            start_node_id: start_id.clone(),
            convergence_points: vec![],
        };

        let node_id = "node_1".to_string();
        let content = Content {
            r#type: "interactive_story_node".to_string(),
            node_id: node_id.clone(),
            text: "Test".to_string(),
            choices: vec![],
            convergence_point: false,
            next_nodes: vec![],
            educational_content: None,
        };
        let node = ContentNode {
            id: node_id.clone(),
            content,
            incoming_edges: 0,
            outgoing_edges: 0,
            generation_metadata: None,
        };

        dag.add_node(node);
        assert_eq!(dag.nodes.len(), 1);
    }
}
