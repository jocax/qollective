//! LLM service implementation for story content generation using shared-types-llm
//!
//! This module provides LLM-based content generation for narrative nodes using shared-types-llm
//! for dynamic multi-provider support.

use shared_types::{
    constants::CONCURRENT_BATCHES,
    traits::llm_service::NodeContext,
    Choice, Content, ContentNode, DAG, GenerationRequest, PromptPackage,
    TaleTrailError,
};
use shared_types_llm::{
    DefaultDynamicLlmClientProvider,
    DynamicLlmClientProvider,
    LlmConfig,
    LlmParameters,
    SystemPromptStyle,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, info_span, warn, Instrument};

/// Story LLM client for content generation using shared-types-llm
#[derive(Clone)]
pub struct StoryLlmClient {
    provider: Arc<DefaultDynamicLlmClientProvider>,
}

impl std::fmt::Debug for StoryLlmClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoryLlmClient").finish()
    }
}

impl StoryLlmClient {
    /// Create new LLM client from configuration
    pub fn new(config: LlmConfig) -> Result<Self, TaleTrailError> {
        let provider = DefaultDynamicLlmClientProvider::new(config);

        Ok(Self {
            provider: Arc::new(provider),
        })
    }

    /// Generate content using prompt package and node context
    pub async fn generate_with_prompts(
        &self,
        prompt_package: &PromptPackage,
        node_context: &NodeContext,
        context: Option<shared_types_llm::RequestContext>,
    ) -> Result<String, TaleTrailError> {
        let span = info_span!(
            "story_content_generation",
            node_position = node_context.node_position,
            total_nodes = node_context.total_nodes,
        );

        async move {
            let start_time = std::time::Instant::now();

            info!(
                "Generating story content at node {}/{}",
                node_context.node_position, node_context.total_nodes
            );

            // Build content generation prompt
            let full_prompt = Self::build_content_prompt(prompt_package, node_context);

            debug!("Sending content prompt to LLM (length: {} chars)", full_prompt.len());

            // Get language code for model selection
            let language_code = match prompt_package.language {
                shared_types::Language::De => "de",
                shared_types::Language::En => "en",
            };

            // Create LLM parameters
            let params = LlmParameters {
                language_code: language_code.to_string(),
                model_name: None,
                system_prompt_style: SystemPromptStyle::ChatML,
                tenant_id: None,
                tenant_config: None,
            };

            // Get dynamic client
            let client = self.provider.get_dynamic_llm_client(&params).await
                .map_err(|e| {
                    error!("Failed to get LLM client: {}", e);
                    TaleTrailError::LLMError(format!("Failed to get LLM client: {}", e))
                })?;

            info!("Using model: {}", client.model_name());

            // Send prompt and get response (pass context)
            let content = client.prompt(&full_prompt, context).await
                .map_err(|e| {
                    error!("LLM content generation failed: {}", e);
                    TaleTrailError::LLMError(format!("Content generation failed: {}", e))
                })?;

            if content.trim().is_empty() {
                warn!("LLM returned empty content");
                return Err(TaleTrailError::GenerationError(
                    "LLM returned empty content".to_string(),
                ));
            }

            let duration_ms = start_time.elapsed().as_millis();
            info!(
                "Story content generated in {}ms (length: {} chars)",
                duration_ms,
                content.len()
            );

            Ok(content)
        }
        .instrument(span)
        .await
    }

    /// Build content generation prompt with node context
    pub fn build_content_prompt(prompt_package: &PromptPackage, node_context: &NodeContext) -> String {
        let mut prompt = format!(
            "System: {}\n\nUser: {}\n\n",
            prompt_package.system_prompt, prompt_package.user_prompt
        );

        // Add node context
        prompt.push_str(&format!(
            "Context:\n- Node position: {}/{}\n",
            node_context.node_position, node_context.total_nodes
        ));

        if let Some(ref prev_content) = node_context.previous_content {
            prompt.push_str(&format!("- Previous content: {}\n", prev_content));
        }

        if !node_context.choices_made.is_empty() {
            prompt.push_str(&format!(
                "- Choices made: {}\n",
                node_context.choices_made.join(" → ")
            ));
        }

        prompt.push_str("\nIMPORTANT: Format your response using Markdown sections:\n\n");
        prompt.push_str("## Narrative\n");
        prompt.push_str("[Your story text here, ~400 words]\n\n");
        prompt.push_str("## Choices\n");
        prompt.push_str("1. [First choice, ~20 words]\n");
        prompt.push_str("2. [Second choice, ~20 words]\n");
        prompt.push_str("3. [Third choice, ~20 words]\n\n");
        prompt.push_str("## Educational Content\n");
        prompt.push_str("[Optional educational information]\n");

        prompt
    }

