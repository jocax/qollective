//! Negotiation protocol for multi-round content corrections
//!
//! This module implements the negotiation protocol that handles multi-round content
//! corrections based on validation failures. It coordinates between the story-generator
//! service and validation services (quality-control and constraint-enforcer) to
//! iteratively improve content.

use shared_types::*;
use crate::validation_issues::{ValidationIssue, IssueSeverity};
use tracing::{info, warn, instrument};

/// Represents a single negotiation round
#[derive(Debug, Clone)]
pub struct NegotiationRound {
    /// Round number (1-based)
    pub iteration: u32,
    /// Issues identified in this round
    pub issues: Vec<ValidationIssue>,
    /// Corrections applied in this round
    pub corrections_applied: Vec<Correction>,
    /// Success flag for this round
    pub success: bool,
}

/// Correction to be applied
#[derive(Debug, Clone)]
pub struct Correction {
    pub node_id: String,
    pub correction_type: CorrectionType,
    pub description: String,
}

/// Type of correction to apply
#[derive(Debug, Clone)]
pub enum CorrectionType {
    LocalFix(String),      // Apply local correction with new content
    Regenerate,            // Request regeneration from story-generator
    Skip,                  // Skip this node (non-critical issue)
}

/// Plan for applying corrections
#[derive(Debug, Clone)]
pub struct CorrectionPlan {
    pub local_fixes: Vec<(String, String)>,  // (node_id, corrected_content)
    pub regenerate_nodes: Vec<String>,        // node_ids to regenerate
    pub skipped_nodes: Vec<String>,           // node_ids with non-critical issues
}

/// Negotiator handles multi-round content improvement
pub struct Negotiator {
    max_rounds: u32,
}

impl Negotiator {
    /// Create new negotiator with max rounds from config
    pub fn new(config: &crate::config::OrchestratorConfig) -> Self {
        Self {
            max_rounds: config.negotiation.max_rounds,
        }
    }

    /// Get the maximum number of negotiation rounds
    pub fn max_rounds(&self) -> u32 {
        self.max_rounds
    }

    /// Negotiate improvements for failed validations
    ///
    /// Returns Ok(Some(plan)) if corrections are needed
    /// Returns Ok(None) if all validations pass
    /// Returns Err if max rounds exceeded or critical failure
    #[instrument(skip(self, quality_results, constraint_results))]
    pub async fn negotiate_improvements(
        &self,
        node: &ContentNode,
        quality_results: Vec<ValidationResult>,
        constraint_results: Vec<ConstraintResult>,
    ) -> Result<Option<CorrectionPlan>> {
        // Collect all issues from quality and constraint results
        let mut issues = Vec::new();

        // Use validation_issues module to aggregate issues
        for result in quality_results {
            let quality_issues = crate::validation_issues::aggregate_quality_issues(&node.id, &result);
            issues.extend(quality_issues);
        }

        for result in constraint_results {
            let constraint_issues = crate::validation_issues::aggregate_constraint_issues(&node.id, &result);
            issues.extend(constraint_issues);
        }

        // Filter to only Critical and Warning issues - Info issues don't require negotiation
        issues.retain(|issue| {
            issue.severity == IssueSeverity::Critical || issue.severity == IssueSeverity::Warning
        });

        // If no actionable issues, return None (all validations passed or only Info issues)
        if issues.is_empty() {
            return Ok(None);
        }

        // Build correction plan based on capabilities
        let plan = self.build_correction_plan(&issues)?;

        Ok(Some(plan))
    }

