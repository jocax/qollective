# TaleTrail Content Generator - Qollective Framework Summary

Educational material for demonstrating envelope-first architecture in distributed systems.

---

## 1. Qollective Envelope Pattern

### Core Structure

```rust
struct Envelope<T> {
    meta: Meta,        // Metadata context
    payload: T,        // Business data
    error: Option<EnvelopeError>,  // Optional error
}

struct Meta {
    // Core fields
    timestamp: Option<DateTime<Utc>>,
    request_id: Option<Uuid>,
    version: Option<String>,
    tenant: Option<String>,

    // Context sections
    security: Option<SecurityMeta>,      // auth, permissions, roles
    tracing: Option<TracingMeta>,        // distributed tracing
    performance: Option<PerformanceMeta>,  // metrics
    monitoring: Option<MonitoringMeta>,   // health, alerts
    debug: Option<DebugMeta>,            // dev diagnostics
    on_behalf_of: Option<OnBehalfOfMeta>, // delegation
    extensions: Option<ExtensionsMeta>,  // custom metadata
}
```

### Purpose

**Envelope-first architecture** wraps all service communication with consistent metadata, ensuring:

- **Unified metadata** across protocols (REST ↔ gRPC ↔ NATS ↔ WebSocket ↔ MCP)
- **Tenant isolation** via automatic context extraction/propagation
- **Distributed tracing** with request_id + trace_id correlation
- **Protocol abstraction** - same envelope, different transport
- **Security context** flow without manual header manipulation

### Envelope Flow

```
┌─────────────────────────────────────────────────────────────┐
│  Client: Create business payload                            │
│    payload = GenerationRequest { theme, language, ... }     │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  Client: Wrap in Envelope with metadata                     │
│    meta = Meta {                                            │
│      tenant: "user-123",                                    │
│      request_id: uuid(),                                    │
│      extensions: TaleTrailCustomMetadata { phase, ... }     │
│    }                                                        │
│    envelope = Envelope::new(meta, payload)                  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼ (Serialize)
┌─────────────────────────────────────────────────────────────┐
│  Transport Layer: NATS / REST / gRPC / WebSocket / MCP      │
│    nats.request("mcp.story.generate", envelope_bytes)       │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼ (Deserialize)
┌─────────────────────────────────────────────────────────────┐
│  Server: Extract envelope and metadata                      │
│    envelope: Envelope<T> = deserialize(bytes)               │
│    tenant_id = envelope.meta.tenant  // Automatic context   │
│    trace_id = envelope.meta.tracing.trace_id                │
│    custom = TaleTrailCustomMetadata::from_extensions(...)   │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  Server: Process business logic + create response envelope  │
│    result = process(envelope.payload)                       │
│    response_envelope = Envelope::new(meta, result)          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼ (Return via same transport)
                  Client
```

### Qollective Feature Gates

Comprehensive feature matrix for transport protocols and capabilities:

