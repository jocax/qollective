# WASM Envelope Architecture Overview

## System Principles

### Unified Communication Format
The envelope pattern provides a consistent JSON wrapper for all communication between UI and backend systems:

```rust
pub struct Envelope<T> {
    pub meta: Meta,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<QollectiveError>,
}
```

### Key Architectural Principles

1. **Type Safety**: Generic `Envelope<T>` ensures compile-time type checking for all data payloads
2. **Context Propagation**: Meta section carries security, tenant, tracing, and operational context
3. **Error Consistency**: Single `QollectiveError` type across all system layers
4. **Protocol Abstraction**: WASM layer handles transport differences transparently
5. **Security by Design**: All remote communication secured via TLS/mTLS

## Overall Architecture

### Component Layers

```
┌─────────────────┐    ┌──────────────────┐
│   Browser/UI    │────│   WASM Client    │
└─────────────────┘    └──────────────────┘
                                │
                    ┌───────────┼───────────┐
                    │           │           │
            ┌───────▼─────┐    ┌▼────────┐  │
            │ HTTPS API   │    │  NATS   │  │
            │  (REST)     │    │Gateway  │  │
            └─────────────┘    └─────────┘  │
                                │           │
                        ┌───────▼─────┐     │
                        │ A2A Registry│     │
                        └─────────────┘     │
                                │           │
                        ┌───────▼─────┐     │
                        │ A2A Agents  │     │
                        └─────────────┘     │
                                │           │
                        ┌───────▼─────┐     │
                        │ MCP Client  │     │
                        └─────────────┘     │
                                │           │
                        ┌───────▼─────┐     │
                        │ MCP Servers │     │
                        └─────────────┘     │
```

### Data Flow Patterns

#### REST Pattern
- **UI → WASM → HTTPS → Backend**
- Synchronous request/response
- Domain entities (User, Product, Order)

#### Agent Pattern
- **UI → WASM → NATS → Registry → Agent → MCP**
- Asynchronous messaging
- JSON-RPC 2.0 protocol

### Context Generation

```rust
// Envelope received at any component
Envelope<JsonRpcRequest> → Context {
    meta: Meta {
        tenant: "abc123",
        request_id: "uuid-456", 
        security: SecurityMeta { ... },
        // ... other context
    }
}
```

## Security Model

### Transport Security
- **All remote communication**: TLS 1.3+ encryption
- **Client authentication**: mTLS with embedded certificates in WASM
- **Message integrity**: Transport-level and application-level validation

### Context Security
- **Tenant isolation**: Multi-tenant context enforced at every layer
- **Request tracing**: End-to-end correlation via request IDs
- **Security propagation**: Authentication context flows through entire call chain

## Benefits

### For Developers
- **Single API Surface**: UI only interacts with WASM interface
- **Type Safety**: Compile-time guarantees for all communication
- **Protocol Agnostic**: Same code works for REST and agent communication
- **Consistent Error Handling**: Unified error types and patterns

### For Operations
- **End-to-end Tracing**: Request correlation across all services
- **Security Compliance**: Consistent security model
- **Multi-tenant Safe**: Built-in tenant isolation
- **Monitoring Ready**: Rich metadata for observability

## Implementation Scenarios

The architecture supports these primary communication patterns:

1. **REST Operations**: Traditional CRUD operations via HTTPS
2. **A2A Agent Communication**: Real-time agent interaction via NATS
3. **Multi-MCP Coordination**: Complex workflows across multiple MCP servers
4. **Error Handling**: Consistent error propagation and translation

Each scenario maintains the same envelope structure while optimizing transport protocols for specific use cases.
