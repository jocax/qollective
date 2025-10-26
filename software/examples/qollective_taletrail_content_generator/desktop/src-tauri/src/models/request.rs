use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};
use crate::constants::validation;

// Import shared types from generated schema
pub use shared_types_generated::{AgeGroup, Language, VocabularyLevel};

/// Represents a generation request for the TaleTrail content pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GenerationRequest {
    /// Unique request identifier
    pub request_id: String,

    /// Tenant identifier for multi-tenancy support
    pub tenant_id: String,

    /// Story theme (e.g., "Space Adventure", "Medieval Quest")
    pub theme: String,

    /// Target age group - uses shared schema enum
    pub age_group: AgeGroup,

    /// Target language - uses shared schema enum
    pub language: Language,

    /// Vocabulary complexity level - uses shared schema enum
    pub vocabulary_level: VocabularyLevel,

    /// Number of story nodes to generate
    pub node_count: u32,

    /// Optional educational focus areas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educational_focus: Option<Vec<String>>,

    /// Optional request constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<RequestConstraints>,

    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<RequestMetadata>,

    /// Predefined story structure preset (Tier 1: Simple).
    /// Mutually exclusive with custom node_count - if provided, orchestrator uses preset configuration.
    /// Values: "guided", "adventure", "epic", "choose_your_path"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_structure: Option<String>,
}

/// Constraints for the generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RequestConstraints {
    /// Maximum number of choices per story node
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_choices_per_node: Option<u32>,

    /// Minimum story length in characters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_story_length: Option<u32>,

    /// Topics to avoid in story generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forbidden_topics: Option<Vec<String>>,

    /// Topics that must be included
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_topics: Option<Vec<String>>,
}

/// Metadata for tracking request lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RequestMetadata {
    /// ISO 8601 timestamp when request was submitted
    pub submitted_at: String,

    /// Optional identifier of the user who submitted the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_by: Option<String>,

    /// Reference to original request ID if this is a replay/modification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_request_id: Option<String>,
}

impl GenerationRequest {
    /// Convert to shared-types-generated::GenerationRequest for orchestrator
    ///
    /// This adapter transforms the desktop's GenerationRequest type into the
    /// shared type format expected by the orchestrator. Key transformations:
    /// - tenant_id: String -> i64 (parsing with error handling)
    /// - node_count: u32 -> Option<i64>
    /// - educational_focus -> educational_goals
    /// - vocabulary_level: VocabularyLevel -> Option<VocabularyLevel>
    ///
    /// # Errors
    /// Returns `AppError::ValidationError` if tenant_id cannot be parsed as i64
    pub fn to_shared_type(&self) -> AppResult<shared_types_generated::GenerationRequest> {
        let tenant_id = self.tenant_id.parse::<i64>()
            .map_err(|e| AppError::ValidationError(
                format!("Invalid tenant_id '{}': must be a valid number (i64): {}", self.tenant_id, e)
            ))?;

        Ok(shared_types_generated::GenerationRequest {
            tenant_id,
            node_count: Some(self.node_count as i64),
            theme: self.theme.clone(),
            age_group: self.age_group.clone(),
            language: self.language.clone(),
            vocabulary_level: Some(self.vocabulary_level.clone()),
            educational_goals: self.educational_focus.clone(),
            required_elements: self.constraints.as_ref()
                .and_then(|c| c.required_topics.clone()),
            tags: None,
            dag_config: None,
            prompt_packages: None,
            author_id: None,
            story_structure: self.story_structure.clone(),
            validation_policy: None,
        })
    }

