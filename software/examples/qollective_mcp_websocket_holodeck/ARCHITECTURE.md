# Holodeck 8-Component MCP Architecture Specification

## Overview

This document defines the comprehensive architecture for the Star Trek TNG Holodeck system, featuring 8 independent components communicating via Model Context Protocol (MCP) with Qollective envelope integration, rig-core LLM orchestration, and Tauri V2 desktop client.

## Architecture Principles

1. **Envelope-First Design**: All MCP communication wrapped in Qollective envelopes
2. **Component Isolation**: Each component runs independently with clear responsibilities  
3. **MCP Protocol First**: Standard MCP for inter-component communication
4. **LLM Agent Integration**: rig-core for OpenAI GPT-4 integration
5. **Transport Abstraction**: HybridTransportClient for protocol flexibility
6. **No Legacy Compatibility**: Clean slate implementation with modern patterns

## Component Definitions

### 1. holodeck-desktop (Tauri V2 Desktop Client)
- **Type**: MCP Consumer / Desktop Application
- **Technology**: Tauri V2 + React + TypeScript
- **Theme**: Enterprise Star Trek TNG visual design
- **Communication**: MCP client connecting to holodeck-coordinator
- **Responsibilities**:
  - User interface for holodeck experiences
  - Real-time display of story progression
  - Character interaction interface
  - Settings and configuration management
  - WebSocket connection to holodeck-storybook for live updates

### 2. holodeck-coordinator (MCP Orchestrator Hub)
- **Type**: MCP Client + Orchestrator
- **Port**: 8447
- **Technology**: Rust + HybridTransportClient + rmcp
- **Communication**: MCP client to all servers, REST to storybook
- **Responsibilities**:
  - Orchestrate all MCP server interactions
  - Manage story generation workflows
  - Coordinate character and environment creation
  - Handle safety protocol enforcement
  - Session state management
  - Error handling and recovery

### 3. holodeck-storybook (Story History Server) 
- **Type**: WebSocket + REST Server
- **Port**: 8080
- **Technology**: Rust + axum + tokio-tungstenite
- **Communication**: REST API + WebSocket events
- **Responsibilities**:
  - Persistent story state storage
  - Session history management
  - Real-time event streaming
  - Player decision tracking
  - Story completion metrics
  - WebSocket connections for live updates

### 4. holodeck-designer (Story Generation MCP Server)
- **Type**: MCP Server + LLM Integration
- **Port**: 8443
- **Technology**: Rust + rmcp + rig-core + OpenAI GPT-4
- **Communication**: MCP Service trait implementation
- **Responsibilities**:
  - Generate story templates from topics
  - Create scene descriptions and narratives
  - Design story branching and decision points
  - Generate dialogue for scenarios
  - Story structure optimization

### 5. holodeck-validator (Story Validation MCP Server)
- **Type**: MCP Server + JSON Schema Validation
- **Port**: 8444  
- **Technology**: Rust + rmcp + jsonschema + content validation
- **Communication**: MCP Service trait implementation
- **Responsibilities**:
  - Validate story templates against schemas
  - Check narrative consistency
  - Verify character interactions
  - Ensure story completability
  - Content appropriateness validation

### 6. holodeck-environment (Environment Generation MCP Server)
- **Type**: MCP Server + LLM Integration
- **Port**: 8445
- **Technology**: Rust + rmcp + rig-core + OpenAI GPT-4
- **Responsibilities**:
  - Generate 3D environment descriptions
  - Create interactive object placements
  - Design lighting and atmospheric conditions
  - Generate environmental sound effects
  - Physics simulation parameters

### 7. holodeck-safety (Safety Monitoring MCP Server)
- **Type**: MCP Server + Content Safety
- **Port**: 8446
- **Technology**: Rust + rmcp + content safety algorithms
- **Responsibilities**:
  - Monitor all generated content for safety
  - Enforce holodeck safety protocols
  - Content filtering and moderation
  - Emergency shutdown capabilities
  - Safety violation reporting

### 8. holodeck-character (Character AI MCP Server)
- **Type**: MCP Server + LLM Integration
- **Port**: 8448
- **Technology**: Rust + rmcp + rig-core + OpenAI GPT-4
- **Responsibilities**:
  - Generate character responses and dialogue
  - Maintain character personality consistency
  - Handle character interactions and relationships
  - Character behavior simulation
  - Voice pattern and speech generation

## Port Allocation Strategy

| Port | Component | Protocol | Purpose |
|------|-----------|----------|---------|
| 8080 | holodeck-storybook | WebSocket + REST | Story persistence and real-time events |
| 8443 | holodeck-designer | MCP over WebSocket/HTTP | Story generation services |
| 8444 | holodeck-validator | MCP over WebSocket/HTTP | Story validation services |
| 8445 | holodeck-environment | MCP over WebSocket/HTTP | Environment generation |
| 8446 | holodeck-safety | MCP over WebSocket/HTTP | Safety monitoring |
| 8447 | holodeck-coordinator | MCP Client Hub | Orchestration and coordination |
| 8448 | holodeck-character | MCP over WebSocket/HTTP | Character AI services |
| 4222 | NATS Server | NATS Protocol | Message bus (optional) |

## MCP Integration Patterns

### Service Implementation Pattern
Each MCP server implements the `rmcp::Service` trait:

```rust
#[async_trait]
impl rmcp::Service for HolodeckService {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult, ErrorData>;
    async fn list_tools(&self) -> Result<ListToolsResult, ErrorData>;
}
```

