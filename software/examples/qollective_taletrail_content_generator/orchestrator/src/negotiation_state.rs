//! Negotiation state tracking for Phase 4
//!
//! Manages the state machine for negotiation rounds, tracking issues,
//! corrections, and round counters.

use crate::validation_issues::ValidationIssue;
use serde::{Deserialize, Serialize};
use shared_types_generated::{CorrectionSummary, ValidationIssueSummary};
use std::collections::HashMap;

/// Maximum negotiation rounds constant (CONSTANTS FIRST principle)
pub const MAX_NEGOTIATION_ROUNDS: u32 = 3;

/// Negotiation state tracking for Phase 4
///
/// Tracks the current state of the negotiation loop including:
/// - Current round number
/// - Validation issues requiring correction
/// - Corrections applied in each round
/// - Success/failure status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationState {
    /// Current negotiation round (1-indexed, 0 = not started)
    pub current_round: u32,

    /// Maximum allowed negotiation rounds
    pub max_rounds: u32,

    /// All validation issues from current round
    pub issues: Vec<ValidationIssue>,

    /// Node IDs that need regeneration in current round
    pub nodes_to_regenerate: Vec<String>,

    /// Node IDs that were skipped due to non-critical unfixable issues
    pub skipped_nodes: Vec<String>,

    /// Number of corrections applied in current round
    pub corrections_applied: usize,

    /// Whether the current round succeeded (all issues resolved)
    pub round_succeeded: bool,

    /// All corrections applied across all rounds (for metrics)
    pub all_corrections: Vec<CorrectionSummary>,

    /// Track correction attempts per node (for metrics)
    pub correction_attempts: HashMap<String, usize>,
}

impl NegotiationState {
    /// Create new negotiation state
    ///
    /// # Arguments
    /// * `max_rounds` - Maximum negotiation rounds allowed
    ///
    /// # Returns
    /// New negotiation state initialized to round 0 (not started)
    pub fn new(max_rounds: u32) -> Self {
        Self {
            current_round: 0,
            max_rounds,
            issues: Vec::new(),
            nodes_to_regenerate: Vec::new(),
            skipped_nodes: Vec::new(),
            corrections_applied: 0,
            round_succeeded: false,
            all_corrections: Vec::new(),
            correction_attempts: HashMap::new(),
        }
    }

    /// Start a new negotiation round
    ///
    /// Increments round counter and resets round-specific state.
    ///
    /// # Returns
    /// Current round number after increment
    pub fn start_round(&mut self) -> u32 {
        self.current_round += 1;
        self.nodes_to_regenerate.clear();
        self.skipped_nodes.clear();
        self.corrections_applied = 0;
        self.round_succeeded = false;
        self.current_round
    }

    /// Check if max rounds reached
    ///
    /// # Returns
    /// True if current_round >= max_rounds
    pub fn max_rounds_reached(&self) -> bool {
        self.current_round >= self.max_rounds
    }

    /// Mark round as succeeded
    ///
    /// Indicates all validation issues were resolved in this round.
    pub fn mark_round_succeeded(&mut self) {
        self.round_succeeded = true;
    }

    /// Add node to regeneration list
    ///
    /// # Arguments
    /// * `node_id` - ID of node to regenerate
    pub fn add_node_to_regenerate(&mut self, node_id: String) {
        if !self.nodes_to_regenerate.contains(&node_id) {
            self.nodes_to_regenerate.push(node_id);
        }
    }

    /// Add node to skipped list
    ///
    /// # Arguments
    /// * `node_id` - ID of node to skip
    pub fn add_skipped_node(&mut self, node_id: String) {
        if !self.skipped_nodes.contains(&node_id) {
            self.skipped_nodes.push(node_id);
        }
    }

    /// Set validation issues for current round
    ///
    /// # Arguments
    /// * `issues` - Vector of validation issues
    pub fn set_issues(&mut self, issues: Vec<ValidationIssue>) {
        self.issues = issues;
    }

    /// Get number of issues remaining
    ///
    /// # Returns
    /// Count of validation issues in current round
    pub fn issues_remaining(&self) -> usize {
        self.issues.len()
    }

    /// Increment corrections applied counter
    pub fn increment_corrections_applied(&mut self) {
        self.corrections_applied += 1;
    }

