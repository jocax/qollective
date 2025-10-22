use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Trail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_coins: Option<Option<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub is_public: bool,
    pub title: String,
    pub status: TrailStatus,
    /// Generated metadata: generation_params, word_count, ai_model, etc.
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

impl Trail {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PromptGenerationRequest {
    pub generation_request: GenerationRequest,
    pub service_target: MCPServiceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_context: Option<NodeContext>,
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
pub struct GenerationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_goals: Option<Vec<String>>,
    pub language: Language,
    /// TaleTrails tenant ID (extracted by Qollective)
    pub tenant_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dag_config: Option<DagStructureConfig>,
    /// Must be even number for convergence calculation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count: Option<i64>,
    pub theme: String,
    /// Cached prompts from prompt-helper (Phase 0.5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_packages: Option<Option<HashMap<String, serde_json::Value>>>,
    pub age_group: AgeGroup,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_elements: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level: Option<VocabularyLevel>,
    /// TaleTrails user ID from JWT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<Option<i64>>,
    /// Predefined story structure preset (Tier 1: Simple). Mutually exclusive with dag_config. Takes priority if both provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_structure: Option<String>,
}

impl GenerationRequest {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrailInsertData {
    /// Trail description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Public visibility
    pub is_public: bool,
    /// Trail metadata JSON (generation params, word count, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
    pub status: TrailStatus,
    /// Categorization tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Price in coins (null for free content)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_coins: Option<Option<i64>>,
    /// Trail title
    pub title: String,
}

impl TrailInsertData {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResult {
    pub correction_capability: CorrectionCapability,
    pub corrections: Vec<CorrectionSuggestion>,
    pub age_appropriate_score: f64,
    pub educational_value_score: f64,
    pub safety_issues: Vec<String>,
    pub is_valid: bool,
}

impl ValidationResult {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerationMetadata {
    pub total_word_count: i64,
    pub validation_rounds: i64,
    pub orchestrator_version: String,
    pub ai_model_version: String,
    pub generation_duration_seconds: i64,
    pub generated_at: String,
}

impl GenerationMetadata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DAG {
    pub convergence_points: Vec<String>,
    pub edges: Vec<Edge>,
    pub nodes: HashMap<String, ContentNode>,
    pub start_node_id: String,
}

impl DAG {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EducationalContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_words: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_objective: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_facts: Option<Vec<String>>,
}

impl EducationalContent {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerationResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<GenerationError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_metadata: Option<GenerationMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_trace: Option<PipelineExecutionTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_generation_metadata: Option<PromptGenerationSummary>,
    pub request_id: String,
    pub progress_percentage: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail: Option<Trail>,
    pub status: GenerationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_steps: Option<Vec<TrailStep>>,
}

impl GenerationResponse {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Choice {
    pub id: String,
    pub next_node_id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Choice {
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
    pub progress_percentage: i64,
    pub job_id: String,
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

/// Pattern for how story branches converge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum ConvergencePattern {
    SingleConvergence,
    MultipleConvergence,
    EndOnly,
    PureBranching,
    ParallelPaths,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VocabularyViolation {
    pub target_level: VocabularyLevel,
    pub word: String,
    pub node_id: String,
    pub current_level: VocabularyLevel,
    pub suggestions: Vec<String>,
}

impl VocabularyViolation {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConstraintResult {
    pub required_elements_present: bool,
    pub vocabulary_violations: Vec<VocabularyViolation>,
    pub corrections: Vec<CorrectionSuggestion>,
    pub correction_capability: CorrectionCapability,
    pub theme_consistency_score: f64,
    pub missing_elements: Vec<String>,
}

impl ConstraintResult {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

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
pub struct ExternalError {
    pub error_message: String,
    pub error_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_possible: Option<bool>,
}

impl ExternalError {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrailStepInsertData {
    /// Sequential order of step
    pub step_order: i64,
    /// Interactive story node content
    pub content_data: HashMap<String, serde_json::Value>,
    /// Step description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether step is required
    pub is_required: bool,
    /// Step metadata (word count, node_id, convergence point, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MappingValidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_age_groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_theme_length: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_languages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_theme_length: Option<i64>,
}

impl MappingValidation {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExternalGenerationRequestV1 {
    /// Story theme (e.g., 'underwater adventure')
    pub theme: String,
    pub age_group: AgeGroup,
    pub language: Language,
}

impl ExternalGenerationRequestV1 {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// MCP service identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum MCPServiceType {
    StoryGenerator,
    QualityControl,
    ConstraintEnforcer,
    PromptHelper,
    Orchestrator,
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
pub struct CorrectionSuggestion {
    pub severity: String,
    pub suggestion: String,
    pub field: String,
    pub issue: String,
}

impl CorrectionSuggestion {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Content {
    pub choices: Vec<Choice>,
    pub next_nodes: Vec<String>,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_content: Option<EducationalContent>,
    pub node_id: String,
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
pub struct ExternalGenerationResponseV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_data: Option<TrailInsertData>,
    /// Trail steps ready for DB insert (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_steps_data: Option<Vec<TrailStepInsertData>>,
    /// Generation statistics and metadata
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ExternalError>,
    /// Job identifier for tracking
    pub job_id: String,
    /// Final job status
    pub status: String,
}

impl ExternalGenerationResponseV1 {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PromptGenerationSummary {
    pub prompt_generation_duration_ms: i64,
    pub prompts_used: HashMap<String, PromptPackage>,
    pub llm_generated_count: i64,
    pub fallback_count: i64,
    pub prompts_generated_at: String,
}

impl PromptGenerationSummary {
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
pub struct MappingDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_goals: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_18_plus: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_12_14: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_15_17: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_9_11: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_15_17: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_18_plus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_9_11: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_6_8: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_elements: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_6_8: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_12_14: Option<String>,
}

impl MappingDefaults {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrailStep {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub is_required: bool,
    pub content_reference: ContentReference,
    pub step_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
pub struct ContentNode {
    pub content: Content,
    pub incoming_edges: i64,
    pub outgoing_edges: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_metadata: Option<HashMap<String, serde_json::Value>>,
    pub id: String,
}

impl ContentNode {
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
pub struct PipelineExecutionTrace {
    /// All pipeline events published to NATS (optional, for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_published: Option<Vec<HashMap<String, serde_json::Value>>>,
    /// Total pipeline execution time in milliseconds
    pub total_duration_ms: i64,
    /// List of pipeline phases completed in order
    pub phases_completed: Vec<GenerationPhase>,
    /// Complete history of all MCP service calls with timing
    pub service_invocations: Vec<ServiceInvocation>,
    /// Request ID for this execution
    pub request_id: String,
}

impl PipelineExecutionTrace {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NodeContext {
    pub incoming_edges: i64,
    pub is_convergence_point: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_content: Option<String>,
    pub node_id: String,
    pub node_position: i64,
}

impl NodeContext {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DagStructureConfig {
    /// Number of choices per decision node
    pub branching_factor: i64,
    /// Position of convergence as ratio (0.5 = midpoint). Required for SingleConvergence, MultipleConvergence, and EndOnly. Must be omitted for PureBranching and ParallelPaths
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convergence_point_ratio: Option<f64>,
    /// Maximum depth of DAG tree
    pub max_depth: i64,
    /// Total number of nodes in story DAG
    pub node_count: i64,
    pub convergence_pattern: ConvergencePattern,
}

impl DagStructureConfig {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LLMConfig {
    pub stop_sequences: Vec<String>,
    pub presence_penalty: f64,
    pub top_p: f64,
    pub temperature: f64,
    pub frequency_penalty: f64,
    pub max_tokens: i64,
}

impl LLMConfig {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
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
pub struct PromptMetadata {
    pub age_group_context: AgeGroup,
    pub generated_at: String,
    pub generation_method: PromptGenerationMethod,
    pub template_version: String,
    pub theme_context: String,
    pub service_target: MCPServiceType,
    pub language_context: Language,
}

impl PromptMetadata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ServiceInvocation {
    /// Batch ID if part of batch processing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<i64>,
    /// Error message if service call failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// MCP tool invoked (generate_structure, validate_content, etc.)
    pub tool_name: String,
    /// Duration of service call in milliseconds
    pub duration_ms: i64,
    /// MCP service name (prompt-helper, story-generator, quality-control, constraint-enforcer)
    pub service_name: String,
    /// Node ID being processed (for content generation and validation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub phase: GenerationPhase,
    /// Whether the service call succeeded
    pub success: bool,
    /// When the service call started
    pub started_at: String,
}

impl ServiceInvocation {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PromptPackage {
    /// LLM user message with context
    pub user_prompt: String,
    /// LLM system instruction
    pub system_prompt: String,
    pub llm_config: LLMConfig,
    /// LLM model identifier
    pub llm_model: String,
    pub prompt_metadata: PromptMetadata,
    pub fallback_used: bool,
    pub language: Language,
}

impl PromptPackage {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Edge {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    pub to_node_id: String,
    pub choice_id: String,
    pub from_node_id: String,
}

impl Edge {
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

/// Method used to generate prompts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum PromptGenerationMethod {
    LLMGenerated,
    TemplateFallback,
    Cached,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BatchInfo {
    pub batch_size: i64,
    pub batch_index: i64,
    pub total_batches: i64,
    pub batch_id: String,
}

impl BatchInfo {
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
    /// Batch identifier for grouping related generation operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    /// Correlation ID for tracking request chains across services
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

impl TaleTrailCustomMetadata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ContentReference {
    pub content: Content,
    pub temp_node_id: String,
}

impl ContentReference {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}
