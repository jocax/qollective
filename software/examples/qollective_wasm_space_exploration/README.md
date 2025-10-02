# 🚀 Space Exploration Demo - Qollective Framework WASM

This workspace demonstrates the Qollective Framework's WASM capabilities through a space exploration theme with a fully functional browser-based mission dashboard.

## Services Overview

| Service | Port | Protocol | Description |
|---------|------|----------|-------------|
| **REST Server** | 8443 | HTTP | Mission data & spacecraft management |
| **WebSocket Server** | 8444 | WebSocket | Real-time telemetry streaming |
| **MCP Server** | 8445 | WebSocket | Tool execution via MCP protocol |
| **Web Dashboard** | 8446 | HTTP | WASM demo interface |

## Architecture

```
WASM CLIENT (Browser) ←→ Space Mission Dashboard ←→ Qollective Framework
                      ↓
                   Real Servers (REST, WebSocket, MCP)
```

## Workspace Structure

### Crates

- **`space-wasm-client`** - Browser WASM client with space mission interface
- **`space-shared`** - Common data structures and space-themed types

### Demo Components

1. **Space Mission Dashboard** - Interactive web interface with WASM integration
2. **WASM Client** - Compiled Rust code running in browser
3. **Space Tools** - Simulated space mission operations (sector scanning, asteroid analysis, etc.)
4. **Mission Control** - Real-time mission monitoring and spacecraft management

## Quick Start

### 1. Build WASM Client
```bash
cd crates/space-wasm-client
wasm-pack build --target web --out-dir pkg
```

### 2. Start Web Server
```bash
# From the space-wasm-client directory
cd crates/space-wasm-client
npx http-server . -p 8446 -c-1
```

### 3. Open Demo
Navigate to: **http://127.0.0.1:8446/space-mission-dashboard.html**

## Complete Build Process

### Build All Components
```bash
# Build WASM client
cd crates/space-wasm-client
wasm-pack build --target web --out-dir pkg

# Build with cargo (for development)
cargo build --lib --release --target wasm32-unknown-unknown
```

### Development Commands
```bash
# Build all workspace crates
cargo build

# Build specific crate
cargo build -p space-wasm-client
cargo build -p space-shared

# Run tests
cargo test
```

## Demo Features

### 🛸 Spacecraft Management
- Launch spacecraft with different types (Explorer, Transport, Research, Mining)
- Get fleet status and monitor active spacecraft
- Real-time spacecraft tracking and status updates

### 🔧 Tool Execution
- Execute space tools: Scan Sector, Analyze Asteroid, Establish Orbit
- Deploy probes and collect samples
- Emergency protocol simulation
- JSON parameter configuration for tool execution

### 📊 Mission Control
- Connect to MCP endpoints for real-time communication
- Monitor mission logs and system status
- Emergency response protocols
- WebSocket-based real-time updates

### 🌌 Space Data Analytics
- Generate space exploration data
- Analyze stellar data across different wavelengths
- Track asteroids and space objects
- Real-time data visualization

## Technical Implementation

### WASM Integration
- **Framework**: Qollective Framework WASM envelope support
- **Language**: Rust compiled to WebAssembly
- **Browser API**: Web APIs via wasm-bindgen
- **Protocol Support**: REST, WebSocket, MCP protocol adapters

### Configuration Files
- **`.cargo/config.toml`** - Handles apple-m3 processor warnings
- **`Cargo.toml`** - Optimized for WASM compilation
- **Target**: wasm32-unknown-unknown with size optimization

### Apple M3 Compatibility
The project includes configuration to handle Apple M3 processor warnings:
```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-cpu=generic"]
```

## Space Exploration Theme

The demo uses a realistic space exploration scenario:

- **Missions**: Deep space exploration, asteroid mining, orbital research
- **Spacecraft**: Various types with specialized capabilities
- **Operations**: Tool execution, emergency protocols, data collection
- **Real-time**: Live mission monitoring and control

This provides engaging, realistic test data while demonstrating full framework capabilities through an interactive web interface.