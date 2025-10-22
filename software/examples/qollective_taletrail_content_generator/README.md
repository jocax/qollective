# TaleTrail Content Generator

A distributed AI content generation system built with Qollective's multi-protocol transport framework, demonstrating MCP orchestration for creating interactive educational story DAGs.

## Overview

TaleTrail generates complete interactive story graphs (Directed Acyclic Graphs) with multiple narrative paths and convergence points for children's educational content. The system uses Model Context Protocol (MCP) servers coordinated via TLS-secured NATS messaging.

### Architecture

```
┌──────────────────────┐
│   Gateway            │  HTTPS API (port 8443)
│  (Qollective Rest)   │
└──────┬───────────────┘
       │
       ↓ NATS (TLS: port 5222)
┌─────────────────────────────────────────┐
│                                         │
│         MCP Orchestrator Client         │
│    (Coordinates generation pipeline)    │
│                                         │
└──┬──────────┬──────────┬───────────────┘
   │          │          │
   ↓          ↓          ↓
┌────────┐ ┌────────┐ ┌────────────┐
│ Story  │ │Quality │ │Constraint  │
│Generator│ │Control │ │ Enforcer   │
│(MCP)   │ │(MCP)   │ │  (MCP)     │
└────────┘ └────────┘ └────────────┘
```

### Components

1. **Gateway** - HTTP API server providing RESTful TLS endpoints via Qollective
2. **Orchestrator** - Coordinates the multi-phase generation pipeline with negotiation protocol
3. **Story Generator** - MCP server creating DAG structure and generating narrative content
4. **Quality Control** - MCP server validating age-appropriateness, safety, and educational value
5. **Constraint Enforcer** - MCP server ensuring vocabulary, theme consistency, and required elements
6. **Prompt Helper** - MCP server generating language-appropriate prompts dynamically
7. **Shared Types** - Common types, constants, and error handling

## Schema-First Development

All types are auto-generated from `schemas/taletrail-content-generator.json` (759 lines, 44 types).

**Regenerate types after schema changes:**
```bash
./regenerate-types.sh
```

**Manual generation:**
```bash
# Validate
cargo run -p generator -- validate schemas/taletrail-content-generator.json --lint

# Generate
cargo run -p generator -- generate schemas/taletrail-content-generator.json \
  --output shared-types/src/generated --format single-file --force
```

**Documentation:** See `SCHEMA.md` for complete details on schema structure, generated types, and extension workflow

## Trait-Based Testability Architecture

TaleTrail uses **dependency injection** with service traits to enable comprehensive unit testing with mocks.

### Service Traits

All service implementations must implement these traits (defined in `shared-types/src/traits/`):

1. **LlmService** - LLM interaction for prompt generation and content generation
2. **PromptHelperService** - Generate language-appropriate prompts dynamically
3. **McpTransport** - MCP tool invocation across NATS
4. **RequestMapper** - External API ↔ Internal API transformation
5. **StoryGeneratorService** - DAG structure and content generation
6. **ValidationService** - Quality and constraint validation

### Testing Pattern

**Production code** uses real implementations:
```rust
let llm_service = Box::new(RigLlmService::new(config));
let prompt_helper = LlmPromptHelper::new(llm_service, templates, models, config);
```

**Test code** uses mocks:
```rust
let mut mock_llm = MockLlmService::new();
mock_llm
    .expect_generate_prompt()
    .returning(|_, _| Ok(("system".into(), "user".into())));
let prompt_helper = LlmPromptHelper::new(Box::new(mock_llm), templates, models, config);
```

**Run tests with mocking feature:**
```bash
cargo test -p shared-types --features mocking
```

### Benefits

- ✅ **Unit tests** without external dependencies (no LLM, no NATS)
- ✅ **Deterministic** test behavior
- ✅ **Fast** test execution
- ✅ **Clear** interface boundaries between components

## MCP Tool Registration & Discovery

TaleTrail implements a comprehensive MCP tool registration and discovery protocol that enables the orchestrator to detect available services, validate their capabilities, and perform pre-flight health checks before pipeline execution.

