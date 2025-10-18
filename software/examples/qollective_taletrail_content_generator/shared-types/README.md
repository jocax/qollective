# Shared Types for TaleTrail Content Generator

This crate provides common types, utilities, and protocols shared across all TaleTrail services.

## Envelope-First Architecture

TaleTrail follows Qollective's envelope-first pattern:

- Use `Envelope<TaleTrailPayload>` directly (not custom wrappers)
- Extend metadata via `Meta.extensions` using `TaleTrailCustomMetadata`
- Use helper functions in `helpers` module for ergonomic envelope creation

## MCP Tool Registration & Discovery Protocol

TaleTrail implements a comprehensive tool discovery protocol that allows the orchestrator to verify service availability and tool inventory before execution.

### Discovery Flow

1. **Orchestrator Startup**: On startup, orchestrator sends discovery requests to all services
2. **Service Response**: Each service responds with `DiscoveryInfo` containing available tools
3. **Validation**: Orchestrator validates that required tools are present
4. **Pre-flight Check**: If critical tools missing, orchestrator fails fast with clear error
5. **Caching**: Discovery results are cached for 5 minutes (configurable via `DISCOVERY_CACHE_TTL_SECS`)

### Discovery Request

The orchestrator sends a discovery request to each service's discovery endpoint:

**NATS Subject Pattern**: `mcp.discovery.list_tools.{service_name}`

Examples:
- `mcp.discovery.list_tools.story-generator`
- `mcp.discovery.list_tools.quality-control`
- `mcp.discovery.list_tools.constraint-enforcer`
- `mcp.discovery.list_tools.prompt-helper`

**Request Envelope**:
```json
{
  "meta": {
    "envelope_id": "uuid-v4",
    "timestamp": "2025-10-18T15:00:00Z",
    "tenant_id": 1,
    "correlation_id": "discovery-correlation-id"
  },
  "payload": {
    "discovery_data": {
      "query_type": "list_tools"
    }
  }
}
```

### Discovery Response

Services respond with `DiscoveryInfo` containing their tool inventory:

**Response Envelope**:
```json
{
  "meta": {
    "envelope_id": "uuid-v4",
    "timestamp": "2025-10-18T15:00:00Z",
    "tenant_id": 1,
    "correlation_id": "discovery-correlation-id"
  },
  "payload": {
    "discovery_data": {
      "query_type": "list_tools_response",
      "tools": [
        {
          "tool_name": "generate_structure",
          "tool_schema": {
            "type": "object",
            "properties": {
              "theme": {"type": "string"},
              "language": {"type": "string", "enum": ["en", "de"]},
              "node_count": {"type": "integer", "minimum": 8}
            },
            "required": ["theme", "language"]
          },
          "service_name": "story-generator",
          "service_version": "0.0.1",
          "capabilities": ["batching", "retry"]
        }
      ],
      "server_info": {
        "service_health": "healthy",
        "uptime_seconds": 3600,
        "available_tools": 3
      }
    }
  }
}
```

### Tool Registration Schema Structure

#### `ToolRegistration`

Complete metadata about a single MCP tool:

```rust
pub struct ToolRegistration {
    /// Tool name (matches rmcp Tool.name field)
    pub tool_name: String,

    /// JSON Schema describing tool parameters
    pub tool_schema: JsonValue,

    /// Service providing this tool
    pub service_name: String,

    /// Service version (semantic versioning)
    pub service_version: String,

    /// Service capabilities for this tool
    pub capabilities: Vec<ServiceCapabilities>,
}
```

#### `DiscoveryInfo`

Complete service discovery response:

```rust
pub struct DiscoveryInfo {
    /// All tools available from this service
    pub available_tools: Vec<ToolRegistration>,

    /// Current service health status
    pub service_health: String,

    /// Service uptime in seconds
    pub uptime_seconds: u64,
}
```

#### `ServiceCapabilities`

Indicates advanced features a service supports:

```rust
pub enum ServiceCapabilities {
    /// Supports batch processing of multiple requests
    Batching,

    /// Supports streaming responses
    Streaming,

    /// Supports response caching
    Caching,

    /// Supports automatic retry on failure
    Retry,
}
```

