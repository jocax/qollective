// ABOUTME: Test-only constants moved from src/constants.rs to keep production code clean
// ABOUTME: Contains mock values, test timeouts, and other testing-specific constants

//! Test constants for qollective testing.
//!
//! This module contains constants that are only used during testing,
//! moved from src/constants.rs to keep production code clean of test-specific values.

/// Mock agent ID for testing
pub const MOCK_AGENT_ID: &str = "test-agent-12345";

/// Mock external agent ID for testing
pub const MOCK_EXTERNAL_AGENT_ID: &str = "external-agent-67890";

/// Mock MCP server ID for testing
pub const MOCK_MCP_SERVER_ID: &str = "test-mcp-server";

/// Test timeout (shorter for faster tests)
pub const TEST_TIMEOUT_SECS: u64 = 5;

/// Default external domain for testing
pub const DEFAULT_EXTERNAL_DOMAIN: &str = "example.com";

/// Test helper to check if URL looks like an external endpoint
pub fn is_external_endpoint(url: &str) -> bool {
    url.contains(DEFAULT_EXTERNAL_DOMAIN) || url.contains("external")
}