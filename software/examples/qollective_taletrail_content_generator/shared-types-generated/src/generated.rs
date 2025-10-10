use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current phase of generation pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum GenerationPhase {
    PromptGeneration,
    Structure,
    Generation,
    Validation,
    Assembly,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GatewayMappingConfig {
    pub defaults: MappingDefaults,
    pub validation: MappingValidation,
}

impl GatewayMappingConfig {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PromptMetadata {
    pub language_context: Language,
    pub template_version: String,
    pub generation_method: PromptGenerationMethod,
    pub theme_context: String,
    pub service_target: MCPServiceType,
    pub generated_at: String,
    pub age_group_context: AgeGroup,
}

impl PromptMetadata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Publication status of trail (matches DB enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum TrailStatus {
    DRAFT,
    PUBLISHED,
    ARCHIVED,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VocabularyViolation {
    pub word: String,
    pub current_level: VocabularyLevel,
    pub node_id: String,
    pub suggestions: Vec<String>,
    pub target_level: VocabularyLevel,
}

impl VocabularyViolation {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ContentReference {
    pub temp_node_id: String,
    pub content: Content,
}

impl ContentReference {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExternalJobStatus {
    /// Human-readable phase description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_phase: Option<String>,
    pub job_id: String,
    pub progress_percentage: i64,
    pub status: String,
    /// Estimated time to completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_completion_seconds: Option<i64>,
}

impl ExternalJobStatus {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MappingDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_9_11: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_15_17: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_18_plus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_12_14: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_6_8: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_9_11: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_6_8: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_goals: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_12_14: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_15_17: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_elements: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_18_plus: Option<i64>,
}

impl MappingDefaults {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Content {
    pub next_nodes: Vec<String>,
    pub choices: Vec<Choice>,
    pub node_id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_content: Option<EducationalContent>,
    pub convergence_point: bool,
    pub text: String,
}

impl Content {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerationError {
    pub timestamp: String,
    pub error_message: String,
    pub retry_possible: bool,
    pub error_code: String,
}

impl GenerationError {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PromptPackage {
    pub language: Language,
    pub prompt_metadata: PromptMetadata,
    pub llm_config: LLMConfig,
    pub fallback_used: bool,
    /// LLM model identifier
    pub llm_model: String,
    /// LLM user message with context
    pub user_prompt: String,
    /// LLM system instruction
    pub system_prompt: String,
}

impl PromptPackage {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrailInsertData {
    pub status: TrailStatus,
    /// Categorization tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Trail title
    pub title: String,
    /// Trail metadata JSON (generation params, word count, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
    /// Trail description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Public visibility
    pub is_public: bool,
    /// Price in coins (null for free content)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_coins: Option<Option<i64>>,
}

impl TrailInsertData {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrailStepInsertData {
    /// Step metadata (word count, node_id, convergence point, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
    /// Step description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether step is required
    pub is_required: bool,
    /// Interactive story node content
    pub content_data: HashMap<String, serde_json::Value>,
    /// Sequential order of step
    pub step_order: i64,
    /// Step title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl TrailStepInsertData {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Target age group for content generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum AgeGroup {
    #[serde(rename = "6-8")]
    _6To8,
    #[serde(rename = "9-11")]
    _9To11,
    #[serde(rename = "12-14")]
    _12To14,
    #[serde(rename = "15-17")]
    _15To17,
    #[serde(rename = "+18")]
    Plus18,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerationMetadata {
    pub generation_duration_seconds: i64,
    pub total_word_count: i64,
    pub generated_at: String,
    pub ai_model_version: String,
    pub validation_rounds: i64,
    pub orchestrator_version: String,
}

impl GenerationMetadata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// External API version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum ApiVersion {
    #[serde(rename = "v1")]
    V1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConstraintResult {
    pub correction_capability: CorrectionCapability,
    pub required_elements_present: bool,
    pub theme_consistency_score: f64,
    pub vocabulary_violations: Vec<VocabularyViolation>,
    pub missing_elements: Vec<String>,
    pub corrections: Vec<CorrectionSuggestion>,
}

impl ConstraintResult {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Choice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub id: String,
    pub next_node_id: String,
    pub text: String,
}

impl Choice {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExternalError {
    pub error_message: String,
    pub error_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_possible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl ExternalError {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Status of content generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum GenerationStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

/// Content language: de (German), en (English)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum Language {
    #[serde(rename = "de")]
    De,
    #[serde(rename = "en")]
    En,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EducationalContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_objective: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_facts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_words: Option<Vec<String>>,
}

impl EducationalContent {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerationResponse {
    pub progress_percentage: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_metadata: Option<GenerationMetadata>,
    pub status: GenerationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_generation_metadata: Option<PromptGenerationSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<GenerationError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_steps: Option<Vec<TrailStep>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail: Option<Trail>,
    pub request_id: String,
}

impl GenerationResponse {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Custom metadata extensions for TaleTrail content generation (stored in Meta.extensions)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TaleTrailCustomMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_phase: Option<GenerationPhase>,
    /// Correlation ID for tracking request chains across services
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// Batch identifier for grouping related generation operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
}

impl TaleTrailCustomMetadata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResult {
    pub age_appropriate_score: f64,
    pub safety_issues: Vec<String>,
    pub corrections: Vec<CorrectionSuggestion>,
    pub is_valid: bool,
    pub educational_value_score: f64,
    pub correction_capability: CorrectionCapability,
}

impl ValidationResult {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MappingValidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_theme_length: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_age_groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_theme_length: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_languages: Option<Vec<String>>,
}

impl MappingValidation {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Edge {
    pub from_node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    pub choice_id: String,
    pub to_node_id: String,
}

impl Edge {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Validation service correction capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum CorrectionCapability {
    CanFixLocally,
    NeedsRevision,
    NoFixPossible,
}

/// Method used to generate prompts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum PromptGenerationMethod {
    LLMGenerated,
    TemplateFallback,
    Cached,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PromptGenerationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_context: Option<NodeContext>,
    pub generation_request: GenerationRequest,
    pub service_target: MCPServiceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_info: Option<BatchInfo>,
}

impl PromptGenerationRequest {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PromptGenerationSummary {
    pub fallback_count: i64,
    pub prompt_generation_duration_ms: i64,
    pub prompts_used: HashMap<String, PromptPackage>,
    pub prompts_generated_at: String,
    pub llm_generated_count: i64,
}

impl PromptGenerationSummary {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DAG {
    pub nodes: HashMap<String, ContentNode>,
    pub convergence_points: Vec<String>,
    pub edges: Vec<Edge>,
    pub start_node_id: String,
}

impl DAG {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CorrectionSuggestion {
    pub issue: String,
    pub severity: String,
    pub suggestion: String,
    pub field: String,
}

impl CorrectionSuggestion {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LLMConfig {
    pub temperature: f64,
    pub max_tokens: i64,
    pub top_p: f64,
    pub stop_sequences: Vec<String>,
    pub presence_penalty: f64,
    pub frequency_penalty: f64,
}

impl LLMConfig {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NodeContext {
    pub is_convergence_point: bool,
    pub node_position: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_content: Option<String>,
    pub incoming_edges: i64,
    pub node_id: String,
}

impl NodeContext {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrailStep {
    pub content_reference: ContentReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub is_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub step_order: i64,
    /// word_count, node_id, convergence_point, etc.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TrailStep {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Trail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub is_public: bool,
    pub title: String,
    pub status: TrailStatus,
    /// Generated metadata: generation_params, word_count, ai_model, etc.
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_coins: Option<Option<i64>>,
}

impl Trail {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExternalGenerationRequestV1 {
    pub age_group: AgeGroup,
    pub language: Language,
    /// Story theme (e.g., 'underwater adventure')
    pub theme: String,
}

impl ExternalGenerationRequestV1 {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Vocabulary complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum VocabularyLevel {
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "intermediate")]
    Intermediate,
    #[serde(rename = "advanced")]
    Advanced,
}

/// MCP service identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MCPServiceType {
    StoryGenerator,
    QualityControl,
    ConstraintEnforcer,
    PromptHelper,
    Orchestrator,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExternalGenerationResponseV1 {
    /// Job identifier for tracking
    pub job_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_data: Option<TrailInsertData>,
    /// Final job status
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ExternalError>,
    /// Generation statistics and metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Trail steps ready for DB insert (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_steps_data: Option<Vec<TrailStepInsertData>>,
}

impl ExternalGenerationResponseV1 {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ContentNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_metadata: Option<HashMap<String, serde_json::Value>>,
    pub incoming_edges: i64,
    pub outgoing_edges: i64,
    pub id: String,
    pub content: Content,
}

impl ContentNode {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BatchInfo {
    pub batch_size: i64,
    pub batch_id: String,
    pub batch_index: i64,
    pub total_batches: i64,
}

impl BatchInfo {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Must be even number for convergence calculation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count: Option<i64>,
    pub age_group: AgeGroup,
    /// TaleTrails tenant ID (extracted by Qollective)
    pub tenant_id: i64,
    /// Cached prompts from prompt-helper (Phase 0.5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_packages: Option<Option<HashMap<String, serde_json::Value>>>,
    pub theme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_goals: Option<Vec<String>>,
    /// TaleTrails user ID from JWT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<Option<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_elements: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level: Option<VocabularyLevel>,
    pub language: Language,
}

impl GenerationRequest {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MappingError {
    pub error_type: String,
    pub message: String,
}

impl MappingError {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}
