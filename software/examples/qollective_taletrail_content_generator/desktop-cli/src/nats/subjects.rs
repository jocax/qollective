/// NATS subject patterns for MCP and TaleTrail communication
///
/// This module defines the subject hierarchy used for NATS messaging:
/// - MCP subjects: mcp.<service>.request for tool invocations
/// - TaleTrail subjects: taletrail.generation.events for pipeline events

// MCP Subject Patterns
pub const MCP_WILDCARD: &str = "mcp.>";

pub const MCP_ORCHESTRATOR_REQUEST: &str = "mcp.orchestrator.request";
pub const MCP_STORY_GENERATOR_REQUEST: &str = "mcp.story-generator.request";
pub const MCP_QUALITY_CONTROL_REQUEST: &str = "mcp.quality-control.request";
pub const MCP_CONSTRAINT_ENFORCER_REQUEST: &str = "mcp.constraint-enforcer.request";
pub const MCP_PROMPT_HELPER_REQUEST: &str = "mcp.prompt-helper.request";

// TaleTrail Subject Patterns
pub const TALETRAIL_WILDCARD: &str = "taletrail.>";

pub const TALETRAIL_GENERATION_EVENTS: &str = "taletrail.generation.events";
pub const TALETRAIL_GENERATION_EVENTS_TENANT: &str = "taletrail.generation.events.{tenant_id}";

/// Get the MCP request subject for a given server name
pub fn mcp_request_subject(server: &str) -> String {
    format!("mcp.{}.request", server)
}

/// Check if a subject is an MCP subject
pub fn is_mcp_subject(subject: &str) -> bool {
    subject.starts_with("mcp.")
}

/// Check if a subject is a TaleTrail subject
pub fn is_taletrail_subject(subject: &str) -> bool {
    subject.starts_with("taletrail.")
}

/// Extract server name from MCP subject
/// Example: "mcp.orchestrator.request" -> Some("orchestrator")
pub fn extract_mcp_server(subject: &str) -> Option<String> {
    if !is_mcp_subject(subject) {
        return None;
    }

    let parts: Vec<&str> = subject.split('.').collect();
    if parts.len() >= 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Extract tenant ID from TaleTrail event subject
/// Example: "taletrail.generation.events.42" -> Some("42")
pub fn extract_tenant_id(subject: &str) -> Option<String> {
    if !subject.starts_with(TALETRAIL_GENERATION_EVENTS) {
        return None;
    }

    let parts: Vec<&str> = subject.split('.').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_request_subject() {
        assert_eq!(mcp_request_subject("orchestrator"), "mcp.orchestrator.request");
        assert_eq!(mcp_request_subject("story-generator"), "mcp.story-generator.request");
    }

    #[test]
    fn test_is_mcp_subject() {
        assert!(is_mcp_subject("mcp.orchestrator.request"));
        assert!(is_mcp_subject("mcp.>"));
        assert!(!is_mcp_subject("taletrail.generation.events"));
    }

    #[test]
    fn test_is_taletrail_subject() {
        assert!(is_taletrail_subject("taletrail.generation.events"));
        assert!(is_taletrail_subject("taletrail.generation.events.42"));
        assert!(!is_taletrail_subject("mcp.orchestrator.request"));
    }

    #[test]
    fn test_extract_mcp_server() {
        assert_eq!(extract_mcp_server("mcp.orchestrator.request"), Some("orchestrator".to_string()));
        assert_eq!(extract_mcp_server("mcp.story-generator.request"), Some("story-generator".to_string()));
        assert_eq!(extract_mcp_server("taletrail.generation.events"), None);
    }

    #[test]
    fn test_extract_tenant_id() {
        assert_eq!(extract_tenant_id("taletrail.generation.events.42"), Some("42".to_string()));
        assert_eq!(extract_tenant_id("taletrail.generation.events"), None);
        assert_eq!(extract_tenant_id("mcp.orchestrator.request"), None);
    }
}
