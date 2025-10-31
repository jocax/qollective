//! MCP Tool Handler Implementations
//!
//! This module contains the actual implementation logic for MCP tools.
//! Each handler function processes the tool parameters and returns the result.

use crate::llm::{generate_nodes_batch, StoryLlmClient};
use crate::mcp_tools::{
    GenerateNodesParams, GenerateNodesResponse, GenerateStructureParams,
    GenerateStructureResponse, ValidatePathsParams, ValidatePathsResponse,
};
use crate::structure::{
    calculate_convergence_points, generate_dag_structure, validate_path_connectivity,
    validate_reachability,
};
use shared_types::{constants::DEFAULT_NODE_COUNT, Choice, Content, ContentNode, GenerationRequest, DAG, EducationalContent, TaleTrailError};
use std::collections::HashMap;
use tracing::warn;

// ============================================================================
// Tool Handler Functions
// ============================================================================

/// Handle generate_structure tool call
///
/// Creates a DAG structure with convergence points based on the generation request.
/// Returns the skeleton DAG with empty content nodes.
///
/// # Arguments
///
/// * `params` - Tool parameters containing generation request
///
/// # Returns
///
/// Result containing GenerateStructureResponse with DAG structure or error
pub fn handle_generate_structure(
    params: GenerateStructureParams,
) -> Result<GenerateStructureResponse, TaleTrailError> {
    // Use dag_config from request if provided, otherwise create default config
    let dag_config = if let Some(config) = &params.generation_request.dag_config {
        config.clone()
    } else {
        // Create default config with SingleConvergence pattern
        use shared_types::{ConvergencePattern, DagStructureConfig};

        let node_count = params
            .generation_request
            .node_count
            .unwrap_or(DEFAULT_NODE_COUNT as i64);

        DagStructureConfig {
            node_count,
            convergence_pattern: ConvergencePattern::SingleConvergence,
            convergence_point_ratio: Some(0.5),
            max_depth: 10,
            branching_factor: 2,
        }
    };

    // Generate DAG structure
    let dag = generate_dag_structure(&dag_config)?;
    let convergence_point_count = dag.convergence_points.len();

    // Return response with structure metadata
    Ok(GenerateStructureResponse {
        node_count: dag.nodes.len(),
        edge_count: dag.edges.len(),
        convergence_point_count,
        dag,
    })
}

/// Handle generate_nodes tool call
///
/// Generates narrative content for specified nodes using LLM via rig-core.
/// If LLM generation fails (e.g., LM Studio not running), falls back to template-based content.
///
/// # Arguments
///
/// * `params` - Tool parameters containing DAG, node IDs, generation request, and prompt package
/// * `llm_client` - LLM client for content generation
/// * `prompt_package` - Pre-generated prompt package with system/user prompts and LLM config
///
/// # Returns
///
/// Result containing GenerateNodesResponse with generated content nodes or error
pub async fn handle_generate_nodes(
    params: GenerateNodesParams,
    llm_client: &StoryLlmClient,
    prompt_package: &shared_types::PromptPackage,
    request_delay_ms: u64,
) -> Result<GenerateNodesResponse, TaleTrailError> {
    // Try LLM generation first
    match generate_nodes_batch(
        llm_client,
        params.node_ids.clone(),
        &params.dag,
        prompt_package,
        &params.generation_request,
        params.expected_choice_counts.as_ref(),
        request_delay_ms,
    )
    .await
    {
        Ok(nodes) => {
            Ok(GenerateNodesResponse {
                generated_count: nodes.len(),
                nodes,
            })
        }
        Err(e) => {
            warn!("LLM generation failed, using fallback content: {}", e);
            // Generate fallback content for requested nodes
            let fallback_nodes = generate_fallback_nodes(
                &params.node_ids,
                &params.dag,
                &params.generation_request,
            )?;
            Ok(GenerateNodesResponse {
                generated_count: fallback_nodes.len(),
                nodes: fallback_nodes,
            })
        }
    }
}

