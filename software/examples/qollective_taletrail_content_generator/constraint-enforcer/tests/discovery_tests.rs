//! Unit tests for Constraint Enforcer discovery protocol

use constraint_enforcer::envelope_handlers::ConstraintEnforcerHandler;
use constraint_enforcer::discovery::DiscoveryHandler;
use qollective::envelope::{Envelope, Meta};
use qollective::server::EnvelopeHandler;
use qollective::types::mcp::{McpData, McpDiscoveryData};
use uuid::Uuid;

#[test]
fn test_get_tool_registrations_returns_two_tools() {
    let registrations = ConstraintEnforcerHandler::get_tool_registrations();
    assert_eq!(registrations.len(), 2, "Should have exactly 2 tools");
}

#[test]
fn test_tool_names_are_correct() {
    let registrations = ConstraintEnforcerHandler::get_tool_registrations();
    let tool_names: Vec<&str> = registrations.iter().map(|t| t.tool_name.as_str()).collect();
    assert!(tool_names.contains(&"enforce_constraints"), "Missing enforce_constraints tool");
    assert!(tool_names.contains(&"suggest_corrections"), "Missing suggest_corrections tool");
}

#[test]
fn test_enforce_constraints_capabilities() {
    let registrations = ConstraintEnforcerHandler::get_tool_registrations();
    let tool = registrations.iter().find(|t| t.tool_name == "enforce_constraints").expect("enforce_constraints tool not found");
    assert_eq!(tool.service_name, "constraint-enforcer");
    assert_eq!(tool.service_version, "0.0.1");
    assert_eq!(tool.capabilities.len(), 2);
    use shared_types::types::tool_registration::ServiceCapabilities;
    assert!(tool.capabilities.contains(&ServiceCapabilities::Batching));
    assert!(tool.capabilities.contains(&ServiceCapabilities::Retry));
}

#[test]
fn test_suggest_corrections_capabilities() {
    let registrations = ConstraintEnforcerHandler::get_tool_registrations();
    let tool = registrations.iter().find(|t| t.tool_name == "suggest_corrections").expect("suggest_corrections tool not found");
    assert_eq!(tool.service_name, "constraint-enforcer");
    assert_eq!(tool.service_version, "0.0.1");
    assert_eq!(tool.capabilities.len(), 1);
    use shared_types::types::tool_registration::ServiceCapabilities;
    assert!(tool.capabilities.contains(&ServiceCapabilities::Retry));
}

#[test]
fn test_tool_schemas_are_valid_json() {
    let registrations = ConstraintEnforcerHandler::get_tool_registrations();
    for registration in registrations {
        assert!(registration.tool_schema.is_object(), "Tool {} schema should be a JSON object", registration.tool_name);
        let schema = registration.tool_schema.as_object().unwrap();
        assert!(schema.contains_key("properties") || schema.contains_key("$schema"), "Tool {} schema should have properties or $schema field", registration.tool_name);
    }
}

#[tokio::test]
async fn test_discovery_handler_response_structure() {
    let handler = DiscoveryHandler::new();
    let mcp_data = McpData {
        tool_call: None,
        tool_response: None,
        tool_registration: None,
        discovery_data: Some(McpDiscoveryData {
            query_type: "list_tools".to_string(),
            tools: None,
            server_info: None,
        }),
    };
    let mut meta = Meta::default();
    meta.tenant = Some("test-tenant".to_string());
    meta.request_id = Some(Uuid::new_v4());
    let envelope = Envelope::new(meta, mcp_data);
    let result = handler.handle(envelope).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    let (response_meta, response_data) = response.extract();
    assert_eq!(response_meta.tenant, Some("test-tenant".to_string()));
    assert!(response_data.discovery_data.is_some());
    let discovery = response_data.discovery_data.unwrap();
    assert_eq!(discovery.query_type, "list_tools_response");
    let server_info = discovery.server_info.unwrap();
    assert_eq!(server_info.server_id, "constraint-enforcer");
    assert!(server_info.health_status.is_healthy);
}
