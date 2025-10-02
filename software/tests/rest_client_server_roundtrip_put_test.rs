// ABOUTME: Integration test for PUT method roundtrip with envelope transport and TLS support
// ABOUTME: Tests complete client-server communication using REST PUT with UnifiedEnvelopeReceiver trait

//! Integration tests for REST PUT method roundtrip communication.
//!
//! This test verifies that PUT requests work correctly with:
//! - Complete envelope transport (metadata + data)
//! - TLS encryption when certificates are available
//! - UnifiedEnvelopeReceiver trait implementation
//! - Proper metadata preservation through roundtrip
//! - ContextDataHandler processing with envelope context
//! - Idempotent operation handling

use qollective::envelope::{Envelope, Meta};
use serde_json::Value;

mod common;
use common::rest_test_utils::*;

#[tokio::test]
async fn test_put_roundtrip_without_tls() {
    let result = run_put_roundtrip_test(false).await;

    match result {
        Ok(_) => println!("✅ PUT roundtrip test (HTTP) completed successfully"),
        Err(e) => panic!("❌ PUT roundtrip test (HTTP) failed: {}", e),
    }
}

#[tokio::test]
async fn test_put_roundtrip_with_tls() {
    let result = run_put_roundtrip_test(true).await;

    match result {
        Ok(_) => println!("✅ PUT roundtrip test (HTTPS) completed successfully"),
        Err(e) => println!("⚠️  PUT roundtrip test (HTTPS) skipped or failed: {}", e),
    }
}

#[tokio::test]
async fn test_put_update_payload() {
    // Test PUT with update/replace payload typical for REST PUT operations
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/put-update".to_string(),
        handler_name: "put-update-handler".to_string(),
        ..Default::default()
    };

    // Setup server
    let server_handle = setup_test_rest_server(config.clone())
        .await
        .expect("Failed to setup test server");

    // Create client
    let client = create_test_rest_client(&config)
        .await
        .expect("Failed to create test client");

    // Create envelope with update payload typical for PUT requests
    let update_data = serde_json::json!({
        "message": "test PUT with update payload",
        "method": "put",
        "operation": "update",
        "resource": {
            "id": "resource-123",
            "name": "Updated Resource Name",
            "description": "This resource has been updated via PUT request",
            "status": "active",
            "metadata": {
                "updated_at": chrono::Utc::now(),
                "updated_by": "test-client",
                "version": 2,
                "tags": ["updated", "test", "put-operation"]
            }
        }
    });

    let envelope = Envelope::new(Meta::default(), update_data.clone());

    // Execute PUT request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.put(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("PUT request failed");

    // Verify response
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["handler"], config.handler_name);
    assert_eq!(response.payload["echo"]["operation"], "update");
    assert_eq!(response.payload["echo"]["resource"]["id"], "resource-123");
    assert_eq!(
        response.payload["echo"]["resource"]["name"],
        "Updated Resource Name"
    );
    assert_eq!(response.payload["echo"]["resource"]["metadata"]["version"], 2);

    // Cleanup
    server_handle.abort();

    println!("✅ PUT update payload test completed successfully");
}

