//! Unit tests for Prompt Helper discovery protocol

use prompt_helper::mcp_tools;
use prompt_helper::discovery::DiscoveryHandler;
use qollective::envelope::{Envelope, Meta};
use qollective::server::EnvelopeHandler;
use qollective::types::mcp::{McpData, McpDiscoveryData};
use uuid::Uuid;

#[test]
fn test_get_tool_registrations_returns_four_tools() {
    let registrations = mcp_tools::get_tool_registrations();
    assert_eq!(registrations.len(), 4, "Should have exactly 4 tools");
}

#[test]
fn test_tool_names_are_correct() {
    let registrations = mcp_tools::get_tool_registrations();
    let tool_names: Vec<&str> = registrations.iter().map(|t| t.tool_name.as_str()).collect();
    assert!(tool_names.contains(&"generate_story_prompts"));
    assert!(tool_names.contains(&"generate_validation_prompts"));
    assert!(tool_names.contains(&"generate_constraint_prompts"));
    assert!(tool_names.contains(&"get_model_for_language"));
}

#[test]
fn test_generate_story_prompts_capabilities() {
    let registrations = mcp_tools::get_tool_registrations();
    let tool = registrations.iter().find(|t| t.tool_name == "generate_story_prompts").unwrap();
    assert_eq!(tool.service_name, "prompt-helper");
    assert_eq!(tool.capabilities.len(), 2);
    use shared_types::types::tool_registration::ServiceCapabilities;
    assert!(tool.capabilities.contains(&ServiceCapabilities::Caching));
    assert!(tool.capabilities.contains(&ServiceCapabilities::Retry));
}

#[test]
fn test_get_model_for_language_capabilities() {
    let registrations = mcp_tools::get_tool_registrations();
    let tool = registrations.iter().find(|t| t.tool_name == "get_model_for_language").unwrap();
    assert_eq!(tool.capabilities.len(), 1);
    use shared_types::types::tool_registration::ServiceCapabilities;
    assert!(tool.capabilities.contains(&ServiceCapabilities::Caching));
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
    let (_, data) = response.extract();
    assert!(data.discovery_data.is_some());
    let discovery = data.discovery_data.unwrap();
    assert_eq!(discovery.query_type, "list_tools_response");
    let server_info = discovery.server_info.unwrap();
    assert_eq!(server_info.server_id, "prompt-helper");
    assert!(server_info.health_status.is_healthy);
}