| Feature Gate          | Client/Server/Both | Transport / Type    | Used in TaleTrail | Status         |
|-----------------------|--------------------|---------------------|:-----------------:|----------------|
| `rest-client`         | Client             | HTTP REST           | —                 | ✓ Implemented  |
| `rest-server`         | Server             | HTTP REST           | —                 | ✓ Implemented  |
| `rest`                | Both               | HTTP REST           | —                 | ✓ Implemented  |
| `grpc-client`         | Client             | gRPC                | —                 | ✓ Implemented  |
| `grpc-server`         | Server             | gRPC                | —                 | ✓ Implemented  |
| `grpc`                | Both               | gRPC                | —                 | ✓ Implemented  |
| `nats-client`         | Client             | NATS Pub/Sub        | **✓**             | ✓ Implemented  |
| `nats-server`         | Server             | NATS Pub/Sub        | **✓**             | ✓ Implemented  |
| `nats`                | Both               | NATS Pub/Sub        | **✓**             | ✓ Implemented  |
| `websocket-client`    | Client             | WebSocket           | —                 | ✓ Implemented  |
| `websocket-server`    | Server             | WebSocket           | —                 | ✓ Implemented  |
| `websocket`           | Both               | WebSocket           | —                 | ✓ Implemented  |
| `jsonrpc-client`      | Client             | JSON-RPC 2.0        | —                 | ✓ Implemented  |
| `jsonrpc-server`      | Server             | JSON-RPC 2.0        | —                 | ✓ Implemented  |
| `jsonrpc`             | Both               | JSON-RPC 2.0        | —                 | ✓ Implemented  |
| `mcp-client`          | Client             | Model Context Proto | **✓**             | ✓ Implemented  |
| `mcp-server`          | Server             | Model Context Proto | **✓**             | ✓ Implemented  |
| `mcp`                 | Both               | Model Context Proto | **✓**             | ✓ Implemented  |
| `a2a-client`          | Client             | Agent-to-Agent      | —                 | ✓ Implemented  |
| `a2a-server`          | Server             | Agent-to-Agent      | —                 | ✓ Implemented  |
| `a2a-standard`        | Both               | A2A (HTTP)          | —                 | ✓ Implemented  |
| `a2a-nats`            | Both               | A2A (NATS)          | —                 | ✓ Implemented  |
| `a2a`                 | Both               | Agent-to-Agent      | —                 | ✓ Implemented  |
| `wasm-client`         | Client             | WASM Browser        | —                 | ✓ Implemented  |
| `wasm-rest`           | Client             | WASM REST           | —                 | ✓ Implemented  |
| `wasm-jsonrpc`        | Client             | WASM JSON-RPC       | —                 | ✓ Implemented  |
| `wasm-mcp`            | Client             | WASM MCP            | —                 | ✓ Implemented  |
| `wasm-enhanced`       | Client             | WASM All Protocols  | —                 | ✓ Implemented  |
| `protobuf`            | Both               | Protocol Buffers    | —                 | ✓ Implemented  |
| `tls`                 | Both               | TLS/mTLS Security   | **✓**             | ✓ Implemented  |
| `tenant-extraction`   | Both               | JWT Multi-tenancy   | **✓**             | ✓ Implemented  |
| `config`              | Both               | TOML Configuration  | **✓**             | ✓ Implemented  |
| `tracing`             | Both               | OpenTelemetry       | **✓**             | ✓ Implemented  |
| `validation`          | Both               | JSON Schema         | **✓**             | ✓ Implemented  |
| `openapi`             | Both               | OpenAPI/Swagger     | —                 | ✓ Implemented  |

**Feature Combinations**:
- `hybrid-transport` = `rest` + `grpc` + `nats` + `websocket` + `jsonrpc` + `mcp` + `a2a-full`
- `server-transport` = All server-side features
- `client-transport` = All client-side features
- `full` = Everything (development)

**TaleTrail Usage**: `nats`, `mcp-client`, `mcp-server`, `tls`, `tenant-extraction`, `config`, `tracing`, `validation`

---

## 2. TaleTrail Content Generator Example

### Purpose

Demonstrates Qollective's envelope-first architecture in a **multi-agent MCP orchestration system** that generates interactive educational stories (16-node DAGs) using:

- **NATS** for transport
- **MCP** (Model Context Protocol) for agent tool execution
- **Qollective envelopes** for metadata propagation
- **TLS + NKey auth** for security
- **Schema-driven code generation** for type safety

### System Architecture

