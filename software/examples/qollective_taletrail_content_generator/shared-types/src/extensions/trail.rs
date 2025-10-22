//! Trail and TrailStep business logic extensions

use crate::{Trail, TrailStep, ContentReference, DAG};
use crate::TrailStatus;
use crate::errors::{TaleTrailError, Result};
use std::collections::{HashMap, VecDeque};

/// Extension trait for DAG to Trail conversion
pub trait DagToTrailExt {
    fn to_trail_with_steps(&self, title: String) -> (Trail, Vec<TrailStep>);
    fn generate_trail_steps(&self) -> Vec<TrailStep>;
}

impl DagToTrailExt for DAG {
    /// Convert DAG to Trail and TrailSteps using BFS traversal
    fn to_trail_with_steps(&self, title: String) -> (Trail, Vec<TrailStep>) {
        let trail = Trail {
            title,
            description: None,
            metadata: HashMap::new(),
            tags: None,
            status: TrailStatus::DRAFT,
            category: Some("story".to_string()),
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

        queue.push_back(self.start_node_id.clone());
        visited.insert(self.start_node_id.clone(), 0);

        while let Some(node_id) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&node_id) {
                let step_order = visited[&node_id];

                let step = TrailStep {
                    step_order,
                    title: Some(format!("Step {}", step_order + 1)),
                    description: None,
                    metadata: HashMap::new(),
                    content_reference: ContentReference {
                        temp_node_id: node_id.clone(),
                        content: node.content.clone(),
                    },
                    is_required: true,
                };

                steps.push(step);

                // Add children to queue
                for edge in &self.edges {
                    if edge.from_node_id == node_id && !visited.contains_key(&edge.to_node_id) {
                        let next_order = steps.len() as i64;
                        visited.insert(edge.to_node_id.clone(), next_order);
                        queue.push_back(edge.to_node_id.clone());
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
pub fn find_step_by_node_id<'a>(steps: &'a [TrailStep], node_id: &str) -> Option<&'a TrailStep> {
    steps.iter()
        .find(|step| step.content_reference.temp_node_id == node_id)
}

/// Extension trait for Trail business logic
pub trait TrailExt {
    fn is_published(&self) -> bool;
    fn is_public_trail(&self) -> bool;
    fn is_free(&self) -> bool;
}

impl TrailExt for Trail {
    /// Check if trail is published
    fn is_published(&self) -> bool {
        matches!(self.status, TrailStatus::PUBLISHED)
    }

    /// Check if trail is public
    fn is_public_trail(&self) -> bool {
        self.is_public
    }

    /// Check if trail is free
    fn is_free(&self) -> bool {
        self.price_coins.is_none() || self.price_coins == Some(Some(0))
    }
}

/// Extension trait for TrailStep business logic
pub trait TrailStepExt {
    fn get_node_id(&self) -> &str;
    fn get_content_text(&self) -> &str;
    fn word_count(&self) -> usize;
}

impl TrailStepExt for TrailStep {
    /// Get the node ID from the content reference
    fn get_node_id(&self) -> &str {
        &self.content_reference.temp_node_id
    }

    /// Get the content text
    fn get_content_text(&self) -> &str {
        &self.content_reference.content.text
    }

    /// Count words in this step's content
    fn word_count(&self) -> usize {
        self.content_reference.content.text
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Content;
    use std::collections::HashMap;

    #[test]
    fn test_validate_sequential_basic() {
        let steps = vec![
            TrailStep {
                step_order: 0,
                title: Some("Step 1".to_string()),
                description: None,
                metadata: HashMap::new(),
                content_reference: ContentReference {
                    temp_node_id: "node_1".to_string(),
                    content: Content {
                        r#type: "interactive_story_node".to_string(),
                        node_id: "node_1".to_string(),
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
                metadata: HashMap::new(),
                content_reference: ContentReference {
                    temp_node_id: "node_2".to_string(),
                    content: Content {
                        r#type: "interactive_story_node".to_string(),
                        node_id: "node_2".to_string(),
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
            metadata: HashMap::new(),
            content_reference: ContentReference {
                temp_node_id: "node_test".to_string(),
                content: Content {
                    r#type: "interactive_story_node".to_string(),
                    node_id: "node_test".to_string(),
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
