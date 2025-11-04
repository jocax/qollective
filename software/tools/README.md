# Qollective Tools

> **Type-safe code generation from JSON Schema for multi-protocol distributed systems**

The Qollective Schema Generator transforms JSON Schema definitions into type-safe, protocol-agnostic code that works seamlessly across REST, gRPC, WebSocket, NATS, MCP, and A2A protocols. Write your schema once, deploy everywhere.

## Quick Command Reference

| Command | Description | Example |
|---------|-------------|---------|
| `generate` | Generate code from JSON Schema | `qollective_tools generate schema.json --output ./src/generated --language rust` |
| `validate` | Validate schema correctness | `qollective_tools validate schema.json --detailed` |
| `info` | Display schema information | `qollective_tools info schema.json --stats` |
| `init` | Initialize new project | `qollective_tools init my-service --template full` |

## Installation

```bash
# Build from source
cd software/generator
cargo build --release

# Install locally
cargo install --path .

# Or run directly
cargo run -- <command> <args>
```

## Usage Examples

### Generate Rust code from schema
```bash
qollective_tools generate schemas/basic/test_simple_struct.json \
  --output ./generated \
  --language rust \
  --format module
```

### Validate a schema with detailed output
```bash
qollective_tools validate schemas/examples/schema_example.json --detailed --lint
```

### Get schema information and statistics
```bash
qollective_tools info schemas/examples/schema_example.json --stats --dependencies
```

### Initialize a new service project
```bash
qollective_tools init user-service --template full --directory ./services/user
```

## Test Schemas

The `schemas/` directory contains test schemas organized by complexity:
- `basic/` - 10 schemas for fundamental types (structs, enums, arrays, UUIDs, dates, etc.)
- `advanced/` - 4 schemas for complex patterns (oneOf, anyOf, nullable, HashMap)
- `examples/` - Comprehensive integration example with 15 interconnected types

## Command Options

### `generate` Command
- `--output, -o`: Output directory for generated code (default: `./generated`)
- `--language, -l`: Target language [`rust`, `typescript`, `java`] (default: `rust`)
- `--format, -f`: Output format [`module`, `crate`, `single-file`] (default: `module`)
- `--package-name, -p`: Override package/module name
- `--skip-validation`: Skip schema validation
- `--force`: Overwrite existing files without confirmation

### `validate` Command
- `--detailed, -d`: Show detailed validation information
- `--lint, -l`: Run additional linting checks

### `info` Command
- `--stats, -s`: Display schema statistics
- `--dependencies, -d`: Show schema dependencies

### `init` Command
- `--directory, -d`: Target directory (defaults to project name)
- `--template, -t`: Project template [`minimal`, `full`, `examples`] (default: `minimal`)

## Why Does This Generator Exist?

### The Problem

Building distributed systems with Qollective requires implementing the same data structures and validation logic across multiple transport protocols:
- REST APIs need request/response DTOs
- gRPC requires protobuf definitions
- WebSocket needs message types
- NATS requires subject-based payloads
- MCP needs tool definitions
- A2A requires agent communication structures

Manually implementing these for each protocol leads to:
- **Inconsistency**: Different implementations drift apart over time
- **Duplication**: Same validation logic repeated 6+ times
- **Errors**: Type mismatches between protocol implementations
- **Maintenance Burden**: Changes must be propagated manually
- **Slow Development**: Writing boilerplate for each protocol

### The Solution

The Qollective Schema Generator solves these problems by:
1. **Single Source of Truth**: One JSON Schema defines all data structures
2. **Automatic Code Generation**: Type-safe code for all protocols
3. **Envelope Pattern Enforcement**: Ensures consistent metadata handling
4. **Multi-Protocol Support**: Generate for any combination of protocols
5. **Type Safety**: Compile-time guarantees across all transports

## Key Features

- **DirectTypifyGenerator**: Simplified Rust code generation using typify directly
- **Intelligent Integer Selection**: Auto-selects optimal types (u8-u128, i8-i128) based on min/max constraints
- **Comprehensive Testing**: 68 tests with compilation verification

## What Is The Generator Good For?

### 1. **Rapid Service Development**
Create a new microservice with full protocol support in minutes:
```json
{
  "qollective": {
    "generation": {
      "targets": ["rust-rest", "rust-grpc", "rust-websocket"]
    }
  }
}
```

### 2. **Protocol-Agnostic APIs**
Write business logic once, support all protocols:
- Same data structures work across REST, gRPC, WebSocket
- Automatic serialization/deserialization
- Protocol-specific optimizations handled automatically

### 3. **Multi-Tenant Applications**
Built-in tenant context management:
- Automatic tenant extraction from JWT tokens
- Context propagation across service boundaries
- Tenant isolation at the framework level

### 4. **AI Agent Systems**
Native support for LLM and agent communication:
- MCP tool generation from schemas
- A2A agent discovery and routing
- Type-safe tool execution

### 5. **Enterprise Integration**
- Consistent error handling across protocols
- Distributed tracing with OpenTelemetry
- Performance metrics collection
- Security context propagation

## How It Works

### 1. **Schema Parsing**
The generator parses your JSON Schema and validates it against:
- JSON Schema 2020-12 specification
- Qollective envelope pattern requirements
- Protocol-specific constraints