    /// Execute negotiation rounds for a batch of nodes
    #[instrument(skip(self, nodes, quality_results, constraint_results))]
    pub async fn execute_negotiation_rounds(
        &self,
        nodes: &[ContentNode],
        quality_results: Vec<Vec<ValidationResult>>,
        constraint_results: Vec<Vec<ConstraintResult>>,
    ) -> Result<Vec<NegotiationRound>> {
        let mut rounds = Vec::new();

        for round_num in 1..=self.max_rounds {
            info!("Starting negotiation round {}/{}", round_num, self.max_rounds);

            let mut round = NegotiationRound {
                iteration: round_num,
                issues: Vec::new(),
                corrections_applied: Vec::new(),
                success: true,
            };

            // Collect issues for this round
            for (i, node) in nodes.iter().enumerate() {
                if let Some(plan) = self.negotiate_improvements(
                    node,
                    quality_results.get(i).cloned().unwrap_or_default(),
                    constraint_results.get(i).cloned().unwrap_or_default(),
                ).await? {
                    // Issues found, mark round as unsuccessful
                    round.success = false;

                    // Track corrections that would be applied
                    // Note: Actual correction application will be integrated with story-generator in Task 4.3
                    for node_id in &plan.regenerate_nodes {
                        round.corrections_applied.push(Correction {
                            node_id: node_id.clone(),
                            correction_type: CorrectionType::Regenerate,
                            description: format!("Regenerate node {}", node_id),
                        });
                    }

                    for (node_id, content) in &plan.local_fixes {
                        round.corrections_applied.push(Correction {
                            node_id: node_id.clone(),
                            correction_type: CorrectionType::LocalFix(content.clone()),
                            description: format!("Apply local fix to node {}", node_id),
                        });
                    }

                    for node_id in &plan.skipped_nodes {
                        round.corrections_applied.push(Correction {
                            node_id: node_id.clone(),
                            correction_type: CorrectionType::Skip,
                            description: format!("Skip node {} (non-critical issue)", node_id),
                        });
                    }
                }
            }

            rounds.push(round.clone());

            // If round succeeded (no issues), stop negotiation
            if round.success {
                info!("Negotiation succeeded in round {}", round_num);
                break;
            }

            // If max rounds reached, warn and stop
            if round_num >= self.max_rounds {
                warn!("Max negotiation rounds ({}) reached", self.max_rounds);
                break;
            }
        }

        Ok(rounds)
    }

    /// Build correction plan from issues
    ///
    /// Public method for use by orchestrator phase_negotiate_failures()
    /// Implements decision matrix logic
    pub fn build_correction_plan(&self, issues: &[ValidationIssue]) -> Result<CorrectionPlan> {
        let mut plan = CorrectionPlan {
            local_fixes: Vec::new(),
            regenerate_nodes: Vec::new(),
            skipped_nodes: Vec::new(),
        };

        for issue in issues {
            match (&issue.correction_capability, &issue.severity) {
                (CorrectionCapability::CanFixLocally, _) => {
                    // Validation service can apply fix
                    // For now, mark for regeneration (will be implemented in Task 4.3 to request corrected content)
                    if !plan.regenerate_nodes.contains(&issue.node_id) {
                        plan.regenerate_nodes.push(issue.node_id.clone());
                    }
                },
                (CorrectionCapability::NeedsRevision, IssueSeverity::Critical | IssueSeverity::Warning) => {
                    // Story-generator must regenerate for critical and warning issues
                    if !plan.regenerate_nodes.contains(&issue.node_id) {
                        plan.regenerate_nodes.push(issue.node_id.clone());
                    }
                },
                (CorrectionCapability::NeedsRevision, IssueSeverity::Info) => {
                    // Info-level issues with NeedsRevision can be regenerated but not critical
                    if !plan.regenerate_nodes.contains(&issue.node_id) {
                        plan.regenerate_nodes.push(issue.node_id.clone());
                    }
                },
                (CorrectionCapability::NoFixPossible, IssueSeverity::Critical) => {
                    // Critical issue that cannot be fixed - return error
                    return Err(TaleTrailError::ValidationError(
                        format!("Critical unfixable issue in node {}: {}", issue.node_id, issue.description)
                    ));
                },
                (CorrectionCapability::NoFixPossible, _) => {
                    // Non-critical unfixable issue - skip
                    if !plan.skipped_nodes.contains(&issue.node_id) {
                        plan.skipped_nodes.push(issue.node_id.clone());
                    }
                },
            }
        }

        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types_llm::{ProviderConfig, parameters::{ProviderType, SystemPromptStyle}};
    use std::collections::HashMap;

    fn create_test_config() -> crate::config::OrchestratorConfig {
        crate::config::OrchestratorConfig {
            service: crate::config::ServiceConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                description: "Test".to_string(),
            },
            nats: crate::config::NatsConfig::default(),
            llm: shared_types_llm::LlmConfig {
                provider: ProviderConfig {
                    provider_type: ProviderType::LmStudio,
                    url: "http://localhost:1234/v1".to_string(),
                    api_key: None,
                    default_model: "test-model".to_string(),
                    models: HashMap::new(),
                    use_default_model_fallback: true,
                    max_tokens: 4096,
                    temperature: 0.7,
                    timeout_secs: 60,
                    system_prompt_style: SystemPromptStyle::Native,
                },
                tenants: HashMap::new(),
            },
            pipeline: crate::config::PipelineConfig::default(),
            batch: crate::config::BatchConfig::default(),
            dag: crate::config::DagConfig::default(),
            negotiation: crate::config::NegotiationConfig {
                max_rounds: 3,
            },
            retry: crate::config::RetryConfig::default(),
        }
    }

    // Severity determination tests are now in validation_issues module
    // These tests verify negotiation logic, not severity calculation
}
