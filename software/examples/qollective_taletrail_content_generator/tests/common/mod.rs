//! Common Test Utilities for TaleTrail Content Generator
//!
//! This module provides shared utilities for integration tests including:
//! - Infrastructure availability checks
//! - Test skipping macros
//! - Test initialization helpers
//! - Common configuration builders
//!
//! # Usage
//!
//! ```rust
//! use common::*;
//!
//! #[tokio::test]
//! async fn infra_nats_connection_test() {
//!     skip_if_no_infra!();
//!     init_rustls();
//!
//!     // test code requiring infrastructure
//! }
//! ```
//!
//! # Environment Variables
//!
//! - `ENABLE_INFRA_TESTS=1` - Enable infrastructure-dependent tests
//! - `NATS_URL` - Override NATS server URL (default: nats://localhost:5222)
//! - `ANTHROPIC_API_KEY` - API key for Anthropic Claude
//! - `OPENAI_API_KEY` - API key for OpenAI

use std::sync::Once;

/// Check if infrastructure tests should run
///
/// Returns true if ENABLE_INFRA_TESTS environment variable is set to any value.
/// This allows tests requiring external infrastructure (NATS, databases, etc.)
/// to be skipped during normal test runs and only enabled when infrastructure is available.
///
/// # Examples
///
/// ```
/// if should_run_infra_tests() {
///     // Run tests requiring NATS, databases, etc.
/// }
/// ```
pub fn should_run_infra_tests() -> bool {
    std::env::var("ENABLE_INFRA_TESTS").is_ok()
}

/// Skip test if infrastructure is not available
///
/// Prints a message to stderr and returns early if ENABLE_INFRA_TESTS is not set.
/// This macro should be called at the beginning of any test that requires external infrastructure.
///
/// # Usage
///
/// ```
/// #[tokio::test]
/// async fn infra_nats_test() {
///     skip_if_no_infra!();
///     // test code requiring infrastructure
/// }
/// ```
#[macro_export]
macro_rules! skip_if_no_infra {
    () => {
        if !$crate::common::should_run_infra_tests() {
            eprintln!("Skipping: Set ENABLE_INFRA_TESTS=1 to run infrastructure tests");
            return;
        }
    };
}

/// Initialize rustls crypto provider once for all tests
///
/// Uses `std::sync::Once` to ensure the crypto provider is only installed once,
/// even if called multiple times across different tests. This is required before
/// using any rustls TLS functionality.
///
/// # Examples
///
/// ```
/// #[tokio::test]
/// async fn test_with_tls() {
///     init_rustls();
///     // use TLS connections
/// }
/// ```
pub fn init_rustls() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        // Install the default ring crypto provider for rustls
        // Ignore error if already installed by another test
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// Initialize tracing subscriber for tests
///
/// Sets up a tracing subscriber with test writer and environment-based filtering.
/// Uses `std::sync::Once` to ensure it's only initialized once per test run.
/// Reads `RUST_LOG` environment variable for log level control (defaults to "info").
///
/// # Examples
///
/// ```
/// #[tokio::test]
/// async fn test_with_logging() {
///     init_test_tracing();
///     tracing::info!("Test starting");
///     // test code with tracing
/// }
/// ```
pub fn init_test_tracing() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .init();
    });
}

/// Create test NATS URL
///
/// Returns the NATS server URL from the NATS_URL environment variable,
/// or defaults to localhost:5222 if not set. This allows tests to connect
/// to different NATS servers in different environments.
///
/// # Examples
///
/// ```
/// let nats_url = test_nats_url();
/// let client = async_nats::connect(&nats_url).await?;
/// ```
pub fn test_nats_url() -> String {
    std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:5222".to_string())
}

/// Get TLS certificate paths for tests
///
/// Returns a tuple of (ca_cert, client_cert, client_key) paths pointing to
/// the test certificates in the `/software/tests/certs` directory.
/// These certificates are used for TLS and mTLS testing.
///
/// # Returns
///
/// Tuple of (&str, &str, &str) containing:
/// - CA certificate path
/// - Client certificate path
/// - Client private key path
///
/// # Examples
///
/// ```
/// let (ca_cert, client_cert, client_key) = test_cert_paths();
/// // Load certificates for TLS configuration
/// ```
pub fn test_cert_paths() -> (&'static str, &'static str, &'static str) {
    const CA_CERT: &str = "../../../tests/certs/ca.pem";
    const CLIENT_CERT: &str = "../../../tests/certs/client-cert.pem";
    const CLIENT_KEY: &str = "../../../tests/certs/client-key.pem";
    (CA_CERT, CLIENT_CERT, CLIENT_KEY)
}

/// Check if API keys are available
///
/// Returns true if both ANTHROPIC_API_KEY and OPENAI_API_KEY environment
/// variables are set. This allows LLM-dependent tests to be skipped when
/// API keys are not available.
///
/// # Examples
///
/// ```
/// if api_keys_available() {
///     // Run tests requiring LLM API calls
/// }
/// ```
pub fn api_keys_available() -> bool {
    std::env::var("ANTHROPIC_API_KEY").is_ok() && std::env::var("OPENAI_API_KEY").is_ok()
}

/// Skip test if API keys are not available
///
/// Prints a message to stderr and returns early if required API keys are not set.
/// This macro should be called at the beginning of any test that requires LLM API access.
///
/// # Usage
///
/// ```
/// #[tokio::test]
/// async fn test_llm_generation() {
///     skip_if_no_api_keys!();
///     // test code requiring API keys
/// }
/// ```
#[macro_export]
macro_rules! skip_if_no_api_keys {
    () => {
        if !$crate::common::api_keys_available() {
            eprintln!("Skipping: Set ANTHROPIC_API_KEY and OPENAI_API_KEY to run LLM tests");
            return;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_run_infra_tests() {
        // This test just verifies the function doesn't panic
        let _ = should_run_infra_tests();
    }

    #[test]
    fn test_init_rustls_multiple_calls() {
        // Should not panic when called multiple times
        init_rustls();
        init_rustls();
    }

    #[test]
    fn test_init_test_tracing_multiple_calls() {
        // Should not panic when called multiple times
        init_test_tracing();
        init_test_tracing();
    }

    #[test]
    fn test_nats_url_default() {
        // Remove env var if set
        std::env::remove_var("NATS_URL");
        let url = test_nats_url();
        assert_eq!(url, "nats://localhost:5222");
    }

    #[test]
    fn test_nats_url_from_env() {
        // Set custom URL
        std::env::set_var("NATS_URL", "nats://custom:4222");
        let url = test_nats_url();
        assert_eq!(url, "nats://custom:4222");
        std::env::remove_var("NATS_URL");
    }

    #[test]
    fn test_cert_paths_function() {
        let (ca, client, key) = test_cert_paths();
        assert!(ca.ends_with("ca.pem"));
        assert!(client.ends_with("client-cert.pem"));
        assert!(key.ends_with("client-key.pem"));
    }

    #[test]
    fn test_api_keys_available() {
        // This test just verifies the function doesn't panic
        let _ = api_keys_available();
    }
}
