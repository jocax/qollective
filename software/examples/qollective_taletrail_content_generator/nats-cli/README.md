# NATS CLI - Envelope-Wrapped MCP Request Tool

Command-line tool for sending Qollective envelope-wrapped MCP (Model Context Protocol) requests to NATS subjects. Built for the TaleTrail Content Generator project.

## Overview

`nats-cli` provides a simple way to test and interact with MCP servers via NATS using Qollective's envelope-first architecture. It handles NKEY authentication, TLS connections, and automatic envelope wrapping/unwrapping.

### Key Features

- ✅ **Envelope-First Architecture**: Automatic wrapping of MCP requests in Qollective envelopes
- ✅ **NKEY Authentication**: Secure authentication using NATS NKeys
- ✅ **TLS Support**: Encrypted communication with certificate validation
- ✅ **Template System**: JSON-based request templates organized by server
- ✅ **Colored Output**: Pretty-printed responses with syntax highlighting
- ✅ **Multi-Tenant Support**: Tenant ID tracking in envelope metadata
- ✅ **Request-Reply Pattern**: Synchronous request-reply with configurable timeout

## Installation

### Prerequisites

- Rust 1.75+ toolchain
- Running NATS server with TLS and NKEY authentication
- NKEY file and CA certificate

### Build from Source

```bash
cd nats-cli
cargo build --release
```

The binary will be available at `target/release/nats-cli`.

## Quick Start

### 1. Configuration

Create or edit `config.toml`:

```toml
[nats]
url = "nats://localhost:5222"
nkey_file = "../nkeys/nats-cli.nk"

[nats.tls]
ca_cert = "../certs/ca.pem"

[client]
default_timeout_secs = 30
default_tenant_id = 1
log_level = "info"
colored_output = true
```

### 2. List Available Templates

```bash
cargo run -- template list
```

### 3. Send a Request

```bash
cargo run -- send \
  --subject mcp.prompt.helper \
  --template templates/prompt-helper/get_model_for_language.json
```

## Usage

### Commands

#### `send` - Send MCP Request

Send an MCP request to a NATS subject using a template.

```bash
nats-cli send --subject <SUBJECT> --template <PATH> [OPTIONS]
```

**Required Arguments:**
- `--subject <SUBJECT>` - NATS subject to send request to
- `--template <PATH>` - Path to JSON template file

**Optional Arguments:**
- `--tenant <ID>` - Tenant ID (overrides config default)
- `--timeout <SECS>` - Request timeout in seconds (overrides config)

**Examples:**

```bash
# Send to prompt-helper with default tenant
nats-cli send \
  --subject mcp.prompt.helper \
  --template templates/prompt-helper/generate_story_prompts.json

# Use custom tenant and timeout
nats-cli send \
  --subject mcp.prompt.helper \
  --template templates/prompt-helper/generate_validation_prompts.json \
  --tenant 42 \
  --timeout 60

# Enable verbose output
nats-cli send \
  --subject mcp.prompt.helper \
  --template templates/prompt-helper/get_model_for_language.json \
  --verbose
```

#### `template list` - List Templates

List all available templates or filter by server.

```bash
nats-cli template list [OPTIONS]
```

**Optional Arguments:**
- `--server <NAME>` - Filter by server name

**Examples:**

```bash
# List all templates
nats-cli template list

# List only prompt-helper templates
nats-cli template list --server prompt-helper
```

### Global Options

These options apply to all commands:

- `--config <PATH>` - Configuration file path (default: `config.toml`)
- `--verbose` - Enable verbose output (includes full envelope in response)
- `--no-color` - Disable colored output
- `--log-level <LEVEL>` - Override log level (`trace`, `debug`, `info`, `warn`, `error`)

**Examples:**

```bash
# Use custom config file
nats-cli --config production.toml send ...

# Disable colors and enable debug logging
nats-cli --no-color --log-level debug send ...
```

## Template System

Templates are JSON files that define MCP tool call requests. They're organized by server name under the `templates/` directory.

### Template Format

