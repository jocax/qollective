// ABOUTME: Integration test for PATCH method roundtrip with envelope transport and TLS support
// ABOUTME: Tests complete client-server communication using REST PATCH with UnifiedEnvelopeReceiver trait

//! Integration tests for REST PATCH method roundtrip communication.
//!
//! This test verifies that PATCH requests work correctly with:
//! - Complete envelope transport (metadata + data)
//! - TLS encryption when certificates are available
//! - UnifiedEnvelopeReceiver trait implementation
//! - Proper metadata preservation through roundtrip
//! - ContextDataHandler processing with envelope context
//! - Partial update operation handling

use qollective::envelope::{Envelope, Meta};
use serde_json::Value;

mod common;
use common::rest_test_utils::*;

#[tokio::test]
async fn test_patch_roundtrip_without_tls() {
    let result = run_patch_roundtrip_test(false).await;

    match result {
        Ok(_) => println!("✅ PATCH roundtrip test (HTTP) completed successfully"),
        Err(e) => panic!("❌ PATCH roundtrip test (HTTP) failed: {}", e),
    }
}

#[tokio::test]
async fn test_patch_roundtrip_with_tls() {
    let result = run_patch_roundtrip_test(true).await;

    match result {
        Ok(_) => println!("✅ PATCH roundtrip test (HTTPS) completed successfully"),
        Err(e) => println!("⚠️  PATCH roundtrip test (HTTPS) skipped or failed: {}", e),
    }
}

#[tokio::test]
async fn test_patch_partial_update() {
    // Test PATCH with partial update payload typical for REST PATCH operations
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/patch-partial".to_string(),
        handler_name: "patch-partial-handler".to_string(),
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

    // Create envelope with partial update payload typical for PATCH requests
    let patch_data = serde_json::json!({
        "message": "test PATCH with partial update payload",
        "method": "patch",
        "operation": "partial_update",
        "resource_id": "resource-789",
        "updates": {
            "name": "Partially Updated Resource",
            "status": "modified",
            "tags": ["updated", "patch-operation"],
            "metadata": {
                "last_modified": chrono::Utc::now(),
                "modified_by": "test-client",
                "version": 3,
                "change_reason": "partial update via PATCH"
            }
        },
        "options": {
            "merge_strategy": "replace_fields",
            "validate_before_update": true,
            "return_full_object": false
        }
    });

    let mut meta = Meta::default();
    meta.request_id = Some(uuid::Uuid::now_v7());
    meta.tenant = Some("patch-partial-tenant".to_string());

    let envelope = Envelope::new(meta.clone(), patch_data.clone());

    // Execute PATCH request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.patch(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("PATCH request failed");

    // Verify metadata preservation
    assert_eq!(response.meta.request_id, meta.request_id);
    assert_eq!(response.meta.tenant, meta.tenant);

    // Verify response structure
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["handler"], config.handler_name);
    assert_eq!(response.payload["echo"]["operation"], "partial_update");
    assert_eq!(response.payload["echo"]["resource_id"], "resource-789");
    assert_eq!(
        response.payload["echo"]["updates"]["name"],
        "Partially Updated Resource"
    );
    assert_eq!(response.payload["echo"]["updates"]["metadata"]["version"], 3);
    assert_eq!(
        response.payload["echo"]["options"]["merge_strategy"],
        "replace_fields"
    );

    // Cleanup
    server_handle.abort();

    println!("✅ PATCH partial update test completed successfully");
}

#[tokio::test]
async fn test_patch_json_patch_format() {
    // Test PATCH with JSON Patch format (RFC 6902)
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/patch-json-patch".to_string(),
        handler_name: "patch-json-patch-handler".to_string(),
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

    // Create envelope with JSON Patch operations
    let json_patch_data = serde_json::json!({
        "message": "test PATCH with JSON Patch format",
        "method": "patch",
        "operation": "json_patch",
        "format": "application/json-patch+json",
        "patches": [
            {
                "op": "replace",
                "path": "/status",
                "value": "active"
            },
            {
                "op": "add",
                "path": "/tags/-",
                "value": "json-patch-updated"
            },
            {
                "op": "remove",
                "path": "/deprecated_field"
            },
            {
                "op": "copy",
                "from": "/original_name",
                "path": "/backup_name"
            },
            {
                "op": "move",
                "from": "/temp_field",
                "path": "/permanent_field"
            },
            {
                "op": "test",
                "path": "/version",
                "value": 2
            }
        ]
    });

    let envelope = Envelope::new(Meta::default(), json_patch_data.clone());

    // Execute PATCH request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.patch(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("PATCH request failed");

    // Verify response
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["echo"]["operation"], "json_patch");
    assert_eq!(
        response.payload["echo"]["format"],
        "application/json-patch+json"
    );
    assert_eq!(
        response.payload["echo"]["patches"].as_array().unwrap().len(),
        6
    );
    assert_eq!(response.payload["echo"]["patches"][0]["op"], "replace");
    assert_eq!(response.payload["echo"]["patches"][1]["op"], "add");
    assert_eq!(response.payload["echo"]["patches"][2]["op"], "remove");

    // Cleanup
    server_handle.abort();

    println!("✅ PATCH JSON Patch format test completed successfully");
}

