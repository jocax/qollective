# Test Utilities Usage Guide

## Overview

The `tests/common/` module provides shared utilities for integration tests across all workspace crates. This guide shows how to use these utilities in your test files.

## Quick Start

### For Tests at Workspace Root Level

If you have a test file in the workspace root `tests/` directory:

```rust
// tests/my_integration_test.rs
mod common;
use common::*;

#[tokio::test]
async fn test_something() {
    skip_if_no_infra!();
    init_rustls();
    init_test_tracing();

    // Your test code
}
```

### For Tests in Workspace Crates

If you have a test file in a crate's `tests/` directory (e.g., `orchestrator/tests/my_test.rs`):

```rust
// orchestrator/tests/my_test.rs

// Path must be relative to the crate's tests directory
#[path = "../../tests/common/mod.rs"]
mod common;
use common::*;

#[tokio::test]
async fn test_something() {
    skip_if_no_infra!();
    init_rustls();

    // Your test code
}
```

## Available Utilities

### Infrastructure Check Functions

#### `should_run_infra_tests() -> bool`
Checks if `ENABLE_INFRA_TESTS` environment variable is set.

```rust
if should_run_infra_tests() {
    // Run infrastructure-dependent tests
} else {
    // Skip or use mocks
}
```

#### `skip_if_no_infra!()`
Macro that returns early from test if infrastructure is not available.

```rust
#[tokio::test]
async fn test_nats_connection() {
    skip_if_no_infra!();  // Will return if ENABLE_INFRA_TESTS not set
    // Test code only runs when infrastructure is available
}
```

### API Key Check Functions

#### `api_keys_available() -> bool`
Checks if both `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` are set.

#### `skip_if_no_api_keys!()`
Macro that returns early from test if API keys are not available.

```rust
#[tokio::test]
async fn test_llm_generation() {
    skip_if_no_api_keys!();  // Will return if API keys not set
    // Test code only runs when API keys are available
}
```

### Initialization Functions

#### `init_rustls()`
Initializes rustls crypto provider. Safe to call multiple times.

```rust
#[tokio::test]
async fn test_with_tls() {
    init_rustls();  // Required before using TLS
    // Use TLS connections
}
```

#### `init_test_tracing()`
Initializes tracing subscriber for logging. Safe to call multiple times.

```rust
#[tokio::test]
async fn test_with_logging() {
    init_test_tracing();
    tracing::info!("Test starting");
    // Your test code with logging
}
```

### Configuration Functions

#### `test_nats_url() -> String`
Returns NATS URL from `NATS_URL` env var or defaults to `nats://localhost:5222`.

```rust
let nats_url = test_nats_url();
let client = async_nats::connect(&nats_url).await?;
```

#### `test_cert_paths() -> (&'static str, &'static str, &'static str)`
Returns tuple of (ca_cert, client_cert, client_key) paths.

```rust
let (ca_cert, client_cert, client_key) = test_cert_paths();
// Load certificates for TLS configuration
```

## Environment Variables

### Running Infrastructure Tests

```bash
# Enable infrastructure tests
export ENABLE_INFRA_TESTS=1
cargo test

# Or inline
ENABLE_INFRA_TESTS=1 cargo test
```

### Running LLM Tests

```bash
# Set API keys
export ANTHROPIC_API_KEY=your_key_here
export OPENAI_API_KEY=your_key_here
cargo test

# Or inline
ANTHROPIC_API_KEY=xxx OPENAI_API_KEY=yyy cargo test
```

### Custom NATS URL

```bash
# Override default NATS URL
export NATS_URL=nats://custom-server:4222
cargo test
```

### Debug Logging

```bash
# Enable debug logging
export RUST_LOG=debug
cargo test
```

## Complete Examples

### Infrastructure Test

```rust
#[path = "../../tests/common/mod.rs"]
mod common;
use common::*;

#[tokio::test]
async fn test_nats_connection() {
    // Skip if infrastructure not available
    skip_if_no_infra!();

    // Initialize requirements
    init_rustls();
    init_test_tracing();

    // Get NATS URL
    let nats_url = test_nats_url();
    tracing::info!("Connecting to NATS at {}", nats_url);

    // Connect to NATS
    let client = async_nats::connect(&nats_url)
        .await
        .expect("Failed to connect to NATS");

    // Test operations
    assert!(client.is_connected());
}
```

