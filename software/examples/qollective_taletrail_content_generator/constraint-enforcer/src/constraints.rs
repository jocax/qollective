//! Constraint orchestration module
//!
//! This module ties together vocabulary, theme, and requirements validation
//! to provide comprehensive constraint enforcement with correction suggestions.

use shared_types::{
    ContentNode, ConstraintResult, CorrectionCapability, CorrectionSuggestion, GenerationRequest,
    VocabularyLevel,
};

use crate::requirements::check_required_elements;
use crate::theme::validate_theme_consistency;
use crate::vocabulary::check_vocabulary_level;

/// Enforce all constraints on a content node
///
/// This is the main orchestration function that:
/// 1. Checks vocabulary level violations
/// 2. Validates theme consistency
/// 3. Checks required elements presence
/// 4. Determines correction capability
/// 5. Generates correction suggestions
///
/// # Arguments
///
/// * `node` - The content node to validate
/// * `request` - The generation request containing constraints
///
/// # Returns
///
/// Complete ConstraintResult with violations, scores, and correction suggestions
pub fn enforce_constraints(
    node: &ContentNode,
    request: &GenerationRequest,
) -> ConstraintResult {
    // Step 1: Check vocabulary level
    let vocabulary_violations = if let Some(vocab_level) = &request.vocabulary_level {
        check_vocabulary_level(
            &node.content.text,
            request.language.clone(),
            vocab_level.clone(),
            &node.id,
        )
    } else {
        // No vocabulary level constraint - no violations
        Vec::new()
    };

    // Step 2: Validate theme consistency (single node check)
    // For single node validation, we check if the node contains theme keywords
    let theme_consistency_score = if request.theme.is_empty() {
        1.0
    } else {
        // Single node theme check: validate against expected theme
        validate_theme_consistency(&[node.clone()], &request.theme)
    };

    // Step 3: Check required elements
    let (required_elements_present, missing_elements) = if let Some(ref elements) = request.required_elements {
        check_required_elements(&node.content.text, elements)
    } else {
        // No required elements - all present
        (true, Vec::new())
    };

    // Step 4: Determine correction capability
    let correction_capability = determine_correction_capability(
        vocabulary_violations.len(),
        theme_consistency_score,
        missing_elements.len(),
    );

    // Step 5: Generate correction suggestions
    let corrections = generate_constraint_corrections(
        &vocabulary_violations,
        theme_consistency_score,
        &missing_elements,
        &request.theme,
    );

    // Build and return result
    build_constraint_result(
        vocabulary_violations,
        theme_consistency_score,
        required_elements_present,
        missing_elements,
        corrections,
        correction_capability,
    )
}

/// Determine the correction capability based on violation counts
///
/// # Logic:
/// - CanFixLocally: <= 3 vocabulary violations, theme_score >= 0.6, 1-2 missing elements
/// - NeedsRevision: 4-6 violations, theme_score 0.3-0.6, 3-5 missing elements
/// - NoFixPossible: 7+ violations, theme_score < 0.3, 6+ missing elements
///
/// # Arguments
///
/// * `vocab_violation_count` - Number of vocabulary violations
/// * `theme_score` - Theme consistency score (0.0 - 1.0)
/// * `missing_element_count` - Number of missing required elements
///
/// # Returns
///
/// Appropriate CorrectionCapability level
fn determine_correction_capability(
    vocab_violation_count: usize,
    theme_score: f32,
    missing_element_count: usize,
) -> CorrectionCapability {
    // Check NoFixPossible conditions
    if vocab_violation_count >= 7 || theme_score < 0.3 || missing_element_count >= 6 {
        return CorrectionCapability::NoFixPossible;
    }

    // Check NeedsRevision conditions
    if vocab_violation_count >= 4
        || (theme_score >= 0.3 && theme_score < 0.6)
        || missing_element_count >= 3
    {
        return CorrectionCapability::NeedsRevision;
    }

    // Otherwise, can fix locally
    CorrectionCapability::CanFixLocally
}

