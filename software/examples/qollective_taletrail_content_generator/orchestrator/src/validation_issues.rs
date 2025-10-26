//! Validation Issue Aggregation Module
//!
//! This module provides types and functions for aggregating validation issues
//! from quality-control and constraint-enforcer services, determining severity
//! levels, and preparing for negotiation.
//!
//! # Architecture
//!
//! The validation phase produces two types of results per node:
//! - **ValidationResult**: Quality scores (age-appropriateness, educational value, safety)
//! - **ConstraintResult**: Constraint violations (vocabulary, theme consistency, required elements)
//!
//! This module aggregates these results into a unified `ValidationIssue` list with:
//! - Severity classification (Critical/Warning/Info)
//! - Node-level tracking
//! - Correction capability information
//!
//! # Severity Thresholds
//!
//! Quality scores (0.0-1.0, higher is better):
//! - **Critical**: age_score < 0.5, edu_score < 0.4 (unacceptable quality)
//! - **Warning**: 0.5/0.4 <= score < 0.7 (needs improvement)
//! - **Info**: score >= 0.7 (acceptable but could be better)
//!
//! Constraint violations (count):
//! - **Critical**: violations > 5 (too many issues)
//! - **Warning**: 1 <= violations <= 5 (some issues)
//! - **Info**: violations = 0 (no issues)

use shared_types::{ConstraintResult, CorrectionCapability, ValidationResult};
use serde::{Deserialize, Serialize};

// ============================================================================
// SEVERITY THRESHOLD CONSTANTS
// ============================================================================

/// Quality score threshold for Critical severity (score < this = Critical)
pub const QUALITY_CRITICAL_THRESHOLD: f64 = 0.5;

/// Quality score threshold for Warning severity (score < this = Warning)
pub const QUALITY_WARNING_THRESHOLD: f64 = 0.7;

/// Educational value threshold for Critical severity (lower than age-appropriateness)
/// Educational content may be implicit in simple stories without explicit keywords
pub const EDUCATIONAL_CRITICAL_THRESHOLD: f64 = 0.4;

/// Constraint violation count threshold for Critical severity (count > this = Critical)
pub const CONSTRAINT_CRITICAL_THRESHOLD: usize = 5;

/// Constraint violation count threshold for Warning severity (count > this = Warning)
pub const CONSTRAINT_WARNING_THRESHOLD: usize = 0;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Issue severity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical issues that must be fixed before content is acceptable
    Critical,
    /// Warning issues that should be addressed but may be acceptable
    Warning,
    /// Informational issues that are nice to address but not required
    Info,
}

/// Type of validation issue
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueType {
    /// Quality validation issue (age-appropriateness, safety, educational value)
    Quality(QualityIssueKind),
    /// Constraint violation issue (vocabulary, theme, required elements)
    Constraint(ConstraintIssueKind),
}

/// Specific kind of quality issue
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityIssueKind {
    /// Age-appropriateness score below threshold
    AgeAppropriateness,
    /// Educational value score below threshold
    EducationalValue,
    /// Safety issues detected
    Safety,
}

/// Specific kind of constraint issue
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConstraintIssueKind {
    /// Vocabulary violations detected
    Vocabulary,
    /// Theme consistency score below threshold
    ThemeConsistency,
    /// Required elements missing
    RequiredElements,
}

/// Aggregated validation issue with severity and correction information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// ID of the node with the issue
    pub node_id: String,
    /// Type of issue (Quality or Constraint)
    pub issue_type: IssueType,
    /// Severity classification (Critical/Warning/Info)
    pub severity: IssueSeverity,
    /// Whether the validator can fix this issue
    pub correction_capability: CorrectionCapability,
    /// Human-readable description of the issue
    pub description: String,
}

impl ValidationIssue {
    /// Create a new ValidationIssue
    pub fn new(
        node_id: String,
        issue_type: IssueType,
        severity: IssueSeverity,
        correction_capability: CorrectionCapability,
        description: String,
    ) -> Self {
        Self {
            node_id,
            issue_type,
            severity,
            correction_capability,
            description,
        }
    }
}

// ============================================================================
// SEVERITY CALCULATION FUNCTIONS
// ============================================================================

