//! ContentNode business logic extensions

use crate::generated::{ContentNode, DAG};
use crate::errors::{TaleTrailError, Result};
use uuid::Uuid;

impl ContentNode {
    /// Calculate word count of the node's text content
    pub fn calculate_word_count(&self) -> usize {
        self.content.text
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .count()
    }

    /// Validate that all choices point to nodes that exist in the DAG
    pub fn validate_choices(&self, dag: &DAG) -> Result<()> {
        for choice in &self.content.choices {
            if !dag.nodes.contains_key(&choice.next_node_id) {
                return Err(TaleTrailError::ValidationError(
                    format!(
                        "Choice {} in node {} points to non-existent node {}",
                        choice.id, self.id, choice.next_node_id
                    )
                ));
            }
        }

        // Verify all next_nodes are covered by choices
        let choice_targets: std::collections::HashSet<_> =
            self.content.choices.iter().map(|c| c.next_node_id).collect();

        for next_node_id in &self.content.next_nodes {
            if !choice_targets.contains(next_node_id) {
                return Err(TaleTrailError::ValidationError(
                    format!(
                        "Node {} has next_node {} not covered by any choice",
                        self.id, next_node_id
                    )
                ));
            }
        }

        Ok(())
    }

    /// Get all next node IDs from choices
    pub fn get_next_nodes(&self) -> Vec<Uuid> {
        self.content.choices
            .iter()
            .map(|choice| choice.next_node_id)
            .collect()
    }

    /// Check if this is a leaf node (no outgoing edges)
    pub fn is_leaf_node(&self) -> bool {
        self.outgoing_edges == 0 || self.content.choices.is_empty()
    }

    /// Check if this node is a convergence point
    pub fn is_convergence_point(&self) -> bool {
        self.content.convergence_point || self.incoming_edges >= 2
    }

    /// Check if this node has educational content
    pub fn has_educational_content(&self) -> bool {
        self.content.educational_content.is_some()
    }

    /// Get the number of choices available at this node
    pub fn choice_count(&self) -> usize {
        self.content.choices.len()
    }

    /// Check if this is the start node (no incoming edges)
    pub fn is_start_node(&self) -> bool {
        self.incoming_edges == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::models::Content;

    #[test]
    fn test_word_count_basic() {
        let node_id = Uuid::now_v7();
        let content = Content {
            content_type: "interactive_story_node".to_string(),
            node_id,
            text: "Hello world test".to_string(),
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

        assert_eq!(node.calculate_word_count(), 3);
    }

    #[test]
    fn test_is_leaf() {
        let node_id = Uuid::now_v7();
        let content = Content {
            content_type: "interactive_story_node".to_string(),
            node_id,
            text: "End".to_string(),
            choices: vec![],
            convergence_point: false,
            next_nodes: vec![],
            educational_content: None,
        };
        let node = ContentNode {
            id: node_id,
            content,
            incoming_edges: 1,
            outgoing_edges: 0,
            generation_metadata: None,
        };

        assert!(node.is_leaf_node());
    }
}
