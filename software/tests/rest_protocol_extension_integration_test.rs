// ABOUTME: Simple HTTP-only integration test for REST protocol metadata extension
// ABOUTME: Tests protocol extension injection and access through Context for unified routing

//! Integration tests for REST protocol metadata extension feature.
//!
//! This test verifies that protocol metadata is correctly:
//! - Injected by REST placeholder handlers 
//! - Available through Context extensions for unified routing
//! - Contains correct HTTP method and URI path information
//! - Preserves optional query parameters and headers
//! - Works across different HTTP methods (GET, POST, PUT, DELETE, PATCH, OPTIONS)
//!
//! **Note**: This test uses HTTP only (no TLS) to focus on core functionality.

use qollective::constants::metadata::PROTOCOL_EXTENSION_KEY;
use qollective::envelope::{Context, Envelope, Meta};
use qollective::error::Result;
use qollective::prelude::{ContextDataHandler, UnifiedEnvelopeReceiver};
use qollective::server::common::ServerConfig;
use qollective::server::rest::{RestProtocolMetadata, RestServer, RestServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

mod common;
use common::{get_available_port, setup_test_environment};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestRequest {
    message: String,
    test_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResponse {
    echo: String,
    test_id: String,
    protocol_info: Option<RestProtocolMetadata>,
    extensions_available: bool,
}

/// Simple test handler that captures protocol metadata from context
struct ProtocolTestHandler {
    name: String,
}

impl ProtocolTestHandler {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl ContextDataHandler<TestRequest, TestResponse> for ProtocolTestHandler {
    async fn handle(&self, context: Option<Context>, data: TestRequest) -> Result<TestResponse> {
        // Extract protocol metadata from context extensions using the PROTOCOL_EXTENSION_KEY
        let protocol_info = context
            .as_ref()
            .and_then(|ctx| ctx.get_extension(PROTOCOL_EXTENSION_KEY))
            .and_then(|ext| serde_json::from_value::<RestProtocolMetadata>(ext.clone()).ok());

        let extensions_available = context
            .as_ref()
            .and_then(|ctx| ctx.extensions_ref())
            .is_some();

        Ok(TestResponse {
            echo: format!("Handler {} received: {}", self.name, data.message),
            test_id: data.test_id,
            protocol_info,
            extensions_available,
        })
    }
}

/// Start a simple HTTP REST server for testing
async fn setup_test_server(endpoint: &str) -> Result<(u16, tokio::task::JoinHandle<()>)> {
    let port = get_available_port();
    
    let server_config = RestServerConfig {
        base: ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut server = RestServer::new(server_config).await?;
    
    // Register our test handler
    server
        .receive_envelope_at(endpoint, ProtocolTestHandler::new("test-handler"))
        .await?;

    // Start server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Test server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(200)).await;

    Ok((port, server_handle))
}

/// Send a simple HTTP request to test endpoint
async fn send_test_request(
    method: &str,
    port: u16,
    endpoint: &str,
    query_params: Option<&HashMap<String, String>>,
    test_data: &TestRequest,
) -> Result<TestResponse> {
    // Build URL with query parameters
    let mut url = format!("http://127.0.0.1:{}{}", port, endpoint);
    if let Some(params) = query_params {
        let query_string: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        if !query_string.is_empty() {
            url.push_str(&format!("?{}", query_string.join("&")));
        }
    }

    // Create envelope with test data
    let envelope = Envelope::new(Meta::default(), test_data.clone());

    // Use reqwest for simple HTTP client
    let client = reqwest::Client::new();
    
    let response = match method {
        "GET" => {
            // For GET, we'll send envelope data as JSON body
            client.get(&url)
                .json(&envelope)
                .send()
                .await
        }
        "POST" => {
            client.post(&url)
                .json(&envelope)
                .send()
                .await
        }
        "PUT" => {
            client.put(&url)
                .json(&envelope)
                .send()
                .await
        }
        "DELETE" => {
            client.delete(&url)
                .json(&envelope)
                .send()
                .await
        }
        "PATCH" => {
            client.patch(&url)
                .json(&envelope)
                .send()
                .await
        }
        "OPTIONS" => {
            client.request(reqwest::Method::OPTIONS, &url)
                .json(&envelope)
                .send()
                .await
        }
        _ => return Err(qollective::error::QollectiveError::transport(
            format!("Unsupported HTTP method: {}", method)
        )),
    }.map_err(|e| qollective::error::QollectiveError::transport(format!("HTTP request failed: {}", e)))?;

    // Parse response
    let response_envelope: Envelope<TestResponse> = response
        .json()
        .await
        .map_err(|e| qollective::error::QollectiveError::transport(format!("Failed to parse response: {}", e)))?;

    Ok(response_envelope.payload)
}

/// Run a protocol extension test for a specific HTTP method
async fn run_protocol_test(
    method: &str,
    endpoint: &str,
    query_params: Option<HashMap<String, String>>,
) -> Result<()> {
    setup_test_environment();
    
    // Start test server
    let (port, server_handle) = setup_test_server(endpoint).await?;

    // Create test data
    let test_data = TestRequest {
        message: format!("Testing {} method", method),
        test_id: format!("test-{}-{}", method.to_lowercase(), chrono::Utc::now().timestamp()),
    };

    // Send request
    let response = send_test_request(
        method,
        port,
        endpoint,
        query_params.as_ref(),
        &test_data,
    ).await?;

    // Verify response
    assert!(response.extensions_available, 
        "Context should have extensions available");
    
    assert!(response.protocol_info.is_some(), 
        "Protocol metadata should be present in context");

    let protocol_info = response.protocol_info.unwrap();
    
    // Verify protocol metadata content
    assert_eq!(protocol_info.protocol_type, "rest", 
        "Protocol type should be 'rest'");
    
    assert_eq!(protocol_info.method, method, 
        "Method should match request method");
    
    assert_eq!(protocol_info.uri_path, endpoint, 
        "URI path should match request endpoint");

    // Verify query parameters if provided
    if let Some(expected_query_params) = &query_params {
        assert_eq!(protocol_info.query_params.as_ref(), Some(expected_query_params),
            "Query parameters should match");
    }

    // Verify test data was processed
    assert_eq!(response.test_id, test_data.test_id,
        "Test ID should match");

    // Stop server
    server_handle.abort();
    
    // Give server time to clean up port binding
    sleep(Duration::from_millis(100)).await;
    
    println!("✅ {} protocol extension test completed successfully", method);
    Ok(())
}

#[tokio::test]
async fn test_post_protocol_extension() {
    let result = run_protocol_test("POST", "/test-post", None).await;
    assert!(result.is_ok(), "POST protocol extension test failed: {:?}", result);
}

#[tokio::test]
async fn test_get_protocol_extension() {
    let result = run_protocol_test("GET", "/test-get", None).await;
    assert!(result.is_ok(), "GET protocol extension test failed: {:?}", result);
}

#[tokio::test]
async fn test_get_with_query_params() {
    let mut query_params = HashMap::new();
    query_params.insert("page".to_string(), "1".to_string());
    query_params.insert("limit".to_string(), "10".to_string());
    
    let result = run_protocol_test("GET", "/test-get-query", Some(query_params)).await;
    assert!(result.is_ok(), "GET with query params protocol extension test failed: {:?}", result);
}

#[tokio::test]
async fn test_put_protocol_extension() {
    let result = run_protocol_test("PUT", "/test-put", None).await;
    assert!(result.is_ok(), "PUT protocol extension test failed: {:?}", result);
}

#[tokio::test]
async fn test_delete_protocol_extension() {
    let result = run_protocol_test("DELETE", "/test-delete", None).await;
    assert!(result.is_ok(), "DELETE protocol extension test failed: {:?}", result);
}

#[tokio::test]
async fn test_patch_protocol_extension() {
    let result = run_protocol_test("PATCH", "/test-patch", None).await;
    assert!(result.is_ok(), "PATCH protocol extension test failed: {:?}", result);
}

#[tokio::test]
async fn test_options_protocol_extension() {
    let result = run_protocol_test("OPTIONS", "/test-options", None).await;
    assert!(result.is_ok(), "OPTIONS protocol extension test failed: {:?}", result);
}

#[tokio::test]
async fn test_complex_endpoint_with_query_params() {
    let mut query_params = HashMap::new();
    query_params.insert("filter".to_string(), "active".to_string());
    query_params.insert("sort".to_string(), "name".to_string());
    query_params.insert("include".to_string(), "details".to_string());

    let result = run_protocol_test("GET", "/api/v1/users/profile", Some(query_params)).await;
    assert!(result.is_ok(), "Complex endpoint protocol extension test failed: {:?}", result);
}

#[tokio::test]
async fn test_protocol_extension_data_integrity() {
    setup_test_environment();
    
    // This test verifies that the protocol metadata doesn't interfere with regular data flow
    let (port, server_handle) = setup_test_server("/test-integrity").await
        .expect("Failed to setup test server");

    let test_data = TestRequest {
        message: "Testing data integrity with protocol extensions".to_string(),
        test_id: "integrity-test-123".to_string(),
    };

    let response = send_test_request("POST", port, "/test-integrity", None, &test_data).await
        .expect("Failed to send test request");

    // Verify both protocol metadata AND regular data processing work
    assert!(response.protocol_info.is_some(), "Protocol info should be available");
    assert_eq!(response.test_id, test_data.test_id, "Regular data should be preserved");
    assert!(response.echo.contains(&test_data.message), "Echo should contain original message");
    
    let protocol_info = response.protocol_info.unwrap();
    assert_eq!(protocol_info.method, "POST");
    assert_eq!(protocol_info.uri_path, "/test-integrity");

    server_handle.abort();
    
    // Give server time to clean up port binding
    sleep(Duration::from_millis(100)).await;
    
    println!("✅ Protocol extension data integrity test completed successfully");
}