# TaleTrail Content Generator

A distributed AI content generation system built with Qollective's multi-protocol transport framework, demonstrating MCP orchestration for creating interactive educational story DAGs.

## Overview

TaleTrail generates complete interactive story graphs (Directed Acyclic Graphs) with multiple narrative paths and convergence points for children's educational content. The system uses Model Context Protocol (MCP) servers coordinated via TLS-secured NATS messaging.

### Architecture

```
┌─────────────┐
│   Gateway   │  HTTP API (port 8080)
│  (Axum)     │
└──────┬──────┘
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

1. **Gateway** - HTTP API server providing RESTful endpoints
2. **Orchestrator** - Coordinates the multi-phase generation pipeline with negotiation protocol
3. **Story Generator** - MCP server creating DAG structure and generating narrative content
4. **Quality Control** - MCP server validating age-appropriateness, safety, and educational value
5. **Constraint Enforcer** - MCP server ensuring vocabulary, theme consistency, and required elements
6. **Shared Types** - Common types, constants, and error handling

## Prerequisites

### TLS-Enabled Infrastructure

TaleTrail demonstrates Qollective's production-ready TLS capabilities with **per-service certificates** for security isolation.

#### Certificate Architecture

Each service has dedicated certificates following security best practices:

| Service | Certificate | Common Name | Purpose |
|---------|-------------|-------------|---------|
| NATS Server | `server-cert.pem` | taletrail-nats | Internal messaging |
| Gateway HTTPS | `gateway-cert.pem` | taletrail-gateway | External API |
| MCP Services | `client-cert.pem` | taletrail-client | NATS connections |

**Security Benefits:**
- ✅ Certificate compromise isolated to one service
- ✅ Independent lifecycle management per service
- ✅ Proper identity (CN) per service role
- ✅ Service-specific SANs configuration

1. **Generate TLS Certificates**
   ```bash
   cd certs
   ./setup-tls.sh
   cd ..
   ```

   This generates 4 certificate pairs (8 files total):
   - CA certificate authority (ca.pem, ca-key.pem)
   - NATS server certificate (server-cert.pem, server-key.pem)
   - Gateway HTTPS certificate (gateway-cert.pem, gateway-key.pem)
   - Client certificate (client-cert.pem, client-key.pem)

2. **Start NATS with TLS**
   ```bash
   ./start-nats.sh
   ```

   Or manually:
   ```bash
   docker-compose up -d
   ```

3. **Verify NATS Health**
   ```bash
   curl http://localhost:9222/healthz
   ```

   Should return: `OK`

4. **Access NATS Monitoring Dashboard**
   Open http://localhost:9222 in your browser to see:
   - Connection statistics
   - Message throughput
   - JetStream status
   - Client connections

### Port Information

- **5222**: NATS TLS client connections
- **9222**: NATS HTTP monitoring dashboard (mapped from internal 8222)
- **8443**: Gateway HTTPS API (TLS-enabled)

### Required Software

- Rust 1.75+
- Docker and Docker Compose
- OpenSSL (for certificate generation)
- LM Studio (optional, for LLM integration in later phases)

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
- ✅ **Per-service TLS architecture** - separate certificates for NATS, Gateway, and MCP services
- ✅ **TLS-enabled Gateway (HTTPS)** with Qollective REST Server on port 8443
- ✅ TLS-enabled NATS infrastructure with Docker Compose (ports 5222/9222)
- ✅ **Gateway envelope-first architecture** - all responses wrapped in UnifiedEnvelope
- ✅ All services compile successfully with rmcp 0.8.0
- ✅ CONSTANTS FIRST principle - all values in `shared-types/src/constants.rs`
- ✅ Configuration inheritance pattern across all services
- ✅ Production-ready certificate generation (4 cert pairs: CA, NATS, Gateway, Client)
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

### NATS Connection Issues

**Problem**: Services fail to connect to NATS

**Solutions**:
1. Verify NATS is running: `docker-compose ps`
2. Check health: `curl http://localhost:9222/healthz`
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
4. **TLS from the Start**: Demonstrates Qollective's production-ready TLS capabilities
5. **Test-Driven**: Write tests before implementation for each component
6. **Build Verification**: Each phase ends with successful build and run verification

## License

MIT

## Acknowledgments

Built with:
- [Qollective](https://github.com/jocax/qollective) - Multi-protocol transport framework
- [rMCP](https://github.com/modelcontextprotocol/rmcp) - Model Context Protocol
- [Axum](https://github.com/tokio-rs/axum) - HTTP framework
- [NATS](https://nats.io/) - Cloud-native messaging system
- [Tokio](https://tokio.rs/) - Async runtime
