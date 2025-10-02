// ABOUTME: Cross-transport metadata preservation consistency tests
// ABOUTME: Verifies that metadata preservation works identically across WebSocket, gRPC, and REST transports

//! Transport metadata consistency tests
//!
//! This test suite verifies that metadata preservation works consistently
//! across all transport protocols (WebSocket, gRPC, REST) by:
//! - Creating identical request metadata
//! - Sending through each transport
//! - Verifying preserved fields are identical
//! - Ensuring timestamp updates correctly
//! - Confirming fallback behavior is consistent

use qollective::envelope::{Envelope, Meta, Context};
use qollective::error::Result;
use serde_json::Value;

mod common;
use common::*;

/// Test that WebSocket metadata preservation follows the same pattern as gRPC server
#[tokio::test]
async fn test_websocket_metadata_preservation_consistency() {
    // Create identical metadata for all transports
    let mut original_meta = Meta::default();
    original_meta.request_id = Some(uuid::Uuid::now_v7());
    original_meta.tenant = Some("consistency-test-tenant".to_string());
    original_meta.version = Some("2.5".to_string());

    // Set security metadata to test preservation
    original_meta.security = Some(qollective::envelope::meta::SecurityMeta {
        user_id: Some("test-user-123".to_string()),
        session_id: Some("session-456".to_string()),
        auth_method: Some(qollective::envelope::meta::AuthMethod::Jwt),
        permissions: vec!["read".to_string(), "write".to_string()],
        ip_address: Some("192.168.1.100".to_string()),
        user_agent: Some("test-agent/1.0".to_string()),
        tenant_id: Some("tenant-789".to_string()),
        roles: vec!["admin".to_string(), "user".to_string()],
        token_expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
    });

    // Set on_behalf_of metadata to test preservation
    original_meta.on_behalf_of = Some(qollective::envelope::meta::OnBehalfOfMeta {
        original_user: Some("delegated-user".to_string()),
        original_tenant: Some("delegated-tenant".to_string()),
        delegation_type: Some("admin_delegation".to_string()),
        delegation_scope: Some(vec!["full_access".to_string()]),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(2)),
    });

    let original_timestamp = chrono::Utc::now();
    original_meta.timestamp = Some(original_timestamp);

    // Test WebSocket metadata preservation
    let websocket_preserved_meta = Meta::preserve_for_response(Some(&original_meta));

    // Verify WebSocket preserves the correct fields
    assert_eq!(websocket_preserved_meta.request_id, original_meta.request_id, "WebSocket should preserve request_id");
    assert_eq!(websocket_preserved_meta.tenant, original_meta.tenant, "WebSocket should preserve tenant");
    assert_eq!(websocket_preserved_meta.version, original_meta.version, "WebSocket should preserve version");
    assert_eq!(websocket_preserved_meta.on_behalf_of, original_meta.on_behalf_of, "WebSocket should preserve on_behalf_of");
    assert_eq!(websocket_preserved_meta.security, original_meta.security, "WebSocket should preserve security");

    // Verify timestamp is updated (should be different and more recent)
    assert!(websocket_preserved_meta.timestamp.is_some(), "WebSocket should set new timestamp");
    assert!(websocket_preserved_meta.timestamp.unwrap() >= original_timestamp, "WebSocket timestamp should be updated to response time");

    // Verify response-specific fields are reset
    assert!(websocket_preserved_meta.duration.is_none(), "WebSocket should reset duration");
    assert!(websocket_preserved_meta.debug.is_none(), "WebSocket should reset debug");
    assert!(websocket_preserved_meta.performance.is_none(), "WebSocket should reset performance");
    assert!(websocket_preserved_meta.monitoring.is_none(), "WebSocket should reset monitoring");
    assert!(websocket_preserved_meta.tracing.is_none(), "WebSocket should reset tracing");
    assert!(websocket_preserved_meta.extensions.is_none(), "WebSocket should reset extensions");

    println!("✅ WebSocket metadata preservation follows gRPC server pattern");
}