### Service Tool Inventory

#### Story Generator

- **Tool**: `generate_structure`
  - **Capabilities**: Batching, Retry
  - **Description**: Generates DAG structure with convergence points

- **Tool**: `generate_nodes`
  - **Capabilities**: Batching, Retry
  - **Description**: Generates content for story nodes

- **Tool**: `validate_paths`
  - **Capabilities**: Retry
  - **Description**: Validates DAG path consistency

#### Quality Control

- **Tool**: `validate_content`
  - **Capabilities**: Batching, Retry
  - **Description**: Validates content quality and age-appropriateness

- **Tool**: `batch_validate`
  - **Capabilities**: Batching, Retry
  - **Description**: Batch validation of multiple nodes

- **Tool**: `suggest_corrections`
  - **Capabilities**: Retry
  - **Description**: Provides correction suggestions

#### Constraint Enforcer

- **Tool**: `enforce_constraints`
  - **Capabilities**: Batching, Retry
  - **Description**: Validates constraints (vocabulary, theme consistency)

- **Tool**: `check_educational_goals`
  - **Capabilities**: Retry
  - **Description**: Validates educational goal compliance

#### Prompt Helper

- **Tool**: `generate_story_prompt`
  - **Capabilities**: Caching, Retry
  - **Description**: Generates prompts for story generation

- **Tool**: `generate_validation_prompt`
  - **Capabilities**: Caching, Retry
  - **Description**: Generates prompts for validation

- **Tool**: `generate_constraint_prompt`
  - **Capabilities**: Caching, Retry
  - **Description**: Generates prompts for constraint checking

- **Tool**: `get_model_for_language`
  - **Capabilities**: Caching
  - **Description**: Returns appropriate LLM model for language

### Health Check Protocol

In addition to tool discovery, services implement health check endpoints.

**NATS Subject Pattern**: `mcp.discovery.health.{service_name}`

**Health Response**:
```json
{
  "meta": {
    "envelope_id": "uuid-v4",
    "timestamp": "2025-10-18T15:00:00Z"
  },
  "payload": {
    "discovery_data": {
      "query_type": "health_response",
      "server_info": {
        "service_health": "healthy",
        "uptime_seconds": 3600,
        "tools_count": 3,
        "tools": ["generate_structure", "generate_nodes", "validate_paths"]
      }
    }
  }
}
```

### Pre-flight Check

The orchestrator performs a pre-flight check on startup:

1. **Discover All Services**: Send discovery requests to all 4 services
2. **Validate Required Tools**:
   - story-generator: `generate_structure`, `generate_nodes`
   - quality-control: `validate_content`
   - constraint-enforcer: `enforce_constraints`
   - prompt-helper: `generate_story_prompts`

3. **Log Discovery Results**: Log all discovered tools at INFO level
4. **Fail Fast**: If critical tools missing, return error and prevent startup
5. **Warn Optional**: Log warnings for missing optional tools

### Configuration

Discovery behavior is controlled by constants in `shared-types/src/constants.rs`:

```rust
/// MCP discovery subject for listing all tools
pub const MCP_DISCOVERY_LIST_TOOLS: &str = "mcp.discovery.list_tools";

/// MCP discovery subject for health checks
pub const MCP_DISCOVERY_HEALTH: &str = "mcp.discovery.health";

/// MCP discovery subject for service-specific info
pub const MCP_DISCOVERY_SERVICE_INFO: &str = "mcp.discovery.service";

/// Discovery cache TTL in seconds (5 minutes)
pub const DISCOVERY_CACHE_TTL_SECS: u64 = 300;
```

## Configuration Management

Configuration is handled per-service using Figment with the following hierarchy:

1. **Defaults** (hardcoded in constants.rs)
2. **config.toml** (service-specific configuration file)
3. **Environment variables** (highest priority)

## Testing

Run the tool registration tests:

```bash
cargo test --package shared-types --test tool_registration_tests
```

All types support:
- JSON serialization/deserialization
- JSON Schema generation (via schemars derive)
- Comprehensive validation