    /// Validate all fields of the generation request
    ///
    /// # Errors
    /// Returns `AppError::ValidationError` if any field fails validation
    pub fn validate(&self) -> AppResult<()> {
        // Validate request_id is not empty
        if self.request_id.trim().is_empty() {
            return Err(AppError::ValidationError(
                "request_id must not be empty".to_string(),
            ));
        }

        // Validate tenant_id is not empty
        if self.tenant_id.trim().is_empty() {
            return Err(AppError::ValidationError(
                "tenant_id must not be empty".to_string(),
            ));
        }

        // Validate theme is not empty
        if self.theme.trim().is_empty() {
            return Err(AppError::ValidationError(
                "theme must not be empty".to_string(),
            ));
        }

        // Note: age_group, language, and vocabulary_level are now enums
        // Type safety is enforced at compile time, no runtime validation needed

        // Validate node_count is within allowed range
        if self.node_count < validation::MIN_NODE_COUNT {
            return Err(AppError::ValidationError(format!(
                "node_count must be at least {}. Got: {}",
                validation::MIN_NODE_COUNT,
                self.node_count
            )));
        }

        if self.node_count > validation::MAX_NODE_COUNT {
            return Err(AppError::ValidationError(format!(
                "node_count must be at most {}. Got: {}",
                validation::MAX_NODE_COUNT,
                self.node_count
            )));
        }

        // Validate story_structure if provided
        if let Some(ref structure) = self.story_structure {
            if !validation::VALID_STORY_STRUCTURES.contains(&structure.as_str()) {
                return Err(AppError::ValidationError(format!(
                    "story_structure must be one of: {}. Got: {}",
                    validation::VALID_STORY_STRUCTURES.join(", "),
                    structure
                )));
            }
        }

        // Validate constraints if present
        if let Some(ref constraints) = self.constraints {
            constraints.validate()?;
        }

        Ok(())
    }

    /// Serialize the request to a JSON string
    ///
    /// # Errors
    /// Returns `AppError::JsonError` if serialization fails
    pub fn to_json(&self) -> AppResult<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// Serialize the request to JSON bytes for NATS publishing
    ///
    /// # Errors
    /// Returns `AppError::JsonError` if serialization fails
    pub fn to_bytes(&self) -> AppResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| e.into())
    }

    /// Create a new builder for constructing a GenerationRequest
    pub fn builder() -> GenerationRequestBuilder {
        GenerationRequestBuilder::default()
    }
}

impl RequestConstraints {
    /// Validate constraint values
    fn validate(&self) -> AppResult<()> {
        // Validate max_choices_per_node if present
        if let Some(max_choices) = self.max_choices_per_node {
            if max_choices < validation::MIN_CHOICES_PER_NODE {
                return Err(AppError::ValidationError(format!(
                    "max_choices_per_node must be at least {}. Got: {}",
                    validation::MIN_CHOICES_PER_NODE,
                    max_choices
                )));
            }

            if max_choices > validation::MAX_CHOICES_PER_NODE {
                return Err(AppError::ValidationError(format!(
                    "max_choices_per_node must be at most {}. Got: {}",
                    validation::MAX_CHOICES_PER_NODE,
                    max_choices
                )));
            }
        }

        // Validate min_story_length if present
        if let Some(min_length) = self.min_story_length {
            if min_length < validation::MIN_STORY_LENGTH {
                return Err(AppError::ValidationError(format!(
                    "min_story_length must be at least {}. Got: {}",
                    validation::MIN_STORY_LENGTH,
                    min_length
                )));
            }

            if min_length > validation::MAX_STORY_LENGTH {
                return Err(AppError::ValidationError(format!(
                    "min_story_length must be at most {}. Got: {}",
                    validation::MAX_STORY_LENGTH,
                    min_length
                )));
            }
        }

        Ok(())
    }
}

/// Builder for constructing GenerationRequest instances
#[derive(Default)]
pub struct GenerationRequestBuilder {
    request_id: Option<String>,
    tenant_id: Option<String>,
    theme: Option<String>,
    age_group: Option<AgeGroup>,
    language: Option<Language>,
    vocabulary_level: Option<VocabularyLevel>,
    node_count: Option<u32>,
    educational_focus: Option<Vec<String>>,
    constraints: Option<RequestConstraints>,
    metadata: Option<RequestMetadata>,
    story_structure: Option<String>,
}