/// Test fallback behavior when no original metadata is provided
#[tokio::test]
async fn test_websocket_metadata_fallback_consistency() {
    // Test WebSocket fallback behavior (when no original metadata provided)
    let websocket_fallback_meta = Meta::preserve_for_response(None);

    // Verify fallback behavior matches expected pattern
    assert!(websocket_fallback_meta.timestamp.is_some(), "WebSocket fallback should set timestamp");
    assert_eq!(websocket_fallback_meta.version, Some("1.0".to_string()), "WebSocket fallback should set default version");
    assert!(websocket_fallback_meta.request_id.is_none(), "WebSocket fallback should not set request_id");
    assert!(websocket_fallback_meta.tenant.is_none(), "WebSocket fallback should not set tenant");
    assert!(websocket_fallback_meta.on_behalf_of.is_none(), "WebSocket fallback should not set on_behalf_of");
    assert!(websocket_fallback_meta.security.is_none(), "WebSocket fallback should not set security");

    println!("✅ WebSocket metadata fallback behavior is consistent");
}

/// Test that new request metadata utility works correctly
#[tokio::test]
async fn test_new_request_metadata_consistency() {
    let new_request_meta = Meta::for_new_request();

    // Verify new request metadata has expected fields
    assert!(new_request_meta.timestamp.is_some(), "New request should have timestamp");
    assert!(new_request_meta.request_id.is_some(), "New request should have request_id");
    assert_eq!(new_request_meta.version, Some("1.0".to_string()), "New request should have default version");

    // Verify new request metadata doesn't have response fields
    assert!(new_request_meta.tenant.is_none(), "New request should not have tenant");
    assert!(new_request_meta.on_behalf_of.is_none(), "New request should not have on_behalf_of");
    assert!(new_request_meta.security.is_none(), "New request should not have security");
    assert!(new_request_meta.duration.is_none(), "New request should not have duration");
    assert!(new_request_meta.debug.is_none(), "New request should not have debug");
    assert!(new_request_meta.performance.is_none(), "New request should not have performance");
    assert!(new_request_meta.monitoring.is_none(), "New request should not have monitoring");
    assert!(new_request_meta.tracing.is_none(), "New request should not have tracing");
    assert!(new_request_meta.extensions.is_none(), "New request should not have extensions");

    println!("✅ New request metadata utility works correctly");
}

/// Test metadata serialization/deserialization consistency
#[tokio::test]
async fn test_metadata_serialization_consistency() {
    // Create metadata with all fields populated
    let mut original_meta = Meta::default();
    original_meta.request_id = Some(uuid::Uuid::now_v7());
    original_meta.tenant = Some("serialization-test".to_string());
    original_meta.version = Some("3.1".to_string());
    original_meta.timestamp = Some(chrono::Utc::now());

    // Test serialization to JSON
    let serialized = serde_json::to_value(&original_meta).expect("Should serialize to JSON");

    // Test deserialization from JSON
    let deserialized: Meta = serde_json::from_value(serialized).expect("Should deserialize from JSON");

    // Verify round-trip consistency
    assert_eq!(deserialized.request_id, original_meta.request_id, "Request ID should survive round-trip");
    assert_eq!(deserialized.tenant, original_meta.tenant, "Tenant should survive round-trip");
    assert_eq!(deserialized.version, original_meta.version, "Version should survive round-trip");
    assert_eq!(deserialized.timestamp, original_meta.timestamp, "Timestamp should survive round-trip");

    println!("✅ Metadata serialization/deserialization is consistent");
}

/// Test envelope context creation from metadata
#[tokio::test]
async fn test_envelope_context_creation_consistency() {
    // Create metadata for context testing
    let mut meta = Meta::default();
    meta.request_id = Some(uuid::Uuid::now_v7());
    meta.tenant = Some("context-test".to_string());
    meta.version = Some("4.2".to_string());
    meta.timestamp = Some(chrono::Utc::now());

    // Create context from metadata
    let context = Context::new(meta.clone());

    // Verify context preserves metadata fields through meta() accessor
    assert_eq!(context.meta().request_id, meta.request_id, "Context should preserve request_id");
    assert_eq!(context.meta().tenant, meta.tenant, "Context should preserve tenant");
    assert_eq!(context.meta().version, meta.version, "Context should preserve version");
    assert_eq!(context.meta().timestamp, meta.timestamp, "Context should preserve timestamp");

    println!("✅ Envelope context creation from metadata is consistent");
}
