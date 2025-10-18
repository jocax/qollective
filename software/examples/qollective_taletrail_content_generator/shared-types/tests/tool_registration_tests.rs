//! Tests for MCP Tool Registration and Discovery types
//!
//! Validates serialization, deserialization, and schema generation
//! for tool registration protocol types.

use serde_json::json;
use shared_types::types::tool_registration::{
    DiscoveryInfo, ServiceCapabilities, ToolRegistration,
};

#[test]
fn test_tool_registration_json_roundtrip() {
    // Create a tool registration with complete schema
    let schema = json!({
        "type": "object",
        "properties": {
            "theme": {
                "type": "string",
                "description": "Story theme"
            },
            "language": {
                "type": "string",
                "enum": ["en", "de"]
            }
        },
        "required": ["theme", "language"]
    });

    let registration = ToolRegistration::new(
        "generate_structure",
        schema.clone(),
        "story-generator",
        "0.0.1",
        vec![ServiceCapabilities::Batching, ServiceCapabilities::Retry],
    );

    // Serialize to JSON
    let json_str = serde_json::to_string(&registration).expect("Should serialize");

    // Deserialize back
    let deserialized: ToolRegistration =
        serde_json::from_str(&json_str).expect("Should deserialize");

    // Verify roundtrip
    assert_eq!(registration, deserialized);
    assert_eq!(deserialized.tool_name, "generate_structure");
    assert_eq!(deserialized.service_name, "story-generator");
    assert_eq!(deserialized.service_version, "0.0.1");
    assert_eq!(deserialized.capabilities.len(), 2);
}

#[test]
fn test_discovery_info_multiple_tools() {
    // Create discovery info with multiple tools
    let schema1 = json!({"type": "object", "properties": {"param1": {"type": "string"}}});
    let schema2 = json!({"type": "object", "properties": {"param2": {"type": "number"}}});

    let tool1 = ToolRegistration::new(
        "tool1",
        schema1,
        "service1",
        "1.0.0",
        vec![ServiceCapabilities::Caching],
    );

    let tool2 = ToolRegistration::new(
        "tool2",
        schema2,
        "service1",
        "1.0.0",
        vec![ServiceCapabilities::Streaming, ServiceCapabilities::Retry],
    );

    let discovery_info = DiscoveryInfo::healthy(vec![tool1, tool2], 7200);

    // Serialize
    let json_str = serde_json::to_string(&discovery_info).expect("Should serialize");

    // Deserialize
    let deserialized: DiscoveryInfo =
        serde_json::from_str(&json_str).expect("Should deserialize");

    // Verify
    assert_eq!(deserialized.available_tools.len(), 2);
    assert_eq!(deserialized.service_health, "healthy");
    assert_eq!(deserialized.uptime_seconds, 7200);
    assert_eq!(deserialized.available_tools[0].tool_name, "tool1");
    assert_eq!(deserialized.available_tools[1].tool_name, "tool2");
}

#[test]
fn test_service_capabilities_all_variants_serialize() {
    // Test all capability variants
    let all_capabilities = vec![
        ServiceCapabilities::Batching,
        ServiceCapabilities::Streaming,
        ServiceCapabilities::Caching,
        ServiceCapabilities::Retry,
    ];

    // Serialize
    let json_str = serde_json::to_string(&all_capabilities).expect("Should serialize");

    // Check JSON format (snake_case)
    assert!(json_str.contains("\"batching\""));
    assert!(json_str.contains("\"streaming\""));
    assert!(json_str.contains("\"caching\""));
    assert!(json_str.contains("\"retry\""));

    // Deserialize back
    let deserialized: Vec<ServiceCapabilities> =
        serde_json::from_str(&json_str).expect("Should deserialize");

    assert_eq!(all_capabilities, deserialized);
}

#[test]
fn test_tool_schema_contains_valid_json_schema() {
    // Create a comprehensive JSON Schema
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "title": "GenerateStructureParams",
        "properties": {
            "theme": {
                "type": "string",
                "description": "Story theme or topic",
                "minLength": 3,
                "maxLength": 100
            },
            "language": {
                "type": "string",
                "enum": ["en", "de"],
                "description": "Content language"
            },
            "node_count": {
                "type": "integer",
                "minimum": 8,
                "maximum": 32,
                "description": "Number of story nodes"
            }
        },
        "required": ["theme", "language"],
        "additionalProperties": false
    });

    let registration = ToolRegistration::new(
        "generate_structure",
        schema.clone(),
        "story-generator",
        "0.0.1",
        vec![ServiceCapabilities::Batching],
    );

    // Validate schema is a valid JSON object
    assert!(registration.tool_schema.is_object());

    let schema_obj = registration.tool_schema.as_object().unwrap();

    // Check required fields
    assert!(schema_obj.contains_key("type"));
    assert!(schema_obj.contains_key("properties"));

    // Check properties structure
    let properties = schema_obj.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("theme"));
    assert!(properties.contains_key("language"));
    assert!(properties.contains_key("node_count"));

    // Check required array
    let required = schema_obj.get("required").unwrap().as_array().unwrap();
    assert_eq!(required.len(), 2);
    assert!(required.contains(&json!("theme")));
    assert!(required.contains(&json!("language")));
}