```json
{
  "tool_name": "generate_story_prompts",
  "arguments": {
    "theme": "Space Adventure",
    "age_group": "6-8",
    "language": "en"
  }
}
```

See [templates/README.md](templates/README.md) for complete documentation.

### Available Templates

#### prompt-helper Server

| Template | Description |
|----------|-------------|
| `generate_story_prompts.json` | Generate story prompts with educational goals |
| `generate_validation_prompts.json` | Generate content validation prompts |
| `generate_constraint_prompts.json` | Generate content constraint prompts |
| `get_model_for_language.json` | Get recommended LLM model for language |

## Configuration

Configuration is loaded hierarchically:

1. **Defaults** - Built-in default values
2. **config.toml** - Configuration file
3. **Environment Variables** - Prefixed with `NATS_CLI_`

### Configuration File Structure

```toml
[nats]
# NATS server URL
url = "nats://localhost:5222"

# Path to NKEY seed file
nkey_file = "../nkeys/nats-cli.nk"

[nats.tls]
# Path to CA certificate for TLS verification
ca_cert = "../certs/ca.pem"

[client]
# Default request timeout in seconds (1-300)
default_timeout_secs = 30

# Default tenant ID for multi-tenant isolation
default_tenant_id = 1

# Logging level: trace, debug, info, warn, error
log_level = "info"

# Enable colored output
colored_output = true

[envelope]
# Envelope version
version = "1.0"

# Validate response envelopes
validate_responses = true
```

### Environment Variables

Override configuration using environment variables:

```bash
# NATS connection
export NATS_CLI_NATS__URL="nats://production:4222"
export NATS_CLI_NATS__NKEY_FILE="/path/to/nkey.nk"

# Client settings
export NATS_CLI_CLIENT__DEFAULT_TIMEOUT_SECS=180
export NATS_CLI_CLIENT__DEFAULT_TENANT_ID=42
export NATS_CLI_CLIENT__LOG_LEVEL="debug"
```

## Output Format

### Response Display

Responses are displayed with colored, structured output:

```
================================================================================

=== Metadata ===
  Request ID: 550e8400-e29b-41d4-a716-446655440000
  Tenant: 1
  Timestamp: 2025-10-08T18:30:45.123Z
  Version: 1.0
  Protocol: mcp

✅ === Tool Result ===
  ✅ Content Block 1:
    {
      "prompts": [
        "Tell me about the solar system...",
        "What would you discover on Mars..."
      ]
    }

================================================================================
```

### Verbose Mode

Add `--verbose` to see the complete envelope structure:

```bash
nats-cli send --subject mcp.prompt.helper \
  --template templates/prompt-helper/get_model_for_language.json \
  --verbose
```

## Examples

### Basic Request

```bash
# Get recommended model for English
cargo run -- send \
  --subject mcp.prompt.helper \
  --template templates/prompt-helper/get_model_for_language.json
```

### Custom Tenant

```bash
# Send request with specific tenant ID
cargo run -- send \
  --subject mcp.prompt.helper \
  --template templates/prompt-helper/generate_story_prompts.json \
  --tenant 5
```

### Extended Timeout

```bash
# Use 60 second timeout for slow operations
cargo run -- send \
  --subject mcp.prompt.helper \
  --template templates/prompt-helper/generate_validation_prompts.json \
  --timeout 60
```

### Debug Mode

```bash
# Enable debug logging to troubleshoot issues
cargo run -- --log-level debug send \
  --subject mcp.prompt.helper \
  --template templates/prompt-helper/get_model_for_language.json
```

## Architecture

### Envelope-First Pattern

All requests are wrapped in Qollective envelopes:

```rust
Envelope {
    meta: Meta {
        request_id: Some(Uuid),
        tenant: Some("1"),
        timestamp: Some(DateTime),
        version: Some("1.0"),
        protocol: Some("mcp"),
        source_service: Some("nats-cli"),
        ...
    },
    payload: McpData {
        tool_call: Some(CallToolRequest {
            params: CallToolParams {
                name: "tool_name",
                arguments: Some({...}),
            }
        }),
        ...
    },
    error: None,
}
```

