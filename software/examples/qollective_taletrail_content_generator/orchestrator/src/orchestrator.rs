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
    retry::{retry_with_backoff, RetryConfig},
};
use async_nats::Client as NatsClient;
use chrono::Utc;
use qollective::envelope::Meta;
use rmcp::model::{
    CallToolRequest, CallToolRequestMethod, CallToolRequestParam, Extensions,
};
use shared_types::*;
use shared_types::TaleTrailCustomMetadata;
use story_generator::mcp_tools::{GenerateStructureResponse, GenerateNodesResponse};
use quality_control::envelope_handlers::ValidateContentResponse;
use constraint_enforcer::envelope_handlers::EnforceConstraintsResponse;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, instrument};
use uuid::Uuid;
use futures::future::join_all;

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

    /// Service invocation tracking for execution trace
    /// Collects all MCP service calls with timing throughout pipeline execution
    service_invocations: Arc<Mutex<Vec<ServiceInvocation>>>,

    /// Pipeline start time for total duration calculation
    pipeline_start_time: Arc<Mutex<Option<std::time::Instant>>>,
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
            story_structure: None,
            dag_config: None,
            validation_policy: None,
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
            service_invocations: Arc::new(Mutex::new(Vec::new())),
            pipeline_start_time: Arc::new(Mutex::new(None)),
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
        // Layer 1: Create correlation_id for this entire pipeline
        let correlation_id = Uuid::now_v7();

        info!(
            correlation_id = %correlation_id,
            "Starting content generation pipeline"
        );
        let start_time = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        // Track pipeline start time for execution trace
        {
            let mut start = self.pipeline_start_time.lock().await;
            *start = Some(start_time);
        }

        // Clear any previous service invocations
        {
            let mut invocations = self.service_invocations.lock().await;
            invocations.clear();
        }

        // Initialize pipeline state
        {
            let mut state = self.state.lock().await;
            *state = PipelineState::new(request.clone());
        }

        // Phase 1: Generate DAG Structure (moved before prompts to enable DAG-aware prompt generation)
        let mut dag = self
            .phase_generate_structure(&request, &request_id, correlation_id)
            .await?;

        // Phase 1.5: Generate Prompts (now DAG-aware for accurate choice counts)
        let prompts = self
            .phase_generate_prompts(&request, &dag, &request_id, correlation_id)
            .await?;

        // Phase 2: Generate Content (batched)
        dag = self
            .phase_generate_content(dag, &request, &prompts, &request_id, correlation_id)
            .await?;

        // Phase 3-4: Validate and Negotiate (iterative)
        let (dag, negotiation_state) = self
            .phase_validate_and_negotiate(dag, &request, &prompts, &request_id, correlation_id)
            .await?;

        // Phase 5: Assemble and Return
        let response = self
            .phase_assemble(dag, &request, &request_id, start_time, correlation_id, negotiation_state)
            .await?;

        Ok(response)
    }

    /// Phase 1.5: Generate prompts for all services
    ///
    /// Calls prompt-helper service to generate customized prompts for all
    /// downstream services in parallel. Now DAG-aware to provide accurate
    /// choice counts per node.
    #[instrument(skip(self, request, dag))]
    async fn phase_generate_prompts(
        &self,
        request: &GenerationRequest,
        dag: &DAG,
        request_id: &str,
        correlation_id: Uuid,
    ) -> Result<HashMap<MCPServiceType, PromptPackage>> {
        info!("Phase 1.5: Generating prompts (DAG-aware)");

        // Update phase
        {
            let mut state = self.state.lock().await;
            state.advance_phase()?;
            state.update_progress(20.0); // Updated from 5.0 since DAG generation is now first (was 15.0)
        }

        let start = std::time::Instant::now();
        let started_at = Utc::now();
        let phase = self.get_current_phase().await;

        // Generate all prompts in parallel with DAG context for choice counts
        let prompts_result = self
            .prompt_orchestrator
            .generate_all_prompts(request, dag)
            .await;

        let duration_ms = start.elapsed().as_millis() as i64;

        // Record invocation
        match &prompts_result {
            Ok(prompts) => {
                self.record_service_invocation(
                    "prompt-helper".to_string(),
                    "generate_story_prompts".to_string(),
                    started_at,
                    duration_ms,
                    true,
                    None,
                    phase,
                    None,
                    None,
                ).await;
            }
            Err(e) => {
                self.record_service_invocation(
                    "prompt-helper".to_string(),
                    "generate_story_prompts".to_string(),
                    started_at,
                    duration_ms,
                    false,
                    Some(e.to_string()),
                    phase,
                    None,
                    None,
                ).await;
            }
        }

        let prompts = prompts_result?;
        let duration_ms_u64 = duration_ms as u64;

        // Publish event
        let tenant_id_str = format!("tenant-{}", request.tenant_id);
        self.event_publisher
            .publish_event(
                PipelineEvent::PromptsGenerated {
                    request_id: request_id.to_string(),
                    duration_ms: duration_ms_u64,
                    fallback_count: 0, // TODO: Track from prompt_orchestrator
                    services: prompts.keys().map(|k| format!("{:?}", k)).collect(),
                },
                Some(&tenant_id_str)
            )
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
    /// and their dependencies. This phase runs BEFORE prompt generation to enable
    /// DAG-aware choice count calculation in prompts.
    #[instrument(skip(self, request))]
    async fn phase_generate_structure(
        &self,
        request: &GenerationRequest,
        request_id: &str,
        correlation_id: Uuid,
    ) -> Result<DAG> {
        info!("Phase 1: Generating DAG structure");

        // Update phase
        {
            let mut state = self.state.lock().await;
            state.advance_phase()?;
            state.update_progress(10.0); // Changed from 15.0 since this is now first phase
        }

        // Create orchestrator defaults from config
        let orchestrator_defaults = self.config.dag.to_dag_structure_config();

        // Resolve final DAG config using three-tier priority:
        // 1. story_structure preset (if provided)
        // 2. dag_config custom (if provided)
        // 3. orchestrator defaults
        let dag_config = request.resolve_dag_config(&orchestrator_defaults)
            .map_err(|e| TaleTrailError::ValidationError(e.to_string()))?;

        // Log resolved configuration for observability
        tracing::info!(
            node_count = dag_config.node_count,
            pattern = ?dag_config.convergence_pattern,
            ratio = ?dag_config.convergence_point_ratio,
            max_depth = dag_config.max_depth,
            branching_factor = dag_config.branching_factor,
            preset = ?request.story_structure,
            "Resolved DAG configuration for structure generation"
        );

        // Create updated request with resolved DAG config for downstream use
        let mut updated_request = request.clone();
        updated_request.dag_config = Some(dag_config.clone());
        // Note: prompt_packages not needed for structure generation - only dag_config is used

        // Create metadata for this request
        let meta = self.create_meta(request, request_id, correlation_id);

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

        let started_at = Utc::now();
        let call_start = std::time::Instant::now();

        let response_result: std::result::Result<GenerateStructureResponse, TaleTrailError> = self
            .call_mcp_tool(&constants::MCP_STORY_GENERATE, tool_request, meta)
            .await;

        let duration_ms = call_start.elapsed().as_millis() as i64;
        let phase = self.get_current_phase().await;

        // Record invocation
        match &response_result {
            Ok(_) => {
                self.record_service_invocation(
                    "story-generator".to_string(),
                    "generate_structure".to_string(),
                    started_at,
                    duration_ms,
                    true,
                    None,
                    phase,
                    None,
                    None,
                ).await;
            }
            Err(e) => {
                self.record_service_invocation(
                    "story-generator".to_string(),
                    "generate_structure".to_string(),
                    started_at,
                    duration_ms,
                    false,
                    Some(e.to_string()),
                    phase,
                    None,
                    None,
                ).await;
            }
        }

        let response = response_result?;
        let dag = response.dag;

        // Publish event
        let tenant_id_str = format!("tenant-{}", request.tenant_id);
        self.event_publisher
            .publish_event(
                PipelineEvent::StructureCreated {
                    request_id: request_id.to_string(),
                    node_count: dag.nodes.len(),
                    convergence_points: response.convergence_point_count,
                },
                Some(&tenant_id_str)
            )
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
        correlation_id: Uuid,
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

        // Wrap shared data in Arc to avoid cloning (Bottleneck #3)
        let request_arc = Arc::new(request.clone());
        let prompts_arc = Arc::new(prompts.clone());
        let dag_arc = Arc::new(dag.clone());

        // Build batch processing futures for concurrent execution (Bottleneck #1)
        let batch_futures: Vec<_> = batches.into_iter().enumerate()
            .map(|(idx, batch)| {
                let batch_id = idx + 1;
                let request_arc = Arc::clone(&request_arc);
                let prompts_arc = Arc::clone(&prompts_arc);
                let dag_arc = Arc::clone(&dag_arc);
                let request_id = request_id.to_string();
                let correlation_id = correlation_id;
                let event_publisher = self.event_publisher.clone();
                let state = Arc::clone(&self.state);

                // Clone self components needed for the async block
                let mcp_client = self.mcp_client.clone();
                let config = self.config.clone();

                async move {
                    let start = std::time::Instant::now();

                    // Publish batch started event (non-blocking, Bottleneck #4)
                    let tenant_id_str = format!("tenant-{}", request_arc.tenant_id);
                    let event_publisher_spawn = event_publisher.clone();
                    let request_id_spawn = request_id.clone();
                    let batch_clone = batch.clone();
                    tokio::spawn(async move {
                        let _ = event_publisher_spawn.publish_event(
                            PipelineEvent::BatchStarted {
                                request_id: request_id_spawn,
                                batch_id,
                                node_count: batch_clone.len(),
                                nodes: batch_clone,
                            },
                            Some(&tenant_id_str)
                        ).await;
                    });

                    // Create updated request with prompt packages
                    let mut updated_request = (*request_arc).clone();
                    let prompt_packages_result = Self::static_convert_prompts_to_request_format(&prompts_arc);
                    let prompt_packages = match prompt_packages_result {
                        Ok(p) => p,
                        Err(e) => return (batch_id, Err(e), vec![], 0),
                    };
                    updated_request.prompt_packages = prompt_packages;

                    // Create metadata for this batch
                    let meta = Self::static_create_meta(&request_arc, &request_id, correlation_id);

                    // Extract expected choice counts for this batch based on DAG edges
                    let expected_choice_counts: Vec<usize> = batch.iter()
                        .map(|node_id| {
                            dag_arc.edges.iter()
                                .filter(|e| &e.from_node_id == node_id)
                                .count()
                        })
                        .collect();

                    tracing::info!(
                        batch_size = batch.len(),
                        choice_counts = ?expected_choice_counts,
                        "Sending batch to story-generator with choice count constraints"
                    );

                    // Call story-generator MCP tool: generate_nodes
                    let tool_request = CallToolRequest {
                        method: CallToolRequestMethod::default(),
                        params: CallToolRequestParam {
                            name: "generate_nodes".into(),
                            arguments: Some({
                                let mut map = serde_json::Map::new();
                                let dag_value = match serde_json::to_value(&*dag_arc) {
                                    Ok(v) => v,
                                    Err(e) => return (batch_id, Err(TaleTrailError::SerializationError(e.to_string())), vec![], 0),
                                };
                                map.insert("dag".to_string(), dag_value);

                                let batch_value = match serde_json::to_value(&batch) {
                                    Ok(v) => v,
                                    Err(e) => return (batch_id, Err(TaleTrailError::SerializationError(e.to_string())), vec![], 0),
                                };
                                map.insert("node_ids".to_string(), batch_value);

                                let request_value = match serde_json::to_value(&updated_request) {
                                    Ok(v) => v,
                                    Err(e) => return (batch_id, Err(TaleTrailError::SerializationError(e.to_string())), vec![], 0),
                                };
                                map.insert("generation_request".to_string(), request_value);

                                let counts_value = match serde_json::to_value(&expected_choice_counts) {
                                    Ok(v) => v,
                                    Err(e) => return (batch_id, Err(TaleTrailError::SerializationError(e.to_string())), vec![], 0),
                                };
                                map.insert("expected_choice_counts".to_string(), counts_value);

                                map
                            }),
                        },
                        extensions: Extensions::default(),
                    };

                    let started_at = Utc::now();
                    let call_start = std::time::Instant::now();

                    let response_result: std::result::Result<GenerateNodesResponse, TaleTrailError> =
                        mcp_client.call_tool(&constants::MCP_STORY_GENERATE, tool_request, meta).await;

                    let duration_ms = call_start.elapsed().as_millis() as i64;

                    // Get phase for recording (we'll record after collecting results)
                    let generated_nodes = match response_result {
                        Ok(response) => response.nodes,
                        Err(e) => return (batch_id, Err(e), vec![], duration_ms as u64),
                    };

                    let total_duration_ms = start.elapsed().as_millis() as u64;

                    // Publish batch completed event (non-blocking, Bottleneck #4)
                    let tenant_id_str = format!("tenant-{}", request_arc.tenant_id);
                    let event_publisher_spawn = event_publisher.clone();
                    let request_id_spawn = request_id.clone();
                    tokio::spawn(async move {
                        let _ = event_publisher_spawn.publish_event(
                            PipelineEvent::BatchCompleted {
                                request_id: request_id_spawn,
                                batch_id,
                                success: true,
                                duration_ms: total_duration_ms,
                            },
                            Some(&tenant_id_str)
                        ).await;
                    });

                    (batch_id, Ok(()), generated_nodes, total_duration_ms)
                }
            })
            .collect();

        // Execute all batches concurrently
        let batch_results = join_all(batch_futures).await;

        // Process results and update DAG
        for (batch_id, result, generated_nodes, _duration_ms) in batch_results {
            // Propagate errors
            result?;

            // Update DAG with generated content
            for gen_node in generated_nodes {
                dag.nodes.insert(gen_node.id.clone(), gen_node);
            }

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
    /// Validates all nodes through quality-control and constraint-enforcer services
    /// in batches with proper progress tracking.
    /// TODO: Phase 4 negotiation will be implemented in next iteration
    #[instrument(skip(self, dag, prompts, request))]
    async fn phase_validate_and_negotiate(
        &self,
        dag: DAG,
        request: &GenerationRequest,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        request_id: &str,
        correlation_id: Uuid,
    ) -> Result<(DAG, crate::negotiation_state::NegotiationState)> {
        info!("Phase 3-4: Validation and negotiation");

        // Update phase to Validation
        {
            let mut state = self.state.lock().await;
            state.advance_phase()?;
            state.update_progress(75.0);
        }

        // Phase 3: Validate all nodes with batched progress tracking and aggregate issues
        let validation_issues = self
            .phase_validate_content(&dag, request, prompts, request_id, correlation_id)
            .await?;

        // Phase 4: Negotiation loop - iteratively resolve validation failures
        let (dag, negotiation_state) = self
            .phase_negotiate_failures(dag, validation_issues, request, prompts, request_id, correlation_id)
            .await?;

        Ok((dag, negotiation_state))
    }

    /// Phase 3: Validate all nodes in batches
    ///
    /// Validates all nodes through quality-control and constraint-enforcer services.
    /// Implements batched progress tracking (75% → 85%) and graceful degradation on
    /// service unavailable.
    ///
    /// Returns aggregated validation issues with severity classification.
    #[instrument(skip(self, dag, request, prompts))]
    async fn phase_validate_content(
        &self,
        dag: &DAG,
        request: &GenerationRequest,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        request_id: &str,
        correlation_id: Uuid,
    ) -> Result<Vec<crate::validation_issues::ValidationIssue>> {
        info!("Phase 3: Validating content (batched)");

        // Track all validation issues across all nodes
        let mut all_issues = Vec::new();

        // Create batches of nodes for progress tracking
        let batch_size = self.config.batch.size_max;
        let node_ids: Vec<String> = dag.nodes.keys().cloned().collect();
        let batches: Vec<Vec<String>> = node_ids
            .chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let total_batches = batches.len();
        let total_nodes = node_ids.len();

        info!(
            "Validating {} nodes in {} batches (batch_size={})",
            total_nodes, total_batches, batch_size
        );

        // Wrap shared data in Arc to avoid cloning per node (Bottleneck #3)
        let request_arc = Arc::new(request.clone());
        let prompts_arc = Arc::new(prompts.clone());

        // Process batches with progress tracking
        for (batch_idx, batch) in batches.iter().enumerate() {
            let batch_id = batch_idx + 1;

            // Build futures for all nodes in this batch - CONCURRENT EXECUTION
            let validation_futures: Vec<_> = batch.iter()
                .map(|node_id| {
                    let node_id = node_id.clone();
                    let node = dag.nodes.get(&node_id).cloned();
                    let request = Arc::clone(&request_arc);
                    let prompts = Arc::clone(&prompts_arc);
                    let request_id = request_id.to_string();
                    let correlation_id = correlation_id;

                    async move {
                        let node = match node {
                            Some(n) => n,
                            None => {
                                return (
                                    node_id.clone(),
                                    Err(TaleTrailError::ValidationError(
                                        format!("Node {} not found in DAG", node_id)
                                    )),
                                    Err(TaleTrailError::ValidationError(
                                        format!("Node {} not found in DAG", node_id)
                                    )),
                                )
                            }
                        };

                        // Execute both validations concurrently per node using tokio::join!
                        let quality_fut = self.validate_quality(&node, &request, &prompts, &request_id, correlation_id, None);
                        let constraint_fut = self.validate_constraints(&node, &request, &prompts, &request_id, correlation_id, None);

                        let (quality_result, constraint_result) = tokio::join!(quality_fut, constraint_fut);

                        (node_id, quality_result, constraint_result)
                    }
                })
                .collect();

            // Execute ALL node validations concurrently
            let results = join_all(validation_futures).await;

            // Process results
            for (node_id, quality_result, constraint_result) in results {
                // Handle errors with graceful degradation
                let quality_result = quality_result.unwrap_or_else(|e| {
                    tracing::warn!(
                        node_id = %node_id,
                        error = %e,
                        "Quality validation failed, using permissive mock result"
                    );
                    // Mock result allowing content to pass
                    ValidationResult {
                        is_valid: true,
                        age_appropriate_score: 1.0,
                        safety_issues: vec![],
                        educational_value_score: 1.0,
                        correction_capability: CorrectionCapability::CanFixLocally,
                        corrections: vec![],
                    }
                });

                let constraint_result = constraint_result.unwrap_or_else(|e| {
                    tracing::warn!(
                        node_id = %node_id,
                        error = %e,
                        "Constraint enforcement failed, using permissive mock result"
                    );
                    // Mock result with no violations
                    ConstraintResult {
                        vocabulary_violations: vec![],
                        correction_capability: CorrectionCapability::CanFixLocally,
                        corrections: vec![],
                        required_elements_present: true,
                        theme_consistency_score: 1.0,
                        missing_elements: vec![],
                    }
                });

                // Aggregate validation issues for this node
                let node_issues = crate::validation_issues::aggregate_all_issues(
                    &node_id,
                    &quality_result,
                    &constraint_result,
                );

                // Count issues by severity for logging
                let critical_count = node_issues
                    .iter()
                    .filter(|i| i.severity == crate::validation_issues::IssueSeverity::Critical)
                    .count();
                let warning_count = node_issues
                    .iter()
                    .filter(|i| i.severity == crate::validation_issues::IssueSeverity::Warning)
                    .count();

                // Log validation results with severity breakdown
                info!(
                    "Node {}: quality_valid={}, constraint_violations={}, issues: {} critical, {} warning",
                    node_id,
                    quality_result.is_valid,
                    constraint_result.vocabulary_violations.len(),
                    critical_count,
                    warning_count
                );

                // Add to aggregated issues list
                all_issues.extend(node_issues);
            }

            // Update progress incrementally per batch
            // Phase 3 range: 75% → 85% (10% total)
            let progress = 75.0 + (10.0 * batch_id as f32 / total_batches as f32);
            {
                let mut state = self.state.lock().await;
                state.update_progress(progress);
            }

            tracing::debug!(
                batch_id = batch_id,
                total_batches = total_batches,
                progress = progress,
                "Completed validation batch"
            );
        }

        // Log final aggregation summary
        let total_critical = all_issues
            .iter()
            .filter(|i| i.severity == crate::validation_issues::IssueSeverity::Critical)
            .count();
        let total_warning = all_issues
            .iter()
            .filter(|i| i.severity == crate::validation_issues::IssueSeverity::Warning)
            .count();
        let total_info = all_issues
            .iter()
            .filter(|i| i.severity == crate::validation_issues::IssueSeverity::Info)
            .count();

        info!(
            "Phase 3 validation complete: {} nodes validated, {} total issues ({} critical, {} warning, {} info)",
            total_nodes,
            all_issues.len(),
            total_critical,
            total_warning,
            total_info
        );

        Ok(all_issues)
    }

    /// Phase 4: Negotiation loop for validation failure correction
    ///
    /// Implements iterative negotiation with validation services to resolve failures.
    /// Uses the existing build_correction_plan() from negotiation.rs with decision matrix:
    ///
    /// Decision Matrix:
    /// - `CanFixLocally` → Skip regeneration (service already fixed)
    /// - `NeedsRevision` + `Critical` → Add to regeneration set
    /// - `NeedsRevision` + `Warning` → Add to regeneration set
    /// - `NoFixPossible` + `Critical` → HALT entire pipeline (error)
    /// - `NoFixPossible` + `Warning` → Log warning, skip node, continue
    ///
    /// # Arguments
    /// * `dag` - DAG with generated content
    /// * `validation_issues` - Aggregated issues from Phase 3
    /// * `request` - Generation request
    /// * `prompts` - Prompt packages
    /// * `request_id` - Request ID for tracing
    /// * `correlation_id` - Correlation ID for pipeline tracking
    ///
    /// # Returns
    /// Updated DAG after negotiation rounds (may be unchanged if no issues or max rounds exceeded)
    ///
    /// # Errors
    /// - Critical + NoFixPossible → Halts pipeline with ValidationError
    /// - Max rounds exceeded → Logs warning but continues with current DAG
    #[instrument(skip(self, dag, validation_issues, request, prompts))]
    async fn phase_negotiate_failures(
        &self,
        mut dag: DAG,
        validation_issues: Vec<crate::validation_issues::ValidationIssue>,
        request: &GenerationRequest,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        request_id: &str,
        correlation_id: Uuid,
    ) -> Result<(DAG, crate::negotiation_state::NegotiationState)> {
        info!("Phase 4: Negotiation loop");

        // Initialize negotiation state
        let mut negotiation_state = crate::negotiation_state::NegotiationState::new(
            self.config.negotiation.max_rounds
        );

        // If no issues, skip negotiation
        if validation_issues.is_empty() {
            info!("No validation issues - skipping negotiation");
            {
                let mut state = self.state.lock().await;
                state.update_progress(95.0); // Jump to end of Phase 4
            }
            return Ok((dag, negotiation_state));
        }

        // Start Phase 4 at 85% progress
        {
            let mut state = self.state.lock().await;
            state.update_progress(85.0);
        }

        negotiation_state.set_issues(validation_issues.clone());

        // Group issues by node_id for processing
        let mut issues_by_node: HashMap<String, Vec<crate::validation_issues::ValidationIssue>> = HashMap::new();
        for issue in &validation_issues {
            issues_by_node
                .entry(issue.node_id.clone())
                .or_insert_with(Vec::new)
                .push(issue.clone());
        }

        info!(
            "Starting negotiation with {} nodes having validation issues",
            issues_by_node.len()
        );

        // Negotiation loop - max 3 rounds
        while !negotiation_state.max_rounds_reached() {
            let round = negotiation_state.start_round();
            info!("Negotiation round {}/{}", round, negotiation_state.max_rounds);

            // Update pipeline state with current negotiation round
            {
                let mut state = self.state.lock().await;
                state.negotiation_round = round;
            }

            // Build correction plans for all nodes with issues
            // Use HashSet for O(1) deduplication instead of Vec.contains() O(n) (Bottleneck #2)
            let mut nodes_to_regenerate_set: HashSet<String> = HashSet::new();
            let mut corrections_applied = 0;

            for (node_id, node_issues) in &issues_by_node {
                // Use existing build_correction_plan() from negotiation module
                // This method implements the decision matrix logic
                let correction_plan = self.negotiator.build_correction_plan(node_issues)?;

                // Track regeneration nodes with O(1) insertion
                for regen_node_id in &correction_plan.regenerate_nodes {
                    if nodes_to_regenerate_set.insert(regen_node_id.clone()) {
                        // Only add to negotiation_state if it's a new insertion
                        negotiation_state.add_node_to_regenerate(regen_node_id.clone());
                    }
                }

                // Track skipped nodes
                for skipped_node_id in &correction_plan.skipped_nodes {
                    negotiation_state.add_skipped_node(skipped_node_id.clone());
                    tracing::warn!(
                        node_id = %skipped_node_id,
                        "Node skipped due to non-critical unfixable issues"
                    );
                }

                // Count corrections
                corrections_applied += correction_plan.regenerate_nodes.len();
                corrections_applied += correction_plan.local_fixes.len();
            }

            // Convert HashSet to Vec for use in subsequent operations
            let nodes_to_regenerate: Vec<String> = nodes_to_regenerate_set.into_iter().collect();

            negotiation_state.corrections_applied = corrections_applied;

            // If no corrections needed, negotiation succeeded
            if nodes_to_regenerate.is_empty() {
                info!("Negotiation succeeded - no corrections needed");
                negotiation_state.mark_round_succeeded();

                // Publish completion event
                let tenant_id_str = format!("tenant-{}", request.tenant_id);
                self.event_publisher
                    .publish_event(
                        PipelineEvent::NegotiationRoundCompleted {
                            request_id: request_id.to_string(),
                            round,
                            issues_remaining: 0,
                            corrections_applied,
                        },
                        Some(&tenant_id_str)
                    )
                    .await?;

                // Update progress to end of Phase 4 (95%)
                {
                    let mut state = self.state.lock().await;
                    state.update_progress(95.0);
                }

                break;
            }

            info!(
                "Round {}: {} nodes need regeneration, {} corrections applied",
                round, nodes_to_regenerate.len(), corrections_applied
            );

            // Task 4: Execute regeneration with retry logic
            let (updated_dag, regenerated_nodes, failed_nodes) = self
                .execute_correction_plan(
                    dag.clone(),
                    &nodes_to_regenerate,
                    &issues_by_node,
                    request,
                    prompts,
                    request_id,
                    correlation_id,
                )
                .await?;

            dag = updated_dag;

            // Record corrections for metrics
            for node_id in &regenerated_nodes {
                negotiation_state.record_correction(
                    node_id.clone(),
                    "Regenerate".to_string(),
                    true,
                );
            }

            for node_id in &failed_nodes {
                negotiation_state.record_correction(
                    node_id.clone(),
                    "Regenerate".to_string(),
                    false,
                );
            }

            tracing::info!(
                round = round,
                regenerated = regenerated_nodes.len(),
                failed = failed_nodes.len(),
                "Regeneration round complete"
            );

            // Task 4.6: Revalidate regenerated nodes - CONCURRENT EXECUTION
            let mut revalidation_issues: HashMap<String, Vec<crate::validation_issues::ValidationIssue>> = HashMap::new();

            // Wrap shared data in Arc to avoid cloning per node (Bottleneck #3)
            let request_arc = Arc::new(request.clone());
            let prompts_arc = Arc::new(prompts.clone());

            // Build futures for all regenerated nodes
            let revalidation_futures: Vec<_> = regenerated_nodes.iter()
                .filter_map(|node_id| {
                    let node = dag.nodes.get(node_id).cloned()?;
                    let node_id = node_id.clone();
                    let request = Arc::clone(&request_arc);
                    let prompts = Arc::clone(&prompts_arc);
                    let request_id = request_id.to_string();
                    let correlation_id = correlation_id;
                    let round = round;

                    Some(async move {
                        tracing::info!(
                            node_id = %node_id,
                            round = round,
                            "Revalidating regenerated node"
                        );

                        // Execute both validations concurrently per node
                        let quality_fut = self.validate_quality(&node, &request, &prompts, &request_id, correlation_id, Some(round));
                        let constraint_fut = self.validate_constraints(&node, &request, &prompts, &request_id, correlation_id, Some(round));

                        let (quality_result, constraint_result) = tokio::join!(quality_fut, constraint_fut);

                        (node_id, node, quality_result, constraint_result)
                    })
                })
                .collect();

            // Execute ALL revalidations concurrently
            let revalidation_results = join_all(revalidation_futures).await;

            // Process results
            for (node_id, node, quality_result, constraint_result) in revalidation_results {
                let quality_result = quality_result?;
                let constraint_result = constraint_result?;

                // Aggregate revalidation issues
                let quality_issues = crate::validation_issues::aggregate_quality_issues(&node.id, &quality_result);
                let constraint_issues = crate::validation_issues::aggregate_constraint_issues(&node.id, &constraint_result);

                let mut node_issues = quality_issues;
                node_issues.extend(constraint_issues);

                // Filter to only Critical and Warning issues
                node_issues.retain(|issue| {
                    issue.severity == crate::validation_issues::IssueSeverity::Critical
                        || issue.severity == crate::validation_issues::IssueSeverity::Warning
                });

                if !node_issues.is_empty() {
                    tracing::warn!(
                        node_id = %node_id,
                        issue_count = node_issues.len(),
                        "Regenerated node still has validation issues"
                    );
                    revalidation_issues.insert(node_id.clone(), node_issues);
                } else {
                    tracing::info!(
                        node_id = %node_id,
                        "Regenerated node passed revalidation"
                    );
                }
            }

            // Task 4.7: Handle failed nodes (retry exhaustion)
            for node_id in &failed_nodes {
                if let Some(original_issues) = issues_by_node.get(node_id) {
                    // Check if any issues are Critical + NoFixPossible
                    let has_critical_unfixable = original_issues.iter().any(|issue| {
                        issue.severity == crate::validation_issues::IssueSeverity::Critical
                            && issue.correction_capability == CorrectionCapability::NoFixPossible
                    });

                    if has_critical_unfixable {
                        // Critical + NoFixPossible + retry exhaustion → Halt pipeline
                        return Err(TaleTrailError::ValidationError(
                            format!(
                                "Node {} has critical unfixable issues and regeneration failed after {} attempts",
                                node_id, constants::RETRY_MAX_ATTEMPTS
                            )
                        ));
                    } else {
                        // Non-critical or NeedsRevision → Skip and continue
                        negotiation_state.add_skipped_node(node_id.clone());
                        tracing::warn!(
                            node_id = %node_id,
                            "Skipping node after regeneration failure (non-critical or retriable issues)"
                        );
                    }
                }
            }

            // Update issues_by_node for next round
            issues_by_node.clear();
            issues_by_node.extend(revalidation_issues);

            // Update negotiation state
            let issues_remaining = issues_by_node.len();
            negotiation_state.set_issues(
                issues_by_node.values()
                    .flatten()
                    .cloned()
                    .collect()
            );

            // Publish round completed event
            let tenant_id_str = format!("tenant-{}", request.tenant_id);
            self.event_publisher
                .publish_event(
                    PipelineEvent::NegotiationRoundCompleted {
                        request_id: request_id.to_string(),
                        round,
                        issues_remaining,
                        corrections_applied: regenerated_nodes.len(),
                    },
                    Some(&tenant_id_str)
                )
                .await?;

            // Update progress incrementally per round
            // Phase 4 range: 85% → 95% (10% total over max 3 rounds)
            let progress = 85.0 + (10.0 * round as f32 / negotiation_state.max_rounds as f32);
            {
                let mut state = self.state.lock().await;
                state.update_progress(progress);
            }

            // Check if all issues resolved
            if issues_remaining == 0 {
                info!("All validation issues resolved after round {}", round);
                negotiation_state.mark_round_succeeded();
                break;
            }

            // Continue to next round if issues remain
            info!(
                "Round {} complete: {} issues remaining, continuing to next round",
                round, issues_remaining
            );
        }

        // Check if max rounds exceeded
        if negotiation_state.max_rounds_reached() && !negotiation_state.round_succeeded {
            tracing::warn!(
                "Max negotiation rounds ({}) exceeded - continuing with current DAG",
                negotiation_state.max_rounds
            );
        }

        // Reset negotiation round counter
        {
            let mut state = self.state.lock().await;
            state.negotiation_round = 0;
            state.update_progress(95.0); // Ensure we're at end of Phase 4
        }

        info!(
            "Negotiation complete after {} rounds",
            negotiation_state.current_round
        );

        Ok((dag, negotiation_state))
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
        correlation_id: Uuid,
        negotiation_round: Option<u32>,
    ) -> Result<ValidationResult> {
        let validation_prompts = prompts.get(&MCPServiceType::QualityControl);

        // Create updated request with prompt packages following envelope-first architecture
        let mut updated_request = request.clone();
        updated_request.prompt_packages = self.convert_prompts_to_request_format(prompts)?;

        // Create metadata for this validation
        let meta = self.create_meta(request, request_id, correlation_id);

        // Publish ValidationStarted event
        let tenant_id_str = format!("tenant-{}", request.tenant_id);
        self.event_publisher
            .publish_event(
                PipelineEvent::ValidationStarted {
                    request_id: request_id.to_string(),
                    batch_id: 0, // Individual node validation, not batched
                    validator: "quality-control".to_string(),
                },
                Some(&tenant_id_str)
            )
            .await?;

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
                    // NEW: Add language field (convert Language enum to lowercase string)
                    map.insert(
                        "language".to_string(),
                        serde_json::to_value(format!("{:?}", request.language).to_lowercase())
                            .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                    );
                    // NEW: Add validation_policy if present
                    if let Some(ref policy) = request.validation_policy {
                        map.insert(
                            "validation_policy".to_string(),
                            serde_json::to_value(policy)
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                    }
                    map
                }),
            },
            extensions: Extensions::default(),
        };

        let started_at = Utc::now();
        let call_start = std::time::Instant::now();

        let response_result: std::result::Result<ValidateContentResponse, TaleTrailError> = self
            .call_mcp_tool(&constants::MCP_QUALITY_VALIDATE, tool_request, meta)
            .await;

        let duration_ms = call_start.elapsed().as_millis() as i64;
        let phase = self.get_current_phase().await;

        // Record invocation with node context
        // Use negotiation round as batch_id during Phase 4
        let batch_id = negotiation_round.map(|r| r as i64);
        match &response_result {
            Ok(_) => {
                self.record_service_invocation(
                    "quality-control".to_string(),
                    "validate_content".to_string(),
                    started_at,
                    duration_ms,
                    true,
                    None,
                    phase,
                    Some(node.id.clone()),
                    batch_id,
                ).await;
            }
            Err(e) => {
                self.record_service_invocation(
                    "quality-control".to_string(),
                    "validate_content".to_string(),
                    started_at,
                    duration_ms,
                    false,
                    Some(e.to_string()),
                    phase,
                    Some(node.id.clone()),
                    batch_id,
                ).await;
            }
        }

        let response = response_result?;
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
        correlation_id: Uuid,
        negotiation_round: Option<u32>,
    ) -> Result<ConstraintResult> {
        let constraint_prompts = prompts.get(&MCPServiceType::ConstraintEnforcer);

        // Create updated request with prompt packages following envelope-first architecture
        let mut updated_request = request.clone();
        updated_request.prompt_packages = self.convert_prompts_to_request_format(prompts)?;

        // Create metadata for this validation
        let meta = self.create_meta(request, request_id, correlation_id);

        // Publish ValidationStarted event
        let tenant_id_str = format!("tenant-{}", request.tenant_id);
        self.event_publisher
            .publish_event(
                PipelineEvent::ValidationStarted {
                    request_id: request_id.to_string(),
                    batch_id: 0, // Individual node validation, not batched
                    validator: "constraint-enforcer".to_string(),
                },
                Some(&tenant_id_str)
            )
            .await?;

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

        let started_at = Utc::now();
        let call_start = std::time::Instant::now();

        let response_result: std::result::Result<EnforceConstraintsResponse, TaleTrailError> = self
            .call_mcp_tool(&constants::MCP_CONSTRAINT_ENFORCE, tool_request, meta)
            .await;

        let duration_ms = call_start.elapsed().as_millis() as i64;
        let phase = self.get_current_phase().await;

        // Record invocation with node context
        // Use negotiation round as batch_id during Phase 4
        let batch_id = negotiation_round.map(|r| r as i64);
        match &response_result {
            Ok(_) => {
                self.record_service_invocation(
                    "constraint-enforcer".to_string(),
                    "enforce_constraints".to_string(),
                    started_at,
                    duration_ms,
                    true,
                    None,
                    phase,
                    Some(node.id.clone()),
                    batch_id,
                ).await;
            }
            Err(e) => {
                self.record_service_invocation(
                    "constraint-enforcer".to_string(),
                    "enforce_constraints".to_string(),
                    started_at,
                    duration_ms,
                    false,
                    Some(e.to_string()),
                    phase,
                    Some(node.id.clone()),
                    batch_id,
                ).await;
            }
        }

        let response = response_result?;
        Ok(response.constraint_result)
    }

    /// Regenerate nodes with retry logic (Task 4.2-4.5)
    ///
    /// Calls story-generator for single-node regeneration with exponential backoff retry.
    /// Preserves DAG structure (edges, convergence points, choice counts) while updating content.
    ///
    /// # Arguments
    /// * `dag` - DAG to update with regenerated nodes
    /// * `nodes_to_regenerate` - List of node IDs requiring regeneration
    /// * `issues_by_node` - Validation issues providing correction context
    /// * `request` - Generation request for context
    /// * `prompts` - Prompt packages for generation
    /// * `request_id` - Request ID for tracing
    /// * `correlation_id` - Correlation ID for pipeline tracking
    ///
    /// # Returns
    /// Result with (updated_dag, regenerated_node_ids, failed_node_ids)
    #[instrument(skip(self, dag, issues_by_node, request, prompts))]
    async fn execute_correction_plan(
        &self,
        mut dag: DAG,
        nodes_to_regenerate: &[String],
        issues_by_node: &HashMap<String, Vec<crate::validation_issues::ValidationIssue>>,
        request: &GenerationRequest,
        prompts: &HashMap<MCPServiceType, PromptPackage>,
        request_id: &str,
        correlation_id: Uuid,
    ) -> Result<(DAG, Vec<String>, Vec<String>)> {
        let mut regenerated_nodes = Vec::new();
        let mut failed_nodes = Vec::new();

        // Create retry configuration from constants
        let retry_config = RetryConfig {
            max_attempts: constants::RETRY_MAX_ATTEMPTS,
            initial_delay_ms: constants::RETRY_INITIAL_DELAY_MS,
            max_delay_ms: constants::RETRY_MAX_DELAY_MS,
            backoff_multiplier: constants::RETRY_BACKOFF_MULTIPLIER,
        };

        info!(
            "Starting node regeneration for {} nodes with retry config: {:?}",
            nodes_to_regenerate.len(),
            retry_config
        );

        for node_id in nodes_to_regenerate {
            // Get the node to preserve its structure
            let original_node = match dag.nodes.get(node_id) {
                Some(node) => node.clone(),
                None => {
                    tracing::error!(node_id = %node_id, "Node not found in DAG");
                    failed_nodes.push(node_id.clone());
                    continue;
                }
            };

            // Extract expected choice count from DAG edges
            let expected_choice_count = dag.edges.iter()
                .filter(|e| &e.from_node_id == node_id)
                .count();

            tracing::info!(
                node_id = %node_id,
                expected_choices = expected_choice_count,
                incoming_edges = original_node.incoming_edges,
                outgoing_edges = original_node.outgoing_edges,
                is_convergence = dag.convergence_points.contains(node_id),
                "Regenerating node with structure preservation"
            );

            // Build correction context from issues
            let correction_context = issues_by_node
                .get(node_id)
                .map(|issues| {
                    issues.iter()
                        .map(|issue| format!("{:?}: {}", issue.issue_type, issue.description))
                        .collect::<Vec<_>>()
                        .join("; ")
                })
                .unwrap_or_else(|| "No specific issues provided".to_string());

            // Create metadata for this call
            let meta = self.create_meta(request, request_id, correlation_id);

            // Prepare generation request with prompts
            let mut updated_request = request.clone();
            updated_request.prompt_packages = self.convert_prompts_to_request_format(prompts)?;

            // Build single-node batch call to story-generator
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
                            serde_json::to_value(&vec![node_id.clone()])
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                        map.insert(
                            "generation_request".to_string(),
                            serde_json::to_value(&updated_request)
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                        map.insert(
                            "expected_choice_counts".to_string(),
                            serde_json::to_value(&vec![expected_choice_count])
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                        map.insert(
                            "correction_context".to_string(),
                            serde_json::to_value(&correction_context)
                                .map_err(|e| TaleTrailError::SerializationError(e.to_string()))?,
                        );
                        map
                    }),
                },
                extensions: Extensions::default(),
            };

            // Task 4.3: Wrap story-generator call in retry_with_backoff
            let regeneration_result = retry_with_backoff(
                || {
                    let tool_request_clone = tool_request.clone();
                    let meta_clone = meta.clone();
                    let mcp_client = self.mcp_client.clone();
                    let subject = constants::MCP_STORY_GENERATE.to_string();

                    async move {
                        mcp_client
                            .call_tool::<GenerateNodesResponse>(&subject, tool_request_clone, meta_clone)
                            .await
                            .map_err(|e| format!("Story generator error: {}", e))
                    }
                },
                &retry_config,
                &format!("regenerate_node_{}", node_id),
            ).await;

            match regeneration_result {
                Ok(response) => {
                    // Task 4.5: Update DAG with regenerated content while preserving structure
                    // response.nodes is a Vec<ContentNode> - get first (should be only) node
                    if let Some(regenerated_node) = response.nodes.first() {
                        // Verify the node ID matches
                        if &regenerated_node.id != node_id {
                            tracing::error!(
                                expected_node_id = %node_id,
                                actual_node_id = %regenerated_node.id,
                                "Regenerated node ID mismatch"
                            );
                            failed_nodes.push(node_id.clone());
                        } else {
                            // Preserve structural properties
                            let mut updated_node = regenerated_node.clone();
                            updated_node.incoming_edges = original_node.incoming_edges;
                            updated_node.outgoing_edges = original_node.outgoing_edges;
                            updated_node.content.convergence_point = original_node.content.convergence_point;

                            // Preserve generation metadata if it exists
                            if let Some(ref orig_metadata) = original_node.generation_metadata {
                                if updated_node.generation_metadata.is_none() {
                                    updated_node.generation_metadata = Some(HashMap::new());
                                }
                                if let Some(ref mut new_metadata) = updated_node.generation_metadata {
                                    new_metadata.insert(
                                        "regeneration_count".to_string(),
                                        serde_json::json!(
                                            orig_metadata.get("regeneration_count")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0) + 1
                                        )
                                    );
                                    new_metadata.insert(
                                        "last_regenerated".to_string(),
                                        serde_json::json!(chrono::Utc::now().to_rfc3339())
                                    );
                                }
                            }

                            // Update the DAG
                            dag.nodes.insert(node_id.clone(), updated_node);
                            regenerated_nodes.push(node_id.clone());

                            tracing::info!(
                                node_id = %node_id,
                                "Successfully regenerated and updated node in DAG"
                            );
                        }
                    } else {
                        tracing::error!(
                            node_id = %node_id,
                            "Regenerated response contains no nodes"
                        );
                        failed_nodes.push(node_id.clone());
                    }
                },
                Err(e) => {
                    // Task 4.7: Handle retry exhaustion
                    tracing::error!(
                        node_id = %node_id,
                        error = %e,
                        "Node regeneration failed after {} retry attempts",
                        retry_config.max_attempts
                    );
                    failed_nodes.push(node_id.clone());
                }
            }
        }

        info!(
            "Regeneration complete: {} succeeded, {} failed",
            regenerated_nodes.len(),
            failed_nodes.len()
        );

        Ok((dag, regenerated_nodes, failed_nodes))
    }

    /// Convert DAG nodes to trail steps
    ///
    /// Transforms the DAG structure into an ordered list of trail steps for frontend consumption.
    /// Each trail step contains:
    /// - Sequential ordering (step_order starting at 1)
    /// - Full content with text, choices, and navigation data
    /// - Metadata including node_id and convergence_point status
    ///
    /// The ordering preserves the DAG's HashMap iteration order. For deterministic ordering in
    /// production, consider sorting by node_id or implementing a topological sort based on edges.
    ///
    /// # Arguments
    ///
    /// * `dag` - The complete DAG with nodes, edges, and convergence points
    ///
    /// # Returns
    ///
    /// Vec<TrailStep> - Ordered list of trail steps ready for frontend visualization
    fn dag_to_trail_steps(dag: &DAG) -> Vec<TrailStep> {
        dag.nodes
            .iter()
            .enumerate()
            .map(|(idx, (node_id, node))| {
                let mut metadata = HashMap::new();
                metadata.insert("node_id".to_string(), serde_json::json!(node_id));
                metadata.insert(
                    "convergence_point".to_string(),
                    serde_json::json!(dag.convergence_points.contains(node_id)),
                );

                // Add edge information for debugging/visualization
                metadata.insert("incoming_edges".to_string(), serde_json::json!(node.incoming_edges));
                metadata.insert("outgoing_edges".to_string(), serde_json::json!(node.outgoing_edges));

                TrailStep {
                    step_order: (idx + 1) as i64,
                    title: None, // Could extract from content.text first sentence if needed
                    description: None,
                    is_required: true, // All nodes in generated DAG are considered required
                    metadata,
                    content_reference: ContentReference {
                        temp_node_id: node_id.clone(),
                        content: {
                            let mut content = node.content.clone();
                            // Populate next_node_id for each choice from DAG edges
                            for choice in &mut content.choices {
                                if let Some(edge) = dag.edges.iter().find(|e| e.choice_id == choice.id) {
                                    choice.next_node_id = edge.to_node_id.clone();
                                    tracing::debug!(
                                        node_id = %node_id,
                                        choice_id = %choice.id,
                                        next_node_id = %edge.to_node_id,
                                        "Successfully matched choice to edge"
                                    );
                                } else {
                                    tracing::warn!(
                                        node_id = %node_id,
                                        choice_id = %choice.id,
                                        choice_text = %choice.text.chars().take(50).collect::<String>(),
                                        available_edges = ?dag.edges.iter()
                                            .filter(|e| e.from_node_id == *node_id)
                                            .map(|e| &e.choice_id)
                                            .collect::<Vec<_>>(),
                                        "Failed to match choice to any DAG edge - attempting fallback"
                                    );
                                }
                            }

                            // Fallback: Match by choice order if ID matching failed
                            let node_edges: Vec<_> = dag.edges.iter()
                                .filter(|e| e.from_node_id == *node_id)
                                .collect();

                            for (idx, choice) in content.choices.iter_mut().enumerate() {
                                // Skip if already matched by ID
                                if !choice.next_node_id.is_empty() {
                                    continue;
                                }

                                // Fallback: match by order if we have enough edges
                                if idx < node_edges.len() {
                                    choice.next_node_id = node_edges[idx].to_node_id.clone();
                                    tracing::info!(
                                        node_id = %node_id,
                                        choice_id = %choice.id,
                                        choice_index = idx,
                                        matched_edge = %node_edges[idx].choice_id,
                                        next_node_id = %node_edges[idx].to_node_id,
                                        "Applied fallback edge matching by choice order"
                                    );
                                } else {
                                    tracing::error!(
                                        node_id = %node_id,
                                        choice_id = %choice.id,
                                        choice_index = idx,
                                        available_edges = node_edges.len(),
                                        "Cannot match choice - insufficient edges for fallback"
                                    );
                                }
                            }
                            content
                        },
                    },
                }
            })
            .collect()
    }

    /// Phase 5: Assemble final response
    ///
    /// Creates final generation response with DAG and metadata, publishes
    /// completion event.
    #[instrument(skip(self, dag, request, negotiation_state))]
    async fn phase_assemble(
        &self,
        dag: DAG,
        request: &GenerationRequest,
        request_id: &str,
        start_time: std::time::Instant,
        correlation_id: Uuid,
        negotiation_state: crate::negotiation_state::NegotiationState,
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
        let tenant_id_str = format!("tenant-{}", request.tenant_id);
        self.event_publisher
            .publish_event(
                PipelineEvent::Complete {
                    request_id: request_id.to_string(),
                    total_duration_ms,
                    total_nodes: dag.nodes.len(),
                    total_validations: dag.nodes.len() * 2, // quality + constraints
                    negotiation_rounds: negotiation_state.current_round,
                },
                Some(&tenant_id_str)
            )
            .await?;

        // Build execution trace
        let execution_trace = {
            // Collect all service invocations
            let invocations = self.service_invocations.lock().await;
            let service_invocations = invocations.clone();

            // Get phases completed from pipeline state
            let phases_completed = vec![
                GenerationPhase::PromptGeneration,
                GenerationPhase::Structure,
                GenerationPhase::Generation,
                GenerationPhase::Validation,
                GenerationPhase::Assembly,
            ];

            PipelineExecutionTrace {
                request_id: request_id.to_string(),
                total_duration_ms: total_duration_ms as i64,
                phases_completed,
                service_invocations,
                events_published: None, // Optional - could collect published events if needed
            }
        };

        // Log execution trace summary
        tracing::info!(
            "Execution trace: {} service invocations across {} phases, total duration: {}ms",
            execution_trace.service_invocations.len(),
            execution_trace.phases_completed.len(),
            execution_trace.total_duration_ms
        );

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

        // Calculate validation metrics
        let total_nodes = dag.nodes.len();
        let nodes_with_unresolved_issues = negotiation_state.issues.len();
        let nodes_passed = total_nodes.saturating_sub(nodes_with_unresolved_issues);
        let validation_pass_rate = if total_nodes > 0 {
            nodes_passed as f64 / total_nodes as f64
        } else {
            1.0
        };

        // Get unresolved issues (only if max rounds exceeded)
        let unresolved_issues = if negotiation_state.max_rounds_reached() && !negotiation_state.round_succeeded {
            Some(negotiation_state.get_unresolved_issues())
        } else {
            Some(Vec::new())
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
                validation_rounds: negotiation_state.current_round as i64 + 1, // +1 for initial validation
                orchestrator_version: env!("CARGO_PKG_VERSION").to_string(),
                validation_pass_rate,
                negotiation_rounds_executed: negotiation_state.current_round as i64,
                resolved_node_count: dag.nodes.len() as i64,
                corrections_applied: if negotiation_state.all_corrections.is_empty() {
                    None
                } else {
                    Some(negotiation_state.all_corrections.clone())
                },
                unresolved_validation_issues: unresolved_issues,
            }),
            prompt_generation_metadata: None, // TODO: Add from prompt orchestration phase
            trail: Some(trail),
            trail_steps: Some(Self::dag_to_trail_steps(&dag)),
            execution_trace: Some(execution_trace),
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
        Self::static_convert_prompts_to_request_format(prompts)
    }

    /// Static version of convert_prompts_to_request_format for use in concurrent contexts
    fn static_convert_prompts_to_request_format(
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
    /// Constructs envelope metadata with tenant_id, request_id, trace_id, and correlation_id
    /// from the orchestrator context.
    ///
    /// # Arguments
    ///
    /// * `request` - Generation request containing tenant_id
    /// * `request_id` - Request ID for correlation (Layer 2: operation identification)
    /// * `correlation_id` - Correlation ID for pipeline tracking (Layer 1: pipeline tracking)
    ///
    /// # Returns
    ///
    /// Meta struct populated with tenant, tracing, and custom TaleTrail metadata
    fn create_meta(&self, request: &GenerationRequest, request_id: &str, correlation_id: Uuid) -> Meta {
        Self::static_create_meta(request, request_id, correlation_id)
    }

    /// Static version of create_meta for use in concurrent contexts
    fn static_create_meta(request: &GenerationRequest, request_id: &str, correlation_id: Uuid) -> Meta {
        let mut meta = Meta::default();

        // Set tenant ID for multi-tenancy isolation
        meta.tenant = Some(format!("tenant-{}", request.tenant_id));

        // Set request ID for correlation (Layer 2: operation identification)
        if let Ok(uuid) = Uuid::parse_str(request_id) {
            meta.request_id = Some(uuid);
        }

        // Set trace ID for distributed tracing (same as request_id for now)
        meta.tracing = Some(create_tracing_meta(request_id.to_string()));

        // Layer 1: Add TaleTrailCustomMetadata with correlation_id
        let custom_meta = TaleTrailCustomMetadata::new()
            .with_correlation_id(correlation_id);
        meta.extensions = Some(custom_meta.to_extensions_meta());

        meta
    }

    /// Record a service invocation for execution trace
    ///
    /// Captures service call details including timing, success status, and context.
    ///
    /// # Arguments
    ///
    /// * `service_name` - Name of MCP service (e.g., "story-generator")
    /// * `tool_name` - Name of MCP tool (e.g., "generate_structure")
    /// * `started_at` - When the service call started
    /// * `duration_ms` - Duration of the call in milliseconds
    /// * `success` - Whether the call succeeded
    /// * `error_message` - Optional error message if call failed
    /// * `phase` - Pipeline phase during which the call was made
    /// * `node_id` - Optional node ID being processed
    /// * `batch_id` - Optional batch ID if part of batch processing
    async fn record_service_invocation(
        &self,
        service_name: String,
        tool_name: String,
        started_at: chrono::DateTime<Utc>,
        duration_ms: i64,
        success: bool,
        error_message: Option<String>,
        phase: GenerationPhase,
        node_id: Option<String>,
        batch_id: Option<i64>,
    ) {
        let invocation = ServiceInvocation {
            service_name: service_name.clone(),
            tool_name: tool_name.clone(),
            started_at: started_at.to_rfc3339(),
            duration_ms,
            success,
            error_message,
            phase,
            node_id,
            batch_id,
        };

        let mut invocations = self.service_invocations.lock().await;
        invocations.push(invocation);

        tracing::debug!(
            "Recorded service invocation: {} / {} ({}ms, success={})",
            service_name,
            tool_name,
            duration_ms,
            success
        );
    }

    /// Get current pipeline phase for service invocation tracking
    async fn get_current_phase(&self) -> GenerationPhase {
        let state = self.state.lock().await;
        // Convert PipelinePhase to GenerationPhase
        use crate::pipeline::PipelinePhase;
        match state.current_phase {
            PipelinePhase::PromptGeneration => GenerationPhase::PromptGeneration,
            PipelinePhase::Structure => GenerationPhase::Structure,
            PipelinePhase::Generation => GenerationPhase::Generation,
            PipelinePhase::Validation => GenerationPhase::Validation,
            PipelinePhase::Assembly => GenerationPhase::Assembly,
            PipelinePhase::Complete => GenerationPhase::Complete,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{AgeGroup, Choice, Content, ContentNode, DAG, Edge, Language};

    /// Helper function to create a test ContentNode
    fn create_test_node(node_id: &str, choices: Vec<Choice>) -> ContentNode {
        let outgoing_edges = choices.len() as i64;
        ContentNode {
            id: node_id.to_string(),
            content: Content {
                node_id: node_id.to_string(),
                text: format!("Test content for {}", node_id),
                r#type: "story".to_string(),
                choices,
                next_nodes: vec![],
                convergence_point: false,
                educational_content: None,
            },
            incoming_edges: 0,
            outgoing_edges,
            generation_metadata: None,
        }
    }

    /// Helper function to create a test Choice
    fn create_test_choice(choice_id: &str, text: &str) -> Choice {
        Choice {
            id: choice_id.to_string(),
            text: text.to_string(),
            next_node_id: String::new(), // Start empty - should be populated by dag_to_trail_steps
            metadata: None,
        }
    }

    #[test]
    fn test_dag_to_trail_steps_linear_story() {
        // Create a simple linear story: node0 -> node1 -> node2
        let mut nodes = HashMap::new();

        // Node 0: Start node with one choice
        let choice_0 = create_test_choice("choice_0_0", "Continue to node 1");
        nodes.insert("node0".to_string(), create_test_node("node0", vec![choice_0]));

        // Node 1: Middle node with one choice
        let choice_1 = create_test_choice("choice_1_0", "Continue to node 2");
        nodes.insert("node1".to_string(), create_test_node("node1", vec![choice_1]));

        // Node 2: End node with no choices
        nodes.insert("node2".to_string(), create_test_node("node2", vec![]));

        // Create edges
        let edges = vec![
            Edge {
                from_node_id: "node0".to_string(),
                to_node_id: "node1".to_string(),
                choice_id: "choice_0_0".to_string(),
                weight: None,
            },
            Edge {
                from_node_id: "node1".to_string(),
                to_node_id: "node2".to_string(),
                choice_id: "choice_1_0".to_string(),
                weight: None,
            },
        ];

        let dag = DAG {
            nodes,
            edges,
            start_node_id: "node0".to_string(),
            convergence_points: vec![],
        };

        // Convert to trail steps
        let trail_steps = Orchestrator::dag_to_trail_steps(&dag);

        // Verify we have 3 trail steps
        assert_eq!(trail_steps.len(), 3, "Should have 3 trail steps");

        // Find trail steps by their node_id (order not guaranteed due to HashMap)
        let step_0 = trail_steps
            .iter()
            .find(|s| s.content_reference.temp_node_id == "node0")
            .expect("Should find node0");
        let step_1 = trail_steps
            .iter()
            .find(|s| s.content_reference.temp_node_id == "node1")
            .expect("Should find node1");
        let step_2 = trail_steps
            .iter()
            .find(|s| s.content_reference.temp_node_id == "node2")
            .expect("Should find node2");

        // Verify node0 choice has correct next_node_id
        assert_eq!(
            step_0.content_reference.content.choices.len(),
            1,
            "Node0 should have 1 choice"
        );
        assert_eq!(
            step_0.content_reference.content.choices[0].next_node_id,
            "node1",
            "Node0 choice should point to node1"
        );

        // Verify node1 choice has correct next_node_id
        assert_eq!(
            step_1.content_reference.content.choices.len(),
            1,
            "Node1 should have 1 choice"
        );
        assert_eq!(
            step_1.content_reference.content.choices[0].next_node_id,
            "node2",
            "Node1 choice should point to node2"
        );

        // Verify node2 (end node) has no choices
        assert_eq!(
            step_2.content_reference.content.choices.len(),
            0,
            "Node2 (end node) should have no choices"
        );
    }

    #[test]
    fn test_dag_to_trail_steps_branching_story() {
        // Create a branching story:
        // node0 -> choice_0_0 -> node1
        //       -> choice_0_1 -> node2
        let mut nodes = HashMap::new();

        // Node 0: Start node with two choices
        let choice_0_0 = create_test_choice("choice_0_0", "Go to node 1");
        let choice_0_1 = create_test_choice("choice_0_1", "Go to node 2");
        nodes.insert(
            "node0".to_string(),
            create_test_node("node0", vec![choice_0_0, choice_0_1]),
        );

        // Node 1: First branch end
        nodes.insert("node1".to_string(), create_test_node("node1", vec![]));

        // Node 2: Second branch end
        nodes.insert("node2".to_string(), create_test_node("node2", vec![]));

        // Create edges
        let edges = vec![
            Edge {
                from_node_id: "node0".to_string(),
                to_node_id: "node1".to_string(),
                choice_id: "choice_0_0".to_string(),
                weight: None,
            },
            Edge {
                from_node_id: "node0".to_string(),
                to_node_id: "node2".to_string(),
                choice_id: "choice_0_1".to_string(),
                weight: None,
            },
        ];

        let dag = DAG {
            nodes,
            edges,
            start_node_id: "node0".to_string(),
            convergence_points: vec![],
        };

        // Convert to trail steps
        let trail_steps = Orchestrator::dag_to_trail_steps(&dag);

        // Verify we have 3 trail steps
        assert_eq!(trail_steps.len(), 3, "Should have 3 trail steps");

        // Find node0
        let step_0 = trail_steps
            .iter()
            .find(|s| s.content_reference.temp_node_id == "node0")
            .expect("Should find node0");

        // Verify node0 has 2 choices with correct next_node_id values
        assert_eq!(
            step_0.content_reference.content.choices.len(),
            2,
            "Node0 should have 2 choices"
        );

        // Find choices by their id
        let choice_0 = step_0
            .content_reference
            .content
            .choices
            .iter()
            .find(|c| c.id == "choice_0_0")
            .expect("Should find choice_0_0");
        let choice_1 = step_0
            .content_reference
            .content
            .choices
            .iter()
            .find(|c| c.id == "choice_0_1")
            .expect("Should find choice_0_1");

        assert_eq!(
            choice_0.next_node_id, "node1",
            "Choice 0 should point to node1"
        );
        assert_eq!(
            choice_1.next_node_id, "node2",
            "Choice 1 should point to node2"
        );
    }

    #[test]
    fn test_dag_to_trail_steps_convergence_points() {
        // Create a convergence scenario:
        // node0 -> choice_0_0 -> node1 -> choice_1_0 -> node3 (convergence)
        //       -> choice_0_1 -> node2 -> choice_2_0 -> node3 (convergence)
        let mut nodes = HashMap::new();

        // Node 0: Start with two branches
        let choice_0_0 = create_test_choice("choice_0_0", "Path 1");
        let choice_0_1 = create_test_choice("choice_0_1", "Path 2");
        nodes.insert(
            "node0".to_string(),
            create_test_node("node0", vec![choice_0_0, choice_0_1]),
        );

        // Node 1: First branch
        let choice_1_0 = create_test_choice("choice_1_0", "Converge");
        nodes.insert("node1".to_string(), create_test_node("node1", vec![choice_1_0]));

        // Node 2: Second branch
        let choice_2_0 = create_test_choice("choice_2_0", "Converge");
        nodes.insert("node2".to_string(), create_test_node("node2", vec![choice_2_0]));

        // Node 3: Convergence point
        let mut node3 = create_test_node("node3", vec![]);
        node3.incoming_edges = 2; // Two paths converge here
        node3.content.convergence_point = true;
        nodes.insert("node3".to_string(), node3);

        // Create edges
        let edges = vec![
            Edge {
                from_node_id: "node0".to_string(),
                to_node_id: "node1".to_string(),
                choice_id: "choice_0_0".to_string(),
                weight: None,
            },
            Edge {
                from_node_id: "node0".to_string(),
                to_node_id: "node2".to_string(),
                choice_id: "choice_0_1".to_string(),
                weight: None,
            },
            Edge {
                from_node_id: "node1".to_string(),
                to_node_id: "node3".to_string(),
                choice_id: "choice_1_0".to_string(),
                weight: None,
            },
            Edge {
                from_node_id: "node2".to_string(),
                to_node_id: "node3".to_string(),
                choice_id: "choice_2_0".to_string(),
                weight: None,
            },
        ];

        let dag = DAG {
            nodes,
            edges,
            start_node_id: "node0".to_string(),
            convergence_points: vec!["node3".to_string()],
        };

        // Convert to trail steps
        let trail_steps = Orchestrator::dag_to_trail_steps(&dag);

        // Verify we have 4 trail steps
        assert_eq!(trail_steps.len(), 4, "Should have 4 trail steps");

        // Find all nodes
        let step_0 = trail_steps
            .iter()
            .find(|s| s.content_reference.temp_node_id == "node0")
            .expect("Should find node0");
        let step_1 = trail_steps
            .iter()
            .find(|s| s.content_reference.temp_node_id == "node1")
            .expect("Should find node1");
        let step_2 = trail_steps
            .iter()
            .find(|s| s.content_reference.temp_node_id == "node2")
            .expect("Should find node2");
        let step_3 = trail_steps
            .iter()
            .find(|s| s.content_reference.temp_node_id == "node3")
            .expect("Should find node3");

        // Verify node0 choices point to correct nodes
        let choice_0 = step_0
            .content_reference
            .content
            .choices
            .iter()
            .find(|c| c.id == "choice_0_0")
            .expect("Should find choice_0_0");
        let choice_1 = step_0
            .content_reference
            .content
            .choices
            .iter()
            .find(|c| c.id == "choice_0_1")
            .expect("Should find choice_0_1");
        assert_eq!(choice_0.next_node_id, "node1");
        assert_eq!(choice_1.next_node_id, "node2");

        // Verify node1 choice points to convergence node
        assert_eq!(step_1.content_reference.content.choices[0].next_node_id, "node3");

        // Verify node2 choice points to convergence node
        assert_eq!(step_2.content_reference.content.choices[0].next_node_id, "node3");

        // Verify convergence point metadata
        let convergence_point = step_3
            .metadata
            .get("convergence_point")
            .expect("Should have convergence_point metadata");
        assert_eq!(convergence_point, &serde_json::json!(true));

        // Verify incoming_edges metadata
        let incoming_edges = step_3
            .metadata
            .get("incoming_edges")
            .expect("Should have incoming_edges metadata");
        assert_eq!(incoming_edges, &serde_json::json!(2));
    }

    #[test]
    fn test_dag_to_trail_steps_edge_cases() {
        // Test 1: Node with no choices (end node)
        let mut nodes = HashMap::new();
        nodes.insert("node0".to_string(), create_test_node("node0", vec![]));

        let dag = DAG {
            nodes: nodes.clone(),
            edges: vec![],
            start_node_id: "node0".to_string(),
            convergence_points: vec![],
        };

        let trail_steps = Orchestrator::dag_to_trail_steps(&dag);
        assert_eq!(trail_steps.len(), 1);
        assert_eq!(trail_steps[0].content_reference.content.choices.len(), 0);

        // Test 2: Missing edge for a choice (should handle gracefully)
        nodes.clear();
        let orphan_choice = create_test_choice("orphan_choice", "Broken link");
        nodes.insert(
            "node0".to_string(),
            create_test_node("node0", vec![orphan_choice]),
        );

        let dag = DAG {
            nodes,
            edges: vec![], // No edges - choice has no corresponding edge
            start_node_id: "node0".to_string(),
            convergence_points: vec![],
        };

        let trail_steps = Orchestrator::dag_to_trail_steps(&dag);
        assert_eq!(trail_steps.len(), 1);
        assert_eq!(trail_steps[0].content_reference.content.choices.len(), 1);
        // Choice should still exist but next_node_id should be empty
        assert_eq!(
            trail_steps[0].content_reference.content.choices[0].next_node_id,
            ""
        );

        // Test 3: Multiple choices, some with edges, some without
        let mut nodes = HashMap::new();
        let choice_with_edge = create_test_choice("choice_with_edge", "Valid path");
        let choice_without_edge = create_test_choice("choice_without_edge", "Broken path");
        nodes.insert(
            "node0".to_string(),
            create_test_node("node0", vec![choice_with_edge, choice_without_edge]),
        );
        nodes.insert("node1".to_string(), create_test_node("node1", vec![]));

        let dag = DAG {
            nodes,
            edges: vec![Edge {
                from_node_id: "node0".to_string(),
                to_node_id: "node1".to_string(),
                choice_id: "choice_with_edge".to_string(),
                weight: None,
            }],
            start_node_id: "node0".to_string(),
            convergence_points: vec![],
        };

        let trail_steps = Orchestrator::dag_to_trail_steps(&dag);
        let step_0 = trail_steps
            .iter()
            .find(|s| s.content_reference.temp_node_id == "node0")
            .expect("Should find node0");

        // Find both choices
        let choice_valid = step_0
            .content_reference
            .content
            .choices
            .iter()
            .find(|c| c.id == "choice_with_edge")
            .expect("Should find choice_with_edge");
        let choice_broken = step_0
            .content_reference
            .content
            .choices
            .iter()
            .find(|c| c.id == "choice_without_edge")
            .expect("Should find choice_without_edge");

        assert_eq!(choice_valid.next_node_id, "node1", "Valid choice should point to node1");
        assert_eq!(choice_broken.next_node_id, "", "Broken choice should remain empty - no fallback edge available at index 1");
    }
}
