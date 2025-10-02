# Protocol and Transport Implementation Overview

## Current Architecture Status

| Protocol/Transport | Client Files (Target) | Server Files (Target) | Transport Files (Target) | Feature Gates | Testability |
|-------------------|----------------------|----------------------|--------------------------|---------------|-------------|
| **NATS** | `src/client/nats.rs`<br/>*Uses `HybridTransportClient`* | `src/server/nats.rs`<br/>`src/transport/nats.rs` | `src/transport/nats.rs`<br/>*Pure NATS protocol logic* | `nats-client`, `nats-server`, `nats` | ✅ Unit testable with mock transport |
| **gRPC** | `src/client/grpc.rs`<br/>*Uses `HybridTransportClient`* | `src/server/grpc.rs`<br/>`src/transport/grpc.rs` | `src/transport/grpc.rs`<br/>*Pure gRPC channel logic* | `grpc-client`, `grpc-server` | ✅ Unit testable with mock transport |
| **HTTP/REST** | `src/client/rest.rs`<br/>*Uses `HybridTransportClient`* | `src/server/rest.rs`<br/>`src/transport/rest.rs` | `src/transport/rest.rs`<br/>*Pure HTTP client logic* | `rest-client`, `rest-server` | ✅ Unit testable with mock transport |
| **WebSocket** | `src/client/websocket.rs`<br/>*Uses `HybridTransportClient`* | `src/server/websocket.rs`<br/>`src/transport/websocket.rs` | `src/transport/websocket.rs`<br/>*Pure WebSocket logic* | `websocket-client`, `websocket-server` | ✅ Unit testable with mock transport |
| **MCP-stdio** | `src/client/mcp_stdio.rs`<br/>*Uses `StdioTransport`* | N/A (process-based) | `src/transport/mcp_stdio.rs`<br/>*Pure process/stdio logic* | `mcp-stdio-client` | ✅ Unit testable with mock process |
| **MCP (rmcp)** | `src/client/mcp.rs`<br/>*Uses `HybridTransportClient`* | `src/server/mcp.rs`<br/>`src/transport/mcp.rs` | `src/transport/mcp.rs`<br/>*Pure MCP protocol logic* | `mcp-client`, `mcp-server`, `mcp` | ✅ Unit testable with mock transport |
| **A2A** | `src/client/a2a.rs`<br/>*Uses `HybridTransportClient`* | `src/server/a2a.rs`<br/>`src/transport/a2a.rs` | `src/transport/a2a.rs`<br/>*Pure A2A/NATS logic* | `a2a-client`, `a2a-server`, `a2a` | ✅ Unit testable with mock transport |

**Target Design Principles:**
1. **Dependency Injection**: Clients accept transport instances, don't create them
2. **Interface Segregation**: Transport interfaces separate from business logic  
3. **Single Responsibility**: Clients handle business logic, transports handle protocol details
4. **Testability**: Easy to unit test with mock transports
5. **Consistency**: Same pattern across all protocols

## Configuration and Transport Dependency Architecture

