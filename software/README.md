# Qollective Framework - Rust Runtime

A comprehensive framework for building distributed applications with enhanced transport capabilities, envelope pattern architecture, and multi-protocol support.

## 🚀 Enhanced Transport Features

### Recent Enhancements (PRP Transport Enhancement)

The Qollective Framework now includes significant transport enhancements with modern protocol support:

- **jsonrpsee 0.25.1** - JSON-RPC 2.0 WebSocket server with browser compatibility
- **rMCP 0.3.0** - Enhanced Model Context Protocol with macro-based tool registration
- **a2a-rs 0.1.0** - Standardized Agent-to-Agent communication protocol

## 🏗️ Architecture

### Core Components

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   Application       │    │   Transport Layer   │    │   Network Layer     │
│   Layer             │    │                     │    │                     │
│                     │    │  ┌─────────────────┐│    │  ┌─────────────────┐│
│  ┌─────────────────┐│    │  │ HybridTransport ││    │  │ JSON-RPC 2.0    ││
│  │ Envelope        ││◄──►│  │ Client          ││◄──►│  │ WebSocket       ││
│  │ Pattern         ││    │  └─────────────────┘│    │  └─────────────────┘│
│  └─────────────────┘│    │                     │    │                     │
│                     │    │  ┌─────────────────┐│    │  ┌─────────────────┐│
│  ┌─────────────────┐│    │  │ Protocol        ││    │  │ Enhanced MCP    ││
│  │ Unified APIs    ││◄──►│  │ Detection       ││◄──►│  │ (rMCP 0.3.0)    ││
│  └─────────────────┘│    │  └─────────────────┘│    │  └─────────────────┘│
│                     │    │                     │    │                     │
│  ┌─────────────────┐│    │  ┌─────────────────┐│    │  ┌─────────────────┐│
│  │ WASM Support    ││◄──►│  │ Feature Gates   ││◄──►│  │ A2A Standard    ││
│  └─────────────────┘│    │  └─────────────────┘│    │  │ (a2a-rs 0.1.0)  ││
└─────────────────────┘    └─────────────────────┘    │  └─────────────────┘│
                                                      └─────────────────────┘
```

### Envelope Pattern

The framework is built around an "envelope first" architecture that wraps all communication:

```rust
use qollective::envelope::Envelope;
use qollective::transport::HybridTransportClient;

// Create envelope with metadata
let envelope = Envelope::new(meta, your_data);

// Send through any transport
let client = HybridTransportClient::new().await?;
let response = client.send_envelope(endpoint, envelope).await?;
```

## 📦 Feature Gates

### Core Transport Features
- `jsonrpsee-integration` - JSON-RPC 2.0 WebSocket enhancements
- `rmcp-enhanced` - rMCP 0.3.0 advanced server capabilities
- `a2a-standard` - a2a-rs standardized Agent-to-Agent protocol

### WASM Support
- `wasm-jsonrpc` - Browser-compatible JSON-RPC client
- `wasm-mcp` - WASM MCP client integration
- `wasm-a2a` - WASM A2A client support
- `wasm-enhanced` - All WASM enhancements

### Deployment Scenarios
- `hybrid-transport` - Multi-protocol transport support
- `browser-transport` - Browser-optimized features
- `server-transport` - Server-side enhancements
- `client-transport` - Client-side optimizations

### Development Features
- `dev` - Development utilities and enhanced logging
- `test` - Testing utilities and mock implementations
- `minimal` - Minimal feature set for resource-constrained environments

## 🚀 Quick Start

### Add to Cargo.toml

```toml
[dependencies]
qollective = { version = "0.0.1", features = [
    "jsonrpsee-integration",
    "rmcp-enhanced",
    "a2a-standard",
    "wasm-enhanced"
] }
```

### JSON-RPC 2.0 WebSocket Server

```rust
use qollective::transport::jsonrpc::JsonRpcWebSocketServer;
use qollective::envelope::Envelope;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create enhanced JSON-RPC server
    let server = JsonRpcWebSocketServer::new("127.0.0.1:8080").await?;
    
    // Start server with envelope support
    server.serve_with_envelope_support().await?;
    
    Ok(())
}
```

### Enhanced MCP Server with rMCP 0.3.0

```rust
use qollective::server::mcp::McpServer;
use rmcp::server_macros::mcp_server;
use qollective::envelope::Envelope;

