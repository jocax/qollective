//! Prompt helper service trait for dynamic prompt generation
//!
//! Abstracts prompt generation logic to enable:
//! - Dynamic LLM-based prompt generation with template fallback
//! - Mock-based testing without real LLM calls
//! - Language-specific model selection
//!
//! # Example Usage
//!
//! ```rust,ignore
//! // Production usage with LLM fallback
//! let prompt_helper = LlmPromptHelper::new(llm_service, config);
//! let story_prompts = prompt_helper
//!     .generate_story_prompts(&request)
//!     .await?;
//! assert!(!story_prompts.fallback_used); // LLM succeeded
//!
//! // Testing with mock
//! let mut mock_helper = MockPromptHelperService::new();
//! mock_helper
//!     .expect_generate_story_prompts()
//!     .returning(|_| Ok(mock_prompt_package()));
//! ```

use async_trait::async_trait;

use crate::errors::TaleTrailError;
use crate::generated::prompts::{PromptPackage, PromptGenerationRequest};
use crate::generated::enums::Language;

/// Statistics for prompt generation tracking
#[derive(Debug, Clone)]
pub struct PromptGenerationStats {
    /// Total prompts generated
    pub total_generated: usize,

    /// Number of times fallback template was used
    pub fallback_count: usize,

    /// Number of times LLM generation succeeded
    pub llm_success_count: usize,
}

/// Abstracts prompt generation logic with LLM and template fallback
///
/// Implementations:
/// - `LlmPromptHelper`: Dynamic LLM generation with config.toml fallback
/// - `TemplatePromptHelper`: Always uses config.toml templates (no LLM)
/// - `MockPromptHelper`: Test mock with predefined PromptPackages
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
#[async_trait]
pub trait PromptHelperService: Send + Sync + std::fmt::Debug {
    /// Generate story generation prompts
    ///
    /// Tries LLM-based generation first, falls back to config.toml templates on failure.
    ///
    /// # Arguments
    /// * `request` - Prompt generation context (theme, age group, language, etc.)
    ///
    /// # Returns
    /// `PromptPackage` with system/user prompts and `fallback_used` indicator
    ///
    /// # Errors
    /// - `TaleTrailError::ConfigError`: Template loading failure (fatal)
    async fn generate_story_prompts(
        &self,
        request: &PromptGenerationRequest,
    ) -> Result<PromptPackage, TaleTrailError>;

    /// Generate validation prompts (for quality-control service)
    ///
    /// Focuses on age-appropriateness, safety, educational value criteria.
    ///
    /// # Arguments
    /// * `request` - Prompt generation context
    ///
    /// # Returns
    /// `PromptPackage` configured for validation tasks
    ///
    /// # Errors
    /// - `TaleTrailError::ConfigError`: Template loading failure
    async fn generate_validation_prompts(
        &self,
        request: &PromptGenerationRequest,
    ) -> Result<PromptPackage, TaleTrailError>;

    /// Generate constraint checking prompts (for constraint-enforcer service)
    ///
    /// Focuses on vocabulary level, theme consistency, required elements.
    ///
    /// # Arguments
    /// * `request` - Prompt generation context
    ///
    /// # Returns
    /// `PromptPackage` configured for constraint checking
    ///
    /// # Errors
    /// - `TaleTrailError::ConfigError`: Template loading failure
    async fn generate_constraint_prompts(
        &self,
        request: &PromptGenerationRequest,
    ) -> Result<PromptPackage, TaleTrailError>;

    /// Get LLM model for language (with fallback to default)
    ///
    /// Maps language to language-specific model (e.g., German → german-qwen).
    /// Never fails - returns default model if language not configured.
    ///
    /// # Arguments
    /// * `language` - Target language
    ///
    /// # Returns
    /// Model identifier string
    ///
    /// # Errors
    /// - `TaleTrailError::ConfigError`: Model configuration loading failure
    async fn get_model_for_language(
        &self,
        language: &Language,
    ) -> Result<String, TaleTrailError>;

    /// Get prompt generation statistics
    ///
    /// # Returns
    /// Statistics including total prompts, fallback count, LLM success count
    fn get_stats(&self) -> PromptGenerationStats;
}
