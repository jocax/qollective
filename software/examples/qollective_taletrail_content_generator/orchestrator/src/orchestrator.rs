//! Main Orchestration Logic
//!
//! Coordinates all MCP services through the complete pipeline, managing phases,
//! batches, validation, and negotiation.
//!
//! # Architecture
//!
//! The orchestrator is an **MCP client** that coordinates 4 MCP servers:
//! 1. **prompt-helper**: Generate prompts (Phase 0.5)
//! 2. **story-generator**: Generate DAG structure and content (Phases 1-2)
//! 3. **quality-control**: Validate content quality (Phase 3)
//! 4. **constraint-enforcer**: Validate constraints (Phase 3)
//!
//! # Pipeline Flow
//!
//! ```text
//! Phase 0.5: Generate Prompts (parallel)
//!     ↓
//! Phase 1: Generate DAG Structure
//!     ↓
//! Phase 2: Generate Content (batched, parallel)
//!     ↓
//! Phase 3: Validate Batches (quality + constraints)
//!     ↓
//! Phase 4: Negotiation (if failures)
//!     ↓
//! Phase 5: Assemble Final DAG
//! ```

use crate::{
    config::OrchestratorConfig,
    events::{EventPublisher, PipelineEvent},
    negotiation::Negotiator,
    pipeline::PipelineState,
    prompt_orchestration::PromptOrchestrator,
};
use async_nats::Client as NatsClient;
use rmcp::model::{
    CallToolRequest, CallToolRequestMethod, CallToolRequestParam, CallToolResult, Extensions,
    RawContent,
};
use shared_types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, instrument};

/// Main orchestrator coordinating all MCP services
pub struct Orchestrator {
    /// NATS client for MCP communication
    nats_client: Arc<NatsClient>,

    /// Configuration
    config: OrchestratorConfig,

    /// Pipeline state (shared, mutable)
    state: Arc<Mutex<PipelineState>>,

    /// Event publisher for progress monitoring
    event_publisher: EventPublisher,

    /// Prompt orchestrator
    prompt_orchestrator: PromptOrchestrator,

    /// Negotiator for corrections
    negotiator: Negotiator,
}

impl Orchestrator {
    /// Create new orchestrator
    ///
    /// # Arguments
    ///
    /// * `nats_client` - NATS client for MCP communication
    /// * `config` - Orchestrator configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use orchestrator::{Orchestrator, OrchestratorConfig};
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = OrchestratorConfig::load()?;
    /// let nats_client = async_nats::connect(&config.nats.url).await?;
    /// let orchestrator = Orchestrator::new(Arc::new(nats_client), config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(nats_client: Arc<NatsClient>, config: OrchestratorConfig) -> Self {
        let event_publisher = EventPublisher::new(
            (*nats_client).clone(),
            constants::MCP_EVENTS_PREFIX.to_string(),
        );

        let prompt_orchestrator = PromptOrchestrator::new(Arc::clone(&nats_client), &config);

        let negotiator = Negotiator::new(&config);

        // Initial state will be created when orchestrate_generation is called
        // Create a minimal dummy request - will be replaced on first call
        let dummy_request = GenerationRequest {
            theme: String::new(),
            age_group: AgeGroup::_6To8,
            language: Language::En,
            node_count: None,
            tenant_id: 0,
            educational_goals: None,
            vocabulary_level: None,
            required_elements: None,
            tags: None,
            prompt_packages: None,
            author_id: None,
        };
        let state = Arc::new(Mutex::new(PipelineState::new(dummy_request)));

