# Qollective Schema Generator

> **Type-safe code generation from JSON Schema for multi-protocol distributed systems**

The Qollective Schema Generator transforms JSON Schema definitions into type-safe, protocol-agnostic code that works seamlessly across REST, gRPC, WebSocket, NATS, MCP, and A2A protocols. Write your schema once, deploy everywhere.

## Quick Command Reference

| Command | Description | Example |
|---------|-------------|---------|
| `generate` | Generate code from JSON Schema | `qollective generate schema.json --output ./src/generated --language rust` |
| `validate` | Validate schema correctness | `qollective validate schema.json --detailed` |
| `info` | Display schema information | `qollective info schema.json --stats` |
| `init` | Initialize new project | `qollective init my-service --template full` |

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
qollective generate ../schemas/user-service.json \
  --output ../src/generated \
  --language rust \
  --format module
```

### Validate a schema with detailed output
```bash
qollective validate ../schemas/user-service.json --detailed --lint
```

### Get schema information and statistics
```bash
qollective info ../schemas/user-service.json --stats --dependencies
```

### Initialize a new service project
```bash
qollective init user-service --template full --directory ./services/user
```

### Generate code with custom derive traits
```bash
# Enable JsonSchema derive for MCP tool definitions
qollective generate schema.json --schemars

# Add additional custom derives
qollective generate schema.json --additional-derives "PartialEq,Hash,Eq"

# Combine both flags
qollective generate schema.json --schemars --additional-derives "Default,PartialOrd"
```

## Command Options

### `generate` Command
- `--output, -o`: Output directory for generated code (default: `./generated`)
- `--language, -l`: Target language [`rust`, `typescript`, `java`] (default: `rust`)
- `--format, -f`: Output format [`module`, `crate`, `single-file`] (default: `module`)
- `--package-name, -p`: Override package/module name
- `--skip-validation`: Skip schema validation
- `--force`: Overwrite existing files without confirmation
- `--schemars`: Enable `schemars::JsonSchema` derive and automatically add schemars dependency
- `--additional-derives`: Additional derive traits (comma-separated, e.g., "PartialEq,Hash,Eq")

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

```json
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

## Example: User Service

### 1. Define Schema (`user-service.json`)
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "allOf": [
    { "$ref": "https://schemas.qollective.io/core/envelope.json" }
  ],
  "$defs": {
    "User": {
      "type": "object",
      "properties": {
        "id": { "type": "string", "format": "uuid" },
        "email": { "type": "string", "format": "email" },
        "name": { "type": "string" }
      }
    }
  },
  "qollective": {
    "version": "1.0.0",
    "generation": {
      "targets": ["rust-rest", "rust-grpc"],
      "outputDir": "./src/generated"
    }
  }
}
```

### 2. Generate Code
```bash
qollective generate user-service.json
```

### 3. Use Generated Types
```rust
use generated::user_service::{User, Envelope};

// Type-safe across all protocols
let user = User {
    id: uuid::Uuid::new_v4().to_string(),
    email: "user@example.com".to_string(),
    name: "John Doe".to_string(),
};

// Automatic envelope wrapping
let response = Envelope::success(user);
```

## Development Workflow

1. **Design**: Create JSON Schema with your data models
2. **Validate**: Run validator to ensure schema correctness
3. **Generate**: Create type-safe code for target protocols
4. **Implement**: Write business logic using generated types
5. **Deploy**: Run services with any supported protocol

## Custom Derive Traits

The generator supports flexible customization of derive traits through CLI flags. This is particularly useful for MCP tool definitions and advanced Rust features.

### --schemars Flag

Enable `schemars::JsonSchema` derive for MCP tool definitions:

```bash
qollective generate schema.json --schemars
```

This automatically:
- Adds `#[derive(JsonSchema)]` to all generated types
- Adds `schemars = "0.8"` to Cargo.toml dependencies
- Imports `use schemars::JsonSchema;`

**Use Case**: MCP (Model Context Protocol) tools require JsonSchema for dynamic type introspection.

### --additional-derives Flag

Add custom derive traits to all generated types:

```bash
qollective generate schema.json --additional-derives "PartialEq,Hash,Eq"
```

**Common Use Cases**:
- `PartialEq,Eq,Hash`: Enable types as HashMap keys
- `Default`: Provide default values for optional fields
- `PartialOrd,Ord`: Enable sorting and comparisons
- `Copy`: Enable cheap copying for small types

### Combining Flags

Use both flags together for maximum flexibility:

```bash
qollective generate schema.json --schemars --additional-derives "Default,PartialOrd"
```

Generated code will include:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default, PartialOrd)]
pub struct User {
    // fields...
}
```

### Important Notes

1. **Breaking Change (v0.1.0)**: `JsonSchema` is no longer derived by default. Use `--schemars` to enable it.
2. **Deduplication**: The generator automatically removes duplicate derives if you specify the same trait in both flags.
3. **Whitespace Handling**: The parser handles spaces correctly: `"PartialEq, Hash, Eq"` works as expected.
4. **Empty Values**: Empty strings and whitespace-only values are safely ignored.
5. **Crate Format Only**: The `schemars` dependency is only injected when using `--format crate`. For `module` or `single-file` formats, you must manually add the dependency to your project.

## Best Practices

1. **Version Your Schemas**: Use semantic versioning in schema $id
2. **Use $defs**: Define reusable types in the $defs section
3. **Document Types**: Include descriptions for generated documentation
4. **Validate First**: Always validate before generating
5. **Feature Gates**: Only generate for protocols you need
6. **Minimal Derives**: Only add derives you actually need to avoid unnecessary trait bounds

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