/// Handle validate_paths tool call
///
/// Validates DAG path connectivity and reachability.
/// Performs comprehensive validation including:
/// - Connectivity checks (incoming/outgoing edges)
/// - Reachability from start node
/// - Path existence to end node
///
/// # Arguments
///
/// * `params` - Tool parameters containing DAG to validate
///
/// # Returns
///
/// Result containing ValidatePathsResponse with validation results
pub fn handle_validate_paths(
    params: ValidatePathsParams,
) -> Result<ValidatePathsResponse, TaleTrailError> {
    let dag = &params.dag;
    let mut validation_messages = Vec::new();
    let mut is_valid = true;

    // Validate path connectivity
    match validate_path_connectivity(dag) {
        Ok(_) => {
            validation_messages.push("Path connectivity validation passed".to_string());
        }
        Err(e) => {
            is_valid = false;
            validation_messages.push(format!("Path connectivity validation failed: {}", e));
        }
    }

    // Validate reachability
    match validate_reachability(dag) {
        Ok(_) => {
            validation_messages.push("Reachability validation passed".to_string());
        }
        Err(e) => {
            is_valid = false;
            validation_messages.push(format!("Reachability validation failed: {}", e));
        }
    }

    // Add summary message
    if is_valid {
        validation_messages.push(format!(
            "All validation checks passed for DAG with {} nodes and {} edges",
            dag.nodes.len(),
            dag.edges.len()
        ));
    } else {
        validation_messages.push(format!(
            "Validation failed for DAG with {} nodes and {} edges",
            dag.nodes.len(),
            dag.edges.len()
        ));
    }

    Ok(ValidatePathsResponse {
        is_valid,
        validation_messages,
        node_count: dag.nodes.len(),
        edge_count: dag.edges.len(),
    })
}

// ============================================================================
// Fallback Content Generation
// ============================================================================

/// Generate fallback content for multiple nodes when LLM is unavailable
///
/// This function provides graceful degradation when the LLM service is unavailable
/// (e.g., LM Studio not running). It generates simple template-based content that
/// maintains story structure while clearly indicating fallback was used.
///
/// # Arguments
///
/// * `node_ids` - List of node IDs to generate content for
/// * `dag` - DAG structure containing all nodes
/// * `generation_request` - Original generation request with theme, age group, etc.
///
/// # Returns
///
/// Vector of ContentNode with fallback-generated content
///
/// # Errors
///
/// Returns error if node IDs are invalid or DAG structure is corrupted
fn generate_fallback_nodes(
    node_ids: &[String],
    dag: &DAG,
    generation_request: &GenerationRequest,
) -> Result<Vec<ContentNode>, TaleTrailError> {
    let mut nodes = Vec::new();

    for node_id in node_ids {
        let dag_node = dag.nodes.get(node_id).ok_or_else(|| {
            TaleTrailError::ValidationError(format!("Node {} not found in DAG", node_id))
        })?;

        let content_node = generate_fallback_content(node_id, dag_node, generation_request)?;
        nodes.push(content_node);
    }

    Ok(nodes)
}