    /// Parse LLM response to extract narrative, choices, and educational content
    ///
    /// Expected Markdown format:
    /// ```markdown
    /// ## Narrative
    /// [Story text]
    ///
    /// ## Choices
    /// 1. [Choice 1]
    /// 2. [Choice 2]
    /// 3. [Choice 3]
    ///
    /// ## Educational Content
    /// [Optional content]
    /// ```
    pub fn parse_content_response(
        response: &str,
    ) -> Result<(String, Vec<String>, Option<String>), TaleTrailError> {
        use shared_types::MarkdownResponseParser;

        // Extract Narrative section (required)
        let narrative = MarkdownResponseParser::extract_section(response, "Narrative")
            .ok_or_else(|| {
                error!("LLM response missing '## Narrative' section");
                TaleTrailError::LLMError("Missing '## Narrative' section in Markdown response".to_string())
            })?;

        if narrative.trim().is_empty() {
            return Err(TaleTrailError::LLMError("Narrative section is empty".to_string()));
        }

        // Extract Choices section (required)
        let choices_section = MarkdownResponseParser::extract_section(response, "Choices")
            .ok_or_else(|| {
                error!("LLM response missing '## Choices' section");
                TaleTrailError::LLMError("Missing '## Choices' section in Markdown response".to_string())
            })?;

        let mut choices = MarkdownResponseParser::extract_numbered_list(&choices_section);

        // Validate we have at least one choice
        if choices.is_empty() {
            warn!("No numbered choices found in Choices section, attempting line-by-line parse");
            // Fallback: split by lines and take first 3 non-empty
            choices = choices_section
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && l.len() > 5)
                .take(3)
                .map(|s| s.to_string())
                .collect();
        }

        // Ensure exactly 3 choices (pad or truncate)
        while choices.len() < 3 {
            choices.push(format!("Choice {}", choices.len() + 1));
        }
        choices.truncate(3);

        // Extract Educational Content section (optional)
        let educational = MarkdownResponseParser::extract_section(response, "Educational Content");

        debug!(
            "Parsed Markdown response: narrative={} chars, choices={}, educational={}",
            narrative.len(),
            choices.len(),
            educational.is_some()
        );

        Ok((narrative, choices, educational))
    }

    /// Extract individual choices from choice section text
    pub fn extract_choices(text: &str) -> Vec<String> {
        let lines: Vec<&str> = text.lines().collect();
        let mut choices = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Remove numbering/bullets (e.g., "1. ", "- ", "• ")
            let cleaned = trimmed
                .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == '-' || c == '•')
                .trim();

            if !cleaned.is_empty() && cleaned.len() > 5 {
                // Reasonable choice length
                choices.push(cleaned.to_string());
            }
        }

        // Ensure we have exactly 3 choices (pad or truncate)
        while choices.len() < 3 {
            choices.push(format!("Choice {}", choices.len() + 1));
        }
        choices.truncate(3);

        choices
    }
}

