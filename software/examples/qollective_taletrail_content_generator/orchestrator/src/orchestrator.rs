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
    discovery::DiscoveryClient,
    events::{EventPublisher, PipelineEvent},
    mcp_client::McpEnvelopeClient,
    negotiation::Negotiator,
    pipeline::PipelineState,
    prompt_orchestration::PromptOrchestrator,
};
use async_nats::Client as NatsClient;
use qollective::envelope::Meta;
use rmcp::model::{
    CallToolRequest, CallToolRequestMethod, CallToolRequestParam, Extensions,
};
use shared_types::*;
use story_generator::mcp_tools::{GenerateStructureResponse, GenerateNodesResponse};
use quality_control::envelope_handlers::ValidateContentResponse;
use constraint_enforcer::envelope_handlers::EnforceConstraintsResponse;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, instrument};
use uuid::Uuid;

/// Helper function to create TracingMeta with minimal fields
fn create_tracing_meta(trace_id: String) -> qollective::envelope::meta::TracingMeta {
    qollective::envelope::meta::TracingMeta {
        trace_id: Some(trace_id),
        span_id: None,
        parent_span_id: None,
        baggage: std::collections::HashMap::new(),
        sampling_rate: None,
        sampled: Some(true),
        trace_state: None,
        operation_name: None,
        span_kind: None,
        span_status: None,
        tags: std::collections::HashMap::new(),
    }
}

/// Main orchestrator coordinating all MCP services
pub struct Orchestrator {
    /// NATS client for MCP communication
    nats_client: Arc<NatsClient>,

    /// MCP envelope client for envelope-first communication
    mcp_client: McpEnvelopeClient,

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

    /// Discovery client for service health checks
    discovery_client: DiscoveryClient,
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
    pub async fn new(nats_client: Arc<NatsClient>, config: OrchestratorConfig) -> Result<Self> {
        let event_publisher = EventPublisher::new(
            nats_client.as_ref().clone(),
            constants::MCP_EVENTS_PREFIX.to_string(),
        );

        let prompt_orchestrator = PromptOrchestrator::new(nats_client.clone(), &config);

        let negotiator = Negotiator::new(&config);

        // Create MCP envelope client with timeout from config
        let mcp_client = McpEnvelopeClient::new(
            nats_client.clone(),
            config.pipeline.generation_timeout_secs,
        );

        // Create discovery client
        let discovery_client = DiscoveryClient::new(nats_client.clone());

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

        let orchestrator = Self {
            nats_client,
            mcp_client,
            config,
            state,
            event_publisher,
            prompt_orchestrator,
            negotiator,
            discovery_client,
        };

        // Run pre-flight check
        orchestrator.pre_flight_check().await?;

        Ok(orchestrator)
    }