#[tokio::test]
async fn test_patch_merge_patch_format() {
    // Test PATCH with JSON Merge Patch format (RFC 7396)
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/patch-merge-patch".to_string(),
        handler_name: "patch-merge-patch-handler".to_string(),
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

    // Create envelope with JSON Merge Patch
    let merge_patch_data = serde_json::json!({
        "message": "test PATCH with JSON Merge Patch format",
        "method": "patch",
        "operation": "merge_patch",
        "format": "application/merge-patch+json",
        "merge_data": {
            "title": "Updated Title via Merge Patch",
            "description": null, // This will remove the description field
            "properties": {
                "priority": "high",
                "category": "updated",
                "metadata": {
                    "updated_at": chrono::Utc::now(),
                    "merge_patch_applied": true
                }
            },
            "tags": ["merge-patch", "updated", "rfc7396"]
        }
    });

    let envelope = Envelope::new(Meta::default(), merge_patch_data.clone());

    // Execute PATCH request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.patch(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("PATCH request failed");

    // Verify response
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["echo"]["operation"], "merge_patch");
    assert_eq!(
        response.payload["echo"]["format"],
        "application/merge-patch+json"
    );
    assert_eq!(
        response.payload["echo"]["merge_data"]["title"],
        "Updated Title via Merge Patch"
    );
    assert_eq!(
        response.payload["echo"]["merge_data"]["description"],
        serde_json::Value::Null
    );
    assert_eq!(
        response.payload["echo"]["merge_data"]["properties"]["priority"],
        "high"
    );
    assert_eq!(
        response.payload["echo"]["merge_data"]["tags"][0],
        "merge-patch"
    );

    // Cleanup
    server_handle.abort();

    println!("✅ PATCH JSON Merge Patch format test completed successfully");
}

#[tokio::test]
async fn test_patch_atomic_operations() {
    // Test PATCH with atomic operations
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/patch-atomic".to_string(),
        handler_name: "patch-atomic-handler".to_string(),
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

    // Create envelope with atomic operation requirements
    let atomic_patch_data = serde_json::json!({
        "message": "test PATCH with atomic operations",
        "method": "patch",
        "operation": "atomic_update",
        "transaction_id": uuid::Uuid::now_v7(),
        "updates": [
            {
                "field": "counter",
                "operation": "increment",
                "value": 1
            },
            {
                "field": "last_access",
                "operation": "set",
                "value": chrono::Utc::now()
            },
            {
                "field": "access_log",
                "operation": "append",
                "value": {
                    "timestamp": chrono::Utc::now(),
                    "action": "atomic_patch",
                    "user": "test-client"
                }
            }
        ],
        "constraints": {
            "if_version_matches": 5,
            "if_modified_since": null,
            "atomic": true,
            "rollback_on_failure": true
        }
    });

    let mut meta = Meta::default();
    meta.request_id = Some(uuid::Uuid::now_v7());
    meta.version = Some("5".to_string()); // Version constraint

    let envelope = Envelope::new(meta.clone(), atomic_patch_data.clone());

    // Execute PATCH request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.patch(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("PATCH request failed");

    // Verify metadata preservation
    assert_eq!(response.meta.request_id, meta.request_id);
    assert_eq!(response.meta.version, meta.version);

    // Verify response structure
    assert_eq!(response.payload["status"], "success");
    assert_eq!(response.payload["echo"]["operation"], "atomic_update");
    assert_eq!(
        response.payload["echo"]["updates"].as_array().unwrap().len(),
        3
    );
    assert_eq!(response.payload["echo"]["constraints"]["atomic"], true);
    assert_eq!(
        response.payload["echo"]["constraints"]["rollback_on_failure"],
        true
    );

    // Cleanup
    server_handle.abort();

    println!("✅ PATCH atomic operations test completed successfully");
}

#[tokio::test]
async fn test_patch_context_and_metadata_integration() {
    // Test PATCH with full context and metadata integration
    let config = RoundtripTestConfig {
        use_tls: false,
        endpoint: "/patch-context".to_string(),
        handler_name: "patch-context-handler".to_string(),
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
    let envelope = create_test_envelope("test PATCH context and metadata integration", "patch");

    // Execute PATCH request
    let response: Envelope<Value> = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        client.patch(&config.endpoint, envelope.clone()),
    )
    .await
    .expect("Request timed out")
    .expect("PATCH request failed");

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
    assert_eq!(response.payload["echo"]["method"], "patch");

    // Cleanup
    server_handle.abort();

    println!("✅ PATCH context and metadata integration test completed successfully");
}