#[mcp_server]
struct MyMcpServer {
    // Tools are automatically registered
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = MyMcpServer::new().await?;
    server.start_with_envelope_integration().await?;
    Ok(())
}
```

### A2A Standard Protocol Client

```rust
use qollective::client::a2a::A2AClient;
use a2a_rs::protocol::AgentMessage;
use qollective::envelope::Envelope;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = A2AClient::new_standard("my-agent").await?;
    
    // Discover other agents
    let agents = client.discover_agents_with_capability("data-processing").await?;
    
    // Send message with envelope
    let message = AgentMessage::new("Process this data");
    let envelope = Envelope::new_with_data(message);
    client.send_envelope_to_agent("target-agent", envelope).await?;
    
    Ok(())
}
```

### Hybrid Transport Client

```rust
use qollective::transport::HybridTransportClient;
use qollective::envelope::Envelope;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with automatic protocol detection
    let client = HybridTransportClient::new_with_all_enhancements().await?;
    
    // Send to different endpoints - protocol auto-detected
    let data = serde_json::json!({"operation": "process", "data": "test"});
    let envelope = Envelope::new_with_data(data);
    
    // JSON-RPC endpoint
    let response1 = client.send_envelope("ws://server1:8080/jsonrpc", envelope.clone()).await?;
    
    // MCP endpoint
    let response2 = client.send_envelope("ws://server2:8080/mcp", envelope.clone()).await?;
    
    // A2A endpoint
    let response3 = client.send_envelope("nats://server3:4222/agents", envelope).await?;
    
    Ok(())
}
```

## 🌐 WASM Support

### Browser WebSocket Client

```rust
use wasm_bindgen::prelude::*;
use qollective::transport::wasm::WasmHybridClient;

#[wasm_bindgen]
pub async fn connect_from_browser() -> Result<(), JsValue> {
    let client = WasmHybridClient::new().await?;
    
    // Connect to JSON-RPC server
    let response = client.send_request(
        "ws://localhost:8080/jsonrpc",
        serde_json::json!({"method": "get_data", "params": {}})
    ).await?;
    
    web_sys::console::log_1(&format!("Response: {:?}", response).into());
    Ok(())
}
```

## 📁 Examples

### Transport Enhancement Demo
[`examples/transport_enhancement_demo/`](examples/transport_enhancement_demo/)
- JSON-RPC 2.0 WebSocket server
- Enhanced MCP server with rMCP 0.3.0
- A2A discovery and communication
- Hybrid transport client

### WASM Space Exploration
[`examples/qollective_wasm_space_exploration/`](examples/qollective_wasm_space_exploration/)
- Browser-based WASM client
- Real-time WebSocket communication
- MCP tool execution from browser
- Space exploration theme

### MCP WebSocket Holodeck
[`examples/qollective_mcp_websocket_holodeck/`](examples/qollective_mcp_websocket_holodeck/)
- Star Trek holodeck simulation
- MCP server coordination
- Desktop and web interfaces
- Real-time story generation

### A2A NATS Enterprise
[`examples/qollective_a2a_nats_enterprise/`](examples/qollective_a2a_nats_enterprise/)
- Star Trek crew simulation
- Agent-to-Agent communication
- NATS-based discovery
- Enterprise scenario

## 🧪 Testing

### Run All Tests
```bash
# Use the test script (recommended)
./test.sh

# Or run with single-threading to avoid race conditions
cargo test -- --test-threads=1
```

**Important**: Tests must be run with `--test-threads=1` to prevent race conditions with environment variables. The framework includes configuration tests that modify environment variables and require test isolation.

### Run Transport Enhancement Tests
```bash
cargo test transport_enhancement_integration_tests
```

### Run Specific Protocol Tests
```bash
cargo test jsonrpsee_websocket_server_integration
cargo test enhanced_mcp_transport_with_rmcp_features
cargo test standardized_a2a_transport_with_a2a_rs
```

### Run WASM Tests
```bash
wasm-pack test --node --lib
```

## 🏃 Development

### Prerequisites
- Rust 1.75+
- Node.js 18+ (for WASM examples)
- NATS server (for A2A examples)

### Build
```bash
cargo build --all-features
```

### Build for WASM
```bash
wasm-pack build --target web --out-dir pkg
```

### Run Examples
```bash
# JSON-RPC WebSocket server
cargo run --bin jsonrpc_websocket_server --features jsonrpsee-integration