### Discovery Protocol Overview

Services register their MCP tools at startup and respond to discovery requests via NATS subjects. The orchestrator uses this protocol to build a complete inventory of available tools before attempting generation workflows.

**NATS Discovery Subjects:**

- `mcp.discovery.list_tools.{service_name}` - Request tool inventory from a specific service
- `mcp.discovery.health.{service_name}` - Request health status from a specific service

**Discovery Flow:**

1. Orchestrator sends envelope-wrapped discovery request to service-specific subject
2. Service responds with ToolRegistration data for all registered tools
3. Orchestrator validates required tools are present
4. Orchestrator caches discovery results (5-minute TTL)
5. Pipeline execution proceeds with validated tool inventory

**ToolRegistration Data Structure:**

```rust
pub struct ToolRegistration {
    tool_name: String,           // Tool identifier (e.g., "generate_structure")
    tool_schema: JsonValue,      // JSON Schema for tool parameters
    service_name: String,        // Service providing this tool
    service_version: String,     // Semantic version (e.g., "0.0.1")
    capabilities: Vec<ServiceCapabilities>, // Service capabilities
}
```

### Service Capabilities

Each tool registration declares which advanced features the service supports. The orchestrator uses this information to optimize execution strategies.

**ServiceCapabilities Enum:**

- `Batching` - Supports batch processing of multiple requests in a single call
- `Streaming` - Supports streaming responses for progressive results
- `Caching` - Supports response caching to reduce redundant computations
- `Retry` - Supports automatic retry on transient failures

**Service-Specific Capabilities:**

| Service | Tools | Capabilities |
|---------|-------|-------------|
| story-generator | generate_structure, generate_nodes, validate_paths | Batching, Retry |
| quality-control | validate_content, batch_validate | Batching, Retry |
| constraint-enforcer | enforce_constraints | Batching, Retry |
| prompt-helper | generate_story_prompts | Caching, Retry |

### Orchestrator Discovery Flow

The orchestrator performs service discovery during startup to ensure all required services are available before accepting requests.

**Pre-Flight Check Process:**

1. **Parallel Discovery** - Send discovery requests to all 4 services concurrently
2. **Response Collection** - Gather tool registrations from each service
3. **Required Service Validation** - Verify critical services (story-generator, quality-control, constraint-enforcer) are present
4. **Optional Service Detection** - Detect prompt-helper presence for optimization
5. **Fail-Fast Behavior** - Exit immediately if required services are missing
6. **Graceful Degradation** - Continue with reduced functionality if optional services unavailable

**Discovery Caching:**

- Tool inventories cached for 5 minutes (300 seconds)
- Reduces NATS traffic and discovery latency
- Cache can be manually cleared for testing
- Automatic cache invalidation on TTL expiration

**Health Monitoring:**

```rust
pub struct DiscoveryInfo {
    available_tools: Vec<ToolRegistration>,
    service_health: String,     // "healthy" | "degraded"
    uptime_seconds: u64,        // Service uptime
}
```

### Health Checks

Services respond to health check requests with current status information.

**Health Check Protocol:**

1. Orchestrator sends health request to `mcp.discovery.health.{service_name}`
2. Service responds with DiscoveryInfo containing health status
3. Orchestrator evaluates service availability

**Health Response Format:**

- `service_health`: "healthy" or "degraded" status indicator
- `uptime_seconds`: Service uptime for monitoring
- `available_tools`: Current tool inventory (may differ if service degraded)

**Health Check Use Cases:**

- Pre-flight validation before pipeline execution
- Continuous monitoring during long-running workflows
- Circuit breaker pattern for service failure detection
- Load balancing decisions across service replicas

### Testing

The discovery protocol has comprehensive test coverage across contract and integration tests.

**Contract Tests** (`orchestrator/tests/contract_discovery.rs`):

- 8 contract tests validating schema consistency and serialization
- No running services required
- Tests JSON roundtrip for ToolRegistration, DiscoveryInfo, and ServiceCapabilities
- Validates envelope metadata preservation
- Ensures consistent schema structure across all registrations