### 2. **Intermediate Representation**
Schemas are converted to an internal AST that captures:
- Type definitions and constraints
- Validation rules
- Protocol-specific metadata
- Generation configuration

### 3. **Code Generation**
Using the [typify](https://github.com/oxidecomputer/typify) library, the generator creates:
- Rust structs with serde derives
- Validation implementations
- Protocol-specific adapters
- Client and server stubs

### 4. **Feature Integration**
Generated code automatically includes:
- Envelope wrapping/unwrapping
- Tenant context extraction
- Error transformation
- Metrics and tracing hooks

## Schema Requirements

Every schema MUST follow the Qollective envelope pattern:

```json5
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://your-domain.com/schemas/service.json",

  // 1. REQUIRED: Extend the envelope
  "allOf": [
    {
      "$ref": "https://schemas.qollective.io/core/envelope.json"
    },
    {
      "properties": {
        "payload": {
          "oneOf": [
            { "$ref": "#/$defs/Request" },
            { "$ref": "#/$defs/Response" },
            { "$ref": "#/$defs/Error" }
          ]
        }
      }
    }
  ],

  // 2. REQUIRED: Define your types in $defs
  "$defs": {
    "Request": { /* ... */ },
    "Response": { /* ... */ },
    "Error": { /* ... */ }
  },

  // 3. REQUIRED: Qollective configuration
  "qollective": {
    "version": "1.0.0",
    "envelope": { "enabled": true },
    "generation": {
      "targets": ["rust-rest"]
    }
  }
}
```

## Supported Generation Targets

### Currently Implemented
- `rust-rest`: REST API client/server with Axum
- `rust-grpc`: gRPC services with Tonic (planned)
- `rust-websocket`: WebSocket handlers with tokio-tungstenite (planned)
- `rust-nats`: NATS pub/sub with async-nats (planned)
- `rust-mcp`: Model Context Protocol with rmcp (planned)
- `rust-a2a`: Agent-to-Agent with a2a-rs (planned)

### Future Targets
- `typescript-rest`: TypeScript/Node.js REST
- `typescript-grpc`: TypeScript gRPC-web
- `java-rest`: Java Spring Boot REST
- `java-grpc`: Java gRPC services

## Architecture Integration

The generator is a critical component of Qollective's architecture:

```
┌─────────────┐     ┌──────────────┐     ┌───────────────┐
│ JSON Schema │────▶│   Generator  │────▶│ Generated Code│
└─────────────┘     └──────────────┘     └───────────────┘
                            │                      │
                            ▼                      ▼
                    ┌──────────────┐      ┌───────────────┐
                    │   Validator  │      │   Protocols   │
                    └──────────────┘      │ REST  gRPC    │
                                          │ WS    NATS    │
                                          │ MCP   A2A     │
                                          └───────────────┘
```

### Envelope-First Design
All generated code follows Qollective's envelope pattern:
- Metadata (request ID, tenant, tracing) handled automatically
- Payload type safety enforced at compile time
- Error responses standardized across protocols

### Multi-Protocol Abstraction
Generated code works with Qollective's `HybridTransportClient`:
- Automatic protocol detection
- Seamless protocol switching
- Consistent API across transports

### Type Safety Guarantees
- Compile-time validation of data structures
- Runtime validation against schema constraints
- Protocol-specific serialization safety

## Example: Simple Struct

Using the test schema `schemas/basic/test_simple_struct.json`:

```bash
# Generate code
qollective_tools generate schemas/basic/test_simple_struct.json --output ./generated

# Use generated types in Rust
use generated::Person;

let person = Person {
    name: "John Doe".to_string(),
    age: 30,
    active: Some(true),
    score: Some(95.5),
};
```

## Development Workflow

1. **Design**: Create JSON Schema with your data models
2. **Validate**: Run validator to ensure schema correctness
3. **Generate**: Create type-safe code for target protocols
4. **Implement**: Write business logic using generated types
5. **Deploy**: Run services with any supported protocol

## Best Practices

1. **Version Your Schemas**: Use semantic versioning in schema $id
2. **Use $defs**: Define reusable types in the $defs section
3. **Document Types**: Include descriptions for generated documentation
4. **Validate First**: Always validate before generating
5. **Feature Gates**: Only generate for protocols you need

## Running Tests

```bash
# Run all tests
cargo test

# Run schema snippet tests
cargo test schema_snippets
```

## Troubleshooting

### Schema Validation Fails
- Ensure schema extends the envelope: `"allOf": [{"$ref": "...envelope.json"}]`
- Check JSON Schema syntax with online validators
- Verify qollective configuration section exists

### Generation Errors
- Confirm target language is supported
- Check output directory permissions
- Ensure schema passes validation first

### Type Conflicts
- Use unique names in $defs
- Avoid reserved keywords for target language
- Check for circular references

## Contributing

The generator uses:
- **Parser**: serde_json for JSON parsing
- **Validator**: jsonschema crate for validation
- **Code Gen**: typify for Rust generation
- **CLI**: clap for argument parsing

To add a new language target:
1. Implement trait in `codegen/mod.rs`
2. Add language option to CLI
3. Create code templates
4. Add integration tests

## License

Part of the Qollective framework - see main project LICENSE.

---

*The generator embodies Qollective's philosophy: Define once, deploy everywhere, with type safety and consistency guaranteed.*