# Enhanced MCP server
cargo run --bin enhanced_mcp_server --features rmcp-enhanced

# A2A discovery client
cargo run --bin a2a_discovery_client --features a2a-standard
```

## 📊 Performance

### Benchmarks
```bash
cargo bench
```

### Optimizations
- **Connection Pooling**: Reuse connections across requests
- **Protocol Detection**: Cache protocol capabilities
- **WASM Bundle Size**: Optimized for browser loading
- **Memory Usage**: Efficient envelope handling

## 🔧 Configuration

### Environment Variables
```bash
# Transport endpoints
export JSONRPC_ENDPOINT="ws://localhost:8080/jsonrpc"
export MCP_ENDPOINT="ws://localhost:8080/mcp"
export A2A_ENDPOINT="nats://localhost:4222"

# Performance tuning
export MAX_CONNECTIONS=100
export REQUEST_TIMEOUT=30

# Debug logging
export DEBUG=1
export RUST_LOG=qollective=debug
```

### Configuration File
```toml
# qollective.toml
[transport]
jsonrpc_endpoint = "ws://localhost:8080/jsonrpc"
mcp_endpoint = "ws://localhost:8080/mcp"
a2a_endpoint = "nats://localhost:4222"
request_timeout = "30s"
max_connections = 100

[transport.jsonrpsee]
enable_browser_support = true
enable_binary_messages = true
ping_interval = "30s"

[transport.rmcp]
server_info_cache_ttl = "300s"
tool_execution_timeout = "60s"
enable_session_management = true

[transport.a2a]
agent_discovery_interval = "30s"
health_check_interval = "10s"
enable_load_balancing = true
```

## 🔍 Debugging

### Enable Debug Logging
```bash
RUST_LOG=qollective=debug cargo run
```

### Transport-specific Logging
```bash
RUST_LOG=qollective::transport=trace cargo run
```

### WASM Debugging
```javascript
// In browser console
localStorage.debug = 'qollective:*';
```

## 📖 Documentation

### API Documentation
```bash
cargo doc --open --all-features
```

### Protocol Documentation
- [JSON-RPC 2.0 Integration](docs/jsonrpc.md)
- [rMCP 0.3.0 Features](docs/rmcp.md)
- [a2a-rs Protocol](docs/a2a.md)
- [WASM Integration](docs/wasm.md)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

### Development Guidelines
- Follow Rust best practices
- Maintain envelope pattern compatibility
- Add comprehensive tests
- Update documentation

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 📞 Support

- GitHub Issues: Report bugs and request features
- Documentation: Comprehensive API and usage docs
- Examples: Working examples for all features
- Community: Join our discussions

## 🗂️ Project Structure

```
qollective/
├── src/
│   ├── lib.rs                 # Main library entry point
│   ├── envelope/              # Envelope pattern implementation
│   ├── transport/             # Enhanced transport layer
│   │   ├── jsonrpc.rs         # JSON-RPC 2.0 transport
│   │   ├── mcp.rs             # rMCP 0.3.0 transport
│   │   └── a2a.rs             # a2a-rs transport
│   ├── client/                # Client implementations
│   ├── server/                # Server implementations
│   └── wasm/                  # WASM-specific code
├── examples/                  # Example projects
│   ├── transport_enhancement_demo/
│   ├── qollective_wasm_space_exploration/
│   ├── qollective_mcp_websocket_holodeck/
│   └── qollective_a2a_nats_enterprise/
├── tests/                     # Integration tests
└── docs/                      # Documentation
```

## 🎯 Roadmap

### Completed
- ✅ jsonrpsee 0.25.1 integration
- ✅ rMCP 0.3.0 enhanced features
- ✅ a2a-rs 0.1.0 standardization
- ✅ WASM browser support
- ✅ Comprehensive testing

### In Progress
- 🔄 Performance optimizations
- 🔄 Additional protocol support
- 🔄 Advanced WASM features

### Planned
- 📋 GraphQL transport
- 📋 gRPC enhancements
- 📋 Kubernetes integration
- 📋 Metrics and monitoring

---

Built with ❤️ by the Qollective team. Part of the larger Qollective ecosystem for distributed application development.