### Tool Registration Pattern
```rust
// holodeck-designer tools
- story.generate: Generate story template from topic
- scene.create: Create detailed scene description
- dialogue.generate: Generate character dialogue
- story.branch: Create story branching points

// holodeck-validator tools  
- story.validate: Validate story template
- scene.check: Check scene consistency
- content.moderate: Content appropriateness check

// holodeck-environment tools
- environment.create: Generate 3D environment
- lighting.setup: Configure scene lighting
- physics.configure: Set physics parameters

// holodeck-safety tools
- content.scan: Scan for inappropriate content
- safety.check: Verify safety protocols
- emergency.shutdown: Emergency stop

// holodeck-character tools
- character.initialize: Set up character AI
- dialogue.respond: Generate character response
- personality.maintain: Ensure character consistency
```

### Envelope Integration Pattern
All MCP communication uses Qollective envelopes:

```rust
// Creating envelope for MCP data
let mcp_data = McpData::with_tool_call(tool_call);
let envelope = Envelope::new(Meta::with_mcp_metadata(metadata), mcp_data);

// Sending via transport
let response: Envelope<McpData> = transport.send_envelope(&endpoint, envelope).await?;
```

## LLM Integration Architecture

### rig-core Integration Pattern
```rust
// Provider → Model → Agent pattern
let openai_client = openai::Client::from_env();
let agent = openai_client.agent("gpt-4")
    .preamble("You are a Star Trek holodeck story designer...")
    .tool(StoryGenerationTool)
    .build();

// Agent execution
let response = agent.prompt(&story_request).await?;
```

### Character AI Specialization
Each character has specialized agents:
- **Captain Picard**: Diplomatic, philosophical, command-focused
- **Data**: Logical, curious, technical analysis
- **Worf**: Security-minded, honorable, tactical
- **Geordi**: Engineering solutions, technical optimism
- **Deanna Troi**: Emotional intelligence, relationship guidance

## Communication Patterns

### 1. Desktop Client to Coordinator
- **Protocol**: MCP over internal Tauri transport
- **Pattern**: Request/Response for holodeck operations
- **Error Handling**: Envelope error propagation

### 2. Coordinator to MCP Servers
- **Protocol**: MCP with Qollective envelope wrapping
- **Transport**: HybridTransportClient (WebSocket/HTTP/gRPC/NATS)
- **Pattern**: Orchestrated tool calls with dependencies

### 3. Coordinator to Storybook
- **Protocol**: REST API for persistence, WebSocket for events
- **Pattern**: CRUD operations + real-time event streaming

### 4. MCP Servers to OpenAI
- **Protocol**: rig-core agent calls
- **Pattern**: Provider → Model → Agent → Tools

### 5. Cross-Server Validation
- **Pattern**: Server-to-server MCP calls for validation
- **Flow**: Designer → Validator, Environment → Safety, Character → Safety

## Data Flow Scenarios

### Story Generation Workflow
1. Desktop client requests story creation (topic: "Diplomatic mission to Risa")
2. Coordinator calls holodeck-designer via MCP
3. Designer uses GPT-4 via rig-core to generate story template
4. Coordinator calls holodeck-validator to validate story
5. If valid, Coordinator calls holodeck-environment for environment setup
6. Coordinator calls holodeck-character to initialize characters
7. Coordinator calls holodeck-safety for final approval
8. Coordinator saves complete holodeck to storybook
9. Desktop client receives ready notification

### Real-time Interaction Flow
1. Player interacts with character through desktop client
2. Coordinator routes interaction to holodeck-character
3. Character server generates response using character-specific GPT-4 agent
4. Response validated by holodeck-safety
5. Character response sent back through coordinator to desktop
6. Interaction logged to storybook with WebSocket event
7. All connected clients receive real-time updates

## Error Handling Strategy

### QollectiveError Integration
- All MCP servers use feature-gated `QollectiveError::McpError`
- Automatic conversion from `rmcp::ErrorData`
- Proper error context preservation through envelopes

### Error Recovery Patterns
- **Service Unavailable**: Retry with exponential backoff
- **Content Safety Violation**: Automatic story regeneration
- **Validation Failure**: Iterative improvement loops
- **LLM API Failure**: Fallback to cached responses

## Security Considerations

### Content Safety Pipeline
1. All generated content flows through holodeck-safety
2. Multi-layer filtering: profanity, violence, inappropriate content
3. Configurable safety levels per user/session
4. Emergency shutdown capabilities
5. Audit logs for all safety decisions

### Transport Security
- TLS encryption for all HTTP/WebSocket communication
- NATS authentication and authorization
- Envelope metadata integrity verification
- Rate limiting and DDoS protection

## Performance Requirements

### Latency Targets
- Story generation: < 30 seconds
- Character response: < 3 seconds  
- Environment setup: < 15 seconds
- Safety validation: < 1 second
- Real-time updates: < 100ms

### Scalability Patterns
- Horizontal scaling of MCP servers
- Connection pooling in coordinator
- Caching of generated content
- Load balancing across LLM providers

## Development Standards

### Constants Management
All ports, timeouts, and subjects defined in `constants.rs`:
```rust
pub mod network {
    pub const HOLODECK_COORDINATOR_PORT: u16 = 8447;
    pub const HOLODECK_STORYBOOK_PORT: u16 = 8080;
    // ... etc
}
```

### Testing Requirements
- Unit tests for all MCP service implementations
- Integration tests for complete workflows
- Single-threaded test execution: `cargo test -- --test-threads=1`
- Envelope-based testing patterns

### Validation Gates
1. **Level 1**: `cargo fmt`, `cargo clippy`
2. **Level 2**: Unit tests with full coverage
3. **Level 3**: Integration test of complete user journey

This architecture provides a robust, scalable foundation for creating immersive Star Trek holodeck experiences with modern Rust patterns, comprehensive MCP integration, and advanced LLM orchestration.