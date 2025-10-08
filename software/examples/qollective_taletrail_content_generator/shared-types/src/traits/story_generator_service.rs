//! Story generator service trait for content creation
//!
//! Abstracts story content generation logic to enable:
//! - DAG structure generation
//! - Single node and batch content generation
//! - Mock-based testing without LLM calls
//!
//! # Example Usage
//!
//! ```rust,ignore
//! // Production story generator
//! let generator = RigStoryGenerator::new(llm_service, config);
//! let dag = generator.generate_structure(&request, &prompt_package).await?;
//!
//! // Testing with mock
//! let mut mock_generator = MockStoryGeneratorService::new();
//! mock_generator
//!     .expect_generate_structure()
//!     .returning(|_, _| Ok(mock_dag_with_16_nodes()));
//! ```

use async_trait::async_trait;

use crate::errors::TaleTrailError;
use crate::{DAG, ContentNode, Content, GenerationRequest, PromptPackage};
use crate::traits::llm_service::NodeContext;

/// Abstracts story content generation logic
///
/// Implementations:
/// - `RigStoryGenerator`: Production implementation using rig-core
/// - `MockStoryGenerator`: Test mock with predefined responses
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
#[async_trait]
pub trait StoryGeneratorService: Send + Sync + std::fmt::Debug {
    /// Generate story structure (DAG without content)
    ///
    /// Creates DAG with:
    /// - Calculated node count based on age group
    /// - Convergence points for story merging
    /// - Valid path connectivity
    /// - Empty content placeholders
    ///
    /// # Arguments
    /// * `request` - Generation parameters (theme, age_group, node_count, etc.)
    /// * `prompt_package` - Pre-generated prompts for structure planning
    ///
    /// # Returns
    /// DAG structure with nodes and edges (content to be filled later)
    ///
    /// # Errors
    /// - `TaleTrailError::GenerationError`: Failed to create valid DAG
    async fn generate_structure(
        &self,
        request: &GenerationRequest,
        prompt_package: &PromptPackage,
    ) -> Result<DAG, TaleTrailError>;

    /// Generate content for single node
    ///
    /// Creates narrative content for one DAG node including:
    /// - Story text (target word count from config)
    /// - Choices leading to next nodes
    /// - Educational content integration
    ///
    /// # Arguments
    /// * `node` - DAG node to populate with content
    /// * `prompt_package` - Pre-generated prompts for content generation
    /// * `context` - Story context (previous content, choices made)
    ///
    /// # Returns
    /// Generated `Content` with story text and choices
    ///
    /// # Errors
    /// - `TaleTrailError::LLMError`: LLM content generation failure
    /// - `TaleTrailError::TimeoutError`: Generation timeout
    async fn generate_node_content(
        &self,
        node: &ContentNode,
        prompt_package: &PromptPackage,
        context: &NodeContext,
    ) -> Result<Content, TaleTrailError>;

    /// Generate batch of nodes in parallel
    ///
    /// Efficiently processes multiple nodes concurrently (typically 4-6 nodes).
    ///
    /// # Arguments
    /// * `nodes` - Batch of DAG nodes to generate content for (owned for mockall compatibility)
    /// * `prompt_package` - Pre-generated prompts for all nodes
    ///
    /// # Returns
    /// Vector of generated Content in same order as input nodes
    ///
    /// # Errors
    /// - `TaleTrailError::GenerationError`: Batch processing failure
    /// - Same errors as `generate_node_content`
    async fn generate_batch(
        &self,
        nodes: Vec<ContentNode>,
        prompt_package: &PromptPackage,
    ) -> Result<Vec<Content>, TaleTrailError>;
}