/// Generate content for a single node
pub async fn generate_node_content(
    llm_client: &StoryLlmClient,
    node_id: &str,
    node_context: NodeContext,
    prompt_package: &PromptPackage,
    _generation_request: &GenerationRequest,
    dag: &DAG,
) -> Result<ContentNode, TaleTrailError> {
    debug!("Generating content for node: {}", node_id);

    // Create request context with node_id for debug dumps
    let context = shared_types_llm::RequestContext::new()
        .with_metadata("node_id", node_id);

    // Call LLM to generate content (pass context)
    let llm_response = llm_client
        .generate_with_prompts(prompt_package, &node_context, Some(context))
        .await?;

    // Parse LLM response
    let (narrative, choice_texts, educational_text) =
        StoryLlmClient::parse_content_response(&llm_response)?;

    // Log successful parsing
    info!(
        "Parsed LLM response for node {}: narrative={} chars, choices={}, educational={}",
        node_id,
        narrative.len(),
        choice_texts.len(),
        educational_text.is_some()
    );

    // Validate narrative is not empty after parsing
    if narrative.trim().is_empty() {
        warn!(
            "LLM returned empty narrative after parsing for node {}",
            node_id
        );
        return Err(TaleTrailError::GenerationError(
            format!("Parsed narrative is empty for node {}", node_id),
        ));
    }

    // Validate we have at least one choice
    if choice_texts.is_empty() {
        warn!(
            "LLM returned no choices after parsing for node {}",
            node_id
        );
        return Err(TaleTrailError::GenerationError(
            format!("No choices parsed for node {}", node_id),
        ));
    }

    // Get DAG node to retrieve structural information
    let dag_node = dag.nodes.get(node_id).ok_or_else(|| {
        TaleTrailError::ValidationError(format!("Node {} not found in DAG", node_id))
    })?;

    // Build Choice objects from parsed text
    let choices: Vec<Choice> = choice_texts
        .iter()
        .enumerate()
        .map(|(idx, text)| Choice {
            id: format!("choice_{}_{}", node_id, idx),
            text: text.clone(),
            next_node_id: dag_node
                .content
                .next_nodes
                .get(idx)
                .cloned()
                .unwrap_or_default(),
            metadata: None,
        })
        .collect();

    // Build educational content if present
    let educational_content = educational_text.map(|text| shared_types::EducationalContent {
        topic: Some("Story Context".to_string()),
        learning_objective: Some(text.clone()),
        educational_facts: Some(vec![text.clone()]),
        vocabulary_words: None,
    });

    // Build Content structure
    let content = Content {
        r#type: "interactive_story_node".to_string(),
        node_id: node_id.to_string(),
        text: narrative,
        choices,
        convergence_point: dag_node.content.convergence_point,
        next_nodes: dag_node.content.next_nodes.clone(),
        educational_content,
    };

    // Build generation metadata
    let mut metadata = HashMap::new();
    metadata.insert(
        "generated_at".to_string(),
        serde_json::json!(chrono::Utc::now().to_rfc3339()),
    );
    metadata.insert(
        "model".to_string(),
        serde_json::json!(prompt_package.llm_model),
    );
    metadata.insert(
        "node_position".to_string(),
        serde_json::json!(node_context.node_position),
    );
    metadata.insert(
        "total_nodes".to_string(),
        serde_json::json!(node_context.total_nodes),
    );

    // Build ContentNode
    let content_node = ContentNode {
        id: node_id.to_string(),
        content,
        incoming_edges: dag_node.incoming_edges,
        outgoing_edges: dag_node.outgoing_edges,
        generation_metadata: Some(metadata),
    };

    debug!(
        "Successfully generated content for node {} (narrative: {} chars, choices: {})",
        node_id,
        content_node.content.text.len(),
        content_node.content.choices.len()
    );

    Ok(content_node)
}

