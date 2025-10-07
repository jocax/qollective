use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Dag {
    pub convergence_points: Vec<serde_json::Value>,
    pub nodes: HashMap<String, serde_json::Value>,
    pub edges: Vec<serde_json::Value>,
    pub start_node_id: String,
}

impl Dag {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Generationmetadata {
    pub generation_duration_seconds: i64,
    pub total_word_count: i64,
    pub ai_model_version: String,
    pub validation_rounds: i64,
    pub orchestrator_version: String,
    pub generated_at: String,
}

impl Generationmetadata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Trailstepinsertdata {
    /// Whether step is required
    pub is_required: bool,
    /// Step metadata (word count, node_id, convergence point, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
    /// Step title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Step description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Interactive story node content
    pub content_data: HashMap<String, serde_json::Value>,
    /// Sequential order of step
    pub step_order: i64,
}

impl Trailstepinsertdata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Promptmetadata {
    pub age_group_context: Agegroup,
    pub language_context: Language,
    pub service_target: Mcpservicetype,
    pub generated_at: String,
    pub template_version: String,
    pub theme_context: String,
    pub generation_method: Promptgenerationmethod,
}

impl Promptmetadata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// MCP service identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mcpservicetype {
    #[serde(rename = "StoryGenerator")]
    Storygenerator,
    #[serde(rename = "QualityControl")]
    Qualitycontrol,
    #[serde(rename = "ConstraintEnforcer")]
    Constraintenforcer,
    #[serde(rename = "PromptHelper")]
    Prompthelper,
    Orchestrator,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Educationalcontent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_words: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_facts: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_objective: Option<String>,
}

impl Educationalcontent {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// External API version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Apiversion {
    #[serde(rename = "v1")]
    V1,
}

/// Validation service correction capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Correctioncapability {
    #[serde(rename = "CanFixLocally")]
    Canfixlocally,
    #[serde(rename = "NeedsRevision")]
    Needsrevision,
    #[serde(rename = "NoFixPossible")]
    Nofixpossible,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Externaljobstatus {
    pub status: String,
    /// Estimated time to completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_completion_seconds: Option<i64>,
    pub progress_percentage: i64,
    /// Human-readable phase description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_phase: Option<String>,
    pub job_id: String,
}

impl Externaljobstatus {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Content language: de (German), en (English)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "de")]
    De,
    #[serde(rename = "en")]
    En,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Promptgenerationsummary {
    pub prompt_generation_duration_ms: i64,
    pub prompts_used: HashMap<String, serde_json::Value>,
    pub llm_generated_count: i64,
    pub fallback_count: i64,
    pub prompts_generated_at: String,
}