**Integration Tests** (`orchestrator/tests/integration_discovery.rs`):

- 8 total integration tests (6 automated + 2 manual)
- **Automated tests** (requires running services):
  - `test_discover_all_services` - Validates all 4 services discovered correctly
  - `test_tool_registration_data_correctness` - Verifies tool schemas and metadata
  - `test_service_capabilities_match_expected` - Validates capability declarations
  - `test_discovery_caching_works` - Confirms cache reduces latency
  - `test_health_check_integration` - Validates health check protocol
  - `test_cache_ttl_expiration` - Verifies cache invalidation logic

- **Manual tests** (require stopping services):
  - `test_graceful_degradation_missing_optional_service` - Tests handling of missing prompt-helper
  - `test_fail_fast_missing_required_service` - Verifies fail-fast for missing story-generator

**Running Discovery Tests:**

```bash
# Contract tests (no services needed)
cargo test --package orchestrator --test contract_discovery

# Integration tests (requires running services)
# 1. Start NATS: ./start-nats.sh
# 2. Start all services:
cargo run -p story-generator &
cargo run -p quality-control &
cargo run -p constraint-enforcer &
cargo run -p prompt-helper &

# 3. Run automated integration tests:
cargo test --package orchestrator --test integration_discovery -- --ignored

# Manual tests require stopping specific services first
```

## Data Structures & Database Alignment

All data structures align with the existing TaleTrails database schema.

### Database Tables

- **`trails`** - Trail metadata (title, description, metadata JSON, tags, status, tenant_id)
- **`trail_steps`** - Ordered steps (step_order, title, description, metadata JSON, content_id, trail_id)
- **`content`** - Actual content (category='story', content JSON field holds interactive story nodes)

### Generated Types

The `schemas/taletrail-content-generator.json` schema defines types that map to these tables:

- **Trail** → `trails` table
- **TrailStep** → `trail_steps` table
- **Content** → `content.content` JSON field

### API Layers

**External API** (simplified, versioned):
- `ExternalGenerationRequestV1` - Minimal fields (theme, age_group, language)
- Gateway applies age-appropriate defaults and enrichment

**Internal MCP API** (complete, rich parameters):
- `GenerationRequest` - Full parameters (educational_goals, node_count, vocabulary_level, etc.)
- Used for MCP service communication

### Data Structure Documentation

For complete field definitions and relationships, see:
- `@.agent-os/specs/2025-10-02-taletrail-content-generator/sub-specs/data-structures.md`
- `schemas/taletrail-content-generator.json`

## Prerequisites

### Required Software

- Rust 1.75+
- Docker and Docker Compose
- OpenSSL (for certificate generation)
- NATS CLI (`nats`) - for NKey generation
- Apache htpasswd (`htpasswd`) - for monitoring authentication
- LM Studio (optional, for LLM integration in later phases)

### Security Architecture