```
                     ┌──────────────────────┐
                     │   HTTP Client        │
                     │  (External Request)  │
                     └──────────┬───────────┘
                                │ REST
                                ▼
                     ┌──────────────────────┐
                     │   Gateway (Optional) │
                     │    REST → NATS       │
                     └──────────┬───────────┘
                                │ NATS
                                ▼
                     ┌──────────────────────┐
                     │    Orchestrator      │◄───────────────┐
                     │   (MCP Client)       │                │
                     │                      │                │
                     │  Phases:             │                │
                     │  1. Prompt Gen       │                │
                     │  2. Structure        │                │
                     │  3. Content          │                │
                     │  4. Validation       │                │
                     │  5. Negotiation      │                │
                     │  6. Assembly         │                │
                     └──────────┬───────────┘                │
                                │                            │
         ┌──────────────────────┼────────────────────┐       │
         │                      │                    │       │
         │ NATS subjects:       │                    │       │
         │ mcp.prompt.helper    │                    │       │
         │ mcp.story.generate   │                    │       │
         │ mcp.quality.validate │                    │       │
         │ mcp.constraint.*     │                    │       │
         │                      │                    │       │
    ┌────▼──────┐     ┌────────▼───────┐    ┌───────▼───────┐    ┌────────────┐
    │  Prompt   │     │     Story      │    │   Quality     │    │Constraint  │
    │  Helper   │     │   Generator    │    │   Control     │    │ Enforcer   │
    │           │     │                │    │               │    │            │
    │ MCP       │     │ MCP Server     │    │ MCP Server    │    │ MCP Server │
    │ Server    │     │                │    │               │    │            │
    │           │     │ Tools:         │    │ Tools:        │    │ Tools:     │
    │ Tools:    │     │ - gen_structure│    │ - validate_   │    │ - enforce_ │
    │ - gen_*_  │     │ - gen_nodes    │    │   quality     │    │   rules    │
    │   prompts │     │                │    │ - suggest_fix │    │ - verify   │
    │ - get_    │     │                │    │               │    │            │
    │   model   │     │                │    │               │    │            │
    └───────┬───┘     └───────┬────────┘    └───────┬───────┘    └─────┬──────┘
            │                 │                     │                   │
            └─────────────────┴─────────────────────┴───────────────────┘
                                        │
                                        ▼
                               ┌─────────────────┐
                               │   LM Studio     │
                               │ (Local LLM API) │
                               │ 127.0.0.1:1234  │
                               └─────────────────┘

Legend:
  → = HTTP/REST communication
  │ = NATS pub/sub (envelope-wrapped)
  MCP = Model Context Protocol (tool calls)
```

### NATS Subject Hierarchy

**TaleTrail uses a simplified single-subject-per-service pattern** (routing via tool_name):

```
mcp.prompt.helper          → Prompt Helper MCP Server
  Queue Group: prompt-helper-group
  Tools: generate_story_prompts, generate_validation_prompts,
         generate_constraint_prompts, get_model_info

mcp.story.generate         → Story Generator MCP Server
  Queue Group: story-generator-group
  Tools: generate_structure, generate_content_nodes

mcp.quality.validate       → Quality Control MCP Server
  Queue Group: quality-control-group
  Tools: validate_quality, suggest_quality_improvements

mcp.constraint.enforce     → Constraint Enforcer MCP Server
  Queue Group: constraint-enforcer-group
  Tools: enforce_constraints, validate_educational_goals

mcp.events                 → Event publishing (monitoring)
  No queue group (broadcast)

mcp.orchestrator.request   → Orchestrator coordination subject
```

**Security**: Each service has NATS NKey authentication with publish/subscribe permissions per subject.

### Component Breakdown

#### 1. Shared Types (`shared-types/`)

**Purpose**: Central type definitions used across all services.

**Key Files**:
- `custom_metadata.rs` - TaleTrail extension of Qollective Meta
- `payloads.rs` - TaleTrailPayload enum (request/response types)
- `constants.rs` - **CONSTANTS FIRST** - all subjects, URLs, timeouts
- `traits/` - Service abstractions (LlmService, McpTransport, etc.)

#### 2. Schema Extension for Qollective

**Pattern**: Extend `Meta.extensions` field with custom metadata.

**TaleTrail Extension**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaleTrailCustomMetadata {
    generation_phase: Option<GenerationPhase>,  // PromptGeneration → Structure → ... → Complete
    batch_id: Option<Uuid>,                     // Groups related operations
    correlation_id: Option<Uuid>,               // Tracks request chains
}