#[test]
fn test_discovery_info_empty_tools() {
    // Test discovery with no tools (service starting up)
    let discovery_info = DiscoveryInfo::degraded(vec![], 10);

    assert_eq!(discovery_info.available_tools.len(), 0);
    assert_eq!(discovery_info.service_health, "degraded");
    assert_eq!(discovery_info.uptime_seconds, 10);

    // Ensure it serializes correctly
    let json_str = serde_json::to_string(&discovery_info).expect("Should serialize");
    let _deserialized: DiscoveryInfo =
        serde_json::from_str(&json_str).expect("Should deserialize");
}

#[test]
fn test_tool_registration_with_complex_nested_schema() {
    // Test with nested schema structures
    let schema = json!({
        "type": "object",
        "properties": {
            "content": {
                "type": "object",
                "properties": {
                    "text": {"type": "string"},
                    "choices": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "string"},
                                "text": {"type": "string"},
                                "next_node_id": {"type": "string"}
                            },
                            "required": ["id", "text", "next_node_id"]
                        }
                    }
                },
                "required": ["text", "choices"]
            }
        },
        "required": ["content"]
    });

    let registration = ToolRegistration::new(
        "generate_nodes",
        schema,
        "story-generator",
        "0.0.1",
        vec![ServiceCapabilities::Batching, ServiceCapabilities::Retry],
    );

    // Serialize and deserialize
    let json_str = serde_json::to_string(&registration).expect("Should serialize");
    let deserialized: ToolRegistration =
        serde_json::from_str(&json_str).expect("Should deserialize");

    // Verify nested schema preserved
    let props = deserialized
        .tool_schema
        .get("properties")
        .unwrap()
        .as_object()
        .unwrap();
    assert!(props.contains_key("content"));

    let content_props = props
        .get("content")
        .unwrap()
        .get("properties")
        .unwrap()
        .as_object()
        .unwrap();
    assert!(content_props.contains_key("text"));
    assert!(content_props.contains_key("choices"));
}

#[test]
fn test_service_capabilities_uniqueness() {
    // Test that capabilities can be used in HashSet (requires Eq + Hash)
    use std::collections::HashSet;

    let mut caps_set: HashSet<ServiceCapabilities> = HashSet::new();
    caps_set.insert(ServiceCapabilities::Batching);
    caps_set.insert(ServiceCapabilities::Batching); // Duplicate
    caps_set.insert(ServiceCapabilities::Caching);

    assert_eq!(caps_set.len(), 2); // Only 2 unique values
    assert!(caps_set.contains(&ServiceCapabilities::Batching));
    assert!(caps_set.contains(&ServiceCapabilities::Caching));
}

#[test]
fn test_discovery_info_builder_methods() {
    let tools = vec![ToolRegistration::new(
        "test_tool",
        json!({"type": "object"}),
        "test-service",
        "1.0.0",
        vec![],
    )];

    // Test healthy builder
    let healthy = DiscoveryInfo::healthy(tools.clone(), 100);
    assert_eq!(healthy.service_health, "healthy");

    // Test degraded builder
    let degraded = DiscoveryInfo::degraded(tools.clone(), 200);
    assert_eq!(degraded.service_health, "degraded");

    // Test custom builder
    let custom = DiscoveryInfo::new(tools, "starting", 50);
    assert_eq!(custom.service_health, "starting");
    assert_eq!(custom.uptime_seconds, 50);
}

#[cfg(feature = "test-utils")]
#[test]
fn test_json_schema_generation() {
    use schemars::schema_for;

    // Test that JsonSchema derive works correctly
    let schema = schema_for!(ToolRegistration);
    let schema_json = serde_json::to_value(&schema).expect("Should serialize schema");

    // Verify schema has expected structure
    assert!(schema_json.is_object());
    let obj = schema_json.as_object().unwrap();
    assert!(obj.contains_key("$schema"));

    // Test DiscoveryInfo schema
    let discovery_schema = schema_for!(DiscoveryInfo);
    let discovery_json =
        serde_json::to_value(&discovery_schema).expect("Should serialize schema");
    assert!(discovery_json.is_object());

    // Test ServiceCapabilities schema
    let caps_schema = schema_for!(ServiceCapabilities);
    let caps_json = serde_json::to_value(&caps_schema).expect("Should serialize schema");
    assert!(caps_json.is_object());
}