impl GenerationRequestBuilder {
    /// Set the request ID
    pub fn request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Set the tenant ID
    pub fn tenant_id(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Set the theme
    pub fn theme(mut self, theme: String) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Set the age group
    pub fn age_group(mut self, age_group: AgeGroup) -> Self {
        self.age_group = Some(age_group);
        self
    }

    /// Set the language
    pub fn language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    /// Set the vocabulary level
    pub fn vocabulary_level(mut self, vocabulary_level: VocabularyLevel) -> Self {
        self.vocabulary_level = Some(vocabulary_level);
        self
    }

    /// Set the node count
    pub fn node_count(mut self, node_count: u32) -> Self {
        self.node_count = Some(node_count);
        self
    }

    /// Set the educational focus
    pub fn educational_focus(mut self, educational_focus: Vec<String>) -> Self {
        self.educational_focus = Some(educational_focus);
        self
    }

    /// Set the constraints
    pub fn constraints(mut self, constraints: RequestConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    /// Set the metadata
    pub fn metadata(mut self, metadata: RequestMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the story structure preset
    pub fn story_structure(mut self, story_structure: String) -> Self {
        self.story_structure = Some(story_structure);
        self
    }

    /// Build the GenerationRequest
    ///
    /// # Errors
    /// Returns `AppError::ValidationError` if required fields are missing
    pub fn build(self) -> AppResult<GenerationRequest> {
        let request = GenerationRequest {
            request_id: self.request_id.ok_or_else(|| {
                AppError::ValidationError("request_id is required".to_string())
            })?,
            tenant_id: self.tenant_id.ok_or_else(|| {
                AppError::ValidationError("tenant_id is required".to_string())
            })?,
            theme: self.theme.ok_or_else(|| {
                AppError::ValidationError("theme is required".to_string())
            })?,
            age_group: self.age_group.ok_or_else(|| {
                AppError::ValidationError("age_group is required".to_string())
            })?,
            language: self.language.ok_or_else(|| {
                AppError::ValidationError("language is required".to_string())
            })?,
            vocabulary_level: self.vocabulary_level.ok_or_else(|| {
                AppError::ValidationError("vocabulary_level is required".to_string())
            })?,
            node_count: self.node_count.ok_or_else(|| {
                AppError::ValidationError("node_count is required".to_string())
            })?,
            educational_focus: self.educational_focus,
            constraints: self.constraints,
            metadata: self.metadata,
            story_structure: self.story_structure,
        };

        // Validate before returning
        request.validate()?;

        Ok(request)
    }
}

impl RequestMetadata {
    /// Create new metadata with current timestamp
    pub fn new() -> Self {
        Self {
            submitted_at: chrono::Utc::now().to_rfc3339(),
            submitted_by: None,
            original_request_id: None,
        }
    }

    /// Create metadata with a specific user
    pub fn with_user(mut self, submitted_by: String) -> Self {
        self.submitted_by = Some(submitted_by);
        self
    }

    /// Mark this as a replay of another request
    pub fn with_original(mut self, original_request_id: String) -> Self {
        self.original_request_id = Some(original_request_id);
        self
    }
}

impl Default for RequestMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> GenerationRequest {
        GenerationRequest {
            request_id: "req-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            theme: "Space Adventure".to_string(),
            age_group: AgeGroup::_6To8,
            language: Language::En,
            vocabulary_level: VocabularyLevel::Basic,
            node_count: 10,
            educational_focus: None,
            constraints: None,
            metadata: None,
            story_structure: None,
        }
    }

    #[test]
    fn test_valid_request_validation() {
        let request = valid_request();
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_empty_request_id() {
        let mut request = valid_request();
        request.request_id = "".to_string();
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_empty_tenant_id() {
        let mut request = valid_request();
        request.tenant_id = "".to_string();
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_empty_theme() {
        let mut request = valid_request();
        request.theme = "".to_string();
        assert!(request.validate().is_err());
    }

    // Tests for invalid enum values are no longer needed - type safety enforced at compile time

    #[test]
    fn test_all_age_groups_serialize() {
        // Test that all age group variants work correctly
        let age_groups = vec![
            AgeGroup::_6To8,
            AgeGroup::_9To11,
            AgeGroup::_12To14,
            AgeGroup::_15To17,
            AgeGroup::Plus18,
        ];

        for age_group in age_groups {
            let mut request = valid_request();
            request.age_group = age_group;
            assert!(request.validate().is_ok());
            // Verify serialization works
            assert!(request.to_json().is_ok());
        }
    }

    #[test]
    fn test_all_languages_serialize() {
        // Test that all language variants work correctly
        let languages = vec![Language::En, Language::De];

        for language in languages {
            let mut request = valid_request();
            request.language = language;
            assert!(request.validate().is_ok());
            assert!(request.to_json().is_ok());
        }
    }

    #[test]
    fn test_all_vocabulary_levels_serialize() {
        // Test that all vocabulary level variants work correctly
        let levels = vec![
            VocabularyLevel::Basic,
            VocabularyLevel::Intermediate,
            VocabularyLevel::Advanced,
        ];

        for level in levels {
            let mut request = valid_request();
            request.vocabulary_level = level;
            assert!(request.validate().is_ok());
            assert!(request.to_json().is_ok());
        }
    }

    #[test]
    fn test_node_count_too_low() {
        let mut request = valid_request();
        request.node_count = validation::MIN_NODE_COUNT - 1;
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("node_count"));
    }

    #[test]
    fn test_node_count_too_high() {
        let mut request = valid_request();
        request.node_count = validation::MAX_NODE_COUNT + 1;
        let result = request.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("node_count"));
    }

    #[test]
    fn test_node_count_boundary_values() {
        let mut request = valid_request();

        // Test minimum
        request.node_count = validation::MIN_NODE_COUNT;
        assert!(request.validate().is_ok());

        // Test maximum
        request.node_count = validation::MAX_NODE_COUNT;
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_constraints_validation_max_choices_too_low() {
        let mut request = valid_request();
        request.constraints = Some(RequestConstraints {
            max_choices_per_node: Some(validation::MIN_CHOICES_PER_NODE - 1),
            min_story_length: None,
            forbidden_topics: None,
            required_topics: None,
        });
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_constraints_validation_max_choices_too_high() {
        let mut request = valid_request();
        request.constraints = Some(RequestConstraints {
            max_choices_per_node: Some(validation::MAX_CHOICES_PER_NODE + 1),
            min_story_length: None,
            forbidden_topics: None,
            required_topics: None,
        });
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_constraints_validation_min_story_length_too_low() {
        let mut request = valid_request();
        request.constraints = Some(RequestConstraints {
            max_choices_per_node: None,
            min_story_length: Some(validation::MIN_STORY_LENGTH - 1),
            forbidden_topics: None,
            required_topics: None,
        });
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_constraints_validation_min_story_length_too_high() {
        let mut request = valid_request();
        request.constraints = Some(RequestConstraints {
            max_choices_per_node: None,
            min_story_length: Some(validation::MAX_STORY_LENGTH + 1),
            forbidden_topics: None,
            required_topics: None,
        });
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_to_json() {
        let request = valid_request();
        let json = request.to_json().unwrap();
        assert!(json.contains("req-123"));
        assert!(json.contains("Space Adventure"));
    }

    #[test]
    fn test_serialization_deserialization() {
        let request = valid_request();
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: GenerationRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.request_id, deserialized.request_id);
        assert_eq!(request.theme, deserialized.theme);
        assert_eq!(request.node_count, deserialized.node_count);
    }

    #[test]
    fn test_builder_pattern() {
        let request = GenerationRequest::builder()
            .request_id("req-789".to_string())
            .tenant_id("tenant-abc".to_string())
            .theme("Medieval Quest".to_string())
            .age_group(AgeGroup::_9To11)
            .language(Language::De)
            .vocabulary_level(VocabularyLevel::Intermediate)
            .node_count(20)
            .build()
            .unwrap();

        assert_eq!(request.request_id, "req-789");
        assert_eq!(request.theme, "Medieval Quest");
        assert_eq!(request.language, Language::De);
    }

    #[test]
    fn test_builder_missing_required_field() {
        let result = GenerationRequest::builder()
            .request_id("req-789".to_string())
            // Missing tenant_id
            .theme("Medieval Quest".to_string())
            .age_group(AgeGroup::_9To11)
            .language(Language::De)
            .vocabulary_level(VocabularyLevel::Intermediate)
            .node_count(20)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_with_constraints() {
        let constraints = RequestConstraints {
            max_choices_per_node: Some(4),
            min_story_length: Some(500),
            forbidden_topics: Some(vec!["violence".to_string()]),
            required_topics: Some(vec!["friendship".to_string()]),
        };

        let request = GenerationRequest::builder()
            .request_id("req-789".to_string())
            .tenant_id("tenant-abc".to_string())
            .theme("Medieval Quest".to_string())
            .age_group(AgeGroup::_9To11)
            .language(Language::De)
            .vocabulary_level(VocabularyLevel::Intermediate)
            .node_count(20)
            .constraints(constraints)
            .build()
            .unwrap();

        assert!(request.constraints.is_some());
        let req_constraints = request.constraints.unwrap();
        assert_eq!(req_constraints.max_choices_per_node, Some(4));
        assert_eq!(req_constraints.min_story_length, Some(500));
    }

    #[test]
    fn test_metadata_creation() {
        let metadata = RequestMetadata::new();
        assert!(metadata.submitted_by.is_none());
        assert!(metadata.original_request_id.is_none());
        assert!(!metadata.submitted_at.is_empty());
    }

    #[test]
    fn test_metadata_with_user() {
        let metadata = RequestMetadata::new()
            .with_user("user-123".to_string());
        assert_eq!(metadata.submitted_by, Some("user-123".to_string()));
    }

    #[test]
    fn test_metadata_with_original() {
        let metadata = RequestMetadata::new()
            .with_original("original-req-456".to_string());
        assert_eq!(metadata.original_request_id, Some("original-req-456".to_string()));
    }

    #[test]
    fn test_json_format_matches_schema() {
        // Test that JSON serialization uses snake_case and correct enum values
        let request = GenerationRequest {
            request_id: "req-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            theme: "Space Adventure".to_string(),
            age_group: AgeGroup::_6To8,
            language: Language::En,
            vocabulary_level: VocabularyLevel::Basic,
            node_count: 10,
            educational_focus: None,
            constraints: None,
            metadata: None,
            story_structure: Some("guided".to_string()),
        };

        let json = request.to_json().unwrap();

        // Verify snake_case field names
        assert!(json.contains("\"request_id\""), "Should use snake_case for request_id");
        assert!(json.contains("\"tenant_id\""), "Should use snake_case for tenant_id");
        assert!(json.contains("\"age_group\""), "Should use snake_case for age_group");
        assert!(json.contains("\"node_count\""), "Should use snake_case for node_count");
        assert!(json.contains("\"vocabulary_level\""), "Should use snake_case for vocabulary_level");
        assert!(json.contains("\"story_structure\""), "Should use snake_case for story_structure");

        // Verify enum values match schema
        assert!(json.contains("\"6-8\""), "AgeGroup should serialize as '6-8'");
        assert!(json.contains("\"en\""), "Language should serialize as 'en'");
        assert!(json.contains("\"basic\""), "VocabularyLevel should serialize as 'basic'");
        assert!(json.contains("\"guided\""), "story_structure value should be 'guided'");

        // Should NOT contain camelCase
        assert!(!json.contains("\"requestId\""), "Should NOT use camelCase");
        assert!(!json.contains("\"tenantId\""), "Should NOT use camelCase");
        assert!(!json.contains("\"ageGroup\""), "Should NOT use camelCase");
    }

    #[test]
    fn test_story_structure_validation() {
        let mut request = valid_request();

        // Valid story structure
        request.story_structure = Some("guided".to_string());
        assert!(request.validate().is_ok());

        request.story_structure = Some("adventure".to_string());
        assert!(request.validate().is_ok());

        request.story_structure = Some("epic".to_string());
        assert!(request.validate().is_ok());

        request.story_structure = Some("choose_your_path".to_string());
        assert!(request.validate().is_ok());

        // Invalid story structure
        request.story_structure = Some("invalid_structure".to_string());
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_to_shared_type_conversion() {
        // Create desktop request with String tenant_id and u32 node_count
        let request = GenerationRequest {
            request_id: "req-123".to_string(),
            tenant_id: "1".to_string(),  // String that parses to i64
            theme: "Space Adventure".to_string(),
            age_group: AgeGroup::_6To8,
            language: Language::En,
            vocabulary_level: VocabularyLevel::Basic,
            node_count: 10,  // u32
            educational_focus: Some(vec!["science".to_string(), "math".to_string()]),
            constraints: Some(RequestConstraints {
                max_choices_per_node: Some(4),
                min_story_length: Some(500),
                forbidden_topics: None,
                required_topics: Some(vec!["friendship".to_string()]),
            }),
            metadata: None,
            story_structure: Some("guided".to_string()),
        };

        // Convert to shared type
        let shared = request.to_shared_type().unwrap();

        // Verify type conversions
        assert_eq!(shared.tenant_id, 1_i64, "tenant_id should be converted from String '1' to i64 1");
        assert_eq!(shared.node_count, Some(10_i64), "node_count should be converted from u32 10 to Option<i64> Some(10)");

        // Verify field mappings
        assert_eq!(shared.theme, "Space Adventure");
        assert_eq!(shared.educational_goals, Some(vec!["science".to_string(), "math".to_string()]),
                   "educational_focus should map to educational_goals");
        assert_eq!(shared.required_elements, Some(vec!["friendship".to_string()]),
                   "required_topics should map to required_elements");
        assert_eq!(shared.story_structure, Some("guided".to_string()));

        // Verify optional fields are set correctly
        assert_eq!(shared.vocabulary_level, Some(VocabularyLevel::Basic));
        assert_eq!(shared.tags, None);
        assert_eq!(shared.dag_config, None);
        assert_eq!(shared.prompt_packages, None);
        assert_eq!(shared.author_id, None);
    }

    #[test]
    fn test_to_shared_type_invalid_tenant_id() {
        // Create request with non-numeric tenant_id
        let request = GenerationRequest {
            request_id: "req-123".to_string(),
            tenant_id: "not-a-number".to_string(),  // Invalid for i64 conversion
            theme: "Space Adventure".to_string(),
            age_group: AgeGroup::_6To8,
            language: Language::En,
            vocabulary_level: VocabularyLevel::Basic,
            node_count: 10,
            educational_focus: None,
            constraints: None,
            metadata: None,
            story_structure: None,
        };

        // Conversion should fail with validation error
        let result = request.to_shared_type();
        assert!(result.is_err(), "Should fail to convert invalid tenant_id");

        let err = result.unwrap_err();
        assert!(err.to_string().contains("tenant_id"), "Error should mention tenant_id");
        assert!(err.to_string().contains("not-a-number"), "Error should include the invalid value");
    }

    #[test]
    fn test_to_shared_type_large_tenant_id() {
        // Test with large but valid i64 tenant_id
        let request = GenerationRequest {
            request_id: "req-123".to_string(),
            tenant_id: "9223372036854775807".to_string(),  // i64::MAX
            theme: "Space Adventure".to_string(),
            age_group: AgeGroup::_6To8,
            language: Language::En,
            vocabulary_level: VocabularyLevel::Basic,
            node_count: 10,
            educational_focus: None,
            constraints: None,
            metadata: None,
            story_structure: None,
        };

        let shared = request.to_shared_type().unwrap();
        assert_eq!(shared.tenant_id, 9223372036854775807_i64);
    }

    #[test]
    fn test_to_shared_type_without_optional_fields() {
        // Test conversion without optional fields
        let request = GenerationRequest {
            request_id: "req-123".to_string(),
            tenant_id: "42".to_string(),
            theme: "Simple Story".to_string(),
            age_group: AgeGroup::_9To11,
            language: Language::De,
            vocabulary_level: VocabularyLevel::Intermediate,
            node_count: 5,
            educational_focus: None,
            constraints: None,
            metadata: None,
            story_structure: None,
        };

        let shared = request.to_shared_type().unwrap();

        assert_eq!(shared.tenant_id, 42);
        assert_eq!(shared.node_count, Some(5));
        assert_eq!(shared.educational_goals, None);
        assert_eq!(shared.required_elements, None);
        assert_eq!(shared.story_structure, None);
    }
}