/// Generate content for multiple nodes in parallel batches
pub async fn generate_nodes_batch(
    llm_client: &StoryLlmClient,
    node_ids: Vec<String>,
    dag: &DAG,
    prompt_package: &PromptPackage,
    generation_request: &GenerationRequest,
    expected_choice_counts: Option<&Vec<usize>>,
) -> Result<Vec<ContentNode>, TaleTrailError> {
    info!(
        "Starting batch generation for {} nodes with concurrency {}",
        node_ids.len(),
        CONCURRENT_BATCHES
    );

    let mut generated_nodes = Vec::new();
    let mut errors = Vec::new();

    // Process nodes in concurrent batches
    let mut tasks = Vec::new();

    for node_id in node_ids {
        // Build node context from DAG
        let node_context = build_node_context(&node_id, dag)?;

        // Clone necessary data for the task
        let llm_client_clone = llm_client.clone();
        let node_id_clone = node_id.clone();
        let prompt_package_clone = prompt_package.clone();
        let generation_request_clone = generation_request.clone();
        let dag_clone = dag.clone();

        // Spawn task for this node
        let task = tokio::spawn(async move {
            generate_node_content(
                &llm_client_clone,
                &node_id_clone,
                node_context,
                &prompt_package_clone,
                &generation_request_clone,
                &dag_clone,
            )
            .await
        });

        tasks.push((node_id, task));

        // Limit concurrent tasks
        if tasks.len() >= CONCURRENT_BATCHES {
            // Wait for current batch to complete
            for (node_id, task) in tasks.drain(..) {
                match task.await {
                    Ok(Ok(content_node)) => {
                        generated_nodes.push(content_node);
                    }
                    Ok(Err(e)) => {
                        error!("Failed to generate content for node {}: {}", node_id, e);
                        errors.push((node_id, e));
                    }
                    Err(e) => {
                        error!("Task panicked for node {}: {}", node_id, e);
                        errors.push((
                            node_id,
                            TaleTrailError::GenerationError(format!("Task panic: {}", e)),
                        ));
                    }
                }
            }
        }
    }

    // Wait for remaining tasks
    for (node_id, task) in tasks {
        match task.await {
            Ok(Ok(content_node)) => {
                generated_nodes.push(content_node);
            }
            Ok(Err(e)) => {
                error!("Failed to generate content for node {}: {}", node_id, e);
                errors.push((node_id, e));
            }
            Err(e) => {
                error!("Task panicked for node {}: {}", node_id, e);
                errors.push((
                    node_id,
                    TaleTrailError::GenerationError(format!("Task panic: {}", e)),
                ));
            }
        }
    }

    info!(
        "Batch generation completed: {} successful, {} failed",
        generated_nodes.len(),
        errors.len()
    );

    if generated_nodes.is_empty() && !errors.is_empty() {
        return Err(TaleTrailError::GenerationError(format!(
            "All {} node generations failed",
            errors.len()
        )));
    }

    // Validate and correct choice counts if constraints were provided
    if let Some(expected_counts) = expected_choice_counts {
        for (node, &expected_count) in generated_nodes.iter_mut().zip(expected_counts.iter()) {
            let actual_count = node.content.choices.len();

            if actual_count != expected_count {
                warn!(
                    node_id = %node.id,
                    expected = expected_count,
                    actual = actual_count,
                    "AI generated wrong choice count - correcting"
                );

                if actual_count > expected_count {
                    // Truncate excess choices
                    node.content.choices.truncate(expected_count);
                    info!(
                        node_id = %node.id,
                        "Truncated {} excess choices",
                        actual_count - expected_count
                    );
                } else {
                    // AI generated too few choices - this indicates a prompt issue
                    error!(
                        node_id = %node.id,
                        expected = expected_count,
                        actual = actual_count,
                        "AI generated too few choices - this indicates prompt issue"
                    );

                    // Add placeholder choices to meet the constraint
                    while node.content.choices.len() < expected_count {
                        let choice_index = node.content.choices.len();
                        node.content.choices.push(Choice {
                            id: format!("fallback_choice_{}_{}", node.id, choice_index),
                            text: format!("Continue the adventure (option {})", choice_index + 1),
                            next_node_id: String::new(),
                            metadata: None,
                        });
                    }

                    warn!(
                        node_id = %node.id,
                        "Added {} placeholder choices to meet constraint",
                        expected_count - actual_count
                    );
                }
            }
        }
    }

    Ok(generated_nodes)
}

