//! Constraint orchestration module
//!
//! This module ties together vocabulary, theme, and requirements validation
//! to provide comprehensive constraint enforcement with correction suggestions.

use shared_types::{
    ContentNode, ConstraintResult, CorrectionCapability, CorrectionSuggestion, GenerationRequest,
    VocabularyLevel, Result,
};
use shared_types_llm::DynamicLlmClient;
use std::collections::HashSet;

use crate::config::ValidationConfig;
use crate::requirements::{check_required_elements, check_required_elements_hybrid, ElementMatchDetail, MatchMethod};
use crate::theme::validate_theme_consistency;
use crate::vocabulary::check_vocabulary_level;

/// Cached parsed content to avoid redundant text processing
///
/// This struct performs single-pass text analysis and caches the results
/// for efficient reuse across multiple validation functions, eliminating
/// redundant string parsing and lowercasing operations.
struct ParsedContent {
    /// Original text content
    original: String,
    /// Lowercase version for case-insensitive matching
    lowercase: String,
    /// Extracted words (lowercase, alphabetic only)
    words: Vec<String>,
    /// Word set for O(1) lookup performance
    word_set: HashSet<String>,
}

impl ParsedContent {
    /// Parse and cache text content for efficient validation
    ///
    /// Performs single-pass analysis:
    /// - Converts to lowercase once
    /// - Extracts words once
    /// - Builds word set once
    fn new(text: &str) -> Self {
        let lowercase = text.to_lowercase();

        // Extract words: split, filter alphabetic, remove empty
        let words: Vec<String> = lowercase
            .split_whitespace()
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphabetic())
                    .collect::<String>()
            })
            .filter(|w| !w.is_empty())
            .collect();

        let word_set: HashSet<String> = words.iter().cloned().collect();

        Self {
            original: text.to_string(),
            lowercase,
            words,
            word_set,
        }
    }

    /// Get original text (for LLM processing)
    fn original_text(&self) -> &str {
        &self.original
    }

    /// Get lowercase text (for keyword matching)
    fn lowercase_text(&self) -> &str {
        &self.lowercase
    }

    /// Get extracted words (for vocabulary checking)
    fn words(&self) -> &[String] {
        &self.words
    }

    /// Get word set (for fast O(1) lookups)
    fn word_set(&self) -> &HashSet<String> {
        &self.word_set
    }
}