impl TaleTrailCustomMetadata {
    // Convert to Qollective's ExtensionsMeta (HashMap<String, Value>)
    pub fn to_extensions_meta(&self) -> ExtensionsMeta {
        let mut sections = HashMap::new();
        if let Some(phase) = &self.generation_phase {
            sections.insert("generation_phase".to_string(), json!(phase));
        }
        if let Some(batch_id) = &self.batch_id {
            sections.insert("batch_id".to_string(), json!(batch_id));
        }
        if let Some(correlation_id) = &self.correlation_id {
            sections.insert("correlation_id".to_string(), json!(correlation_id));
        }
        ExtensionsMeta { sections }
    }

    // Extract from Qollective's ExtensionsMeta
    pub fn from_extensions_meta(extensions: &ExtensionsMeta) -> Option<Self> {
        let generation_phase = extensions.sections.get("generation_phase")
            .and_then(|v| serde_json::from_value(v.clone()).ok());
        let batch_id = extensions.sections.get("batch_id")
            .and_then(|v| serde_json::from_value(v.clone()).ok());
        let correlation_id = extensions.sections.get("correlation_id")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        Some(Self { generation_phase, batch_id, correlation_id })
    }
}
```

#### 3. Orchestrator (MCP Client)

**Role**: Coordinates entire pipeline by calling MCP server tools via NATS.

**Envelope Construction**:
```rust
// orchestrator/src/mcp_client.rs

pub struct McpEnvelopeClient {
    nats_client: Arc<NatsClient>,
    timeout_secs: u64,
    concurrency_limit: Arc<Semaphore>,  // Backpressure control
}

impl McpEnvelopeClient {
    pub async fn call_tool<T: DeserializeOwned>(
        &self,
        subject: &str,           // e.g., "mcp.story.generate"
        request: CallToolRequest, // MCP tool call
        meta: Meta,               // Qollective metadata
    ) -> Result<T> {
        // Step 1: Wrap MCP request in McpData
        let mcp_data = McpData {
            tool_call: Some(request),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        // Step 2: Wrap McpData in Qollective Envelope with metadata
        let envelope = Envelope::new(meta, mcp_data);

        // Step 3: Serialize envelope to bytes
        let envelope_bytes = serde_json::to_vec(&envelope)?;

        // Step 4: Send via NATS request/reply with timeout
        let response = self.nats_client
            .request_raw(subject, &envelope_bytes, Duration::from_secs(self.timeout_secs))
            .await?;

        // Step 5: Deserialize response envelope
        let response_envelope: Envelope<McpData> = serde_json::from_slice(&response.payload)?;

        // Step 6: Extract tool response and deserialize result
        let tool_response = response_envelope.payload.tool_response
            .ok_or(TaleTrailError::GenerationError("Missing tool response".into()))?;

        // Step 7: Parse tool result content as expected type T
        let result: T = serde_json::from_value(tool_response.content.first()?.clone())?;
        Ok(result)
    }
}
```

**Pipeline Execution**:
```rust
// orchestrator/src/orchestrator.rs

pub struct Orchestrator {
    mcp_client: McpEnvelopeClient,
    config: OrchestratorConfig,
}

impl Orchestrator {
    pub async fn generate_story(&self, request: ExternalGenerationRequestV1) -> Result<Trail> {
        let correlation_id = Uuid::now_v7();

        // Phase 0.5: Generate prompts (parallel)
        let prompts = self.generate_prompts(&request, correlation_id).await?;

        // Phase 1: Generate DAG structure
        let meta = self.build_meta(GenerationPhase::Structure, correlation_id);
        let structure: DagStructure = self.mcp_client.call_tool(
            MCP_STORY_GENERATE,
            CallToolRequest { name: "generate_structure", arguments: json!(request) },
            meta
        ).await?;

        // Phase 2: Generate content for each node (batched)
        let meta = self.build_meta(GenerationPhase::Generation, correlation_id);
        let nodes = self.generate_nodes_batched(&structure, &prompts, meta).await?;

        // Phase 3: Validate quality and constraints (parallel)
        let meta = self.build_meta(GenerationPhase::Validation, correlation_id);
        let (quality_result, constraint_result) = tokio::join!(
            self.validate_quality(&nodes, meta.clone()),
            self.enforce_constraints(&nodes, meta)
        );

        // Phase 4: Negotiation if failures detected
        if quality_result.has_failures() || constraint_result.has_violations() {
            self.negotiate_fixes(quality_result, constraint_result).await?;
        }

        // Phase 5: Assemble final trail
        let meta = self.build_meta(GenerationPhase::Assembly, correlation_id);
        Ok(self.assemble_trail(structure, nodes, meta).await?)
    }

