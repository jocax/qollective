/// MCP Request History Data Model
///
/// Minimal wrapper for MCP request/response history.
/// Embeds rmcp types directly per architectural decision.
use chrono::{DateTime, Utc};
use rmcp::model::{CallToolRequest, CallToolResult};
use serde::{Deserialize, Serialize};

/// Status of a history entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HistoryStatus {
    Success,
    Error,
    Timeout,
}

/// Minimal wrapper for MCP request/response history
/// Embeds rmcp types directly (NO duplication)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHistoryEntry {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Timestamp when the request was made
    pub timestamp: DateTime<Utc>,
    /// Server name (e.g., "orchestrator", "story-generator")
    pub server_name: String,
    /// Tool name from CallToolRequest
    pub tool_name: String,
    /// Tenant ID for multi-tenancy support
    pub tenant_id: i32,
    /// Status of the request execution
    pub status: HistoryStatus,
    /// Duration of the request in milliseconds
    pub duration_ms: u64,

    // rmcp types embedded directly (NO duplication)
    /// Original MCP request
    pub request: CallToolRequest,
    /// MCP response from server
    pub response: CallToolResult,
}

/// Query parameters for loading history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryQuery {
    /// Page number (0-indexed)
    pub page: usize,
    /// Number of entries per page
    pub page_size: usize,
    /// Optional filter by server name
    pub server_filter: Option<String>,
    /// Optional filter by status
    pub status_filter: Option<HistoryStatus>,
    /// Optional search term (searches in tool_name)
    pub search_term: Option<String>,
}

impl Default for HistoryQuery {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: crate::constants::history::DEFAULT_PAGE_SIZE,
            server_filter: None,
            status_filter: None,
            search_term: None,
        }
    }
}

/// Paginated history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPage {
    /// List of history entries for this page
    pub entries: Vec<McpHistoryEntry>,
    /// Total number of entries (after filtering)
    pub total_count: usize,
    /// Current page number (0-indexed)
    pub page: usize,
    /// Number of entries per page
    pub page_size: usize,
    /// Total number of pages
    pub total_pages: usize,
}
