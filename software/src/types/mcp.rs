// ABOUTME: Shared MCP type definitions for client-server communication
// ABOUTME: Contains envelope data structures, server info, and discovery types

//! Shared MCP types for client-server communication
//!
//! This module contains type definitions that are shared between MCP client and server
//! components to ensure consistent data structures for communication.

// Import comprehensive rmcp types for standardized MCP protocol handling
use rmcp::model::{
    CallToolRequest, CallToolResult, Tool
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

// ============================================================================
// ENVELOPE DATA STRUCTURES (Used in Qollective envelopes)
// ============================================================================

/// MCP data structure that goes inside Qollective envelopes (ENVELOPE-FIRST ARCHITECTURE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpData {
    /// MCP tool call data
    pub tool_call: Option<CallToolRequest>,
    /// MCP tool response data
    pub tool_response: Option<CallToolResult>,
    /// MCP tool registration data
    pub tool_registration: Option<Tool>,
    /// MCP discovery query/response data
    pub discovery_data: Option<McpDiscoveryData>,
}

/// MCP discovery data for tool and server discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpDiscoveryData {
    /// Discovery query type
    pub query_type: String,
    /// Tools found in discovery
    pub tools: Option<Vec<Tool>>,
    /// Server information  
    pub server_info: Option<McpServerInfo>,
}

// ============================================================================
// SERVER INFORMATION STRUCTURES
// ============================================================================

/// MCP server information for registry tracking
/// Used by both client (for tracking servers) and server (for registration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    /// Unique server identifier
    pub server_id: String,
    /// Human-readable server name
    pub server_name: String,
    /// Tools provided by this server
    pub tools: Vec<Tool>,
    /// Server capabilities
    pub capabilities: Vec<String>,
    /// Server metadata
    pub metadata: ServerMetadata,
    /// Async configuration for server connectivity
    pub async_config: Option<AsyncConfig>,
    /// Current health status
    pub health_status: HealthStatus,
}

/// Metadata about an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetadata {
    /// Server description
    pub description: Option<String>,
    /// Server version
    pub version: String,
    /// Contact information
    pub contact: Option<String>,
    /// Documentation URL
    pub documentation_url: Option<String>,
    /// Server tags for categorization
    pub tags: Vec<String>,
}

/// Async configuration for MCP server connectivity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncConfig {
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// SSL configuration
    pub ssl_config: Option<SslConfig>,
}

/// SSL configuration for secure connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    /// Enable SSL/TLS
    pub enabled: bool,
    /// Certificate file path
    pub cert_file: Option<String>,
    /// Private key file path
    pub key_file: Option<String>,
    /// Certificate authority file path
    pub ca_file: Option<String>,
    /// Verify server certificate
    pub verify_cert: bool,
}

/// Health status tracking for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the server is currently healthy
    pub is_healthy: bool,
    /// Last health check timestamp
    pub last_check: SystemTime,
    /// Average response time
    pub response_time: Duration,
    /// Number of errors since last successful check
    pub error_count: u32,
    /// Server uptime
    pub uptime: Duration,
}

// ============================================================================
// UTILITY IMPLEMENTATIONS
// ============================================================================

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            is_healthy: true,
            last_check: SystemTime::now(),
            response_time: Duration::from_millis(0),
            error_count: 0,
            uptime: Duration::from_secs(0),
        }
    }
}

impl Default for AsyncConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            max_concurrent_requests: 10,
            ssl_config: None,
        }
    }
}

impl Default for SslConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_file: None,
            key_file: None,
            ca_file: None,
            verify_cert: true,
        }
    }
}

impl McpData {
    /// Create a new McpData with tool call
    pub fn with_tool_call(tool_call: CallToolRequest) -> Self {
        Self {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        }
    }

    /// Create a new McpData with tool response
    pub fn with_tool_response(tool_response: CallToolResult) -> Self {
        Self {
            tool_call: None,
            tool_response: Some(tool_response),
            tool_registration: None,
            discovery_data: None,
        }
    }

    /// Create a new McpData with discovery data
    pub fn with_discovery(discovery_data: McpDiscoveryData) -> Self {
        Self {
            tool_call: None,
            tool_response: None,
            tool_registration: None,
            discovery_data: Some(discovery_data),
        }
    }

    /// Create a new McpData with tool registration - uses rmcp Tool type
    pub fn with_tool_registration(tool: Tool) -> Self {
        Self {
            tool_call: None,
            tool_response: None,
            tool_registration: Some(tool),
            discovery_data: None,
        }
    }
}

impl McpDiscoveryData {
    /// Create a tools list query
    pub fn tools_list_query() -> Self {
        Self {
            query_type: "list_tools".to_string(),
            tools: None,
            server_info: None,
        }
    }

    /// Create a tools list response
    pub fn tools_list_response(tools: Vec<Tool>) -> Self {
        Self {
            query_type: "list_tools_response".to_string(),
            tools: Some(tools),
            server_info: None,
        }
    }

    /// Create a server info query
    pub fn server_info_query() -> Self {
        Self {
            query_type: "server_info".to_string(),
            tools: None,
            server_info: None,
        }
    }

    /// Create a server info response
    pub fn server_info_response(server_info: McpServerInfo) -> Self {
        Self {
            query_type: "server_info_response".to_string(),
            tools: None,
            server_info: Some(server_info),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    use rmcp_macros::*;

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[tokio::test]
    async fn test_rmcp_macros_compilation() {
        // Test that rmcp-macros compiles and can be used
        // This test ensures the dependency is working correctly
        
        // Create basic rmcp types to verify compilation
        use rmcp::model::{CallToolRequest, CallToolRequestParam};
        
        let tool_request = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "test_tool".to_string().into(),
                arguments: None,
            },
            extensions: Default::default(),
        };
        
        // Test envelope integration
        let mcp_data = McpData::with_tool_call(tool_request);
        assert!(mcp_data.tool_call.is_some());
        assert!(mcp_data.tool_response.is_none());
    }

    #[cfg(any(feature = "mcp-client", feature = "mcp-server"))]
    #[test]
    fn test_envelope_serialization_with_rmcp_types() {
        // Test that envelope serialization works with rmcp types
        use rmcp::model::{CallToolResult, CallToolRequestParam, CallToolRequest};
        
        let tool_request = CallToolRequest {
            method: Default::default(),
            params: CallToolRequestParam {
                name: "test_tool".to_string().into(),
                arguments: Some({
                    let mut map = serde_json::Map::new();
                    map.insert("key".to_string(), serde_json::Value::String("value".to_string()));
                    map
                }),
            },
            extensions: Default::default(),
        };
        
        let mcp_data = McpData::with_tool_call(tool_request);
        
        // Verify serialization works
        let serialized = serde_json::to_string(&mcp_data).expect("Should serialize");
        let _deserialized: McpData = serde_json::from_str(&serialized).expect("Should deserialize");
    }
}