/// Generate fallback content for a single node
///
/// Creates simple template-based narrative and choices when LLM generation fails.
/// The content is age-appropriate but clearly marked as fallback.
///
/// # Arguments
///
/// * `node_id` - ID of the node to generate content for
/// * `content_node` - Content node structure with metadata
/// * `generation_request` - Generation request with theme and parameters
///
/// # Returns
///
/// ContentNode with fallback-generated narrative and choices
fn generate_fallback_content(
    node_id: &str,
    content_node: &ContentNode,
    generation_request: &GenerationRequest,
) -> Result<ContentNode, TaleTrailError> {
    let theme = &generation_request.theme;
    let age_group = &generation_request.age_group;

    // Generate simple narrative based on theme
    let narrative = format!(
        "In this part of your {} adventure, you find yourself at an important moment. \
        The path ahead offers different possibilities, each leading to new experiences. \
        What will you choose to do next?",
        theme.to_lowercase()
    );

    // Generate choices based on outgoing edges
    let mut choices = Vec::new();
    let outgoing_count = content_node.outgoing_edges;

    for i in 0..outgoing_count.min(3) {
        let choice_text = match i {
            0 => format!("Take the first path in your {} journey", theme.to_lowercase()),
            1 => format!("Choose the second option and see where it leads"),
            2 => format!("Try the third way forward"),
            _ => format!("Option {}", i + 1),
        };

        choices.push(Choice {
            id: format!("fallback_choice_{}_{}", node_id, i),
            text: choice_text,
            next_node_id: String::new(), // Will be filled by orchestrator
            metadata: None,
        });
    }

    // Add educational content hint based on age group
    let age_label = match age_group {
        shared_types::AgeGroup::_6To8 => "6-8",
        shared_types::AgeGroup::_9To11 => "9-11",
        shared_types::AgeGroup::_12To14 => "12-14",
        shared_types::AgeGroup::_15To17 => "15-17",
        shared_types::AgeGroup::Plus18 => "18+",
    };

    let educational_content = Some(EducationalContent {
        topic: Some(format!("{} adventure", theme)),
        learning_objective: Some(format!(
            "This story is designed for readers aged {}. Remember to think carefully about your choices!",
            age_label
        )),
        educational_facts: None,
        vocabulary_words: None,
    });

    let content = Content {
        r#type: "interactive_story_node".to_string(),
        node_id: node_id.to_string(),
        text: narrative,
        choices,
        convergence_point: content_node.content.convergence_point,
        next_nodes: content_node.content.next_nodes.clone(),
        educational_content,
    };

    // Build metadata indicating fallback was used
    let mut metadata = HashMap::new();
    metadata.insert(
        "generated_at".to_string(),
        serde_json::json!(chrono::Utc::now().to_rfc3339()),
    );
    metadata.insert(
        "generation_method".to_string(),
        serde_json::json!("fallback"),
    );
    metadata.insert(
        "fallback_reason".to_string(),
        serde_json::json!("LLM unavailable"),
    );
    metadata.insert(
        "theme".to_string(),
        serde_json::json!(theme),
    );

    Ok(ContentNode {
        id: node_id.to_string(),
        content,
        incoming_edges: content_node.incoming_edges,
        outgoing_edges: content_node.outgoing_edges,
        generation_metadata: Some(metadata),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{AgeGroup, Language};

    const DEFAULT_NODE_COUNT: usize = 16;

    fn create_test_generation_request() -> shared_types::GenerationRequest {
        shared_types::GenerationRequest {
            theme: "Test Adventure".to_string(),
            age_group: AgeGroup::_9To11,
            language: Language::En,
            tenant_id: 1,
            node_count: Some(DEFAULT_NODE_COUNT as i64),
            vocabulary_level: None,
            educational_goals: None,
            required_elements: None,
            tags: None,
            author_id: None,
            prompt_packages: None,
            story_structure: None,
            dag_config: None,
            validation_policy: None,
        }
    }

    #[test]
    fn test_handle_generate_structure_basic() {
        let params = GenerateStructureParams {
            generation_request: create_test_generation_request(),
        };

        let result = handle_generate_structure(params);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.node_count, DEFAULT_NODE_COUNT);
        assert!(response.edge_count > 0);
        assert!(response.convergence_point_count > 0);
        assert_eq!(response.dag.nodes.len(), DEFAULT_NODE_COUNT);
    }

    #[test]
    fn test_handle_generate_structure_custom_node_count() {
        let mut request = create_test_generation_request();
        request.node_count = Some(8);

        let params = GenerateStructureParams {
            generation_request: request,
        };

        let result = handle_generate_structure(params);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.node_count, 8);
    }

    // NOTE: This test is disabled because handle_generate_nodes is now async
    // and requires an LLM client. See tests/generation_tests.rs for comprehensive tests.
    //
    // #[tokio::test]
    // async fn test_handle_generate_nodes_with_llm() {
    //     // This test would require a running LLM client
    // }

    // NOTE: This test is disabled because handle_generate_nodes is now async
    // See tests/generation_tests.rs for error handling tests.
    //
    // #[tokio::test]
    // async fn test_handle_generate_nodes_invalid_node_id() {
    //     // This test would require a running LLM client
    // }

    #[test]
    fn test_handle_validate_paths_valid_dag() {
        // Generate valid structure
        let params = GenerateStructureParams {
            generation_request: create_test_generation_request(),
        };
        let structure_response = handle_generate_structure(params).unwrap();

        // Validate it
        let validate_params = ValidatePathsParams {
            dag: structure_response.dag,
        };

        let result = handle_validate_paths(validate_params);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.is_valid, "Generated DAG should be valid");
        assert!(response.validation_messages.len() >= 3); // Should have multiple validation messages
        assert_eq!(response.node_count, DEFAULT_NODE_COUNT);
    }

    #[test]
    fn test_handle_validate_paths_empty_dag() {
        use shared_types::DAG;
        use std::collections::HashMap;

        // Create empty DAG
        let empty_dag = DAG {
            nodes: HashMap::new(),
            edges: Vec::new(),
            start_node_id: "0".to_string(),
            convergence_points: Vec::new(),
        };

        let validate_params = ValidatePathsParams { dag: empty_dag };

        let result = handle_validate_paths(validate_params);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.is_valid, "Empty DAG should be invalid");
    }

    #[test]
    fn test_integration_generate_and_validate() {
        // Generate structure
        let params = GenerateStructureParams {
            generation_request: create_test_generation_request(),
        };
        let structure_response = handle_generate_structure(params).unwrap();

        // Validate the generated structure
        let validate_params = ValidatePathsParams {
            dag: structure_response.dag.clone(),
        };
        let validate_response = handle_validate_paths(validate_params).unwrap();

        assert!(
            validate_response.is_valid,
            "Generated structure should pass validation"
        );

        // NOTE: Content generation test removed as handle_generate_nodes is now async
        // See tests/generation_tests.rs for comprehensive content generation tests
    }
}
