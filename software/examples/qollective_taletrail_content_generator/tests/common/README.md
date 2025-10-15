# Common Test Utilities

Shared utilities for integration tests in the TaleTrail Content Generator project.

## Overview

This module provides:
- Infrastructure availability checks
- Test skipping macros
- Test initialization helpers (rustls, tracing)
- Common configuration builders

## Usage

### Basic Import

In your test file (e.g., `tests/my_integration_test.rs`):

```rust
mod common;
use common::*;
```

### Infrastructure Tests

Tests that require external services (NATS, databases, etc.):

```rust
#[tokio::test]
async fn test_nats_connection() {
    // Skip if ENABLE_INFRA_TESTS not set
    skip_if_no_infra!();

    // Initialize TLS support
    init_rustls();

    // Get NATS URL (from env or default)
    let nats_url = test_nats_url();

    // Your test code here
}
```

### LLM API Tests

Tests that require API keys:

```rust
#[tokio::test]
async fn test_llm_generation() {
    // Skip if API keys not available
    skip_if_no_api_keys!();

    // Initialize tracing for logging
    init_test_tracing();

    // Your test code here
}
```

### TLS Tests

Tests requiring TLS certificates:

```rust
#[tokio::test]
async fn test_tls_connection() {
    skip_if_no_infra!();
    init_rustls();

    // Get certificate paths
    let (ca_cert, client_cert, client_key) = test_cert_paths();

    // Load certificates and test
}
```

## Environment Variables

### Required for Infrastructure Tests

```bash
export ENABLE_INFRA_TESTS=1
```

### Optional Configuration

```bash
# Override NATS URL (default: nats://localhost:5222)
export NATS_URL=nats://custom-server:4222

# Enable debug logging
export RUST_LOG=debug
```

### Required for LLM Tests

```bash
export ANTHROPIC_API_KEY=your_key_here
export OPENAI_API_KEY=your_key_here
```

## Running Tests

### All tests (skip infrastructure tests):
```bash
cargo test
```

### With infrastructure tests:
```bash
ENABLE_INFRA_TESTS=1 cargo test
```

### With LLM tests:
```bash
ANTHROPIC_API_KEY=xxx OPENAI_API_KEY=yyy cargo test
```

### With everything:
```bash
ENABLE_INFRA_TESTS=1 \
ANTHROPIC_API_KEY=xxx \
OPENAI_API_KEY=yyy \
RUST_LOG=debug \
cargo test
```

### Single-threaded (required for some tests):
```bash
cargo test -- --test-threads=1
```

## API Reference

### Functions

#### `should_run_infra_tests() -> bool`
Check if infrastructure tests should run.

#### `init_rustls()`
Initialize rustls crypto provider (safe to call multiple times).

#### `init_test_tracing()`
Initialize tracing subscriber for tests (safe to call multiple times).

#### `test_nats_url() -> String`
Get NATS URL from env or default.

#### `test_cert_paths() -> (&'static str, &'static str, &'static str)`
Get paths to test certificates (ca, client_cert, client_key).

#### `api_keys_available() -> bool`
Check if LLM API keys are available.

### Macros

#### `skip_if_no_infra!()`
Skip test if infrastructure not available.

#### `skip_if_no_api_keys!()`
Skip test if API keys not available.

## Examples

See `tests/example_usage.rs` for comprehensive examples of all features.

## Certificate Paths

The test certificates are located at:
- CA Certificate: `../../../tests/certs/ca.pem`
- Client Certificate: `../../../tests/certs/client-cert.pem`
- Client Key: `../../../tests/certs/client-key.pem`

These paths are relative to the test file location and point to the shared test certificates in `/software/tests/certs/`.

## Design Principles

1. **Safe defaults**: Tests should be skippable when infrastructure is unavailable
2. **Single initialization**: Use `std::sync::Once` to prevent multiple initialization
3. **Clear error messages**: Print helpful messages when tests are skipped
4. **Environment-driven**: Use env vars for configuration flexibility
5. **Zero dependencies**: Use only std lib where possible

## Integration with Existing Tests

To add these utilities to existing integration tests:

1. Add the import:
   ```rust
   mod common;
   use common::*;
   ```

2. Add skip checks at the beginning of infrastructure tests:
   ```rust
   #[tokio::test]
   async fn my_existing_test() {
       skip_if_no_infra!();  // Add this line
       // existing test code...
   }
   ```

3. Replace hardcoded initialization with utility functions:
   ```rust
   // Before:
   rustls::crypto::ring::default_provider().install_default().unwrap();

   // After:
   init_rustls();
   ```

## Testing the Common Module

The module includes its own test suite:

```bash
cargo test --lib common
```

This verifies that all utility functions work correctly without requiring external infrastructure.