### LLM Test

```rust
#[path = "../../tests/common/mod.rs"]
mod common;
use common::*;

#[tokio::test]
async fn test_llm_generation() {
    // Skip if API keys not available
    skip_if_no_api_keys!();

    // Initialize tracing
    init_test_tracing();

    // Your LLM test code
    tracing::info!("Testing LLM generation");
    // ...
}
```

### TLS Test

```rust
#[path = "../../tests/common/mod.rs"]
mod common;
use common::*;

#[tokio::test]
async fn test_tls_connection() {
    skip_if_no_infra!();
    init_rustls();
    init_test_tracing();

    // Get certificate paths
    let (ca_cert, client_cert, client_key) = test_cert_paths();

    // Load certificates
    tracing::info!("Loading certs from: {}, {}, {}", ca_cert, client_cert, client_key);
    // ... load and use certificates
}
```

### Combined Infrastructure and LLM Test

```rust
#[path = "../../tests/common/mod.rs"]
mod common;
use common::*;

#[tokio::test]
async fn test_full_integration() {
    // Check both requirements
    skip_if_no_infra!();
    skip_if_no_api_keys!();

    // Initialize everything
    init_rustls();
    init_test_tracing();

    let nats_url = test_nats_url();
    tracing::info!("Full integration test with NATS: {}", nats_url);

    // Your full integration test code
}
```

## Adding to Existing Tests

To add these utilities to existing integration tests:

1. **Add the import at the top:**
   ```rust
   #[path = "../../tests/common/mod.rs"]
   mod common;
   use common::*;
   ```

2. **Add skip checks for infrastructure tests:**
   ```rust
   #[tokio::test]
   async fn existing_infra_test() {
       skip_if_no_infra!();  // Add this line
       // existing test code...
   }
   ```

3. **Replace hardcoded initialization:**
   ```rust
   // Before:
   rustls::crypto::ring::default_provider().install_default().unwrap();

   // After:
   init_rustls();
   ```

4. **Use configuration helpers:**
   ```rust
   // Before:
   let nats_url = "nats://localhost:5222";

   // After:
   let nats_url = test_nats_url();
   ```

## Running Tests

### All Tests (Skip Infrastructure)
```bash
cargo test
```

### With Infrastructure
```bash
ENABLE_INFRA_TESTS=1 cargo test
```

### Specific Workspace Member
```bash
cargo test -p orchestrator
```

### Single Test
```bash
cargo test test_nats_connection
```

### With Debug Output
```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Single-Threaded (Required for some tests)
```bash
cargo test -- --test-threads=1
```

## Best Practices

1. **Always use skip macros for infrastructure tests** - This ensures tests pass in CI/CD without infrastructure
2. **Initialize rustls before TLS operations** - Required for all TLS connections
3. **Use test_nats_url() instead of hardcoded URLs** - Allows flexibility in different environments
4. **Call init_test_tracing() for debugging** - Helps troubleshoot test failures
5. **Combine checks when needed** - Some tests may need both infrastructure and API keys

## Troubleshooting

### Test is skipped unexpectedly
- Check if you set `ENABLE_INFRA_TESTS=1`
- Verify API keys are set if using `skip_if_no_api_keys!()`

### TLS errors
- Make sure to call `init_rustls()` before any TLS operations
- Verify certificate paths are correct with `test_cert_paths()`

### NATS connection fails
- Check if NATS is running: `docker ps` or `nats-server -v`
- Override URL with `NATS_URL` environment variable
- Verify port is correct (default: 5222)

### No log output
- Set `RUST_LOG=debug` or `RUST_LOG=trace`
- Call `init_test_tracing()` in your test
- Use `-- --nocapture` flag with cargo test

## Additional Resources

- See `tests/common/README.md` for detailed API documentation
- See `tests/example_usage.rs` for comprehensive examples
- Check individual crate test files for real-world usage patterns
