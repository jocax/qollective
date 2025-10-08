//! Validation service trait for content quality and constraint checking
//!
//! Abstracts validation logic to enable:
//! - Quality validation (age-appropriateness, safety, educational value)
//! - Constraint enforcement (vocabulary, theme consistency, required elements)
//! - DAG-level validation
//! - Mock-based testing without real validation rules
//!
//! # Example Usage
//!
//! ```rust,ignore
//! // Production quality validator
//! let validator = RubricQualityValidator::new(config);
//! let quality_result = validator
//!     .validate_quality(&content, &prompts, &request)
//!     .await?;
//!
//! // Testing with mock
//! let mut mock_validator = MockValidationService::new();
//! mock_validator
//!     .expect_validate_quality()
//!     .returning(|_, _, _| Ok(passing_quality_result()));
//! ```

use async_trait::async_trait;

use crate::errors::TaleTrailError;
use crate::{Content, DAG, GenerationRequest, PromptPackage, ValidationResult, ConstraintResult};

/// Abstracts validation logic for quality control and constraint enforcement
///
/// Implementations:
/// - `RubricQualityValidator`: Production quality validation using config rubrics
/// - `VocabularyConstraintValidator`: Production constraint enforcement
/// - `MockValidator`: Test mock with predefined validation results
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
#[async_trait]
pub trait ValidationService: Send + Sync + std::fmt::Debug {
    /// Validate content quality (quality-control service)
    ///
    /// Checks:
    /// - Age-appropriateness (language complexity, themes)
    /// - Safety (violence, inappropriate content)
    /// - Educational value (alignment with educational goals)
    ///
    /// Returns correction suggestions with `CorrectionCapability` indicating
    /// if validator can fix locally, needs revision, or cannot fix.
    ///
    /// # Arguments
    /// * `content` - Content to validate
    /// * `prompt_package` - Validation prompts with rubric criteria
    /// * `request` - Original generation request with age group and goals
    ///
    /// # Returns
    /// `ValidationResult` with pass/fail status and correction suggestions
    ///
    /// # Errors
    /// - `TaleTrailError::ValidationError`: Validation process failure
    async fn validate_quality(
        &self,
        content: &Content,
        prompt_package: &PromptPackage,
        request: &GenerationRequest,
    ) -> Result<ValidationResult, TaleTrailError>;

    /// Validate content constraints (constraint-enforcer service)
    ///
    /// Checks:
    /// - Vocabulary level (words appropriate for age group)
    /// - Theme consistency (matches requested theme)
    /// - Required elements (moral lessons, science facts, etc.)
    ///
    /// Returns violations with specific word-level suggestions.
    ///
    /// # Arguments
    /// * `content` - Content to check
    /// * `prompt_package` - Constraint checking prompts
    /// * `request` - Generation request with vocabulary level and theme
    ///
    /// # Returns
    /// `ConstraintResult` with violations and correction suggestions
    ///
    /// # Errors
    /// - `TaleTrailError::ValidationError`: Constraint checking failure
    async fn validate_constraints(
        &self,
        content: &Content,
        prompt_package: &PromptPackage,
        request: &GenerationRequest,
    ) -> Result<ConstraintResult, TaleTrailError>;

    /// Validate entire DAG structure and content
    ///
    /// Performs holistic validation:
    /// - All paths from start to end are valid
    /// - Convergence points function correctly
    /// - Story coherence across all nodes
    /// - Educational progression makes sense
    ///
    /// # Arguments
    /// * `dag` - Complete DAG with all nodes populated
    /// * `request` - Original generation request
    ///
    /// # Returns
    /// `ValidationResult` with overall pass/fail and per-node issues
    ///
    /// # Errors
    /// - `TaleTrailError::ValidationError`: DAG validation failure
    async fn validate_dag(
        &self,
        dag: &DAG,
        request: &GenerationRequest,
    ) -> Result<ValidationResult, TaleTrailError>;
}