/// Calculate severity from quality validation score
///
/// # Arguments
/// * `score` - Quality score (0.0-1.0, higher is better)
/// * `metric_name` - Name of the quality metric for description
///
/// # Returns
/// Tuple of (severity, description)
///
/// # Thresholds
/// - score < 0.5: Critical
/// - 0.5 <= score < 0.7: Warning
/// - score >= 0.7: Info
pub fn severity_from_quality_score(score: f64, metric_name: &str) -> (IssueSeverity, String) {
    if score < QUALITY_CRITICAL_THRESHOLD {
        (
            IssueSeverity::Critical,
            format!("{} score critically low: {:.2}", metric_name, score),
        )
    } else if score < QUALITY_WARNING_THRESHOLD {
        (
            IssueSeverity::Warning,
            format!("{} score below target: {:.2}", metric_name, score),
        )
    } else {
        (
            IssueSeverity::Info,
            format!("{} score acceptable: {:.2}", metric_name, score),
        )
    }
}

/// Calculate severity from constraint violation count
///
/// # Arguments
/// * `violation_count` - Number of violations detected
/// * `constraint_name` - Name of the constraint for description
///
/// # Returns
/// Tuple of (severity, description)
///
/// # Thresholds
/// - violations > 5: Critical
/// - 1 <= violations <= 5: Warning
/// - violations = 0: Info (no issue)
pub fn severity_from_violation_count(
    violation_count: usize,
    constraint_name: &str,
) -> (IssueSeverity, String) {
    if violation_count > CONSTRAINT_CRITICAL_THRESHOLD {
        (
            IssueSeverity::Critical,
            format!(
                "{} violations critical: {} violations detected",
                constraint_name, violation_count
            ),
        )
    } else if violation_count > CONSTRAINT_WARNING_THRESHOLD {
        (
            IssueSeverity::Warning,
            format!(
                "{} violations detected: {} issues found",
                constraint_name, violation_count
            ),
        )
    } else {
        (
            IssueSeverity::Info,
            format!("{} check passed: no violations", constraint_name),
        )
    }
}

// ============================================================================
// ISSUE AGGREGATION FUNCTIONS
// ============================================================================

/// Aggregate quality validation issues from ValidationResult
///
/// Creates ValidationIssue entries for:
/// - Age-appropriateness score
/// - Educational value score
/// - Safety issues (if any)
///
/// # Arguments
/// * `node_id` - ID of the node being validated
/// * `result` - ValidationResult from quality-control service
///
/// # Returns
/// Vector of ValidationIssue instances
pub fn aggregate_quality_issues(node_id: &str, result: &ValidationResult) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Age-appropriateness issue
    let (severity, description) =
        severity_from_quality_score(result.age_appropriate_score, "Age-appropriateness");
    issues.push(ValidationIssue::new(
        node_id.to_string(),
        IssueType::Quality(QualityIssueKind::AgeAppropriateness),
        severity,
        result.correction_capability,
        description,
    ));

    // Educational value issue - uses custom threshold (0.4 instead of 0.5)
    let (severity, description) = if result.educational_value_score < EDUCATIONAL_CRITICAL_THRESHOLD {
        (
            IssueSeverity::Critical,
            format!("Educational value score critically low: {:.2}", result.educational_value_score),
        )
    } else if result.educational_value_score < QUALITY_WARNING_THRESHOLD {
        (
            IssueSeverity::Warning,
            format!("Educational value score below target: {:.2}", result.educational_value_score),
        )
    } else {
        (
            IssueSeverity::Info,
            format!("Educational value score acceptable: {:.2}", result.educational_value_score),
        )
    };
    issues.push(ValidationIssue::new(
        node_id.to_string(),
        IssueType::Quality(QualityIssueKind::EducationalValue),
        severity,
        result.correction_capability,
        description,
    ));

    // Safety issues
    if !result.safety_issues.is_empty() {
        let description = format!(
            "Safety concerns detected: {}",
            result.safety_issues.join(", ")
        );
        issues.push(ValidationIssue::new(
            node_id.to_string(),
            IssueType::Quality(QualityIssueKind::Safety),
            IssueSeverity::Critical, // Safety issues are always critical
            result.correction_capability,
            description,
        ));
    }

    issues
}

