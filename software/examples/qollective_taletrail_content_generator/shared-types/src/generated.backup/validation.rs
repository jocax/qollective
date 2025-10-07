//! Validation and constraint checking types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::enums::*;

/// Quality control validation output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Overall validation result
    pub is_valid: bool,

    /// Age appropriateness score (0.0-1.0)
    pub age_appropriate_score: f32,

    /// List of safety concerns
    pub safety_issues: Vec<String>,

    /// Educational content score (0.0-1.0)
    pub educational_value_score: f32,

    /// Suggested corrections
    pub corrections: Vec<CorrectionSuggestion>,

    /// Correction capability
    pub correction_capability: CorrectionCapability,
}

/// Constraint enforcement validation output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintResult {
    /// Words above grade level
    pub vocabulary_violations: Vec<VocabularyViolation>,

    /// Theme consistency across story (0.0-1.0)
    pub theme_consistency_score: f32,

    /// All required elements included
    pub required_elements_present: bool,

    /// List of missing required elements
    pub missing_elements: Vec<String>,

    /// Suggested corrections
    pub corrections: Vec<CorrectionSuggestion>,

    /// Correction capability
    pub correction_capability: CorrectionCapability,
}

/// Field-level correction suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionSuggestion {
    /// Field that needs correction
    pub field: String,

    /// Issue description
    pub issue: String,

    /// Suggested fix
    pub suggestion: String,

    /// Issue severity
    pub severity: CorrectionSeverity,
}

/// Vocabulary violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyViolation {
    /// Word that violates vocabulary level
    pub word: String,

    /// Node where violation occurs
    pub node_id: Uuid,

    /// Current complexity level of word
    pub current_level: VocabularyLevel,

    /// Target complexity level
    pub target_level: VocabularyLevel,

    /// Suggested replacements
    pub suggestions: Vec<String>,
}

/// Quality validation result (alias for ValidationResult)
pub type QualityResult = ValidationResult;