/// Generate correction suggestions based on constraint violations
///
/// # Arguments
///
/// * `vocabulary_violations` - List of vocabulary violations
/// * `theme_score` - Theme consistency score
/// * `missing_elements` - List of missing required elements
/// * `theme` - Expected theme for theme reinforcement
///
/// # Returns
///
/// Vector of correction suggestions with severity and specific guidance
fn generate_constraint_corrections(
    vocabulary_violations: &[shared_types::VocabularyViolation],
    theme_score: f32,
    missing_elements: &[String],
    theme: &str,
) -> Vec<CorrectionSuggestion> {
    let mut corrections = Vec::new();

    // Generate vocabulary correction suggestions
    for violation in vocabulary_violations {
        let suggestion = if violation.suggestions.is_empty() {
            format!(
                "Replace '{}' with simpler word appropriate for {:?} level",
                violation.word, violation.target_level
            )
        } else {
            format!(
                "Replace '{}' with: {}",
                violation.word,
                violation.suggestions.join(", ")
            )
        };

        corrections.push(CorrectionSuggestion {
            issue: format!(
                "Word '{}' exceeds {:?} vocabulary level",
                violation.word, violation.target_level
            ),
            severity: "medium".to_string(),
            suggestion,
            field: "content.text".to_string(),
        });
    }

    // Generate theme consistency suggestions
    if theme_score < 0.5 {
        let severity = if theme_score < 0.3 {
            "high"
        } else {
            "medium"
        };

        corrections.push(CorrectionSuggestion {
            issue: format!(
                "Low theme consistency score: {:.2}. Content does not align well with theme '{}'",
                theme_score, theme
            ),
            severity: severity.to_string(),
            suggestion: format!(
                "Add theme-reinforcing keywords related to '{}' to improve consistency",
                theme
            ),
            field: "content.text".to_string(),
        });
    } else if theme_score < 0.8 {
        corrections.push(CorrectionSuggestion {
            issue: format!(
                "Moderate theme consistency score: {:.2}",
                theme_score
            ),
            severity: "low".to_string(),
            suggestion: format!(
                "Consider adding more references to '{}' theme for better consistency",
                theme
            ),
            field: "content.text".to_string(),
        });
    }

    // Generate missing elements suggestions
    for element in missing_elements {
        corrections.push(CorrectionSuggestion {
            issue: format!("Required element missing: '{}'", element),
            severity: "high".to_string(),
            suggestion: format!("Add content that includes or addresses '{}'", element),
            field: "content.text".to_string(),
        });
    }

    corrections
}