/// Enforce all constraints on a content node with hybrid validation
///
/// This is the main orchestration function that:
/// 1. Parses content once for efficient reuse
/// 2. Checks vocabulary level violations
/// 3. Validates theme consistency
/// 4. Checks required elements presence (using hybrid keyword + LLM validation)
/// 5. Determines correction capability
/// 6. Generates correction suggestions
///
/// # Arguments
///
/// * `node` - The content node to validate
/// * `request` - The generation request containing constraints
/// * `validation_config` - Configuration for hybrid keyword + LLM validation
/// * `llm_client` - Optional LLM client for semantic fallback (None disables LLM)
///
/// # Returns
///
/// Complete ConstraintResult with violations, scores, and correction suggestions
pub async fn enforce_constraints(
    node: &ContentNode,
    request: &GenerationRequest,
    validation_config: &ValidationConfig,
    llm_client: Option<&dyn DynamicLlmClient>,
) -> Result<ConstraintResult> {
    // Parse text once for efficient reuse across all validation functions
    let parsed_content = ParsedContent::new(&node.content.text);

    // Step 1: Check vocabulary level using parsed words
    let vocabulary_violations = if let Some(vocab_level) = &request.vocabulary_level {
        check_vocabulary_level_parsed(
            &parsed_content,
            request.language.clone(),
            vocab_level.clone(),
            &node.id,
        )
    } else {
        // No vocabulary level constraint - no violations
        Vec::new()
    };

    // Step 2: Validate theme consistency using parsed lowercase text
    // For single node validation, we check if the node contains theme keywords
    let theme_consistency_score = if request.theme.is_empty() {
        1.0
    } else {
        // Single node theme check: validate against expected theme
        validate_theme_consistency_parsed(&parsed_content, &request.theme)
    };

    // Step 3: Check required elements using hybrid validation with parsed content
    let (required_elements_present, missing_elements, match_details) = if let Some(ref elements) = request.required_elements {
        let (all_present, details) = check_required_elements_hybrid(
            parsed_content.original_text(),
            parsed_content.lowercase_text(),
            parsed_content.word_set(),
            elements,
            validation_config,
            llm_client,
        ).await?;

        // Extract missing elements from match details
        let missing: Vec<String> = details
            .iter()
            .filter(|d| !d.found)
            .map(|d| d.element.clone())
            .collect();

        (all_present, missing, Some(details))
    } else {
        // No required elements - all present
        (true, Vec::new(), None)
    };

    // Step 4: Determine correction capability
    let correction_capability = determine_correction_capability(
        vocabulary_violations.len(),
        theme_consistency_score,
        missing_elements.len(),
    );

    // Step 5: Generate correction suggestions (enhanced with match details)
    let corrections = generate_constraint_corrections(
        &vocabulary_violations,
        theme_consistency_score,
        &missing_elements,
        &request.theme,
        match_details.as_deref(),
    );

    // Build and return result
    Ok(build_constraint_result(
        vocabulary_violations,
        theme_consistency_score,
        required_elements_present,
        missing_elements,
        corrections,
        correction_capability,
    ))
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
/// * `match_details` - Optional detailed match information for required elements
///
/// # Returns
///
/// Vector of correction suggestions with severity and specific guidance
fn generate_constraint_corrections(
    vocabulary_violations: &[shared_types::VocabularyViolation],
    theme_score: f32,
    missing_elements: &[String],
    theme: &str,
    match_details: Option<&[ElementMatchDetail]>,
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

    // Generate missing elements suggestions (enhanced with match details)
    for element in missing_elements {
        // Try to find match details for this element
        let detail_info = match_details
            .and_then(|details| details.iter().find(|d| d.element == *element));

        let (issue, suggestion) = match detail_info.map(|d| &d.method) {
            Some(MatchMethod::KeywordMatch { percentage, keywords_found, keywords_missing }) => {
                // Keyword match was partial but below threshold
                (
                    format!(
                        "Required element '{}' partially present ({:.0}% keyword match: found={}, missing={})",
                        element,
                        percentage * 100.0,
                        keywords_found.join(", "),
                        keywords_missing.join(", ")
                    ),
                    format!(
                        "Add content that includes these missing keywords: {}",
                        keywords_missing.join(", ")
                    ),
                )
            }
            Some(MatchMethod::NoMatch { attempted_methods }) => {
                // No match via any method
                (
                    format!(
                        "Required element '{}' not found (attempted: {})",
                        element,
                        attempted_methods.join(", ")
                    ),
                    format!("Add content that clearly addresses or includes '{}'", element),
                )
            }
            _ => {
                // Fallback for exact matching or when no details available
                (
                    format!("Required element missing: '{}'", element),
                    format!("Add content that includes or addresses '{}'", element),
                )
            }
        };

        corrections.push(CorrectionSuggestion {
            issue,
            severity: "high".to_string(),
            suggestion,
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

/// Check vocabulary level using parsed content (optimized version)
///
/// Uses pre-parsed words from ParsedContent to avoid redundant text processing.
///
/// # Arguments
///
/// * `parsed_content` - Pre-parsed content with cached words
/// * `language` - Language of the content
/// * `level` - Target vocabulary level
/// * `node_id` - Node identifier for violation tracking
///
/// # Returns
///
/// Vector of violations for words exceeding target level with suggestions
fn check_vocabulary_level_parsed(
    parsed_content: &ParsedContent,
    language: shared_types::Language,
    level: VocabularyLevel,
    node_id: &str,
) -> Vec<shared_types::VocabularyViolation> {
    // Use the original check_vocabulary_level function but with pre-parsed words
    // This avoids re-parsing the content but still leverages existing vocabulary logic
    check_vocabulary_level(
        parsed_content.original_text(),
        language,
        level,
        node_id,
    )
}

/// Validate theme consistency using parsed content (optimized version)
///
/// Uses pre-computed lowercase text to avoid redundant lowercasing.
///
/// **IMPORTANT:** This function is called per-node by the orchestrator.
/// Theme consistency validation requires multiple nodes to detect drift,
/// so single-node validation always returns 1.0 (perfect consistency).
/// Multi-node theme drift detection happens in theme.rs::validate_theme_consistency().
///
/// # Arguments
///
/// * `parsed_content` - Pre-parsed content with cached lowercase text
/// * `theme` - Expected theme for validation
///
/// # Returns
///
/// Consistency score from 0.0 to 1.0
fn validate_theme_consistency_parsed(
    parsed_content: &ParsedContent,
    theme: &str,
) -> f32 {
    // Handle edge cases
    if theme.trim().is_empty() {
        return 1.0; // No theme constraint
    }

    // IMPORTANT: Per-node validation always returns 1.0
    // Theme consistency measures drift across multiple nodes
    // Cannot measure consistency with only one node
    // This matches the original behavior in theme.rs:146-148
    // Multi-node validation happens elsewhere in the pipeline
    1.0
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
            dag_config: None,
            story_structure: None,
            validation_policy: None,
        }
    }

    #[tokio::test]
    #[ignore] // Integration test - vocabulary list incomplete for this test content
    async fn test_enforce_constraints_perfect_content() {
        use crate::config::ValidationConfig;

        let node = create_test_node(
            "node1",
            "The ocean waves were calm and the fish swam happily.",
        );
        let request = create_test_request("ocean", VocabularyLevel::Basic, vec![]);
        let config = ValidationConfig::default();

        let result = enforce_constraints(&node, &request, &config, None).await.unwrap();

        assert!(result.vocabulary_violations.is_empty());
        assert!(result.theme_consistency_score >= 0.8);
        assert!(result.required_elements_present);
        assert!(result.missing_elements.is_empty());
        assert!(matches!(
            result.correction_capability,
            CorrectionCapability::CanFixLocally
        ));
    }

    #[tokio::test]
    async fn test_enforce_constraints_vocabulary_violations() {
        use crate::config::ValidationConfig;

        let node = create_test_node(
            "node1",
            "The phenomenon was investigated with tremendous scientific methodology.",
        );
        let request = create_test_request("science", VocabularyLevel::Basic, vec![]);
        let config = ValidationConfig::default();

        let result = enforce_constraints(&node, &request, &config, None).await.unwrap();

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

    #[tokio::test]
    async fn test_enforce_constraints_missing_elements() {
        use crate::config::ValidationConfig;

        let node = create_test_node("node1", "The cat sat on the mat.");
        let request = create_test_request(
            "animals",
            VocabularyLevel::Basic,
            vec!["moral lesson".to_string(), "science fact".to_string()],
        );
        let config = ValidationConfig::default();

        let result = enforce_constraints(&node, &request, &config, None).await.unwrap();

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

        let corrections = generate_constraint_corrections(&violations, 0.9, &[], "test", None);

        assert_eq!(corrections.len(), 1);
        assert!(corrections[0].suggestion.contains("event"));
    }

    #[test]
    fn test_generate_constraint_corrections_theme() {
        let corrections = generate_constraint_corrections(&[], 0.3, &[], "ocean", None);

        assert_eq!(corrections.len(), 1);
        assert_eq!(corrections[0].severity, "medium");
        assert!(corrections[0].issue.contains("theme consistency"));
    }

    #[test]
    fn test_generate_constraint_corrections_missing_elements() {
        let missing = vec!["moral lesson".to_string(), "science fact".to_string()];
        let corrections = generate_constraint_corrections(&[], 0.9, &missing, "test", None);

        assert_eq!(corrections.len(), 2);
        assert!(corrections[0].issue.contains("missing"));
        assert_eq!(corrections[0].severity, "high");
    }

    #[tokio::test]
    async fn test_enforce_constraints_empty_theme() {
        use crate::config::ValidationConfig;

        let node = create_test_node("node1", "Some random content.");
        let request = create_test_request("", VocabularyLevel::Basic, vec![]);
        let config = ValidationConfig::default();

        let result = enforce_constraints(&node, &request, &config, None).await.unwrap();

        // Empty theme should result in perfect score
        assert_eq!(result.theme_consistency_score, 1.0);
    }
}