/// Aggregate constraint violation issues from ConstraintResult
///
/// Creates ValidationIssue entries for:
/// - Vocabulary violations
/// - Theme consistency score
/// - Missing required elements
///
/// # Arguments
/// * `node_id` - ID of the node being validated
/// * `result` - ConstraintResult from constraint-enforcer service
///
/// # Returns
/// Vector of ValidationIssue instances
pub fn aggregate_constraint_issues(
    node_id: &str,
    result: &ConstraintResult,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Vocabulary violations
    let violation_count = result.vocabulary_violations.len();
    let (severity, description) = severity_from_violation_count(violation_count, "Vocabulary");
    issues.push(ValidationIssue::new(
        node_id.to_string(),
        IssueType::Constraint(ConstraintIssueKind::Vocabulary),
        severity,
        result.correction_capability,
        description,
    ));

    // Theme consistency
    let (severity, description) =
        severity_from_quality_score(result.theme_consistency_score, "Theme consistency");
    issues.push(ValidationIssue::new(
        node_id.to_string(),
        IssueType::Constraint(ConstraintIssueKind::ThemeConsistency),
        severity,
        result.correction_capability,
        description,
    ));

    // Missing required elements
    if !result.missing_elements.is_empty() {
        let description = format!(
            "Missing required elements: {}",
            result.missing_elements.join(", ")
        );
        issues.push(ValidationIssue::new(
            node_id.to_string(),
            IssueType::Constraint(ConstraintIssueKind::RequiredElements),
            IssueSeverity::Critical, // Missing required elements is critical
            result.correction_capability,
            description,
        ));
    }

    issues
}

