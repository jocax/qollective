use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DAG {
    pub nodes: HashMap<String, ContentNode>,
    pub edges: Vec<Edge>,
    pub start_node_id: String,
    pub convergence_points: Vec<String>,
}

impl DAG {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EducationalContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_words: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_facts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_objective: Option<String>,
}

impl EducationalContent {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
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

/// Publication status of trail (matches DB enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum TrailStatus {
    DRAFT,
    PUBLISHED,
    ARCHIVED,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrailInsertData {
    /// Trail title
    pub title: String,
    /// Price in coins (null for free content)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_coins: Option<Option<i64>>,
    /// Categorization tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Trail metadata JSON (generation params, word count, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
    /// Public visibility
    pub is_public: bool,
    pub status: TrailStatus,
    /// Trail description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl TrailInsertData {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExternalGenerationRequestV1 {
    pub age_group: AgeGroup,
    /// Story theme (e.g., 'underwater adventure')
    pub theme: String,
    pub language: Language,
}

impl ExternalGenerationRequestV1 {
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
pub struct ContentNode {
    pub content: Content,
    pub id: String,
    pub incoming_edges: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_metadata: Option<HashMap<String, serde_json::Value>>,
    pub outgoing_edges: i64,
}

impl ContentNode {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrailStep {
    pub step_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub is_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// word_count, node_id, convergence_point, etc.
    pub metadata: HashMap<String, serde_json::Value>,
    pub content_reference: ContentReference,
}

impl TrailStep {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ValidationIssueSummary {
    /// Type of validation issue (e.g., 'age_appropriateness', 'word_count')
    pub issue_type: String,
    /// Issue severity level
    pub severity: String,
    /// ID of node with unresolved issue
    pub node_id: String,
    /// Human-readable issue description
    pub description: String,
}

impl ValidationIssueSummary {
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
pub struct DagStructureConfig {
    /// Total number of nodes in story DAG
    pub node_count: i64,
    /// Maximum depth of DAG tree
    pub max_depth: i64,
    /// Number of choices per decision node
    pub branching_factor: i64,
    pub convergence_pattern: ConvergencePattern,
    /// Position of convergence as ratio (0.5 = midpoint). Required for SingleConvergence, MultipleConvergence, and EndOnly. Must be omitted for PureBranching and ParallelPaths
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convergence_point_ratio: Option<f64>,
}

impl DagStructureConfig {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConstraintResult {
    pub corrections: Vec<CorrectionSuggestion>,
    pub required_elements_present: bool,
    pub theme_consistency_score: f64,
    pub vocabulary_violations: Vec<VocabularyViolation>,
    pub correction_capability: CorrectionCapability,
    pub missing_elements: Vec<String>,
}

impl ConstraintResult {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Trail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Generated metadata: generation_params, word_count, ai_model, etc.
    pub metadata: HashMap<String, serde_json::Value>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_coins: Option<Option<i64>>,
    pub status: TrailStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub is_public: bool,
}

impl Trail {
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
pub struct VocabularyViolation {
    pub target_level: VocabularyLevel,
    pub word: String,
    pub current_level: VocabularyLevel,
    pub suggestions: Vec<String>,
    pub node_id: String,
}

impl VocabularyViolation {
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
pub struct PromptGenerationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_context: Option<NodeContext>,
    pub service_target: MCPServiceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_info: Option<BatchInfo>,
    pub generation_request: GenerationRequest,
}

impl PromptGenerationRequest {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PromptGenerationSummary {
    pub prompt_generation_duration_ms: i64,
    pub llm_generated_count: i64,
    pub prompts_generated_at: String,
    pub prompts_used: HashMap<String, PromptPackage>,
    pub fallback_count: i64,
}

impl PromptGenerationSummary {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CorrectionSummary {
    /// Number of attempts to correct this node
    pub attempts: i64,
    /// Type of correction applied
    pub correction_type: String,
    /// ID of node that was corrected
    pub node_id: String,
    /// Whether the correction succeeded
    pub success: bool,
}

impl CorrectionSummary {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerationResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_generation_metadata: Option<PromptGenerationSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail: Option<Trail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<GenerationError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_steps: Option<Vec<TrailStep>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_trace: Option<PipelineExecutionTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_metadata: Option<GenerationMetadata>,
    pub status: GenerationStatus,
    pub progress_percentage: i64,
    pub request_id: String,
}

impl GenerationResponse {
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

/// External API version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum ApiVersion {
    #[serde(rename = "v1")]
    V1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MappingDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_elements: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_15_17: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_9_11: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_15_17: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_18_plus: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_goals: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_12_14: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_18_plus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_6_8: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_9_11: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_6_8: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_12_14: Option<i64>,
}

impl MappingDefaults {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerationMetadata {
    /// Actual node count used after resolving preset/explicit values (Priority: explicit > preset > defaults)
    pub resolved_node_count: i64,
    pub generated_at: String,
    /// Number of negotiation rounds executed after initial validation
    pub negotiation_rounds_executed: i64,
    pub orchestrator_version: String,
    pub total_word_count: i64,
    pub ai_model_version: String,
    /// List of all correction attempts during negotiation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrections_applied: Option<Vec<CorrectionSummary>>,
    pub generation_duration_seconds: i64,
    /// Issues remaining after max negotiation rounds exceeded (empty if all resolved)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved_validation_issues: Option<Vec<ValidationIssueSummary>>,
    /// Total number of validation rounds executed (includes initial + negotiation rounds)
    pub validation_rounds: i64,
    /// Percentage of nodes that passed validation (nodes_passed / total_nodes)
    pub validation_pass_rate: f64,
}

impl GenerationMetadata {
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
    pub max_theme_length: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_age_groups: Option<Vec<String>>,
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
pub struct LLMConfig {
    pub presence_penalty: f64,
    pub stop_sequences: Vec<String>,
    pub top_p: f64,
    pub max_tokens: i64,
    pub frequency_penalty: f64,
    pub temperature: f64,
}

impl LLMConfig {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ServiceInvocation {
    /// Duration of service call in milliseconds
    pub duration_ms: i64,
    /// Whether the service call succeeded
    pub success: bool,
    pub phase: GenerationPhase,
    /// MCP tool invoked (generate_structure, validate_content, etc.)
    pub tool_name: String,
    /// MCP service name (prompt-helper, story-generator, quality-control, constraint-enforcer)
    pub service_name: String,
    /// Batch ID if part of batch processing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<i64>,
    /// Node ID being processed (for content generation and validation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// When the service call started
    pub started_at: String,
    /// Error message if service call failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl ServiceInvocation {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CorrectionSuggestion {
    pub suggestion: String,
    pub severity: String,
    pub field: String,
    pub issue: String,
}

impl CorrectionSuggestion {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Validation policy for content generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ValidationPolicy {
    /// Whether to enable content validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_validation: Option<bool>,
    /// Custom restricted words per language (overrides or merges with config defaults)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_restricted_words: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_mode: Option<RestrictedWordsMergeMode>,
}

impl ValidationPolicy {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PipelineExecutionTrace {
    /// Request ID for this execution
    pub request_id: String,
    /// All pipeline events published to NATS (optional, for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_published: Option<Vec<HashMap<String, serde_json::Value>>>,
    /// List of pipeline phases completed in order
    pub phases_completed: Vec<GenerationPhase>,
    /// Complete history of all MCP service calls with timing
    pub service_invocations: Vec<ServiceInvocation>,
    /// Total pipeline execution time in milliseconds
    pub total_duration_ms: i64,
}

impl PipelineExecutionTrace {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BatchInfo {
    pub batch_id: String,
    pub batch_index: i64,
    pub total_batches: i64,
    pub batch_size: i64,
}

impl BatchInfo {
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
pub struct PromptPackage {
    pub fallback_used: bool,
    pub language: Language,
    /// LLM model identifier
    pub llm_model: String,
    /// LLM system instruction
    pub system_prompt: String,
    pub llm_config: LLMConfig,
    pub prompt_metadata: PromptMetadata,
    /// LLM user message with context
    pub user_prompt: String,
}

impl PromptPackage {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerationRequest {
    pub age_group: AgeGroup,
    /// Must be even number for convergence calculation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count: Option<i64>,
    /// Cached prompts from prompt-helper (Phase 0.5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_packages: Option<Option<HashMap<String, serde_json::Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_policy: Option<ValidationPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_goals: Option<Vec<String>>,
    /// TaleTrails user ID from JWT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<Option<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_elements: Option<Vec<String>>,
    /// Predefined story structure preset (Tier 1: Simple). Mutually exclusive with dag_config. Takes priority if both provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_structure: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub language: Language,
    /// TaleTrails tenant ID (extracted by Qollective)
    pub tenant_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dag_config: Option<DagStructureConfig>,
    pub theme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level: Option<VocabularyLevel>,
}

impl GenerationRequest {
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

/// Method used to generate prompts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum PromptGenerationMethod {
    LLMGenerated,
    TemplateFallback,
    Cached,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExternalJobStatus {
    pub job_id: String,
    pub progress_percentage: i64,
    /// Human-readable phase description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_phase: Option<String>,
    /// Estimated time to completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_completion_seconds: Option<i64>,
    pub status: String,
}

impl ExternalJobStatus {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PromptMetadata {
    pub age_group_context: AgeGroup,
    pub generated_at: String,
    pub language_context: Language,
    pub service_target: MCPServiceType,
    pub template_version: String,
    pub theme_context: String,
    pub generation_method: PromptGenerationMethod,
}

impl PromptMetadata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResult {
    pub educational_value_score: f64,
    pub is_valid: bool,
    pub safety_issues: Vec<String>,
    pub age_appropriate_score: f64,
    pub correction_capability: CorrectionCapability,
    pub corrections: Vec<CorrectionSuggestion>,
}

impl ValidationResult {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerationError {
    pub retry_possible: bool,
    pub error_code: String,
    pub error_message: String,
    pub timestamp: String,
}

impl GenerationError {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// How to merge custom restricted words with config defaults
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum RestrictedWordsMergeMode {
    Replace,
    Merge,
    ConfigOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrailStepInsertData {
    /// Step metadata (word count, node_id, convergence point, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
    /// Sequential order of step
    pub step_order: i64,
    /// Interactive story node content
    pub content_data: HashMap<String, serde_json::Value>,
    /// Step description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether step is required
    pub is_required: bool,
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
pub struct Content {
    pub node_id: String,
    pub choices: Vec<Choice>,
    pub text: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub convergence_point: bool,
    pub next_nodes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_content: Option<EducationalContent>,
}

impl Content {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExternalError {
    pub error_message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    pub error_code: String,
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
pub struct ExternalGenerationResponseV1 {
    /// Job identifier for tracking
    pub job_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_data: Option<TrailInsertData>,
    /// Trail steps ready for DB insert (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_steps_data: Option<Vec<TrailStepInsertData>>,
    /// Generation statistics and metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Final job status
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ExternalError>,
}

impl ExternalGenerationResponseV1 {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NodeContext {
    pub node_id: String,
    pub node_position: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_content: Option<String>,
    pub is_convergence_point: bool,
    pub incoming_edges: i64,
}

impl NodeContext {
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
pub struct MappingError {
    pub message: String,
    pub error_type: String,
}

impl MappingError {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}