        Self {
            nats_client,
            config,
            state,
            event_publisher,
            prompt_orchestrator,
            negotiator,
        }
    }

    /// Main orchestration method - coordinates complete pipeline
    ///
    /// Executes the full 5-phase pipeline from prompt generation through final assembly.
    ///
    /// # Arguments
    ///
    /// * `request` - Generation request with theme, age group, language, etc.
    ///
    /// # Returns
    ///
    /// Complete generation response with DAG and metadata
    ///
    /// # Errors
    ///
    /// Returns error if any critical phase fails
    #[instrument(skip(self, request))]
    pub async fn orchestrate_generation(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationResponse> {
        info!("Starting content generation pipeline");
        let start_time = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        // Initialize pipeline state
        {
            let mut state = self.state.lock().await;
            *state = PipelineState::new(request.clone());
        }

        // Phase 0.5: Generate Prompts
        let prompts = self
            .phase_generate_prompts(&request, &request_id)
            .await?;

        // Phase 1: Generate DAG Structure
        let mut dag = self
            .phase_generate_structure(&request, &prompts, &request_id)
            .await?;

        // Phase 2: Generate Content (batched)
        dag = self
            .phase_generate_content(dag, &prompts, &request_id)
            .await?;

        // Phase 3-4: Validate and Negotiate (iterative)
        dag = self
            .phase_validate_and_negotiate(dag, &prompts, &request_id)
            .await?;

        // Phase 5: Assemble and Return
        let response = self
            .phase_assemble(dag, &request, &request_id, start_time)
            .await?;

        Ok(response)
    }

    /// Phase 0.5: Generate prompts for all services
    ///
    /// Calls prompt-helper service to generate customized prompts for all
    /// downstream services in parallel.
    #[instrument(skip(self, request))]
    async fn phase_generate_prompts(
        &self,
        request: &GenerationRequest,
        request_id: &str,
    ) -> Result<HashMap<MCPServiceType, PromptPackage>> {
        info!("Phase 0.5: Generating prompts");

        // Update phase
        {
            let mut state = self.state.lock().await;
            state.advance_phase()?;
            state.update_progress(5.0);
        }

        let start = std::time::Instant::now();

        // Generate all prompts in parallel
        let prompts = self
            .prompt_orchestrator
            .generate_all_prompts(request)
            .await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Publish event
        self.event_publisher
            .publish_event(PipelineEvent::PromptsGenerated {
                request_id: request_id.to_string(),
                duration_ms,
                fallback_count: 0, // TODO: Track from prompt_orchestrator
                services: prompts.keys().map(|k| format!("{:?}", k)).collect(),
            })
            .await?;

        // Store in state
        {
            let mut state = self.state.lock().await;
            state.prompt_packages = prompts.clone();
        }

        Ok(prompts)
    }

    /// Phase 1: Generate DAG structure
    ///
    /// Calls story-generator MCP tool to create the DAG structure with all nodes
    /// and their dependencies.
    #[instrument(skip(self, request, prompts))]
    async fn phase_generate_structure(
        &self,
        request: &GenerationRequest,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        request_id: &str,
    ) -> Result<DAG> {
        info!("Phase 1: Generating DAG structure");

        // Update phase
        {
            let mut state = self.state.lock().await;
            state.advance_phase()?;
            state.update_progress(15.0);
        }

        // Get story prompt package
        let story_prompts = prompts
            .get(&MCPServiceType::StoryGenerator)
            .ok_or_else(|| TaleTrailError::GenerationError("Missing story prompts".to_string()))?;

        // Call story-generator MCP tool: generate_structure
        let tool_request = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "generate_structure".into(),
                arguments: Some({
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "request".to_string(),
                        serde_json::to_value(request)
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    map.insert(
                        "prompts".to_string(),
                        serde_json::to_value(story_prompts)
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    map
                }),
            },
            extensions: Extensions::default(),
        };

        let dag: DAG = self
            .call_mcp_tool(&constants::MCP_STORY_GENERATE, tool_request)
            .await?;

        // Publish event
        self.event_publisher
            .publish_event(PipelineEvent::StructureCreated {
                request_id: request_id.to_string(),
                node_count: dag.nodes.len(),
                convergence_points: 0, // TODO: Calculate from DAG
            })
            .await?;

        // Store in state
        {
            let mut state = self.state.lock().await;
            state.dag = Some(dag.clone());
        }

        Ok(dag)
    }

    /// Phase 2: Generate content for all nodes (batched)
    ///
    /// Processes nodes in batches with limited concurrency, calling story-generator
    /// to populate node content.
    #[instrument(skip(self, dag, prompts))]
    async fn phase_generate_content(
        &self,
        mut dag: DAG,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        request_id: &str,
    ) -> Result<DAG> {
        info!("Phase 2: Generating content (batched)");

        // Update phase
        {
            let mut state = self.state.lock().await;
            state.advance_phase()?;
            state.update_progress(25.0);
        }

        let story_prompts = prompts
            .get(&MCPServiceType::StoryGenerator)
            .ok_or_else(|| TaleTrailError::GenerationError("Missing story prompts".to_string()))?;

        // Create batches of nodes
        let batch_size = self.config.batch.size_max;
        let node_ids: Vec<String> = dag.nodes.keys().cloned().collect();
        let batches: Vec<Vec<String>> = node_ids
            .chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let total_batches = batches.len();
        info!(
            "Processing {} batches of {} nodes each",
            total_batches, batch_size
        );

        // Process batches with limited concurrency
        let mut batch_id = 0;
        for batch in batches {
            batch_id += 1;

            // Publish batch started event
            self.event_publisher
                .publish_event(PipelineEvent::BatchStarted {
                    request_id: request_id.to_string(),
                    batch_id,
                    node_count: batch.len(),
                    nodes: batch.clone(),
                })
                .await?;

            let start = std::time::Instant::now();

            // Call story-generator MCP tool: generate_nodes
            let tool_request = CallToolRequest {
                method: CallToolRequestMethod::default(),
                params: CallToolRequestParam {
                    name: "generate_nodes".into(),
                    arguments: Some({
                        let mut map = serde_json::Map::new();
                        map.insert(
                            "dag".to_string(),
                            serde_json::to_value(&dag)
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                        map.insert(
                            "node_ids".to_string(),
                            serde_json::to_value(&batch)
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                        map.insert(
                            "prompts".to_string(),
                            serde_json::to_value(story_prompts)
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                        map
                    }),
                },
                extensions: Extensions::default(),
            };

            let generated_nodes: Vec<ContentNode> = self
                .call_mcp_tool(&constants::MCP_STORY_GENERATE, tool_request)
                .await?;

            // Update DAG with generated content
            for gen_node in generated_nodes {
                dag.nodes.insert(gen_node.id.clone(), gen_node);
            }

            let duration_ms = start.elapsed().as_millis() as u64;

            // Publish batch completed event
            self.event_publisher
                .publish_event(PipelineEvent::BatchCompleted {
                    request_id: request_id.to_string(),
                    batch_id,
                    success: true,
                    duration_ms,
                })
                .await?;

            // Update progress
            let progress = 25.0 + (50.0 * batch_id as f32 / total_batches as f32);
            {
                let mut state = self.state.lock().await;
                state.update_progress(progress);
            }
        }

        Ok(dag)
    }

    /// Phase 3-4: Validate and negotiate corrections
    ///
    /// Validates all nodes through quality-control and constraint-enforcer services.
    /// For now, validation only; negotiation will be implemented in next iteration.
    #[instrument(skip(self, dag, prompts))]
    async fn phase_validate_and_negotiate(
        &self,
        dag: DAG,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        request_id: &str,
    ) -> Result<DAG> {
        info!("Phase 3-4: Validation and negotiation");

        // Update phase to Validation
        {
            let mut state = self.state.lock().await;
            state.advance_phase()?;
            state.update_progress(75.0);
        }

        // For now, just validate without negotiation
        // TODO: Implement full negotiation loop in next iteration

        // Validate all nodes
        for node in dag.nodes.values() {
            // Call quality-control
            let quality_result = self.validate_quality(node, prompts, request_id).await?;

            // Call constraint-enforcer
            let constraint_result = self
                .validate_constraints(node, prompts, request_id)
                .await?;

            // Log validation results
            info!(
                "Node {}: quality_valid={}, constraint_violations={}",
                node.id,
                quality_result.is_valid,
                constraint_result.vocabulary_violations.len()
            );
        }

        Ok(dag)
    }

    /// Validate node quality
    ///
    /// Calls quality-control MCP service to validate content quality.
    async fn validate_quality(
        &self,
        node: &ContentNode,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        _request_id: &str,
    ) -> Result<ValidationResult> {
        let validation_prompts = prompts.get(&MCPServiceType::QualityControl);

        let tool_request = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "validate_content".into(),
                arguments: Some({
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "node".to_string(),
                        serde_json::to_value(node)
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    if let Some(p) = validation_prompts {
                        map.insert(
                            "prompts".to_string(),
                            serde_json::to_value(p)
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                    }
                    map
                }),
            },
            extensions: Extensions::default(),
        };

        self.call_mcp_tool(&constants::MCP_QUALITY_VALIDATE, tool_request)
            .await
    }

    /// Validate node constraints
    ///
    /// Calls constraint-enforcer MCP service to check content constraints.
    async fn validate_constraints(
        &self,
        node: &ContentNode,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        _request_id: &str,
    ) -> Result<ConstraintResult> {
        let constraint_prompts = prompts.get(&MCPServiceType::ConstraintEnforcer);

        let tool_request = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "enforce_constraints".into(),
                arguments: Some({
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "node".to_string(),
                        serde_json::to_value(node)
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    if let Some(p) = constraint_prompts {
                        map.insert(
                            "prompts".to_string(),
                            serde_json::to_value(p)
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                    }
                    map
                }),
            },
            extensions: Extensions::default(),
        };

        self.call_mcp_tool(&constants::MCP_CONSTRAINT_ENFORCE, tool_request)
            .await
    }

    /// Phase 5: Assemble final response
    ///
    /// Creates final generation response with DAG and metadata, publishes
    /// completion event.
    #[instrument(skip(self, dag, request))]
    async fn phase_assemble(
        &self,
        dag: DAG,
        request: &GenerationRequest,
        request_id: &str,
        start_time: std::time::Instant,
    ) -> Result<GenerationResponse> {
        info!("Phase 5: Assembling final response");

        // Update phase
        {
            let mut state = self.state.lock().await;
            state.advance_phase()?;
            state.update_progress(100.0);
        }

        let total_duration_ms = start_time.elapsed().as_millis() as u64;

        // Publish completion event
        self.event_publisher
            .publish_event(PipelineEvent::Complete {
                request_id: request_id.to_string(),
                total_duration_ms,
                total_nodes: dag.nodes.len(),
                total_validations: dag.nodes.len() * 2, // quality + constraints
                negotiation_rounds: 0,                  // TODO: Track from negotiation
            })
            .await?;

        // Create response with Trail from DAG
        // Build metadata map
        let mut metadata = HashMap::new();
        metadata.insert(
            "generation_params".to_string(),
            serde_json::to_value(request)
                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
        );
        metadata.insert(
            "node_count".to_string(),
            serde_json::json!(dag.nodes.len()),
        );
        metadata.insert(
            "start_node_id".to_string(),
            serde_json::json!(dag.start_node_id.clone()),
        );

        let trail = Trail {
            title: request.theme.clone(),
            description: Some(format!(
                "Interactive story for {} age group",
                match request.age_group {
                    AgeGroup::_6To8 => "6-8",
                    AgeGroup::_9To11 => "9-11",
                    AgeGroup::_12To14 => "12-14",
                    AgeGroup::_15To17 => "15-17",
                    AgeGroup::Plus18 => "18+",
                }
            )),
            tags: request.tags.clone(),
            status: TrailStatus::DRAFT,
            is_public: false,
            metadata,
            category: None,
            price_coins: None,
        };

        let response = GenerationResponse {
            request_id: request_id.to_string(),
            status: GenerationStatus::Completed,
            progress_percentage: 100,
            generation_metadata: Some(GenerationMetadata {
                generation_duration_seconds: (total_duration_ms / 1000) as i64,
                total_word_count: 0, // TODO: Calculate from DAG
                generated_at: chrono::Utc::now().to_rfc3339(),
                ai_model_version: self.config.llm.provider.default_model.clone(),
                validation_rounds: 1, // TODO: Track actual negotiation rounds
                orchestrator_version: env!("CARGO_PKG_VERSION").to_string(),
            }),
            prompt_generation_metadata: None, // TODO: Add from prompt orchestration phase
            trail: Some(trail),
            trail_steps: None, // TODO: Convert DAG nodes to trail steps
            errors: None,
        };

        Ok(response)
    }

    /// Generic MCP tool call helper
    ///
    /// Serializes request, sends via NATS with timeout, deserializes response.
    ///
    /// # Arguments
    ///
    /// * `subject` - NATS subject for the MCP service
    /// * `request` - MCP tool call request
    ///
    /// # Type Parameters
    ///
    /// * `T` - Expected response type (must be deserializable)
    ///
    /// # Errors
    ///
    /// - Serialization errors
    /// - NATS timeout
    /// - NATS connection errors
    /// - MCP tool errors
    /// - Response deserialization errors
    async fn call_mcp_tool<T: serde::de::DeserializeOwned>(
        &self,
        subject: &str,
        request: CallToolRequest,
    ) -> Result<T> {
        // Serialize request
        let request_bytes = serde_json::to_vec(&request)
            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?;

        // Send via NATS with timeout
        let timeout_duration =
            std::time::Duration::from_secs(self.config.pipeline.generation_timeout_secs);

        let response_bytes = tokio::time::timeout(
            timeout_duration,
            self.nats_client
                .request(subject.to_string(), request_bytes.into()),
        )
        .await
        .map_err(|_| TaleTrailError::TimeoutError)?
        .map_err(|e| TaleTrailError::NatsError(e.to_string()))?;

        // Deserialize response
        let tool_result: CallToolResult = serde_json::from_slice(&response_bytes.payload)
            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?;

        // Check for errors
        if tool_result.is_error == Some(true) {
            return Err(TaleTrailError::GenerationError(format!(
                "MCP tool error: {:?}",
                tool_result.content
            )));
        }

        // Extract result from content
        let first_content = tool_result
            .content
            .first()
            .ok_or_else(|| TaleTrailError::GenerationError("Empty MCP response".to_string()))?;

        let json_str = match &first_content.raw {
            RawContent::Text(text_content) => &text_content.text,
            _ => {
                return Err(TaleTrailError::GenerationError(
                    "Unexpected content type".to_string(),
                ))
            }
        };

        let result: T = serde_json::from_str(json_str)
            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?;

        Ok(result)
    }
}