    /// Run pre-flight discovery check
    ///
    /// Discovers all required MCP services and validates they expose the required tools
    /// before starting orchestration. This prevents runtime failures due to missing services.
    ///
    /// # Errors
    ///
    /// Returns TaleTrailError if:
    /// - Required service is not discoverable
    /// - Required tool is missing from a service
    async fn pre_flight_check(&self) -> Result<()> {
        info!("Running pre-flight discovery check...");

        let all_services = self.discovery_client.discover_all_services().await?;

        // Define required tools per service
        let required_tools = vec![
            ("story-generator", vec!["generate_structure", "generate_nodes"]),
            ("quality-control", vec!["validate_content"]),
            ("constraint-enforcer", vec!["enforce_constraints"]),
            ("prompt-helper", vec!["generate_story_prompts"]),
        ];

        for (service_name, required) in required_tools {
            let tools = all_services.get(service_name).ok_or_else(|| {
                TaleTrailError::DiscoveryError(format!(
                    "Required service not found: {}",
                    service_name
                ))
            })?;

            for required_tool in required {
                if !tools.iter().any(|t| t.tool_name == required_tool) {
                    return Err(TaleTrailError::DiscoveryError(format!(
                        "Missing required tool {} from service {}",
                        required_tool, service_name
                    )));
                }
            }

            info!(
                "✅ Discovered {} with {} tools: {:?}",
                service_name,
                tools.len(),
                tools.iter().map(|t| &t.tool_name).collect::<Vec<_>>()
            );
        }

        // Log warnings for optional tools
        if let Some(qc_tools) = all_services.get("quality-control") {
            if !qc_tools.iter().any(|t| t.tool_name == "batch_validate") {
                tracing::warn!("Optional tool batch_validate not available from quality-control");
            }
        }

        info!("✅ Pre-flight discovery check passed");
        Ok(())
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
            .phase_generate_content(dag, &request, &prompts, &request_id)
            .await?;

        // Phase 3-4: Validate and Negotiate (iterative)
        dag = self
            .phase_validate_and_negotiate(dag, &request, &prompts, &request_id)
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

        // Create updated request with prompt packages following envelope-first architecture
        let mut updated_request = request.clone();
        updated_request.prompt_packages = self.convert_prompts_to_request_format(prompts)?;

        // Create metadata for this request
        let meta = self.create_meta(request, request_id);

        // Call story-generator MCP tool: generate_structure
        let tool_request = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "generate_structure".into(),
                arguments: Some({
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "generation_request".to_string(),
                        serde_json::to_value(&updated_request)
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    map
                }),
            },
            extensions: Extensions::default(),
        };

        let response: GenerateStructureResponse = self
            .call_mcp_tool(&constants::MCP_STORY_GENERATE, tool_request, meta)
            .await?;
        let dag = response.dag;

        // Publish event
        self.event_publisher
            .publish_event(PipelineEvent::StructureCreated {
                request_id: request_id.to_string(),
                node_count: dag.nodes.len(),
                convergence_points: response.convergence_point_count,
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
    #[instrument(skip(self, dag, prompts, request))]
    async fn phase_generate_content(
        &self,
        mut dag: DAG,
        request: &GenerationRequest,
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

            // Create updated request with prompt packages following envelope-first architecture
            let mut updated_request = request.clone();
            updated_request.prompt_packages = self.convert_prompts_to_request_format(prompts)?;

            // Create metadata for this batch
            let meta = self.create_meta(request, request_id);

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
                            "generation_request".to_string(),
                            serde_json::to_value(&updated_request)
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                        map
                    }),
                },
                extensions: Extensions::default(),
            };

            let response: GenerateNodesResponse = self
                .call_mcp_tool(&constants::MCP_STORY_GENERATE, tool_request, meta)
                .await?;
            let generated_nodes = response.nodes;

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
    #[instrument(skip(self, dag, prompts, request))]
    async fn phase_validate_and_negotiate(
        &self,
        dag: DAG,
        request: &GenerationRequest,
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
            let quality_result = self.validate_quality(node, request, prompts, request_id).await?;

            // Call constraint-enforcer
            let constraint_result = self
                .validate_constraints(node, request, prompts, request_id)
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
        request: &GenerationRequest,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        request_id: &str,
    ) -> Result<ValidationResult> {
        let validation_prompts = prompts.get(&MCPServiceType::QualityControl);

        // Create updated request with prompt packages following envelope-first architecture
        let mut updated_request = request.clone();
        updated_request.prompt_packages = self.convert_prompts_to_request_format(prompts)?;

        // Create metadata for this validation
        let meta = self.create_meta(request, request_id);

        let tool_request = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "validate_content".into(),
                arguments: Some({
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "content_node".to_string(),
                        serde_json::to_value(node)
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    map.insert(
                        "age_group".to_string(),
                        serde_json::to_value(&request.age_group)
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    map.insert(
                        "educational_goals".to_string(),
                        serde_json::to_value(&request.educational_goals.clone().unwrap_or_default())
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    map
                }),
            },
            extensions: Extensions::default(),
        };

        let response: ValidateContentResponse = self
            .call_mcp_tool(&constants::MCP_QUALITY_VALIDATE, tool_request, meta)
            .await?;
        Ok(response.validation_result)
    }

    /// Validate node constraints
    ///
    /// Calls constraint-enforcer MCP service to check content constraints.
    async fn validate_constraints(
        &self,
        node: &ContentNode,
        request: &GenerationRequest,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        request_id: &str,
    ) -> Result<ConstraintResult> {
        let constraint_prompts = prompts.get(&MCPServiceType::ConstraintEnforcer);

        // Create updated request with prompt packages following envelope-first architecture
        let mut updated_request = request.clone();
        updated_request.prompt_packages = self.convert_prompts_to_request_format(prompts)?;

        // Create metadata for this validation
        let meta = self.create_meta(request, request_id);

        let tool_request = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "enforce_constraints".into(),
                arguments: Some({
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "content_node".to_string(),
                        serde_json::to_value(node)
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    map.insert(
                        "generation_request".to_string(),
                        serde_json::to_value(&updated_request)
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    map
                }),
            },
            extensions: Extensions::default(),
        };

        let response: EnforceConstraintsResponse = self
            .call_mcp_tool(&constants::MCP_CONSTRAINT_ENFORCE, tool_request, meta)
            .await?;
        Ok(response.constraint_result)
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

    /// Convert prompt packages from internal format to GenerationRequest format
    ///
    /// Transforms HashMap<MCPServiceType, PromptPackage> into the nested Option
    /// structure required by GenerationRequest.prompt_packages field.
    ///
    /// # Arguments
    ///
    /// * `prompts` - HashMap of service types to prompt packages from Phase 0.5
    ///
    /// # Returns
    ///
    /// Option<Option<HashMap<String, serde_json::Value>>> for GenerationRequest
    ///
    /// # Errors
    ///
    /// Returns SerializationError if prompt package cannot be serialized
    fn convert_prompts_to_request_format(
        &self,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
    ) -> Result<Option<Option<HashMap<String, serde_json::Value>>>> {
        if prompts.is_empty() {
            return Ok(None);
        }

        let mut prompt_map = HashMap::new();

        for (service_type, prompt_package) in prompts {
            // Convert enum to string key
            let key = match service_type {
                MCPServiceType::StoryGenerator => "story_generator",
                MCPServiceType::QualityControl => "quality_control",
                MCPServiceType::ConstraintEnforcer => "constraint_enforcer",
                MCPServiceType::PromptHelper => "prompt_helper",
                MCPServiceType::Orchestrator => "orchestrator",
            };

            // Serialize PromptPackage to JSON Value
            let value = serde_json::to_value(prompt_package)
                .map_err(|e| TaleTrailError::SerializationError(
                    format!("Failed to serialize prompt package for {}: {}", key, e)
                ))?;

            prompt_map.insert(key.to_string(), value);
        }

        Ok(Some(Some(prompt_map)))
    }

    /// Create metadata for MCP requests
    ///
    /// Constructs envelope metadata with tenant_id, request_id, and trace_id
    /// from the orchestrator context.
    ///
    /// # Arguments
    ///
    /// * `request` - Generation request containing tenant_id
    /// * `request_id` - Request ID for correlation
    ///
    /// # Returns
    ///
    /// Meta struct populated with tenant and tracing information
    fn create_meta(&self, request: &GenerationRequest, request_id: &str) -> Meta {
        let mut meta = Meta::default();

        // Set tenant ID for multi-tenancy isolation
        meta.tenant = Some(format!("tenant-{}", request.tenant_id));

        // Set request ID for correlation
        if let Ok(uuid) = Uuid::parse_str(request_id) {
            meta.request_id = Some(uuid);
        }

        // Set trace ID for distributed tracing (same as request_id for now)
        meta.tracing = Some(create_tracing_meta(request_id.to_string()));

        meta
    }

    /// Generic MCP tool call helper with envelope wrapping
    ///
    /// Wraps request in Envelope<McpData>, sends via NATS, unwraps response.
    /// This method now uses the McpEnvelopeClient for envelope-first communication.
    ///
    /// # Arguments
    ///
    /// * `subject` - NATS subject for the MCP service
    /// * `request` - MCP tool call request
    /// * `meta` - Envelope metadata (tenant_id, request_id, trace_id)
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
        meta: Meta,
    ) -> Result<T> {
        self.mcp_client.call_tool(subject, request, meta).await
    }
}
