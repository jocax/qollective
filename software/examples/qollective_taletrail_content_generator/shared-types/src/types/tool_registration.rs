//! MCP Tool Registration and Discovery Types
//!
//! This module provides types for MCP tool discovery protocol.
//! Services register their tools and capabilities, and the orchestrator
//! discovers available tools before execution.
//!
//! # Discovery Flow
//!
//! 1. Orchestrator sends discovery request to `mcp.discovery.list_tools.{service_name}`
//! 2. Service responds with `DiscoveryInfo` containing available tools
//! 3. Orchestrator validates required tools are present
//! 4. Pipeline execution proceeds with validated tool inventory

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Tool registration information for MCP discovery
///
/// Contains complete metadata about a single MCP tool including
/// its JSON Schema definition, service location, and capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ToolRegistration {
    /// Tool name (matches rmcp Tool.name field)
    pub tool_name: String,

    /// JSON Schema describing tool parameters
    /// Generated from parameter types using schemars
    pub tool_schema: JsonValue,

    /// Service providing this tool (e.g., "story-generator")
    pub service_name: String,

    /// Service version (semantic versioning)
    pub service_version: String,

    /// Service capabilities for this tool
    pub capabilities: Vec<ServiceCapabilities>,
}

impl ToolRegistration {
    /// Create a new tool registration
    pub fn new(
        tool_name: impl Into<String>,
        tool_schema: JsonValue,
        service_name: impl Into<String>,
        service_version: impl Into<String>,
        capabilities: Vec<ServiceCapabilities>,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            tool_schema,
            service_name: service_name.into(),
            service_version: service_version.into(),
            capabilities,
        }
    }
}

/// Discovery information returned by MCP services
///
/// Complete service discovery response including all registered tools,
/// health status, and uptime information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveryInfo {
    /// All tools available from this service
    pub available_tools: Vec<ToolRegistration>,

    /// Current service health status
    pub service_health: String,

    /// Service uptime in seconds
    pub uptime_seconds: u64,
}

impl DiscoveryInfo {
    /// Create a new discovery info response
    pub fn new(
        available_tools: Vec<ToolRegistration>,
        service_health: impl Into<String>,
        uptime_seconds: u64,
    ) -> Self {
        Self {
            available_tools,
            service_health: service_health.into(),
            uptime_seconds,
        }
    }

    /// Create a healthy discovery response
    pub fn healthy(available_tools: Vec<ToolRegistration>, uptime_seconds: u64) -> Self {
        Self::new(available_tools, "healthy", uptime_seconds)
    }

    /// Create a degraded discovery response
    pub fn degraded(available_tools: Vec<ToolRegistration>, uptime_seconds: u64) -> Self {
        Self::new(available_tools, "degraded", uptime_seconds)
    }
}

/// Service capabilities for tool execution
///
/// Indicates what advanced features a service supports for a given tool.
/// Used by orchestrator to optimize execution strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ServiceCapabilities {
    /// Supports batch processing of multiple requests
    Batching,

    /// Supports streaming responses
    Streaming,

    /// Supports response caching
    Caching,

    /// Supports automatic retry on failure
    Retry,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_registration_creation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "param1": {"type": "string"}
            }
        });

        let registration = ToolRegistration::new(
            "test_tool",
            schema.clone(),
            "test-service",
            "0.0.1",
            vec![ServiceCapabilities::Batching, ServiceCapabilities::Retry],
        );

        assert_eq!(registration.tool_name, "test_tool");
        assert_eq!(registration.service_name, "test-service");
        assert_eq!(registration.service_version, "0.0.1");
        assert_eq!(registration.capabilities.len(), 2);
        assert!(registration.capabilities.contains(&ServiceCapabilities::Batching));
        assert!(registration.capabilities.contains(&ServiceCapabilities::Retry));
    }

    #[test]
    fn test_discovery_info_creation() {
        let schema = json!({"type": "object"});
        let tool = ToolRegistration::new(
            "tool1",
            schema,
            "service1",
            "1.0.0",
            vec![ServiceCapabilities::Caching],
        );

        let info = DiscoveryInfo::healthy(vec![tool], 3600);

        assert_eq!(info.service_health, "healthy");
        assert_eq!(info.uptime_seconds, 3600);
        assert_eq!(info.available_tools.len(), 1);
        assert_eq!(info.available_tools[0].tool_name, "tool1");
    }

    #[test]
    fn test_discovery_info_degraded() {
        let info = DiscoveryInfo::degraded(vec![], 100);
        assert_eq!(info.service_health, "degraded");
        assert_eq!(info.uptime_seconds, 100);
    }

    #[test]
    fn test_service_capabilities_all_variants() {
        let capabilities = vec![
            ServiceCapabilities::Batching,
            ServiceCapabilities::Streaming,
            ServiceCapabilities::Caching,
            ServiceCapabilities::Retry,
        ];

        // Test serialization
        let json = serde_json::to_string(&capabilities).unwrap();
        assert!(json.contains("batching"));
        assert!(json.contains("streaming"));
        assert!(json.contains("caching"));
        assert!(json.contains("retry"));

        // Test deserialization
        let deserialized: Vec<ServiceCapabilities> = serde_json::from_str(&json).unwrap();
        assert_eq!(capabilities, deserialized);
    }
}