impl Promptgenerationsummary {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Trailstep {
    /// word_count, node_id, convergence_point, etc.
    pub metadata: HashMap<String, serde_json::Value>,
    pub is_required: bool,
    pub content_reference: Contentreference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub step_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Trailstep {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Gatewaymappingconfig {
    pub defaults: Mappingdefaults,
    pub validation: Mappingvalidation,
}

impl Gatewaymappingconfig {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Nodecontext {
    pub node_id: String,
    pub node_position: i64,
    pub incoming_edges: i64,
    pub is_convergence_point: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_content: Option<String>,
}

impl Nodecontext {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Contentnode {
    pub content: Content,
    pub outgoing_edges: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_metadata: Option<HashMap<String, serde_json::Value>>,
    pub id: String,
    pub incoming_edges: i64,
}

impl Contentnode {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Method used to generate prompts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Promptgenerationmethod {
    #[serde(rename = "LLMGenerated")]
    Llmgenerated,
    #[serde(rename = "TemplateFallback")]
    Templatefallback,
    Cached,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Mappingvalidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_age_groups: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_languages: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_theme_length: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_theme_length: Option<i64>,
}

impl Mappingvalidation {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Target age group for content generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Agegroup {
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

/// Vocabulary complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Vocabularylevel {
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "intermediate")]
    Intermediate,
    #[serde(rename = "advanced")]
    Advanced,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Generationrequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_goals: Option<Vec<serde_json::Value>>,
    /// Cached prompts from prompt-helper (Phase 0.5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_packages: Option<Option<HashMap<String, serde_json::Value>>>,
    pub age_group: Agegroup,
    /// TaleTrails user ID from JWT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<Option<i64>>,
    /// Must be even number for convergence calculation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_elements: Option<Vec<serde_json::Value>>,
    pub theme: String,
    pub language: Language,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level: Option<Vocabularylevel>,
    /// TaleTrails tenant ID (extracted by Qollective)
    pub tenant_id: i64,
}

impl Generationrequest {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Promptgenerationrequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_context: Option<Nodecontext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_info: Option<Batchinfo>,
    pub generation_request: Generationrequest,
    pub service_target: Mcpservicetype,
}

impl Promptgenerationrequest {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Publication status of trail (matches DB enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trailstatus {
    #[serde(rename = "DRAFT")]
    Draft,
    #[serde(rename = "PUBLISHED")]
    Published,
    #[serde(rename = "ARCHIVED")]
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Generationerror {
    pub timestamp: String,
    pub error_code: String,
    pub error_message: String,
    pub retry_possible: bool,
}

impl Generationerror {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Status of content generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Generationstatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Mappingerror {
    pub error_type: String,
    pub message: String,
}

impl Mappingerror {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Trailinsertdata {
    /// Trail description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: Trailstatus,
    /// Categorization tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<serde_json::Value>>,
    /// Public visibility
    pub is_public: bool,
    /// Price in coins (null for free content)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_coins: Option<Option<i64>>,
    /// Trail metadata JSON (generation params, word count, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
    /// Trail title
    pub title: String,
}

impl Trailinsertdata {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Vocabularyviolation {
    pub word: String,
    pub node_id: String,
    pub current_level: Vocabularylevel,
    pub suggestions: Vec<serde_json::Value>,
    pub target_level: Vocabularylevel,
}

impl Vocabularyviolation {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Externalgenerationrequestv1 {
    pub language: Language,
    /// Story theme (e.g., 'underwater adventure')
    pub theme: String,
    pub age_group: Agegroup,
}

impl Externalgenerationrequestv1 {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_content: Option<Educationalcontent>,
    pub text: String,
    pub next_nodes: Vec<serde_json::Value>,
    pub choices: Vec<serde_json::Value>,
    pub convergence_point: bool,
    pub node_id: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

impl Content {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Choice {
    pub next_node_id: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub text: String,
}

impl Choice {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Batchinfo {
    pub batch_index: i64,
    pub batch_id: String,
    pub batch_size: i64,
    pub total_batches: i64,
}

impl Batchinfo {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Externalgenerationresponsev1 {
    /// Trail steps ready for DB insert (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_steps_data: Option<Vec<serde_json::Value>>,
    /// Job identifier for tracking
    pub job_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Externalerror>,
    /// Final job status
    pub status: String,
    /// Generation statistics and metadata
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_data: Option<Trailinsertdata>,
}

impl Externalgenerationresponsev1 {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Constraintresult {
    pub theme_consistency_score: f64,
    pub correction_capability: Correctioncapability,
    pub vocabulary_violations: Vec<serde_json::Value>,
    pub required_elements_present: bool,
    pub corrections: Vec<serde_json::Value>,
    pub missing_elements: Vec<serde_json::Value>,
}

impl Constraintresult {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Current phase of generation pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Generationphase {
    #[serde(rename = "PromptGeneration")]
    Promptgeneration,
    Structure,
    Generation,
    Validation,
    Assembly,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Contentreference {
    pub content: Content,
    pub temp_node_id: String,
}

impl Contentreference {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Correctionsuggestion {
    pub field: String,
    pub suggestion: String,
    pub issue: String,
    pub severity: String,
}

impl Correctionsuggestion {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Generationresponse {
    pub progress_percentage: i64,
    pub status: Generationstatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail: Option<Trail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_metadata: Option<Generationmetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<serde_json::Value>>,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_generation_metadata: Option<Promptgenerationsummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_steps: Option<Vec<serde_json::Value>>,
}

impl Generationresponse {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Trail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_coins: Option<Option<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: Trailstatus,
    pub title: String,
    /// Generated metadata: generation_params, word_count, ai_model, etc.
    pub metadata: HashMap<String, serde_json::Value>,
    pub is_public: bool,
}

impl Trail {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Mappingdefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_18_plus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_goals: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_9_11: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_9_11: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_18_plus: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_6_8: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_15_17: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_12_14: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_level_6_8: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_elements: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_15_17: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count_12_14: Option<i64>,
}

impl Mappingdefaults {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Externalerror {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_possible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    pub error_code: String,
    pub error_message: String,
}

impl Externalerror {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Edge {
    pub choice_id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

impl Edge {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Promptpackage {
    /// LLM system instruction
    pub system_prompt: String,
    /// LLM user message with context
    pub user_prompt: String,
    pub fallback_used: bool,
    /// LLM model identifier
    pub llm_model: String,
    pub prompt_metadata: Promptmetadata,
    pub llm_config: Llmconfig,
    pub language: Language,
}

impl Promptpackage {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Validationresult {
    pub correction_capability: Correctioncapability,
    pub age_appropriate_score: f64,
    pub corrections: Vec<serde_json::Value>,
    pub educational_value_score: f64,
    pub is_valid: bool,
    pub safety_issues: Vec<serde_json::Value>,
}

impl Validationresult {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Llmconfig {
    pub presence_penalty: f64,
    pub frequency_penalty: f64,
    pub max_tokens: i64,
    pub top_p: f64,
    pub temperature: f64,
    pub stop_sequences: Vec<serde_json::Value>,
}

impl Llmconfig {
    /// Validate this instance against the JSON Schema constraints
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