/// Aggregate all validation issues from both quality and constraint results
///
/// # Arguments
/// * `node_id` - ID of the node being validated
/// * `quality_result` - ValidationResult from quality-control service
/// * `constraint_result` - ConstraintResult from constraint-enforcer service
///
/// # Returns
/// Combined vector of all ValidationIssue instances
pub fn aggregate_all_issues(
    node_id: &str,
    quality_result: &ValidationResult,
    constraint_result: &ConstraintResult,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    issues.extend(aggregate_quality_issues(node_id, quality_result));
    issues.extend(aggregate_constraint_issues(node_id, constraint_result));
    issues
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Test Helper Functions
    // ========================================================================

    fn create_quality_result(
        age_score: f64,
        edu_score: f64,
        safety_issues: Vec<String>,
    ) -> ValidationResult {
        ValidationResult {
            is_valid: age_score >= 0.7 && edu_score >= 0.7 && safety_issues.is_empty(),
            age_appropriate_score: age_score,
            safety_issues,
            educational_value_score: edu_score,
            correction_capability: CorrectionCapability::CanFixLocally,
            corrections: vec![],
        }
    }

    fn create_constraint_result(
        vocab_count: usize,
        theme_score: f64,
        missing: Vec<String>,
    ) -> ConstraintResult {
        use shared_types::VocabularyViolation;
        use shared_types::VocabularyLevel;

        let violations: Vec<VocabularyViolation> = (0..vocab_count)
            .map(|i| VocabularyViolation {
                word: format!("word{}", i),
                node_id: "test-node".to_string(),
                current_level: VocabularyLevel::Advanced,
                target_level: VocabularyLevel::Basic,
                suggestions: vec![],
            })
            .collect();

        ConstraintResult {
            vocabulary_violations: violations,
            correction_capability: CorrectionCapability::NeedsRevision,
            corrections: vec![],
            required_elements_present: missing.is_empty(),
            theme_consistency_score: theme_score,
            missing_elements: missing,
        }
    }

    // ========================================================================
    // Severity Calculation Tests (Task 2.1)
    // ========================================================================

    #[test]
    fn test_quality_score_critical_threshold() {
        // Exactly at threshold (0.5) should be Warning, not Critical
        let (severity, _) = severity_from_quality_score(0.5, "Test");
        assert_eq!(severity, IssueSeverity::Warning);

        // Just below threshold should be Critical
        let (severity, _) = severity_from_quality_score(0.49, "Test");
        assert_eq!(severity, IssueSeverity::Critical);

        // Just above threshold should be Warning
        let (severity, _) = severity_from_quality_score(0.51, "Test");
        assert_eq!(severity, IssueSeverity::Warning);
    }

    #[test]
    fn test_quality_score_warning_threshold() {
        // Exactly at threshold (0.7) should be Info, not Warning
        let (severity, _) = severity_from_quality_score(0.7, "Test");
        assert_eq!(severity, IssueSeverity::Info);

        // Just below threshold should be Warning
        let (severity, _) = severity_from_quality_score(0.69, "Test");
        assert_eq!(severity, IssueSeverity::Warning);

        // Just above threshold should be Info
        let (severity, _) = severity_from_quality_score(0.71, "Test");
        assert_eq!(severity, IssueSeverity::Info);
    }

    #[test]
    fn test_quality_score_edge_cases() {
        // Minimum value (0.0) should be Critical
        let (severity, _) = severity_from_quality_score(0.0, "Test");
        assert_eq!(severity, IssueSeverity::Critical);

        // Maximum value (1.0) should be Info
        let (severity, _) = severity_from_quality_score(1.0, "Test");
        assert_eq!(severity, IssueSeverity::Info);
    }

    #[test]
    fn test_violation_count_critical_threshold() {
        // Exactly at threshold (5) should be Warning, not Critical
        let (severity, _) = severity_from_violation_count(5, "Test");
        assert_eq!(severity, IssueSeverity::Warning);

        // Just above threshold (6) should be Critical
        let (severity, _) = severity_from_violation_count(6, "Test");
        assert_eq!(severity, IssueSeverity::Critical);

        // Well above threshold should be Critical
        let (severity, _) = severity_from_violation_count(10, "Test");
        assert_eq!(severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_violation_count_warning_threshold() {
        // Exactly 0 violations should be Info
        let (severity, _) = severity_from_violation_count(0, "Test");
        assert_eq!(severity, IssueSeverity::Info);

        // 1 violation should be Warning
        let (severity, _) = severity_from_violation_count(1, "Test");
        assert_eq!(severity, IssueSeverity::Warning);

        // Between 1-5 should be Warning
        let (severity, _) = severity_from_violation_count(3, "Test");
        assert_eq!(severity, IssueSeverity::Warning);
    }

    #[test]
    fn test_violation_count_edge_cases() {
        // 0 violations should be Info
        let (severity, _) = severity_from_violation_count(0, "Test");
        assert_eq!(severity, IssueSeverity::Info);

        // Large number should be Critical
        let (severity, _) = severity_from_violation_count(100, "Test");
        assert_eq!(severity, IssueSeverity::Critical);
    }

    // ========================================================================
    // Quality Issue Aggregation Tests (Task 2.3)
    // ========================================================================

    #[test]
    fn test_aggregate_quality_issues_all_good() {
        let result = create_quality_result(0.9, 0.85, vec![]);
        let issues = aggregate_quality_issues("node-1", &result);

        // Should have 2 issues (age + educational), no safety issue
        assert_eq!(issues.len(), 2);

        // Both should be Info severity
        assert!(issues.iter().all(|i| i.severity == IssueSeverity::Info));
    }

    #[test]
    fn test_aggregate_quality_issues_critical_scores() {
        let result = create_quality_result(0.3, 0.2, vec![]);
        let issues = aggregate_quality_issues("node-1", &result);

        // Should have 2 issues
        assert_eq!(issues.len(), 2);

        // Both should be Critical severity
        assert!(issues
            .iter()
            .all(|i| i.severity == IssueSeverity::Critical));
    }

    #[test]
    fn test_aggregate_quality_issues_with_safety() {
        let result = create_quality_result(0.9, 0.9, vec!["Violence detected".to_string()]);
        let issues = aggregate_quality_issues("node-1", &result);

        // Should have 3 issues (age + educational + safety)
        assert_eq!(issues.len(), 3);

        // Safety issue should be Critical
        let safety_issue = issues
            .iter()
            .find(|i| matches!(i.issue_type, IssueType::Quality(QualityIssueKind::Safety)))
            .expect("Safety issue should exist");
        assert_eq!(safety_issue.severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_aggregate_quality_issues_warning_range() {
        let result = create_quality_result(0.6, 0.65, vec![]);
        let issues = aggregate_quality_issues("node-1", &result);

        // Should have 2 issues
        assert_eq!(issues.len(), 2);

        // Both should be Warning severity
        assert!(issues.iter().all(|i| i.severity == IssueSeverity::Warning));
    }

    // ========================================================================
    // Constraint Issue Aggregation Tests (Task 2.4)
    // ========================================================================

    #[test]
    fn test_aggregate_constraint_issues_all_good() {
        let result = create_constraint_result(0, 0.9, vec![]);
        let issues = aggregate_constraint_issues("node-1", &result);

        // Should have 2 issues (vocabulary + theme), no missing elements
        assert_eq!(issues.len(), 2);

        // Both should be Info severity
        assert!(issues.iter().all(|i| i.severity == IssueSeverity::Info));
    }

    #[test]
    fn test_aggregate_constraint_issues_critical_violations() {
        let result = create_constraint_result(10, 0.3, vec![]);
        let issues = aggregate_constraint_issues("node-1", &result);

        // Should have 2 issues
        assert_eq!(issues.len(), 2);

        // Vocabulary should be Critical (10 violations)
        let vocab_issue = issues
            .iter()
            .find(|i| {
                matches!(
                    i.issue_type,
                    IssueType::Constraint(ConstraintIssueKind::Vocabulary)
                )
            })
            .expect("Vocabulary issue should exist");
        assert_eq!(vocab_issue.severity, IssueSeverity::Critical);

        // Theme should be Critical (0.3 score)
        let theme_issue = issues
            .iter()
            .find(|i| {
                matches!(
                    i.issue_type,
                    IssueType::Constraint(ConstraintIssueKind::ThemeConsistency)
                )
            })
            .expect("Theme issue should exist");
        assert_eq!(theme_issue.severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_aggregate_constraint_issues_with_missing_elements() {
        let result = create_constraint_result(0, 0.9, vec!["moral lesson".to_string()]);
        let issues = aggregate_constraint_issues("node-1", &result);

        // Should have 3 issues (vocabulary + theme + missing)
        assert_eq!(issues.len(), 3);

        // Missing elements issue should be Critical
        let missing_issue = issues
            .iter()
            .find(|i| {
                matches!(
                    i.issue_type,
                    IssueType::Constraint(ConstraintIssueKind::RequiredElements)
                )
            })
            .expect("Missing elements issue should exist");
        assert_eq!(missing_issue.severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_aggregate_constraint_issues_warning_range() {
        let result = create_constraint_result(3, 0.65, vec![]);
        let issues = aggregate_constraint_issues("node-1", &result);

        // Should have 2 issues
        assert_eq!(issues.len(), 2);

        // Vocabulary should be Warning (3 violations)
        let vocab_issue = issues
            .iter()
            .find(|i| {
                matches!(
                    i.issue_type,
                    IssueType::Constraint(ConstraintIssueKind::Vocabulary)
                )
            })
            .expect("Vocabulary issue should exist");
        assert_eq!(vocab_issue.severity, IssueSeverity::Warning);

        // Theme should be Warning (0.65 score)
        let theme_issue = issues
            .iter()
            .find(|i| {
                matches!(
                    i.issue_type,
                    IssueType::Constraint(ConstraintIssueKind::ThemeConsistency)
                )
            })
            .expect("Theme issue should exist");
        assert_eq!(theme_issue.severity, IssueSeverity::Warning);
    }

    #[test]
    fn test_aggregate_constraint_issues_boundary_violations() {
        // Exactly 5 violations should be Warning
        let result = create_constraint_result(5, 0.9, vec![]);
        let issues = aggregate_constraint_issues("node-1", &result);

        let vocab_issue = issues
            .iter()
            .find(|i| {
                matches!(
                    i.issue_type,
                    IssueType::Constraint(ConstraintIssueKind::Vocabulary)
                )
            })
            .expect("Vocabulary issue should exist");
        assert_eq!(vocab_issue.severity, IssueSeverity::Warning);

        // 6 violations should be Critical
        let result = create_constraint_result(6, 0.9, vec![]);
        let issues = aggregate_constraint_issues("node-1", &result);

        let vocab_issue = issues
            .iter()
            .find(|i| {
                matches!(
                    i.issue_type,
                    IssueType::Constraint(ConstraintIssueKind::Vocabulary)
                )
            })
            .expect("Vocabulary issue should exist");
        assert_eq!(vocab_issue.severity, IssueSeverity::Critical);
    }

    // ========================================================================
    // Combined Aggregation Tests (Task 2.5)
    // ========================================================================

    #[test]
    fn test_aggregate_all_issues_combines_both() {
        let quality = create_quality_result(0.9, 0.85, vec![]);
        let constraint = create_constraint_result(0, 0.9, vec![]);
        let issues = aggregate_all_issues("node-1", &quality, &constraint);

        // Should have 4 issues total (2 quality + 2 constraint)
        assert_eq!(issues.len(), 4);

        // Should have quality issues
        assert!(issues
            .iter()
            .any(|i| matches!(i.issue_type, IssueType::Quality(_))));

        // Should have constraint issues
        assert!(issues
            .iter()
            .any(|i| matches!(i.issue_type, IssueType::Constraint(_))));
    }

    #[test]
    fn test_aggregate_all_issues_with_all_problems() {
        let quality = create_quality_result(0.3, 0.2, vec!["Violence".to_string()]);
        let constraint = create_constraint_result(10, 0.3, vec!["moral".to_string()]);
        let issues = aggregate_all_issues("node-1", &quality, &constraint);

        // Should have 7 issues:
        // - 3 quality (age + edu + safety)
        // - 4 constraint (vocab + theme + missing) + no safety issues from constraint
        // Wait, let me recalculate: 3 quality (age, edu, safety) + 3 constraint (vocab, theme, missing) = 6
        assert_eq!(issues.len(), 6);

        // Should have multiple critical issues
        let critical_count = issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .count();
        assert!(critical_count >= 4); // safety + vocab + theme + missing
    }

    #[test]
    fn test_aggregate_all_issues_node_id_consistency() {
        let quality = create_quality_result(0.9, 0.85, vec![]);
        let constraint = create_constraint_result(0, 0.9, vec![]);
        let issues = aggregate_all_issues("test-node-123", &quality, &constraint);

        // All issues should have the same node_id
        assert!(issues.iter().all(|i| i.node_id == "test-node-123"));
    }

    #[test]
    fn test_aggregate_all_issues_correction_capability() {
        let quality = create_quality_result(0.6, 0.65, vec![]);
        let constraint = create_constraint_result(3, 0.65, vec![]);
        let issues = aggregate_all_issues("node-1", &quality, &constraint);

        // Quality issues should have CanFixLocally
        let quality_issues: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.issue_type, IssueType::Quality(_)))
            .collect();
        assert!(quality_issues
            .iter()
            .all(|i| i.correction_capability == CorrectionCapability::CanFixLocally));

        // Constraint issues should have NeedsRevision
        let constraint_issues: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.issue_type, IssueType::Constraint(_)))
            .collect();
        assert!(constraint_issues
            .iter()
            .all(|i| i.correction_capability == CorrectionCapability::NeedsRevision));
    }

    // ========================================================================
    // ValidationIssue Struct Tests (Task 2.2)
    // ========================================================================

    #[test]
    fn test_validation_issue_creation() {
        let issue = ValidationIssue::new(
            "node-1".to_string(),
            IssueType::Quality(QualityIssueKind::AgeAppropriateness),
            IssueSeverity::Critical,
            CorrectionCapability::CanFixLocally,
            "Test description".to_string(),
        );

        assert_eq!(issue.node_id, "node-1");
        assert_eq!(
            issue.issue_type,
            IssueType::Quality(QualityIssueKind::AgeAppropriateness)
        );
        assert_eq!(issue.severity, IssueSeverity::Critical);
        assert_eq!(
            issue.correction_capability,
            CorrectionCapability::CanFixLocally
        );
        assert_eq!(issue.description, "Test description");
    }

    #[test]
    fn test_validation_issue_serialization() {
        let issue = ValidationIssue::new(
            "node-1".to_string(),
            IssueType::Constraint(ConstraintIssueKind::Vocabulary),
            IssueSeverity::Warning,
            CorrectionCapability::NeedsRevision,
            "Test description".to_string(),
        );

        // Test that it can be serialized and deserialized
        let json = serde_json::to_string(&issue).expect("Serialization should succeed");
        let deserialized: ValidationIssue =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(issue, deserialized);
    }
}