TaleTrail implements **defense-in-depth** security with multiple layers:

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 1: TLS Encryption (All Traffic)                      │
│  ├─ HTTPS Gateway (port 8443)                               │
│  ├─ NATS TLS (port 5222)                                    │
│  └─ HTTPS Monitoring (port 9222)                            │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Authentication                                     │
│  ├─ NKey Cryptographic Authentication (NATS clients)        │
│  └─ Basic Auth (Monitoring dashboard)                       │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Authorization                                      │
│  └─ Subject-Level Permissions (per service)                 │
└─────────────────────────────────────────────────────────────┘
```

#### Security Features

**🔐 NKey Authentication**
- Ed25519 public-key cryptography for NATS client authentication
- No passwords or shared secrets transmitted
- Each service has unique cryptographic identity
- Server never sees or stores private keys

**🔒 TLS Encryption**
- All NATS traffic encrypted with TLS 1.2/1.3
- HTTPS for gateway API (port 8443)
- HTTPS for monitoring dashboard (port 9222)
- Per-service certificate architecture

**🛡️ Subject-Level Authorization**
- Principle of least privilege per service
- Story Generator: Can only publish stories and subscribe to requests
- Quality Control: Can only validate and publish results
- Constraint Enforcer: Can only enforce constraints
- Orchestrator: Full coordination access
- Gateway: Can only send requests and receive events

**🚫 Monitoring Security**
- HTTPS with TLS encryption
- Basic Authentication (username/password)
- No public internet exposure
- Unauthenticated health checks only

#### Certificate & Key Architecture

| Component | Authentication | Certificate/Key | Purpose |
|-----------|---------------|-----------------|---------|
| NATS Server | N/A | `server-cert.pem` | TLS encryption |
| Gateway HTTPS | N/A | `gateway-cert.pem` | External API TLS |
| Story Generator | NKey | `story-generator.nk` | NATS authentication |
| Quality Control | NKey | `quality-control.nk` | NATS authentication |
| Constraint Enforcer | NKey | `constraint-enforcer.nk` | NATS authentication |
| Orchestrator | NKey | `orchestrator.nk` | NATS authentication |
| Gateway NATS | NKey | `gateway.nk` | NATS authentication |
| Monitoring Proxy | Basic Auth | `.htpasswd` | Dashboard access |

### Quick Setup (5 Steps)

#### 1. Generate TLS Certificates
```bash
cd certs
./setup-tls.sh
cd ..
```

This generates 4 certificate pairs (8 files total):
- CA certificate authority (ca.pem, ca-key.pem)
- NATS server certificate (server-cert.pem, server-key.pem)
- Gateway HTTPS certificate (gateway-cert.pem, gateway-key.pem)
- Client certificate (client-cert.pem, client-key.pem) [Not used with NKey auth]

#### 2. Setup Security (NKeys + Basic Auth)
```bash
./setup-security.sh
```

This automated script:
- ✅ Generates NKey pairs for all 5 services
- ✅ Creates Basic Auth credentials for monitoring
- ✅ Sets proper file permissions (600 for private keys)
- ✅ Displays public keys for nats-server.conf

**Monitoring Credentials:**
- URL: https://localhost:9222
- Username: `admin`
- Password: (you'll set this during setup)

#### 3. Update NATS Configuration (Already Done!)

The `nats-server.conf` file already contains the NKey public keys from setup-security.sh. No manual editing needed!

**Authorization Configuration:**
```toml
authorization {
  users: [
    { nkey: "UC..." permissions: { publish: ["mcp.story.>"] subscribe: ["mcp.story.generate"] } }
    { nkey: "UB..." permissions: { publish: ["mcp.quality.>"] subscribe: ["mcp.quality.validate"] } }
    # ... (3 more services)
  ]
}
```

#### 4. Start NATS with Security
```bash
./start-nats.sh
```

Or manually:
```bash
docker-compose up -d
```

This starts:
- **NATS Server** - Port 5222 with NKey authentication
- **Nginx Monitoring Proxy** - Port 9222 with HTTPS + Basic Auth

#### 5. Verify Security

**Test NATS Health:**
```bash
# Unauthenticated health check (allowed)
curl --insecure https://localhost:9222/healthz
# Should return: OK
```

**Test Monitoring Dashboard:**
```bash
# Requires authentication
open https://localhost:9222
# Login with: admin / <your-password>
```

**View Monitoring Dashboard:**
- Connection statistics
- Message throughput
- JetStream status
- Client connections (should show 5 authenticated services)

### Port Information

- **5222**: NATS TLS client connections (NKey authentication required)
- **9222**: NATS HTTPS monitoring (Basic Auth: admin/<password>)
- **8443**: Gateway HTTPS API (TLS-enabled)

## Configuration

Copy the example environment file:

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```bash
# NATS Configuration (TLS-enabled)
NATS_URL=nats://localhost:5222
NATS_TLS_CA_CERT=./certs/ca.pem
NATS_TLS_CLIENT_CERT=./certs/client-cert.pem
NATS_TLS_CLIENT_KEY=./certs/client-key.pem
NATS_MONITOR_URL=http://localhost:9222

