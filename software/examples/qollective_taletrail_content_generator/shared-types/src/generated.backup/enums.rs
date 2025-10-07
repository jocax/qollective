//! Enumeration types for TaleTrail content generation

use serde::{Deserialize, Serialize};

/// Target age group for content generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgeGroup {
    #[serde(rename = "6-8")]
    SixToEight,
    #[serde(rename = "9-11")]
    NineToEleven,
    #[serde(rename = "12-14")]
    TwelveToFourteen,
    #[serde(rename = "15-17")]
    FifteenToSeventeen,
    #[serde(rename = "+18")]
    EighteenPlus,
}

/// Content language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// German
    #[serde(rename = "de")]
    De,
    /// English
    #[serde(rename = "en")]
    En,
}

/// Vocabulary complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VocabularyLevel {
    Basic,
    Intermediate,
    Advanced,
}

/// Status of content generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Publication status of trail (matches DB enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrailStatus {
    Draft,
    Published,
    Archived,
}

/// Validation service correction capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrectionCapability {
    CanFixLocally,
    NeedsRevision,
    NoFixPossible,
}

/// Current phase of generation pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenerationPhase {
    PromptGeneration,
    Structure,
    Generation,
    Validation,
    Assembly,
    Complete,
}

/// MCP service identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MCPServiceType {
    StoryGenerator,
    QualityControl,
    ConstraintEnforcer,
    PromptHelper,
    Orchestrator,
}

/// Method used to generate prompts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PromptGenerationMethod {
    LLMGenerated,
    TemplateFallback,
    Cached,
}

/// External API version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiVersion {
    V1,
}

/// Correction suggestion severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CorrectionSeverity {
    Low,
    Medium,
    High,
}
