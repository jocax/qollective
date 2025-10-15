//! Example Integration Test demonstrating common test utilities usage
//!
//! This file shows how to use the shared test utilities module.

// Import the common utilities module
mod common;

// Import specific items we need
use common::*;

/// Example test that requires infrastructure
///
/// This test will be skipped unless ENABLE_INFRA_TESTS=1 is set
#[tokio::test]
async fn example_infra_test() {
    // Skip if infrastructure not available
    skip_if_no_infra!();

    // Initialize rustls for TLS connections
    init_rustls();

    // Initialize tracing for logging
    init_test_tracing();

    // Get NATS URL (from env or default)
    let nats_url = test_nats_url();
    tracing::info!("Would connect to NATS at: {}", nats_url);

    // Get certificate paths
    let (ca_cert, client_cert, client_key) = test_cert_paths();
    tracing::info!(
        "Certificate paths: ca={}, client={}, key={}",
        ca_cert,
        client_cert,
        client_key
    );

    // Your actual test code would go here
    assert!(true, "Infrastructure test passed");
}

/// Example test that requires API keys
///
/// This test will be skipped unless both ANTHROPIC_API_KEY and OPENAI_API_KEY are set
#[tokio::test]
async fn example_llm_test() {
    // Skip if API keys not available
    skip_if_no_api_keys!();

    // Initialize tracing for logging
    init_test_tracing();

    tracing::info!("API keys are available");

    // Your actual LLM test code would go here
    assert!(true, "LLM test passed");
}

/// Example test that always runs (no infrastructure requirements)
#[tokio::test]
async fn example_unit_test() {
    // Initialize tracing (optional for unit tests)
    init_test_tracing();

    tracing::info!("Running unit test");

    // Your actual unit test code
    assert_eq!(2 + 2, 4);
}

/// Example test combining both infrastructure and API key checks
#[tokio::test]
async fn example_full_integration_test() {
    // Check both infrastructure and API keys
    skip_if_no_infra!();
    skip_if_no_api_keys!();

    // Initialize everything
    init_rustls();
    init_test_tracing();

    let nats_url = test_nats_url();
    tracing::info!("Full integration test with NATS: {}", nats_url);

    // Your actual full integration test code would go here
    assert!(true, "Full integration test passed");
}