# LM Studio Configuration (Phase 2+)
LM_STUDIO_URL=http://127.0.0.1:1234
LM_STUDIO_MODEL=local-model

# JWT Secret for authentication (Phase 5+)
JWT_SECRET=your-secret-change-in-production
```

### Service-Specific Configuration

Each service has its own `config.toml`:

- `story-generator/config.toml` - NATS subject: `mcp.story.generate`
- `quality-control/config.toml` - NATS subject: `mcp.quality.validate`
- `constraint-enforcer/config.toml` - NATS subject: `mcp.constraint.enforce`
- `gateway/config.toml` - HTTP server settings

## Build Instructions

Build all workspace members:

```bash
./build.sh
```

Or build individual services:

```bash
cargo build -p shared-types
cargo build -p orchestrator
cargo build -p story-generator
cargo build -p quality-control
cargo build -p constraint-enforcer
cargo build -p gateway
```

## Running Services

**Important**: Start NATS first!

```bash
# 1. Start NATS with TLS
./start-nats.sh

# 2. In separate terminals, start each service:

# Terminal 1: Story Generator
cd story-generator && cargo run

# Terminal 2: Quality Control
cd quality-control && cargo run

# Terminal 3: Constraint Enforcer
cd constraint-enforcer && cargo run

# Terminal 4: Orchestrator
cd orchestrator && cargo run

# Terminal 5: Gateway
cd gateway && cargo run
```

### Testing

Test the HTTPS health endpoint with envelope-wrapped response:

```bash
curl --insecure -X POST https://localhost:8443/health \
  -H "Content-Type: application/json" -d 'null'
```

Expected response (envelope-wrapped):
```json
{
  "meta": {
    "timestamp": "2025-10-06T19:15:05.277325Z"
  },
  "payload": {
    "status": "healthy",
    "service": "taletrail-gateway",
    "version": "0.1.0",
    "timestamp": "2025-10-06T19:15:05.277310+00:00"
  }
}
```

**Note:** The `--insecure` flag is needed for self-signed certificates in development.

Or use the IntelliJ HTTP client file:
```bash
# Open http-tests/health.http in IntelliJ IDEA
```

## Development Status

### Phase 0: Project Scaffolding ✅ **COMPLETE** (2025-10-06)

**Summary:** Complete TaleTrail workspace with TLS-enabled infrastructure and Qollective REST gateway

**Achievements:**
- ✅ Complete workspace structure with 6 crates (shared-types, orchestrator, 3 MCP services, gateway)
- ✅ **Defense-in-depth security architecture**:
  - NKey cryptographic authentication for all NATS clients
  - TLS encryption for all traffic (NATS, Gateway, Monitoring)
  - Subject-level authorization with least privilege per service
  - Basic Auth + HTTPS for monitoring dashboard
  - Nginx reverse proxy for monitoring security
- ✅ **Per-service TLS architecture** - separate certificates for NATS, Gateway
- ✅ **TLS-enabled Gateway (HTTPS)** with Qollective REST Server on port 8443
- ✅ TLS-enabled NATS infrastructure with Docker Compose (ports 5222/9222)
- ✅ **Gateway envelope-first architecture** - all responses wrapped in UnifiedEnvelope
- ✅ All services compile successfully with rmcp 0.8.0
- ✅ CONSTANTS FIRST principle - all values in `shared-types/src/constants.rs`
- ✅ Configuration inheritance pattern across all services
- ✅ Production-ready certificate generation and NKey management
- ✅ Automated security setup script (`setup-security.sh`)
- ✅ Verified HTTPS health endpoint with envelope-wrapped responses

**Key Milestones:**
1. Fixed qollective library rmcp 0.8 integration (6 CallToolResult + 3 Implementation fixes)
2. Redesigned gateway from basic Axum to Qollective REST Server
3. All services use TLS NATS connection with proper certificate loading
4. HTTP health test demonstrates envelope-first architecture
5. Build infrastructure (build.sh, test.sh) and comprehensive README

**Gateway Test Result (HTTPS with TLS):**
```bash
curl --insecure -X POST https://localhost:8443/health -H "Content-Type: application/json" -d 'null'
# Response (envelope-wrapped):
{
  "meta": {"timestamp": "2025-10-06T19:15:05.277325Z"},
  "payload": {
    "status": "healthy",
    "service": "taletrail-gateway",
    "version": "0.1.0",
    "timestamp": "2025-10-06T19:15:05.277310+00:00"
  }
}