    /// Record a correction attempt for metrics
    ///
    /// # Arguments
    /// * `node_id` - ID of node being corrected
    /// * `correction_type` - Type of correction (LocalFix, Regenerate, Skip)
    /// * `success` - Whether the correction succeeded
    pub fn record_correction(&mut self, node_id: String, correction_type: String, success: bool) {
        // Increment attempt counter for this node
        let attempts = self.correction_attempts.entry(node_id.clone()).or_insert(0);
        *attempts += 1;

        // Add correction summary
        self.all_corrections.push(CorrectionSummary {
            node_id,
            correction_type,
            success,
            attempts: *attempts as i64,
        });
    }

    /// Get unresolved validation issues for metrics
    ///
    /// Returns issues that remain after max rounds exceeded
    pub fn get_unresolved_issues(&self) -> Vec<ValidationIssueSummary> {
        self.issues
            .iter()
            .map(|issue| {
                // Convert IssueType enum to string representation
                let issue_type_str = match &issue.issue_type {
                    crate::validation_issues::IssueType::Quality(kind) => {
                        format!("Quality::{:?}", kind)
                    }
                    crate::validation_issues::IssueType::Constraint(kind) => {
                        format!("Constraint::{:?}", kind)
                    }
                };

                ValidationIssueSummary {
                    node_id: issue.node_id.clone(),
                    severity: match issue.severity {
                        crate::validation_issues::IssueSeverity::Critical => "Critical".to_string(),
                        crate::validation_issues::IssueSeverity::Warning => "Warning".to_string(),
                        crate::validation_issues::IssueSeverity::Info => "Info".to_string(),
                    },
                    issue_type: issue_type_str,
                    description: issue.description.clone(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_negotiation_state() {
        let state = NegotiationState::new(3);

        assert_eq!(state.current_round, 0);
        assert_eq!(state.max_rounds, 3);
        assert!(state.issues.is_empty());
        assert!(state.nodes_to_regenerate.is_empty());
        assert!(state.skipped_nodes.is_empty());
        assert_eq!(state.corrections_applied, 0);
        assert!(!state.round_succeeded);
    }

    #[test]
    fn test_start_round() {
        let mut state = NegotiationState::new(3);

        let round = state.start_round();
        assert_eq!(round, 1);
        assert_eq!(state.current_round, 1);

        state.add_node_to_regenerate("node-1".to_string());
        state.increment_corrections_applied();

        // Starting new round should reset round-specific state
        let round = state.start_round();
        assert_eq!(round, 2);
        assert_eq!(state.current_round, 2);
        assert!(state.nodes_to_regenerate.is_empty());
        assert_eq!(state.corrections_applied, 0);
    }

    #[test]
    fn test_max_rounds_reached() {
        let mut state = NegotiationState::new(3);

        assert!(!state.max_rounds_reached());

        state.start_round(); // Round 1
        assert!(!state.max_rounds_reached());

        state.start_round(); // Round 2
        assert!(!state.max_rounds_reached());

        state.start_round(); // Round 3
        assert!(state.max_rounds_reached());
    }

    #[test]
    fn test_add_node_to_regenerate_deduplication() {
        let mut state = NegotiationState::new(3);

        state.add_node_to_regenerate("node-1".to_string());
        state.add_node_to_regenerate("node-2".to_string());
        state.add_node_to_regenerate("node-1".to_string()); // Duplicate

        assert_eq!(state.nodes_to_regenerate.len(), 2);
        assert!(state.nodes_to_regenerate.contains(&"node-1".to_string()));
        assert!(state.nodes_to_regenerate.contains(&"node-2".to_string()));
    }

    #[test]
    fn test_add_skipped_node_deduplication() {
        let mut state = NegotiationState::new(3);

        state.add_skipped_node("node-1".to_string());
        state.add_skipped_node("node-2".to_string());
        state.add_skipped_node("node-1".to_string()); // Duplicate

        assert_eq!(state.skipped_nodes.len(), 2);
        assert!(state.skipped_nodes.contains(&"node-1".to_string()));
        assert!(state.skipped_nodes.contains(&"node-2".to_string()));
    }

    #[test]
    fn test_round_succeeded() {
        let mut state = NegotiationState::new(3);

        assert!(!state.round_succeeded);

        state.mark_round_succeeded();
        assert!(state.round_succeeded);

        // Starting new round should reset success flag
        state.start_round();
        assert!(!state.round_succeeded);
    }
}
