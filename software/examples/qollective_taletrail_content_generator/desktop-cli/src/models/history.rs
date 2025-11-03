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

impl std::fmt::Display for HistoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryStatus::Success => write!(f, "success"),
            HistoryStatus::Error => write!(f, "error"),
            HistoryStatus::Timeout => write!(f, "timeout"),
        }
    }
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

impl McpHistoryEntry {
    /// Create a new history entry
    pub fn new(
        server_name: String,
        tool_name: String,
        tenant_id: i32,
        status: HistoryStatus,
        duration_ms: u64,
        request: CallToolRequest,
        response: CallToolResult,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            server_name,
            tool_name,
            tenant_id,
            status,
            duration_ms,
            request,
            response,
        }
    }

    /// Get formatted timestamp
    pub fn formatted_timestamp(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get short timestamp (time only)
    pub fn short_timestamp(&self) -> String {
        self.timestamp.format("%H:%M:%S").to_string()
    }

    /// Get duration in seconds with decimals
    pub fn duration_seconds(&self) -> f64 {
        self.duration_ms as f64 / 1000.0
    }
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
            page_size: 20, // Default page size
            server_filter: None,
            status_filter: None,
            search_term: None,
        }
    }
}

impl HistoryQuery {
    /// Create a new query with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set page number
    pub fn with_page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }

    /// Set page size
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Set server filter
    pub fn with_server(mut self, server: String) -> Self {
        self.server_filter = Some(server);
        self
    }

    /// Set status filter
    pub fn with_status(mut self, status: HistoryStatus) -> Self {
        self.status_filter = Some(status);
        self
    }

    /// Set search term
    pub fn with_search(mut self, term: String) -> Self {
        self.search_term = Some(term);
        self
    }

    /// Check if entry matches this query's filters
    pub fn matches(&self, entry: &McpHistoryEntry) -> bool {
        // Check server filter
        if let Some(ref server) = self.server_filter {
            if &entry.server_name != server {
                return false;
            }
        }

        // Check status filter
        if let Some(ref status) = self.status_filter {
            if &entry.status != status {
                return false;
            }
        }

        // Check search term
        if let Some(ref term) = self.search_term {
            let term_lower = term.to_lowercase();
            if !entry.tool_name.to_lowercase().contains(&term_lower)
                && !entry.server_name.to_lowercase().contains(&term_lower)
            {
                return false;
            }
        }

        true
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

impl HistoryPage {
    /// Create a new history page
    pub fn new(
        entries: Vec<McpHistoryEntry>,
        total_count: usize,
        page: usize,
        page_size: usize,
    ) -> Self {
        let total_pages = if page_size > 0 {
            (total_count + page_size - 1) / page_size
        } else {
            0
        };

        Self {
            entries,
            total_count,
            page,
            page_size,
            total_pages,
        }
    }

    /// Check if there is a next page
    pub fn has_next(&self) -> bool {
        self.page + 1 < self.total_pages
    }

    /// Check if there is a previous page
    pub fn has_previous(&self) -> bool {
        self.page > 0
    }

    /// Get the range of entries shown (e.g., "1-20 of 100")
    pub fn range_display(&self) -> String {
        if self.total_count == 0 {
            "0 of 0".to_string()
        } else {
            let start = self.page * self.page_size + 1;
            let end = (start + self.entries.len() - 1).min(self.total_count);
            format!("{}-{} of {}", start, end, self.total_count)
        }
    }
}