# Certificate: gateway-cert.pem (CN=taletrail-gateway)
# Per-service TLS: Not shared with NATS server
```

**Next:** Phase 1 - Foundation Implementation (Data structures, envelopes, error handling)

### Phase 1: Foundation Implementation (Next)

- Data structures: ContentNode, DAG, GenerationRequest
- TaleTrailEnvelope implementation
- Comprehensive error handling
- Full constants definition
- Unit tests

### Future Phases

- Phase 2: MCP Story Generator Server
- Phase 3: Quality Control & Constraint Enforcer Servers
- Phase 4: MCP Orchestrator Client
- Phase 5: HTTP API Gateway
- Phase 6: Documentation & Finalization

## DAG Configuration

TaleTrail uses a **two-tier DAG configuration model** that balances simplicity with advanced control. Users can choose between predefined presets (Tier 1) for common use cases or provide complete custom DAG configurations (Tier 2) for advanced scenarios.

### Two-Tier Model Overview

**Tier 1: Story Structure Presets** - Simple, predefined configurations
- Use `story_structure` field in request JSON
- 4 presets covering common storytelling patterns
- No manual parameter tuning required
- Ideal for most educational content

**Tier 2: Custom DAG Configuration** - Full parameter control
- Use `dag_config` object in request JSON
- 5 parameters for complete DAG specification
- Fine-grained control over story structure
- Ideal for specialized content or experimentation

**Tier 3: Orchestrator Defaults** - Fallback configuration
- Defined in `orchestrator/config.toml`
- Used when neither preset nor custom config provided
- Provides sensible defaults for basic story generation

### Tier 1: Story Structure Presets

Predefined presets optimized for different storytelling styles:

| Preset | Nodes | Pattern | Convergence | Max Depth | Branching | Use Case |
|--------|-------|---------|-------------|-----------|-----------|----------|
| `guided` | 12 | SingleConvergence | 50% (node 6) | 8 | 2 | Linear story with one major turning point |
| `adventure` | 16 | MultipleConvergence | 60% intervals | 10 | 2 | Branching story with multiple convergence points |
| `epic` | 24 | EndOnly | 90% (node ~21) | 12 | 2 | Complex branching that converges at climax |
| `choose_your_path` | 16 | PureBranching | None | 10 | 3 | Pure choice tree with multiple endings |

**Example Request with Preset:**
```json
{
  "theme": "Ocean Adventure with Marine Biology",
  "age_group": "9-11",
  "language": "en",
  "educational_goals": ["ocean ecosystem", "marine life"],
  "vocabulary_level": "intermediate",
  "story_structure": "guided"
}
```

### Tier 2: Custom DAG Configuration

For advanced control, specify all 5 DAG parameters directly:

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| `node_count` | integer | 4-100 | Total nodes in story DAG |
| `convergence_pattern` | enum | See patterns below | How story branches converge |
| `convergence_point_ratio` | float | 0.0-1.0 | Position of convergence as ratio (0.5 = midpoint) |
| `max_depth` | integer | 3-20 | Maximum depth of DAG tree structure |
| `branching_factor` | integer | 2-4 | Number of choices per decision node |

**Convergence Patterns:**
- `SingleConvergence` - One major convergence point (requires `convergence_point_ratio`)
- `MultipleConvergence` - Multiple convergence at intervals (requires `convergence_point_ratio`)
- `EndOnly` - Converges only at story climax (requires `convergence_point_ratio`)
- `PureBranching` - No convergence, multiple endings (no ratio needed)
- `ParallelPaths` - Parallel story tracks (no ratio needed)

**Example Request with Custom DAG:**
```json
{
  "theme": "Medieval Quest for the Sacred Artifact",
  "age_group": "12-14",
  "language": "en",
  "educational_goals": ["medieval history", "ethics", "strategy"],
  "vocabulary_level": "intermediate",
  "dag_config": {
    "node_count": 20,
    "convergence_pattern": "MultipleConvergence",
    "convergence_point_ratio": 0.33,
    "max_depth": 15,
    "branching_factor": 2
  }
}
```

### Configuration Priority

When multiple configuration sources are present, TaleTrail uses this priority order:

1. **Priority 1 (Highest)**: `story_structure` preset in request JSON
2. **Priority 2 (Middle)**: Custom `dag_config` object in request JSON
3. **Priority 3 (Lowest)**: Orchestrator defaults from `config.toml`

**Example - Preset Wins:**
```json
{
  "theme": "Space Exploration",
  "age_group": "9-11",
  "language": "en",
  "story_structure": "guided",
  "dag_config": {
    "node_count": 20,
    "convergence_pattern": "Epic"
  }
}
```
In this case, the `guided` preset configuration will be used, and `dag_config` will be logged but ignored.

### Testing with NATS-CLI

TaleTrail includes template files for testing each configuration approach:

**Test Preset Configurations:**
```bash
# Test guided preset (12 nodes, single convergence)
cargo run -p nats-cli -- send --template templates/orchestrator/request_guided.json

