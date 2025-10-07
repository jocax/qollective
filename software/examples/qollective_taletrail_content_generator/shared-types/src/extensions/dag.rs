//! DAG (Directed Acyclic Graph) business logic extensions

use crate::generated::{ContentNode, DAG, Edge};
use crate::errors::{TaleTrailError, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

impl DAG {
    /// Add a node to the DAG
    pub fn add_node(&mut self, node: ContentNode) {
        self.nodes.insert(node.id, node);
    }

    /// Add an edge to the DAG and update node edge counts
    pub fn add_edge(&mut self, edge: Edge) -> Result<()> {
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
    pub fn validate_structure(&self) -> Result<()> {
        // Check start node exists
        if !self.nodes.contains_key(&self.start_node_id) {
            return Err(TaleTrailError::ValidationError(
                "Start node not found in DAG".to_string()
            ));
        }

        // Check for cycles using DFS
        if self.has_cycle() {
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
        let reachable = self.get_reachable_nodes(self.start_node_id);
        if reachable.len() != self.nodes.len() {
            return Err(TaleTrailError::ValidationError(
                format!("Not all nodes are reachable from start node. Reachable: {}, Total: {}",
                    reachable.len(), self.nodes.len())
            ));
        }

        Ok(())
    }

    /// Find all paths from start to end node
    pub fn find_paths(&self, start: Uuid, end: Uuid) -> Vec<Vec<Uuid>> {
        let mut paths = Vec::new();
        let mut current_path = vec![start];
        let mut visited = HashSet::new();

        self.dfs_paths(start, end, &mut current_path, &mut visited, &mut paths);
        paths
    }

    /// Detect convergence points (nodes with 2+ incoming edges)
    pub fn detect_convergence_points(&self) -> Vec<Uuid> {
        self.nodes
            .values()
            .filter(|node| node.incoming_edges >= 2)
            .map(|node| node.id)
            .collect()
    }

    /// Check if a target node is reachable from a source node
    pub fn is_reachable(&self, from: Uuid, to: Uuid) -> bool {
        let reachable = self.get_reachable_nodes(from);
        reachable.contains(&to)
    }

    // Private helper methods

    fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                if self.has_cycle_dfs(*node_id, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        false
    }

    fn has_cycle_dfs(
        &self,
        node: Uuid,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
    ) -> bool {
        visited.insert(node);
        rec_stack.insert(node);

        // Get all outgoing edges from this node
        for edge in &self.edges {
            if edge.from_node_id == node {
                let neighbor = edge.to_node_id;
                if !visited.contains(&neighbor) {
                    if self.has_cycle_dfs(neighbor, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(&neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(&node);
        false
    }

    fn get_reachable_nodes(&self, start: Uuid) -> HashSet<Uuid> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        reachable.insert(start);

        while let Some(current) = queue.pop_front() {
            for edge in &self.edges {
                if edge.from_node_id == current && !reachable.contains(&edge.to_node_id) {
                    reachable.insert(edge.to_node_id);
                    queue.push_back(edge.to_node_id);
                }
            }
        }

        reachable
    }

    fn dfs_paths(
        &self,
        current: Uuid,
        end: Uuid,
        current_path: &mut Vec<Uuid>,
        visited: &mut HashSet<Uuid>,
        paths: &mut Vec<Vec<Uuid>>,
    ) {
        if current == end {
            paths.push(current_path.clone());
            return;
        }

        visited.insert(current);

        for edge in &self.edges {
            if edge.from_node_id == current && !visited.contains(&edge.to_node_id) {
                current_path.push(edge.to_node_id);
                self.dfs_paths(edge.to_node_id, end, current_path, visited, paths);
                current_path.pop();
            }
        }

        visited.remove(&current);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::models::Content;

    #[test]
    fn test_add_node_basic() {
        let mut dag = DAG {
            nodes: Default::default(),
            edges: vec![],
            start_node_id: Uuid::now_v7(),
            convergence_points: vec![],
        };

        let node_id = Uuid::now_v7();
        let content = Content {
            content_type: "interactive_story_node".to_string(),
            node_id,
            text: "Test".to_string(),
            choices: vec![],
            convergence_point: false,
            next_nodes: vec![],
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
        assert_eq!(dag.nodes.len(), 1);
    }
}
