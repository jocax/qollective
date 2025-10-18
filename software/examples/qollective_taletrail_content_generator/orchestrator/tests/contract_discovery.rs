//! Contract tests for MCP discovery protocol
//!
//! These tests verify schema consistency and serialization contracts
//! without requiring running services.

use shared_types::types::tool_registration::{ToolRegistration, DiscoveryInfo, ServiceCapabilities};
use serde_json;

#[test]
fn test_tool_registration_json_roundtrip() {
    // ARRANGE
    let registration = ToolRegistration::new(
        "test_tool",
        serde_json::json!({"type": "object", "properties": {"param": {"type": "string"}}}),
        "test-service",
        "0.0.1",
        vec![ServiceCapabilities::Batching, ServiceCapabilities::Retry],
    );

    // ACT
    let json = serde_json::to_string(&registration).expect("Serialization failed");
    let deserialized: ToolRegistration = serde_json::from_str(&json).expect("Deserialization failed");

    // ASSERT
    assert_eq!(registration.tool_name, deserialized.tool_name);
    assert_eq!(registration.service_name, deserialized.service_name);
    assert_eq!(registration.service_version, deserialized.service_version);
    assert_eq!(registration.capabilities, deserialized.capabilities);
    assert_eq!(registration.tool_schema, deserialized.tool_schema);
}

#[test]
fn test_discovery_info_json_roundtrip() {
    // ARRANGE
    let tools = vec![
        ToolRegistration::new(
            "tool1",
            serde_json::json!({"type": "object"}),
            "service1",
            "1.0.0",
            vec![ServiceCapabilities::Caching],
        ),
        ToolRegistration::new(
            "tool2",
            serde_json::json!({"type": "object"}),
            "service1",
            "1.0.0",
            vec![ServiceCapabilities::Retry],
        ),
    ];

    let discovery_info = DiscoveryInfo::healthy(tools.clone(), 3600);

    // ACT
    let json = serde_json::to_string(&discovery_info).expect("Serialization failed");
    let deserialized: DiscoveryInfo = serde_json::from_str(&json).expect("Deserialization failed");

    // ASSERT
    assert_eq!(discovery_info.service_health, deserialized.service_health);
    assert_eq!(discovery_info.uptime_seconds, deserialized.uptime_seconds);
    assert_eq!(discovery_info.available_tools.len(), deserialized.available_tools.len());
    assert_eq!(discovery_info.available_tools[0].tool_name, deserialized.available_tools[0].tool_name);
}

#[test]
fn test_service_capabilities_all_variants_serialize() {
    // ARRANGE
    let capabilities = vec![
        ServiceCapabilities::Batching,
        ServiceCapabilities::Streaming,
        ServiceCapabilities::Caching,
        ServiceCapabilities::Retry,
    ];

    // ACT
    let json = serde_json::to_string(&capabilities).expect("Serialization failed");
    let deserialized: Vec<ServiceCapabilities> = serde_json::from_str(&json).expect("Deserialization failed");

    // ASSERT
    assert_eq!(capabilities, deserialized);

    // Verify JSON format (snake_case)
    assert!(json.contains("batching"));
    assert!(json.contains("streaming"));
    assert!(json.contains("caching"));
    assert!(json.contains("retry"));
}

#[test]
fn test_tool_registration_schema_consistency() {
    // This test ensures that all ToolRegistration instances use consistent schema structure

    let registrations = vec![
        ToolRegistration::new("tool1", serde_json::json!({"type": "object"}), "s1", "1.0", vec![]),
        ToolRegistration::new("tool2", serde_json::json!({"type": "object"}), "s1", "1.0", vec![]),
    ];

    for reg in registrations {
        // All tool schemas should be JSON objects
        assert!(reg.tool_schema.is_object(), "Tool schema must be JSON object");

        // Service name should be non-empty
        assert!(!reg.service_name.is_empty());

        // Service version should follow semver (basic check)
        assert!(reg.service_version.contains('.'));
    }
}

#[test]
fn test_envelope_metadata_preserved_in_discovery() {
    // This test verifies that envelope metadata structure is compatible with discovery responses
    // We test the schema structure without requiring actual NATS connection

    use qollective::envelope::{Envelope, Meta};
    use qollective::types::mcp::{McpData, McpDiscoveryData};
    use uuid::Uuid;

    // ARRANGE - Create discovery request envelope
    let mut meta = Meta::default();
    meta.request_id = Some(Uuid::new_v4());
    meta.tenant = Some("test-tenant".to_string());

    let discovery_data = McpDiscoveryData {
        query_type: "list_tools".to_string(),
        tools: None,
        server_info: None,
    };

    let mcp_data = McpData {
        tool_call: None,
        tool_response: None,
        tool_registration: None,
        discovery_data: Some(discovery_data),
    };

    let envelope = Envelope::new(meta.clone(), mcp_data);

    // ACT - Serialize and deserialize
    let json = serde_json::to_string(&envelope).expect("Serialization failed");
    let deserialized: Envelope<McpData> = serde_json::from_str(&json).expect("Deserialization failed");

    // ASSERT - Metadata preserved
    assert_eq!(envelope.meta.request_id, deserialized.meta.request_id);
    assert_eq!(envelope.meta.tenant, deserialized.meta.tenant);
}

#[test]
fn test_discovery_data_serialization_roundtrip() {
    use qollective::types::mcp::McpDiscoveryData;

    let discovery_data = McpDiscoveryData {
        query_type: "list_tools".to_string(),
        tools: None,
        server_info: None,
    };

    let json = serde_json::to_string(&discovery_data).expect("Serialization failed");
    let deserialized: McpDiscoveryData = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(discovery_data.query_type, deserialized.query_type);
}

#[test]
fn test_tool_registration_with_empty_capabilities() {
    // ARRANGE
    let registration = ToolRegistration::new(
        "minimal_tool",
        serde_json::json!({"type": "object"}),
        "minimal-service",
        "0.1.0",
        vec![], // No capabilities
    );

    // ACT
    let json = serde_json::to_string(&registration).expect("Serialization failed");
    let deserialized: ToolRegistration = serde_json::from_str(&json).expect("Deserialization failed");

    // ASSERT
    assert_eq!(registration.capabilities.len(), 0);
    assert_eq!(deserialized.capabilities.len(), 0);
}

#[test]
fn test_discovery_info_degraded_status() {
    // ARRANGE
    let tools = vec![
        ToolRegistration::new(
            "partial_tool",
            serde_json::json!({"type": "object"}),
            "degraded-service",
            "1.0.0",
            vec![ServiceCapabilities::Retry],
        ),
    ];

    let discovery_info = DiscoveryInfo::degraded(tools, 100);

    // ACT
    let json = serde_json::to_string(&discovery_info).expect("Serialization failed");
    let deserialized: DiscoveryInfo = serde_json::from_str(&json).expect("Deserialization failed");

    // ASSERT
    assert_eq!(discovery_info.service_health, "degraded");
    assert_eq!(deserialized.service_health, "degraded");
    assert_eq!(discovery_info.uptime_seconds, 100);
}