# Test adventure preset (16 nodes, multiple convergence)
cargo run -p nats-cli -- send --template templates/orchestrator/request_adventure.json

# Test epic preset (24 nodes, end-only convergence)
cargo run -p nats-cli -- send --template templates/orchestrator/request_epic.json

# Test choose_your_path preset (16 nodes, pure branching)
cargo run -p nats-cli -- send --template templates/orchestrator/request_choose_your_path.json
```

**Test Custom DAG Configuration:**
```bash
# Test custom DAG with 20 nodes and multiple convergence
cargo run -p nats-cli -- send --template templates/orchestrator/request_custom_dag.json
```

### Validation Rules

TaleTrail validates all DAG configurations before generation:

- **node_count**: Must be 4-100 (minimum for meaningful story, maximum for performance)
- **convergence_pattern**: Must be one of the 5 valid patterns
- **convergence_point_ratio**:
  - Required for `SingleConvergence`, `MultipleConvergence`, and `EndOnly`
  - Must be omitted for `PureBranching` and `ParallelPaths`
  - Must be between 0.0 and 1.0 when provided
- **max_depth**: Must be 3-20 (minimum for branching structure, maximum for complexity)
- **branching_factor**: Must be 2-4 (2-3 typical for readability, 4 for highly complex stories)

**Validation Errors:**
- Invalid preset name → Error message lists valid options
- Missing `convergence_point_ratio` → Validation error explains requirement
- Out-of-range values → Error specifies valid range
- Both preset and custom config → Warning logged, preset takes priority

For complete configuration details, validation rules, and migration guides, see `CONFIGURATION.md`.

## Troubleshooting

### NKey Authentication Failures

**Problem**: Services fail to connect with "authentication violation" errors

**Solutions**:
1. **Verify NKey files exist:**
   ```bash
   ls -l nkeys/*.nk nkeys/*.pub
   ```

2. **Check NKey permissions:**
   ```bash
   # Private keys should be 600 (owner read/write only)
   chmod 600 nkeys/*.nk
   ```

3. **Verify public keys in nats-server.conf match:**
   ```bash
   # Compare public keys
   cat nkeys/story-generator.pub
   grep "story-generator" nats-server.conf
   ```

4. **Regenerate NKeys if needed:**
   ```bash
   ./setup-security.sh
   # Then update nats-server.conf with new public keys
   docker-compose restart taletrail-nats
   ```

5. **Check NATS logs for auth errors:**
   ```bash
   docker-compose logs taletrail-nats | grep -i "auth"
   ```

### Monitoring Access Issues

**Problem**: Cannot access monitoring dashboard at https://localhost:9222

**Solutions**:
1. **Verify nginx proxy is running:**
   ```bash
   docker-compose ps nats-monitor-proxy
   ```

2. **Test Basic Auth credentials:**
   ```bash
   curl -u admin:<password> --insecure https://localhost:9222/varz
   ```

3. **Regenerate Basic Auth:**
   ```bash
   htpasswd -c nginx/.htpasswd admin
   docker-compose restart nats-monitor-proxy
   ```

4. **Check nginx logs:**
   ```bash
   docker-compose logs nats-monitor-proxy
   ```

### Subject Authorization Errors

**Problem**: Services get "permissions violation" when publishing/subscribing

**Solutions**:
1. **Check service permissions in nats-server.conf:**
   Each service should only have access to specific subjects.

2. **Example permission fix for story-generator:**
   ```toml
   # In nats-server.conf
   {
     nkey: "UCJEP5KGI..."  # story-generator public key
     permissions: {
       publish: { allow: ["mcp.story.response.>", "mcp.events.story.>"] }
       subscribe: { allow: ["mcp.story.generate", "mcp.orchestrator.story.>"] }
     }
   }
   ```

3. **Restart NATS after config changes:**
   ```bash
   docker-compose restart taletrail-nats
   ```

### NATS Connection Issues

**Problem**: Services fail to connect to NATS

**Solutions**:
1. Verify NATS is running: `docker-compose ps`
2. Check health: `curl --insecure https://localhost:9222/healthz`
3. View logs: `docker-compose logs -f taletrail-nats`
4. Restart NATS: `docker-compose restart taletrail-nats`

### TLS Certificate Problems

**Problem**: TLS verification errors

**Solutions**:
1. Regenerate certificates: `cd certs && ./setup-tls.sh && cd ..`
2. Verify certificate files exist:
   ```bash
   ls -l certs/*.pem
   ```
3. Check certificate validity:
   ```bash
   openssl x509 -in certs/server-cert.pem -text -noout
   ```

### Port Conflicts

**Problem**: Port already in use

**Solutions**:
1. NATS ports (5222, 9222): Stop conflicting services or change ports in `docker-compose.yml`
2. Gateway port (8080): Change `GATEWAY_PORT` in `.env` or `gateway/config.toml`

### Build Errors

**Problem**: Compilation failures

**Solutions**:
1. Clean build: `cargo clean && cargo build --workspace`
2. Update dependencies: `cargo update`
3. Verify Rust version: `rustc --version` (must be 1.75+)

## Architecture Principles

1. **CONSTANTS FIRST**: All hardcoded values in `shared-types/src/constants.rs`
2. **Configuration Inheritance**: Each service inherits config from parent via constructor parameters
3. **Envelope-First Architecture**: All communication wrapped in TaleTrailEnvelope
4. **Defense-in-Depth Security**: Multiple layers of security controls
   - Layer 1: TLS encryption for all traffic
   - Layer 2: NKey cryptographic authentication
   - Layer 3: Subject-level authorization (least privilege)
   - Layer 4: Monitoring protection (HTTPS + Basic Auth)
5. **Zero Trust Architecture**: Never trust, always verify
   - All services must authenticate with NKeys
   - All traffic encrypted with TLS
   - Subject-level permissions enforced
6. **Test-Driven**: Write tests before implementation for each component
7. **Build Verification**: Each phase ends with successful build and run verification

## License

MIT

## Acknowledgments

Built with:
- [Qollective](https://github.com/jocax/qollective) - Multi-protocol transport framework
- [rMCP](https://github.com/modelcontextprotocol/rmcp) - Model Context Protocol
- [Axum](https://github.com/tokio-rs/axum) - HTTP framework
- [NATS](https://nats.io/) - Cloud-native messaging system
- [Tokio](https://tokio.rs/) - Async runtime