    fn build_meta(&self, phase: GenerationPhase, correlation_id: Uuid) -> Meta {
        let mut meta = Meta::default();
        meta.tenant = Some("taletrail-system".to_string());
        meta.request_id = Some(Uuid::now_v7());

        // Add TaleTrail custom metadata
        let custom = TaleTrailCustomMetadata::new()
            .with_phase(phase)
            .with_correlation_id(correlation_id);
        meta.extensions = Some(custom.to_extensions_meta());

        meta
    }
}
```

#### 4. MCP Servers

**Pattern**: Each server implements MCP protocol and handles envelope-wrapped tool calls.

**Example: Story Generator Server**:
```rust
// story-generator/src/envelope_handlers.rs

pub struct StoryGeneratorEnvelopeHandler {
    llm_service: Arc<dyn LlmService>,
    config: StoryGeneratorConfig,
}

impl StoryGeneratorEnvelopeHandler {
    pub async fn handle_envelope_request(
        &self,
        envelope_bytes: &[u8],
    ) -> Result<Vec<u8>> {
        // Step 1: Deserialize incoming envelope
        let request_envelope: Envelope<McpData> = serde_json::from_slice(envelope_bytes)?;

        // Step 2: Extract metadata and custom extensions
        let tenant_id = request_envelope.meta.tenant.clone();
        let request_id = request_envelope.meta.request_id;
        let custom_meta = request_envelope.meta.extensions.as_ref()
            .and_then(|e| TaleTrailCustomMetadata::from_extensions_meta(e));

        info!(
            tenant_id = ?tenant_id,
            request_id = ?request_id,
            phase = ?custom_meta.as_ref().and_then(|c| c.generation_phase.as_ref()),
            "Received envelope request"
        );

        // Step 3: Extract MCP tool call
        let tool_call = request_envelope.payload.tool_call
            .ok_or(TaleTrailError::GenerationError("Missing tool_call".into()))?;

        // Step 4: Route to appropriate tool handler
        let tool_result = match tool_call.params.name.as_str() {
            "generate_structure" => self.handle_generate_structure(tool_call).await?,
            "generate_content_nodes" => self.handle_generate_nodes(tool_call).await?,
            _ => return Err(TaleTrailError::UnsupportedTool(tool_call.params.name)),
        };

        // Step 5: Wrap result in MCP response
        let mcp_response = McpData {
            tool_call: None,
            tool_response: Some(CallToolResult {
                content: vec![json!(tool_result)],
                is_error: false,
                ..Default::default()
            }),
            tool_registration: None,
            discovery_data: None,
        };

        // Step 6: Create response envelope (preserve/update metadata)
        let mut response_meta = request_envelope.meta.clone();
        response_meta.duration = Some(start_time.elapsed().as_secs_f64());

        let response_envelope = Envelope::new(response_meta, mcp_response);

        // Step 7: Serialize and return
        Ok(serde_json::to_vec(&response_envelope)?)
    }
}

// NATS integration
pub async fn run_server(config: StoryGeneratorConfig) -> Result<()> {
    let nats_client = connect_nats(&config.nats).await?;
    let handler = Arc::new(StoryGeneratorEnvelopeHandler::new(config));

    // Subscribe to subject with queue group (load balancing)
    let mut subscriber = nats_client
        .queue_subscribe(
            config.nats.subject.clone(),      // "mcp.story.generate"
            config.nats.queue_group.clone(),  // "story-generator-group"
        )
        .await?;

    info!("Story Generator listening on subject: {}", config.nats.subject);

    while let Some(message) = subscriber.next().await {
        let handler = handler.clone();
        tokio::spawn(async move {
            match handler.handle_envelope_request(&message.payload).await {
                Ok(response) => {
                    if let Some(reply) = message.reply {
                        let _ = message.respond(response).await;
                    }
                }
                Err(e) => error!("Error handling request: {}", e),
            }
        });
    }

    Ok(())
}
```

**MCP Tool Registration**:
```rust
// story-generator/src/mcp_tools.rs

use rmcp::model::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateStructureParams {
    pub theme: String,
    pub language: Language,
    pub age_group: AgeGroup,
    pub node_count: u32,
}

pub fn create_generate_structure_tool() -> Tool {
    Tool {
        name: "generate_structure".to_string(),
        description: Some("Generate DAG structure for interactive story".to_string()),
        input_schema: schemars::schema_for!(GenerateStructureParams),
    }
}
```

#### 5. NATS Configuration

**Connection Setup** (TLS + NKey auth):
```rust
// shared-types/src/constants.rs

pub const NATS_DEFAULT_URL: &str = "nats://localhost:5222";
pub const NATS_TLS_CA_CERT_PATH: &str = "./certs/ca.pem";
pub const NATS_TLS_CLIENT_CERT_PATH: &str = "./certs/client-cert.pem";
pub const NATS_TLS_CLIENT_KEY_PATH: &str = "./certs/client-key.pem";

pub const MCP_PROMPT_HELPER: &str = "mcp.prompt.helper";
pub const MCP_STORY_GENERATE: &str = "mcp.story.generate";
pub const MCP_QUALITY_VALIDATE: &str = "mcp.quality.validate";
pub const MCP_CONSTRAINT_ENFORCE: &str = "mcp.constraint.enforce";

pub const PROMPT_HELPER_GROUP: &str = "prompt-helper-group";
pub const STORY_GENERATOR_GROUP: &str = "story-generator-group";
pub const QUALITY_CONTROL_GROUP: &str = "quality-control-group";
pub const CONSTRAINT_ENFORCER_GROUP: &str = "constraint-enforcer-group";
```

**Configuration Files** (per-service):
```toml
# story-generator/config.toml

[nats]
url = "nats://localhost:5222"
subject = "mcp.story.generate"
queue_group = "story-generator-group"
tls_enabled = true
tls_ca_cert_path = "./certs/ca.pem"
tls_client_cert_path = "./certs/client-cert.pem"
tls_client_key_path = "./certs/client-key.pem"

[llm]
provider = "lm-studio"
base_url = "http://127.0.0.1:1234"
model = "mistral-7b-instruct"
temperature = 0.7
max_tokens = 2000
```

#### 6. NATS-CLI (Manual Testing Tool)

**Purpose**: Send envelope-wrapped MCP requests manually for testing.

```bash
# Example: Call prompt helper to generate story prompt
nats-cli send \
  --subject mcp.prompt.helper \
  --tool-name generate_story_prompts \
  --arguments '{"theme":"space exploration","age_group":"9-11","language":"de"}' \
  --tenant taletrail-test \
  --timeout 30

# nats-cli wraps the arguments in:
# 1. CallToolRequest (MCP)
# 2. McpData { tool_call: Some(...) }
# 3. Envelope<McpData> with Meta { tenant, request_id, ... }
# 4. Serialize and send via NATS request/reply

# Response is automatically deserialized and pretty-printed
```

### Component Interaction Flow

**End-to-End Request Flow with Envelope Propagation**:

```
1. HTTP Gateway receives external request
   ├─ Extract JWT token → tenant_id
   ├─ Create initial Meta { tenant, request_id }
   └─ Forward to Orchestrator via NATS

2. Orchestrator: Phase 0.5 - Generate Prompts (parallel)
   ├─ Create Meta with extensions { phase: PromptGeneration, correlation_id }
   ├─ Build 4 MCP CallToolRequests (story, validation, constraint, model)
   ├─ Wrap each in Envelope<McpData>
   └─ Send to mcp.prompt.helper (4 concurrent requests)
       │
       ▼
   Prompt Helper MCP Server
       ├─ Deserialize Envelope<McpData>
       ├─ Extract tenant_id from meta.tenant
       ├─ Extract phase from meta.extensions
       ├─ Execute tool (call LLM via rig-core)
       ├─ Wrap result in CallToolResult
       ├─ Create response Envelope<McpData>
       └─ Return via NATS reply
       │
       ▼
   Orchestrator receives 4 responses
       └─ Aggregate prompts

3. Orchestrator: Phase 1 - Generate Structure
   ├─ Update Meta { phase: Structure, correlation_id (same) }
   ├─ Build CallToolRequest("generate_structure")
   ├─ Wrap in Envelope<McpData>
   └─ Send to mcp.story.generate
       │
       ▼
   Story Generator MCP Server
       ├─ Deserialize Envelope<McpData>
       ├─ Extract metadata (tenant, phase, correlation_id)
       ├─ Execute generate_structure tool
       ├─ Create response Envelope with DagStructure
       └─ Return via NATS
       │
       ▼
   Orchestrator receives DagStructure

4. Orchestrator: Phase 2 - Generate Content (batched)
   ├─ Update Meta { phase: Generation, batch_id }
   ├─ Split 16 nodes into batches (e.g., 4 batches of 4 nodes)
   ├─ For each batch:
   │   ├─ Create CallToolRequest("generate_content_nodes", nodes: [1,2,3,4])
   │   ├─ Wrap in Envelope<McpData>
   │   └─ Send to mcp.story.generate
   └─ Await all batches concurrently (Semaphore limits concurrency)
       │
       ▼
   Story Generator processes batches in parallel
       └─ Returns 16 generated nodes (ContentNode[])

5. Orchestrator: Phase 3 - Validation (parallel)
   ├─ Update Meta { phase: Validation, correlation_id }
   ├─ Create 2 requests (quality + constraints)
   └─ Send concurrently via tokio::join!
       │
       ├─────────────────────┬──────────────────────┐
       ▼                     ▼                      ▼
   Quality Control    Constraint Enforcer
       │                     │
       └─────────────────────┴──────────────────────┘
                             │
                             ▼
   Orchestrator receives ValidationResult + ConstraintResult

6. Orchestrator: Phase 4 - Negotiation (if needed)
   ├─ Analyze failures/violations
   ├─ Determine correction strategy
   └─ Re-run failed operations with updated metadata

7. Orchestrator: Phase 5 - Assembly
   ├─ Update Meta { phase: Assembly, correlation_id }
   ├─ Combine structure + nodes + metadata
   └─ Return final Trail to Gateway

8. HTTP Gateway
   ├─ Wrap Trail in HTTP response
   └─ Return to client

Metadata propagates through entire pipeline:
  - tenant_id: Consistent multi-tenancy
  - correlation_id: Distributed tracing
  - generation_phase: Pipeline stage tracking
  - request_id: Per-request identification
```

### Key Benefits Demonstrated

1. **Protocol Abstraction**: Same envelope structure works across NATS, MCP, REST
2. **Metadata Consistency**: Tenant, tracing, custom data flows automatically
3. **Type Safety**: Schema-driven code generation prevents runtime errors
4. **Observability**: correlation_id enables end-to-end request tracking
5. **Security**: TLS + NKey auth + tenant isolation via envelope
6. **Extensibility**: Custom metadata (TaleTrailCustomMetadata) without breaking Qollective
7. **Scalability**: NATS queue groups enable horizontal scaling of MCP servers
8. **Testability**: nats-cli allows manual envelope testing without full system

---

## Summary

**Qollective** provides envelope-first architecture that unifies metadata across protocols. **TaleTrail** demonstrates this by:

- Wrapping MCP tool calls in `Envelope<McpData>` with custom metadata extensions
- Propagating tenant context automatically across 4 MCP servers via NATS
- Using correlation_id for distributed tracing across 6 pipeline phases
- Extending `Meta.extensions` with TaleTrail-specific metadata (phase, batch_id, correlation_id)
- Abstracting NATS transport behind Qollective's envelope pattern

**For students**: Focus on how the envelope pattern eliminates protocol-specific metadata handling and enables consistent context propagation across heterogeneous services.
