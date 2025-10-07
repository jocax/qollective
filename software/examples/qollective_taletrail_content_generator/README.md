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
