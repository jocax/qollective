//! LLM service trait for AI model interaction
//!
//! Abstracts LLM interaction (LM Studio, OpenAI, Anthropic, etc.) to enable:
//! - Dependency injection of different LLM providers
//! - Mock-based testing without calling real LLM APIs
//! - Consistent error handling across implementations
//!
//! # Example Usage
//!
//! ```rust,ignore
//! // Production usage with rig-core
//! let llm = RigLlmService::new(&config.llm_studio_url, &config.model_name)?;
//! let (system_prompt, user_prompt) = llm
//!     .generate_prompt(meta_prompt, &context)
//!     .await?;
//!
//! // Testing with mock
//! let mut mock_llm = MockLlmService::new();
//! mock_llm
//!     .expect_generate_prompt()
//!     .returning(|_, _| Ok(("system".into(), "user".into())));
//! ```

use async_trait::async_trait;

use crate::errors::TaleTrailError;
use crate::generated::prompts::{PromptPackage, PromptGenerationRequest};

/// Node context for content generation
#[derive(Debug, Clone)]
pub struct NodeContext {
    /// Previous content leading to this node
    pub previous_content: Option<String>,

    /// Choices made to reach this node
    pub choices_made: Vec<String>,

    /// Current node position in DAG
    pub node_position: usize,

    /// Total nodes in story
    pub total_nodes: usize,
}

/// Abstracts LLM interaction for prompt and content generation
///
/// Implementations:
/// - `RigLlmService`: Production implementation using rig-core
/// - `MockLlmService`: Test mock with predefined responses (via mockall)
/// - `OpenAiLlmService`: Future direct OpenAI API integration
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
#[async_trait]
pub trait LlmService: Send + Sync + std::fmt::Debug {
    /// Generate prompts using meta-prompt (for prompt-helper service)
    ///
    /// Uses LLM to dynamically create system and user prompts based on meta-prompt instructions.
    ///
    /// # Arguments
    /// * `meta_prompt` - Instructions telling LLM how to generate prompts
    /// * `context` - Request context (age group, language, theme, educational goals)
    ///
    /// # Returns
    /// Tuple of `(system_prompt, user_prompt)`
    ///
    /// # Errors
    /// - `TaleTrailError::LLMError`: LLM API communication failure
    /// - `TaleTrailError::TimeoutError`: Request timeout
    async fn generate_prompt(
        &self,
        meta_prompt: &str,
        context: &PromptGenerationRequest,
    ) -> Result<(String, String), TaleTrailError>;

    /// Generate content using prepared prompt package
    ///
    /// # Arguments
    /// * `prompt_package` - Pre-generated prompts with LLM configuration
    /// * `node_context` - Story context (previous content, choices made)
    ///
    /// # Returns
    /// Generated content string
    ///
    /// # Errors
    /// - `TaleTrailError::LLMError`: Content generation failure
    /// - `TaleTrailError::TimeoutError`: Request timeout
    async fn generate_content(
        &self,
        prompt_package: &PromptPackage,
        node_context: &NodeContext,
    ) -> Result<String, TaleTrailError>;

    /// List available LLM models
    ///
    /// # Returns
    /// Vector of model identifiers (e.g., "gpt-4", "claude-3-opus")
    ///
    /// # Errors
    /// - `TaleTrailError::LLMError`: Failed to fetch model list
    async fn list_models(&self) -> Result<Vec<String>, TaleTrailError>;

    /// Check if specific model is available
    ///
    /// # Arguments
    /// * `model_name` - Model identifier to check
    ///
    /// # Returns
    /// `true` if model exists and is available
    ///
    /// # Errors
    /// - `TaleTrailError::LLMError`: Failed to check model availability
    async fn model_exists(&self, model_name: &str) -> Result<bool, TaleTrailError>;
}