### Request Flow

1. **Template Loading** - JSON template parsed into `CallToolRequest`
2. **Envelope Creation** - Request wrapped with metadata (tenant, timestamps, IDs)
3. **Serialization** - Envelope serialized to JSON
4. **NATS Request** - Sent to subject with request-reply pattern
5. **Response Parsing** - Response envelope deserialized and validated
6. **Output Display** - Pretty-printed with colored formatting

### Security

- **NKEY Authentication**: Cryptographic signatures for client identity
- **TLS Encryption**: All communication encrypted with TLS 1.2/1.3
- **CA Verification**: Server certificates validated against CA cert
- **Tenant Isolation**: Multi-tenant support with tenant ID tracking

## Troubleshooting

### Connection Failed

```
❌ Failed to connect to NATS: connection refused
```

**Solutions:**
- Verify NATS server is running: `nats-server --config nats-server.conf`
- Check URL in config: `url = "nats://localhost:5222"`
- Verify network connectivity

### Authentication Error

```
❌ Authentication error: invalid signature
```

**Solutions:**
- Check NKEY file path: `nkey_file = "../nkeys/nats-cli.nk"`
- Verify NKEY is registered in NATS server config
- Ensure NKEY file contains valid seed

### TLS Error

```
❌ TLS error: unable to get local issuer certificate
```

**Solutions:**
- Check CA cert path: `ca_cert = "../certs/ca.pem"`
- Verify CA cert matches server's certificate authority
- Ensure CA cert file is readable

### Template Not Found

```
❌ Template not found: templates/prompt-helper/missing.json
```

**Solutions:**
- Verify template file exists
- Check path is relative to current directory
- Use `nats-cli template list` to see available templates

### Request Timeout

```
❌ Request timed out after 30 seconds
```

**Solutions:**
- Increase timeout: `--timeout 60`
- Check server is processing requests
- Verify subject name matches server subscription

### Invalid Response

```
❌ Server error: Tool execution failed
```

**Solutions:**
- Check server logs for detailed error messages
- Verify template arguments match tool requirements
- Use `--verbose` to see full response envelope

## Development

### Project Structure

```
nats-cli/
├── Cargo.toml                  # Package manifest
├── config.toml                 # Configuration file
├── README.md                   # This file
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── client.rs               # NATS client wrapper
│   ├── config.rs               # Configuration management
│   ├── templates.rs            # Template discovery/loading
│   ├── output.rs               # Response formatting
│   ├── errors.rs               # Error types
│   └── constants.rs            # Constants (CONSTANTS FIRST)
└── templates/
    ├── README.md               # Template documentation
    └── prompt-helper/
        ├── generate_story_prompts.json
        ├── generate_validation_prompts.json
        ├── generate_constraint_prompts.json
        └── get_model_for_language.json
```

### Adding New Templates

1. Create JSON file in `templates/<server-name>/`
2. Follow template format (see [templates/README.md](templates/README.md))
3. Test with `nats-cli template list`

### Code Style

- **CONSTANTS FIRST**: All hardcoded values in `constants.rs`
- **Configuration Inheritance**: Defaults → config.toml → Environment
- **Error Handling**: Structured errors with `thiserror`
- **Logging**: Tracing with configurable levels

## Dependencies

- **qollective** - Envelope-first architecture and types
- **shared-types** - TaleTrail common types and NKEY helpers
- **rmcp** - MCP protocol types
- **async-nats** - NATS client with async support
- **clap** - Command-line argument parsing
- **figment** - Hierarchical configuration
- **colored** - Terminal output coloring
- **serde/serde_json** - Serialization

## License

Part of the Qollective TaleTrail Content Generator project.

## Related Documentation

- [TaleTrail Project README](../README.md)
- [Template Documentation](templates/README.md)
- [NKEY Authentication Implementation](../NKEY_AUTHENTICATION_IMPLEMENTATION.md)
- [Qollective Framework Documentation](../../../README.md)