/// Build NodeContext from DAG structure
pub fn build_node_context(node_id: &str, dag: &DAG) -> Result<NodeContext, TaleTrailError> {
    // Verify node exists
    let node = dag.nodes.get(node_id).ok_or_else(|| {
        TaleTrailError::ValidationError(format!("Node {} not found in DAG", node_id))
    })?;

    // Find position in DAG (simplified: parse node ID if it's numeric)
    let node_position = node_id
        .parse::<usize>()
        .unwrap_or(0);

    let total_nodes = dag.nodes.len();

    // Extract previous content from incoming nodes
    let previous_content = if node.incoming_edges > 0 {
        // Find predecessor nodes
        let predecessors = find_predecessor_nodes(node_id, dag);

        if !predecessors.is_empty() {
            // Get content from first predecessor (simplified)
            let pred_id = &predecessors[0];
            dag.nodes
                .get(pred_id)
                .and_then(|n| {
                    if !n.content.text.is_empty() {
                        Some(n.content.text.clone())
                    } else {
                        None
                    }
                })
        } else {
            None
        }
    } else {
        None
    };

    // Extract choices made (simplified: extract from edges)
    let choices_made = vec![]; // TODO: Track actual choices if needed

    Ok(NodeContext {
        previous_content,
        choices_made,
        node_position,
        total_nodes,
    })
}

/// Find predecessor nodes in DAG
fn find_predecessor_nodes(node_id: &str, dag: &DAG) -> Vec<String> {
    let mut predecessors = Vec::new();

    for edge in &dag.edges {
        if edge.to_node_id == node_id {
            predecessors.push(edge.from_node_id.clone());
        }
    }

    predecessors
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{AgeGroup, Language};

    #[test]
    fn test_extract_choices() {
        let text = "1. Go left\n2. Go right\n3. Stay here";
        let choices = StoryLlmClient::extract_choices(text);

        assert_eq!(choices.len(), 3);
        assert_eq!(choices[0], "Go left");
        assert_eq!(choices[1], "Go right");
        assert_eq!(choices[2], "Stay here");
    }

    #[test]
    fn test_extract_choices_with_bullets() {
        let text = "- Choice one\n- Choice two\n- Choice three";
        let choices = StoryLlmClient::extract_choices(text);

        assert_eq!(choices.len(), 3);
        assert_eq!(choices[0], "Choice one");
    }

    #[test]
    fn test_parse_markdown_response() {
        let response = r#"## Narrative
This is the story text

## Choices
1. Choice 1
2. Choice 2
3. Choice 3

## Educational Content
Educational information"#;

        let result = StoryLlmClient::parse_content_response(response);
        assert!(result.is_ok());

        let (narrative, choices, educational) = result.unwrap();
        assert_eq!(narrative, "This is the story text");
        assert_eq!(choices.len(), 3);
        assert!(educational.is_some());
    }

    #[test]
    fn test_build_content_prompt() {
        let prompt_package = PromptPackage {
            system_prompt: "System instructions".to_string(),
            user_prompt: "User template".to_string(),
            language: Language::En,
            llm_model: "test-model".to_string(),
            llm_config: shared_types::LLMConfig {
                temperature: 0.7,
                max_tokens: 500,
                top_p: 1.0,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
                stop_sequences: vec![],
            },
            prompt_metadata: shared_types::PromptMetadata {
                theme_context: "Adventure".to_string(),
                age_group_context: AgeGroup::_9To11,
                language_context: Language::En,
                service_target: shared_types::MCPServiceType::StoryGenerator,
                generation_method: shared_types::PromptGenerationMethod::LLMGenerated,
                template_version: "1.0".to_string(),
                generated_at: "2024-01-01T00:00:00Z".to_string(),
            },
            fallback_used: false,
        };

        let node_context = NodeContext {
            previous_content: Some("Beginning of story".to_string()),
            choices_made: vec!["Choice 1".to_string()],
            node_position: 2,
            total_nodes: 16,
        };

        let prompt = StoryLlmClient::build_content_prompt(&prompt_package, &node_context);

        assert!(prompt.contains("System: System instructions"));
        assert!(prompt.contains("User: User template"));
        assert!(prompt.contains("Node position: 2/16"));
        assert!(prompt.contains("Previous content: Beginning of story"));
        assert!(prompt.contains("Choices made: Choice 1"));
    }
}