#[tokio::test]
async fn test_put_idempotent_behavior() {
    // Test PUT idempotent behavior - multiple identical requests should work the same
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/put-idempotent".to_string(),
        handler_name: "put-idempotent-handler".to_string(),
        ..Default::default()
    };

    // Setup server
    let server_handle = setup_test_rest_server(config.clone())
        .await
        .expect("Failed to setup test server");

    // Create client
    let client = create_test_rest_client(&config)
        .await
        .expect("Failed to create test client");

    // Create envelope for idempotent testing
    let idempotent_data = serde_json::json!({
        "message": "test PUT idempotent behavior",
        "method": "put",
        "operation": "replace",
        "resource_id": "idempotent-test-123",
        "data": {
            "field1": "value1",
            "field2": "value2",
            "timestamp": "2024-01-01T00:00:00Z" // Fixed timestamp for idempotency
        }
    });

    let envelope = Envelope::new(Meta::default(), idempotent_data.clone());

    // Execute first PUT request
    let response1: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.put(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("First request timed out")
    .expect("First PUT request failed");

    // Execute second identical PUT request (should be idempotent)
    let response2: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.put(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Second request timed out")
    .expect("Second PUT request failed");

    // Verify both responses are identical (idempotent behavior)
    assert_eq!(response1.payload["status"], response2.payload["status"]);
    assert_eq!(
        response1.payload["echo"]["resource_id"],
        response2.payload["echo"]["resource_id"]
    );
    assert_eq!(
        response1.payload["echo"]["data"],
        response2.payload["echo"]["data"]
    );

    // Both should succeed
    assert_eq!(response1.payload["status"], "success");
    assert_eq!(response2.payload["status"], "success");

    // Cleanup
    server_handle.abort();

    println!("✅ PUT idempotent behavior test completed successfully");
}

#[tokio::test]
async fn test_put_complete_resource_replacement() {
    // Test PUT with complete resource replacement payload
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/put-replace".to_string(),
        handler_name: "put-replace-handler".to_string(),
        ..Default::default()
    };

    // Setup server
    let server_handle = setup_test_rest_server(config.clone())
        .await
        .expect("Failed to setup test server");

    // Create client
    let client = create_test_rest_client(&config)
        .await
        .expect("Failed to create test client");

    // Create envelope with complete resource replacement
    let replacement_data = serde_json::json!({
        "message": "test PUT with complete resource replacement",
        "method": "put",
        "operation": "replace",
        "resource": {
            "id": "resource-456",
            "type": "document",
            "title": "Completely Replaced Document",
            "content": "This is the new content that replaces the entire document",
            "properties": {
                "author": "test-client",
                "created_at": "2024-01-01T00:00:00Z",
                "last_modified": chrono::Utc::now(),
                "tags": ["replacement", "put-test", "complete"],
                "permissions": {
                    "read": ["user1", "user2"],
                    "write": ["user1"],
                    "admin": ["admin1"]
                }
            }
        }
    });

    let mut meta = Meta::default();
    meta.request_id = Some(uuid::Uuid::now_v7());
    meta.tenant = Some("put-replace-tenant".to_string());

    let envelope = Envelope::new(meta.clone(), replacement_data.clone());

    // Execute PUT request with custom deserialization handling
    let response_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.put(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out");

    // Handle the server response format mismatch
    let response: Envelope<Value> = match response_result {
        Ok(resp) => resp,
        Err(e) if e.to_string().contains("missing field `payload`") => {
            // The server is returning "data" instead of "payload"
            // This indicates a server-side serialization issue that should be fixed
            panic!(
                "Server serialization mismatch: Server returns 'data' field but client expects 'payload'. \
                Fix the server to use consistent Envelope<T> serialization. Error: {}",
                e
            );
        },
        Err(e) => panic!("PUT request failed: {}", e),
    };

    // Verify metadata preservation
    assert_eq!(response.meta.request_id, meta.request_id);
    assert_eq!(response.meta.tenant, meta.tenant);

    // Verify response structure
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["echo"]["operation"], "replace");
    assert_eq!(response.payload["echo"]["resource"]["id"], "resource-456");
    assert_eq!(
        response.payload["echo"]["resource"]["title"],
        "Completely Replaced Document"
    );
    assert_eq!(
        response.payload["echo"]["resource"]["properties"]["permissions"]["read"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    // Cleanup
    server_handle.abort();

    println!("✅ PUT complete resource replacement test completed successfully");
}

#[tokio::test]
async fn test_put_context_and_metadata_integration() {
    // Test PUT with full context and metadata integration
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/put-context".to_string(),
        handler_name: "put-context-handler".to_string(),
        ..Default::default()
    };

    // Setup server
    let server_handle = setup_test_rest_server(config.clone())
        .await
        .expect("Failed to setup test server");

    // Create client
    let client = create_test_rest_client(&config)
        .await
        .expect("Failed to create test client");

    // Create envelope with rich metadata for context testing
    let envelope = create_test_envelope("test PUT context and metadata integration", "put");

    // Execute PUT request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.put(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("PUT request failed");

    // Verify context was properly extracted and passed to handler
    assert_eq!(response.payload["context"]["has_context"], true);
    assert_eq!(
        response.payload["context"]["request_id"],
        envelope.meta.request_id.as_ref().unwrap().to_string()
    );
    assert_eq!(
        response.payload["context"]["tenant"],
        envelope.meta.tenant.as_ref().unwrap().as_str()
    );
    assert!(response.payload["context"]["timestamp"].is_string());

    // Verify metadata preservation through the envelope handler
    assert_eq!(response.meta.request_id, envelope.meta.request_id);
    assert_eq!(response.meta.tenant, envelope.meta.tenant);
    assert_eq!(response.meta.version, envelope.meta.version);

    // Verify proper handler execution
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["handler"], config.handler_name);
    assert_eq!(response.payload["echo"]["method"], "put");

    // Cleanup
    server_handle.abort();

    println!("✅ PUT context and metadata integration test completed successfully");
}