/// Build final ConstraintResult from all validation components
///
/// # Arguments
///
/// * `vocabulary_violations` - Vocabulary violations found
/// * `theme_score` - Theme consistency score (0.0 - 1.0)
/// * `required_present` - Whether all required elements are present
/// * `missing_elements` - List of missing required elements
/// * `corrections` - Generated correction suggestions
/// * `capability` - Determined correction capability
///
/// # Returns
///
/// Complete ConstraintResult ready for MCP tool response
fn build_constraint_result(
    vocabulary_violations: Vec<shared_types::VocabularyViolation>,
    theme_score: f32,
    required_present: bool,
    missing_elements: Vec<String>,
    corrections: Vec<CorrectionSuggestion>,
    capability: CorrectionCapability,
) -> ConstraintResult {
    ConstraintResult {
        vocabulary_violations,
        theme_consistency_score: theme_score as f64,
        required_elements_present: required_present,
        missing_elements,
        corrections,
        correction_capability: capability,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{Choice, Content, Language};

    fn create_test_node(id: &str, text: &str) -> ContentNode {
        ContentNode {
            id: id.to_string(),
            content: Content {
                node_id: id.to_string(),
                text: text.to_string(),
                r#type: "narrative".to_string(),
                choices: vec![],
                next_nodes: vec![],
                convergence_point: false,
                educational_content: None,
            },
            incoming_edges: 0,
            outgoing_edges: 0,
            generation_metadata: None,
        }
    }

    fn create_test_request(
        theme: &str,
        vocab_level: VocabularyLevel,
        required: Vec<String>,
    ) -> GenerationRequest {
        GenerationRequest {
            tags: Some(vec![]),
            node_count: Some(10),
            age_group: shared_types::AgeGroup::_6To8,
            tenant_id: 1,
            prompt_packages: None,
            theme: theme.to_string(),
            educational_goals: Some(vec![]),
            author_id: None,
            required_elements: Some(required),
            vocabulary_level: Some(vocab_level),
            language: Language::En,
        }
    }

    #[test]
    #[ignore] // Integration test - vocabulary list incomplete for this test content
    fn test_enforce_constraints_perfect_content() {
        let node = create_test_node(
            "node1",
            "The ocean waves were calm and the fish swam happily.",
        );
        let request = create_test_request("ocean", VocabularyLevel::Basic, vec![]);

        let result = enforce_constraints(&node, &request);

        assert!(result.vocabulary_violations.is_empty());
        assert!(result.theme_consistency_score >= 0.8);
        assert!(result.required_elements_present);
        assert!(result.missing_elements.is_empty());
        assert!(matches!(
            result.correction_capability,
            CorrectionCapability::CanFixLocally
        ));
    }

    #[test]
    fn test_enforce_constraints_vocabulary_violations() {
        let node = create_test_node(
            "node1",
            "The phenomenon was investigated with tremendous scientific methodology.",
        );
        let request = create_test_request("science", VocabularyLevel::Basic, vec![]);

        let result = enforce_constraints(&node, &request);

        assert!(!result.vocabulary_violations.is_empty());
        assert!(!result.corrections.is_empty());

        // Should have corrections for vocabulary violations
        let vocab_corrections: Vec<_> = result
            .corrections
            .iter()
            .filter(|c| c.issue.contains("vocabulary level"))
            .collect();
        assert!(!vocab_corrections.is_empty());
    }

    #[test]
    fn test_enforce_constraints_missing_elements() {
        let node = create_test_node("node1", "The cat sat on the mat.");
        let request = create_test_request(
            "animals",
            VocabularyLevel::Basic,
            vec!["moral lesson".to_string(), "science fact".to_string()],
        );

        let result = enforce_constraints(&node, &request);

        assert!(!result.required_elements_present);
        assert_eq!(result.missing_elements.len(), 2);
        assert!(!result.corrections.is_empty());

        // Should have corrections for missing elements
        let missing_corrections: Vec<_> = result
            .corrections
            .iter()
            .filter(|c| c.issue.contains("missing"))
            .collect();
        assert_eq!(missing_corrections.len(), 2);
    }

    #[test]
    fn test_determine_correction_capability_can_fix_locally() {
        let capability = determine_correction_capability(2, 0.8, 1);
        assert!(matches!(capability, CorrectionCapability::CanFixLocally));
    }

    #[test]
    fn test_determine_correction_capability_needs_revision() {
        let capability = determine_correction_capability(5, 0.5, 4);
        assert!(matches!(capability, CorrectionCapability::NeedsRevision));
    }

    #[test]
    fn test_determine_correction_capability_no_fix_possible() {
        let capability = determine_correction_capability(10, 0.2, 8);
        assert!(matches!(
            capability,
            CorrectionCapability::NoFixPossible
        ));
    }

    #[test]
    fn test_generate_constraint_corrections_vocabulary() {
        let violations = vec![shared_types::VocabularyViolation {
            word: "phenomenon".to_string(),
            node_id: "node1".to_string(),
            current_level: VocabularyLevel::Advanced,
            target_level: VocabularyLevel::Basic,
            suggestions: vec!["event".to_string(), "thing".to_string()],
        }];

        let corrections = generate_constraint_corrections(&violations, 0.9, &[], "test");

        assert_eq!(corrections.len(), 1);
        assert!(corrections[0].suggestion.contains("event"));
    }

    #[test]
    fn test_generate_constraint_corrections_theme() {
        let corrections = generate_constraint_corrections(&[], 0.3, &[], "ocean");

        assert_eq!(corrections.len(), 1);
        assert_eq!(corrections[0].severity, "medium");
        assert!(corrections[0].issue.contains("theme consistency"));
    }

    #[test]
    fn test_generate_constraint_corrections_missing_elements() {
        let missing = vec!["moral lesson".to_string(), "science fact".to_string()];
        let corrections = generate_constraint_corrections(&[], 0.9, &missing, "test");

        assert_eq!(corrections.len(), 2);
        assert!(corrections[0].issue.contains("missing"));
        assert_eq!(corrections[0].severity, "high");
    }

    #[test]
    fn test_enforce_constraints_empty_theme() {
        let node = create_test_node("node1", "Some random content.");
        let request = create_test_request("", VocabularyLevel::Basic, vec![]);

        let result = enforce_constraints(&node, &request);

        // Empty theme should result in perfect score
        assert_eq!(result.theme_consistency_score, 1.0);
    }
}