| Protocol/Transport | Config Files | Config→Transport | Config→Client | Config→Server | Transport→Client | Transport→Server |
|-------------------|--------------|------------------|---------------|---------------|------------------|------------------|
| **NATS** | `src/config/nats.rs` | ✅ `NatsConfig` → `NatsTransport::new(config)` | ✅ `A2AConfig` → `A2AClient::new(config)` | ✅ `NatsServerConfig` → `NatsServer::new(config)` | ✅ `NatsTransport` → `A2AClient::with_transport(transport)` | ✅ `NatsTransport` → `NatsServer::with_transport(transport)` |
| **gRPC** | `src/config/grpc.rs` | ✅ `GrpcConfig` → `GrpcTransport::new(config)` | ✅ `GrpcClientConfig` → `GrpcClient::new(config)` | ✅ `GrpcServerConfig` → `GrpcServer::new(config)` | ✅ `GrpcTransport` → `GrpcClient::with_transport(transport)` | ✅ `GrpcTransport` → `GrpcServer::with_transport(transport)` |
| **HTTP/REST** | `src/config/rest.rs` | ✅ `RestConfig` → `HttpTransport::new(config)` | ✅ `RestClientConfig` → `RestClient::new(config)` | ✅ `RestServerConfig` → `RestServer::new(config)` | ✅ `HttpTransport` → `RestClient::with_transport(transport)` | ✅ `HttpTransport` → `RestServer::with_transport(transport)` |
| **WebSocket** | `src/config/websocket.rs`<br/>*🔨 Needs creation* | 🔨 `WsConfig` → `WsTransport::new(config)` | 🔨 `WsClientConfig` → `WsClient::new(config)` | 🔨 `WsServerConfig` → `WsServer::new(config)` | 🔨 `WsTransport` → `WsClient::with_transport(transport)` | 🔨 `WsTransport` → `WsServer::with_transport(transport)` |
| **MCP-stdio** | `src/config/mcp.rs` | ✅ `McpStdioConfig` → `StdioTransport::new(config)` | ✅ `McpStdioClientConfig` → `McpStdioClient::new(config)` | N/A (process-based) | ✅ `StdioTransport` → `McpStdioClient::with_transport(transport)` | N/A |
| **MCP (rmcp)** | `src/config/mcp.rs` | ✅ `McpConfig` → `McpTransport::new(config)` | ✅ `McpClientConfig` → `McpClient::new(config)` | ✅ `McpServerConfig` → `McpServer::new(config)` | ✅ `McpTransport` → `McpClient::with_transport(transport)` | ✅ `McpTransport` → `McpServer::with_transport(transport)` |
| **A2A** | `src/config/a2a.rs` | ✅ `A2ATransportConfig` → `A2ATransport::new(config)` | ✅ `A2AClientConfig` → `A2AClient::new(config)` | ✅ `A2AServerConfig` → `A2AServer::new(config)` | ✅ `A2ATransport` → `A2AClient::with_transport(transport)` | ✅ `A2ATransport` → `A2AServer::with_transport(transport)` |

**Legend:**
- ✅ **Target Implementation**: Should be implemented
- 🔨 **Needs Creation**: Missing components that need to be built
- N/A **Not Applicable**: Doesn't apply to this protocol type

## Transport Selection and Discovery Architecture

| Component | Current Behavior | Target Behavior | Transport Selection Logic |
|-----------|------------------|-----------------|---------------------------|
| **Client** | Hard-coded transport creation | Dynamic transport selection | `HybridTransportClient::discover_best_transport(endpoint)` |
| **Server** | Single transport binding | Multi-transport listening | `TransportManager::bind_available_transports(configs)` |
| **Config** | Protocol-specific configs | Unified transport config | `TransportConfigRegistry::get_available_transports()` |
| **Discovery** | Manual endpoint configuration | Automatic capability detection | `TransportCapabilityDetector::probe_endpoint_capabilities()` |

## Framework Integration Pattern

```rust
// Target usage pattern for framework consumers:
let config = QollectiveConfig::from_file("qollective.toml")?;
let transport_manager = TransportManager::new(config.transports)?;

// Option 1: Let framework choose best transport
let client = QollectiveClient::new(transport_manager)?;
client.connect_to("https://api.example.com").await?; // Auto-detects: REST
client.connect_to("nats://nats.example.com").await?; // Auto-detects: NATS

// Option 2: Force specific transport
let grpc_transport = transport_manager.get_transport("grpc")?;
let grpc_client = GrpcClient::with_transport(grpc_transport)?;

// Option 3: Transport fallback chain
let client = QollectiveClient::with_fallback_chain([
    "grpc", "rest", "websocket"
])?;
```

Additional Infrastructure:
- Config: Protocol configs in src/config/{nats,grpc,rest,mcp,a2a}.rs
- Transport Layer: Universal transport abstraction in src/transport/mod.rs
- Envelope Support: All protocols support envelope wrapping via src/envelope/

Feature Gate Architecture:
- Individual client/server gates: {protocol}-client, {protocol}-server
- Combined gates: nats, mcp, a2a (enable both client & server)
- Default bundle includes all protocols except WebSocket server
