//! Trail and TrailStep business logic extensions

use crate::generated::DAG;
use crate::generated::models::{Trail, TrailStep, ContentReference};
use crate::generated::enums::TrailStatus;
use crate::errors::{TaleTrailError, Result};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

impl DAG {
    /// Convert DAG to Trail and TrailSteps using BFS traversal
    pub fn to_trail_with_steps(&self, title: String) -> (Trail, Vec<TrailStep>) {
        let trail = Trail {
            title,
            description: None,
            metadata: serde_json::json!({}),
            tags: None,
            status: TrailStatus::Draft,
            category: "story".to_string(),
            is_public: false,
            price_coins: None,
        };

        let trail_steps = self.generate_trail_steps();

        (trail, trail_steps)
    }

    /// Generate trail steps using BFS traversal from start node
    fn generate_trail_steps(&self) -> Vec<TrailStep> {
        let mut steps = Vec::new();
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();

        queue.push_back(self.start_node_id);
        visited.insert(self.start_node_id, 0);

        while let Some(node_id) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&node_id) {
                let step_order = visited[&node_id];

                let step = TrailStep {
                    step_order,
                    title: Some(format!("Step {}", step_order + 1)),
                    description: None,
                    metadata: serde_json::json!({}),
                    content_reference: ContentReference {
                        temp_node_id: node_id,
                        content: node.content.clone(),
                    },
                    is_required: true,
                };

                steps.push(step);

                // Add children to queue
                for edge in &self.edges {
                    if edge.from_node_id == node_id && !visited.contains_key(&edge.to_node_id) {
                        let next_order = steps.len() as i32;
                        visited.insert(edge.to_node_id, next_order);
                        queue.push_back(edge.to_node_id);
                    }
                }
            }
        }

        // Sort by step_order to ensure sequential ordering
        steps.sort_by_key(|s| s.step_order);
        steps
    }
}

/// Validate that trail steps have sequential ordering
pub fn validate_trail_steps(steps: &[TrailStep]) -> Result<()> {
    if steps.is_empty() {
        return Err(TaleTrailError::ValidationError(
            "Trail must have at least one step".to_string()
        ));
    }

    // Check first step starts at 0
    if steps[0].step_order != 0 {
        return Err(TaleTrailError::ValidationError(
            format!("First step must have step_order 0, found {}", steps[0].step_order)
        ));
    }

    // Check sequential ordering
    for i in 1..steps.len() {
        let expected = steps[i - 1].step_order + 1;
        if steps[i].step_order != expected {
            return Err(TaleTrailError::ValidationError(
                format!(
                    "Step ordering not sequential: expected {}, found {}",
                    expected, steps[i].step_order
                )
            ));
        }
    }

    // Verify all content references are valid
    for step in steps {
        if step.content_reference.content.text.is_empty() {
            return Err(TaleTrailError::ValidationError(
                format!("Step {} has empty content text", step.step_order)
            ));
        }
    }

    Ok(())
}

/// Count total words across all trail steps
pub fn count_total_words(steps: &[TrailStep]) -> usize {
    steps.iter()
        .map(|step| {
            step.content_reference.content.text
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .count()
        })
        .sum()
}

/// Find a trail step by its node ID
pub fn find_step_by_node_id(steps: &[TrailStep], node_id: Uuid) -> Option<&TrailStep> {
    steps.iter()
        .find(|step| step.content_reference.temp_node_id == node_id)
}

impl Trail {
    /// Check if trail is published
    pub fn is_published(&self) -> bool {
        matches!(self.status, TrailStatus::Published)
    }

    /// Check if trail is public
    pub fn is_public(&self) -> bool {
        self.is_public
    }

    /// Check if trail is free
    pub fn is_free(&self) -> bool {
        self.price_coins.is_none() || self.price_coins == Some(0)
    }
}

impl TrailStep {
    /// Get the node ID from the content reference
    pub fn get_node_id(&self) -> Uuid {
        self.content_reference.temp_node_id
    }

    /// Get the content text
    pub fn get_content_text(&self) -> &str {
        &self.content_reference.content.text
    }

    /// Count words in this step's content
    pub fn word_count(&self) -> usize {
        self.content_reference.content.text
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::models::Content;

    #[test]
    fn test_validate_sequential_basic() {
        let steps = vec![
            TrailStep {
                step_order: 0,
                title: Some("Step 1".to_string()),
                description: None,
                metadata: serde_json::json!({}),
                content_reference: ContentReference {
                    temp_node_id: Uuid::now_v7(),
                    content: Content {
                        content_type: "interactive_story_node".to_string(),
                        node_id: Uuid::now_v7(),
                        text: "Test".to_string(),
                        choices: vec![],
                        convergence_point: false,
                        next_nodes: vec![],
                        educational_content: None,
                    },
                },
                is_required: true,
            },
            TrailStep {
                step_order: 1,
                title: Some("Step 2".to_string()),
                description: None,
                metadata: serde_json::json!({}),
                content_reference: ContentReference {
                    temp_node_id: Uuid::now_v7(),
                    content: Content {
                        content_type: "interactive_story_node".to_string(),
                        node_id: Uuid::now_v7(),
                        text: "Test 2".to_string(),
                        choices: vec![],
                        convergence_point: false,
                        next_nodes: vec![],
                        educational_content: None,
                    },
                },
                is_required: true,
            },
        ];

        assert!(validate_trail_steps(&steps).is_ok());
    }

    #[test]
    fn test_trail_step_word_count() {
        let step = TrailStep {
            step_order: 0,
            title: Some("Test".to_string()),
            description: None,
            metadata: serde_json::json!({}),
            content_reference: ContentReference {
                temp_node_id: Uuid::now_v7(),
                content: Content {
                    content_type: "interactive_story_node".to_string(),
                    node_id: Uuid::now_v7(),
                    text: "One two three four five".to_string(),
                    choices: vec![],
                    convergence_point: false,
                    next_nodes: vec![],
                    educational_content: None,
                },
            },
            is_required: true,
        };

        assert_eq!(step.word_count(), 5);
    }
}
