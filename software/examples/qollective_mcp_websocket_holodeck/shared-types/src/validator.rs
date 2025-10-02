// ABOUTME: Shared validation types for holodeck content validation
// ABOUTME: Provides validation result structures, criteria, and assessment metrics

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Holodeck content type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum HolodeckContentType {
    Story,
    Template,
    Character,
    Environment,
    Dialogue,
    Scenario,
}

/// Canon strictness level for Star Trek consistency validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum CanonStrictnessLevel {
    Lenient,    // Allow creative interpretation
    Standard,   // Normal canon compliance
    Strict,     // Rigid canon adherence
    Ultra,      // Extremely rigid canon compliance
}

/// Content validation result structure  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentValidationResult {
    pub content_id: Uuid,
    pub is_valid: bool,
    pub overall_score: u32,
    pub structure_analysis: StructureAnalysis,
    pub quality_assessment: QualityAssessment,
    pub safety_analysis: SafetyAnalysis,
    pub improvement_suggestions: Vec<String>,
    pub validation_timestamp: DateTime<Utc>,
    pub approved: bool,
}

/// Structure analysis details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureAnalysis {
    pub has_clear_beginning: bool,
    pub has_developed_middle: bool,
    pub has_satisfying_conclusion: bool,
    pub character_development_score: u32,
    pub plot_coherence_score: u32,
    pub pacing_score: u32,
}

/// Quality assessment metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub quality_score: u32,
    pub narrative_depth: u32,
    pub character_authenticity: u32,
    pub dialogue_quality: u32,
    pub descriptive_richness: u32,
    pub emotional_engagement: u32,
}

/// Safety analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAnalysis {
    pub safety_score: u32,
    pub safety_level: crate::SafetyLevel,
    pub content_warnings: Vec<String>,
    pub age_appropriate: bool,
    pub violence_level: u32,
    pub language_appropriateness: u32,
}

/// Canon consistency validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonValidationResult {
    pub content_id: Uuid,
    pub is_canon_compliant: bool,
    pub compliance_score: u32,
    pub compliance_details: CanonComplianceDetails,
    pub era_consistency: bool,
    pub character_accuracy: bool,
    pub technology_consistency: bool,
    pub violations: Vec<CanonViolation>,
    pub validation_timestamp: DateTime<Utc>,
}

/// Canon compliance analysis details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonComplianceDetails {
    pub universe_consistency_score: u32,
    pub character_behavior_score: u32,
    pub technology_accuracy_score: u32,
    pub timeline_consistency_score: u32,
    pub cultural_accuracy_score: u32,
}

/// Canon violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonViolation {
    pub violation_type: String,
    pub description: String,
    pub severity: ViolationSeverity,
    pub suggested_correction: Option<String>,
}

/// Canon violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

/// Quality assessment result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessmentResult {
    pub content_id: Uuid,
    pub overall_quality_score: u32,
    pub quality_metrics: Vec<QualityMetric>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub recommendations: Vec<String>,
    pub target_audience_appropriateness: bool,
    pub assessment_timestamp: DateTime<Utc>,
}

/// Individual quality metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetric {
    pub metric_name: String,
    pub score: u32,
    pub description: String,
    pub weight: f32,
}

/// Validation criteria for different story types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCriteria {
    pub structure_weight: f32,
    pub character_weight: f32,
    pub dialogue_weight: f32,
    pub pacing_weight: f32,
    pub theme_weight: f32,
    pub safety_weight: f32,
    pub required_elements: Vec<String>,
    pub optional_elements: Vec<String>,
    pub restrictions: Vec<String>,
}