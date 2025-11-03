/// MCP Testing Interface State Management
///
/// Manages state for the MCP Testing UI including server selection,
/// template browsing, request editing, response viewing, and history tracking

use crate::models::history::{HistoryPage, HistoryQuery, McpHistoryEntry};
use crate::models::mcp::{McpServerConfig, TemplateInfo};
use std::sync::{Arc, RwLock};

/// Represents the different tabs in the MCP Testing Interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpTab {
    Templates,
    Editor,
    Response,
    History,
}

impl McpTab {
    /// Get the display name for the tab
    pub fn display_name(&self) -> &'static str {
        match self {
            McpTab::Templates => "Templates",
            McpTab::Editor => "Request Editor",
            McpTab::Response => "Response",
            McpTab::History => "History",
        }
    }

    /// Get the tab number (1-4)
    pub fn tab_number(&self) -> usize {
        match self {
            McpTab::Templates => 1,
            McpTab::Editor => 2,
            McpTab::Response => 3,
            McpTab::History => 4,
        }
    }

    /// Get tab from number (1-4)
    pub fn from_number(number: usize) -> Option<Self> {
        match number {
            1 => Some(McpTab::Templates),
            2 => Some(McpTab::Editor),
            3 => Some(McpTab::Response),
            4 => Some(McpTab::History),
            _ => None,
        }
    }

    /// Get next tab in order
    pub fn next(&self) -> Self {
        match self {
            McpTab::Templates => McpTab::Editor,
            McpTab::Editor => McpTab::Response,
            McpTab::Response => McpTab::History,
            McpTab::History => McpTab::Templates,
        }
    }

    /// Get previous tab in order
    pub fn previous(&self) -> Self {
        match self {
            McpTab::Templates => McpTab::History,
            McpTab::Editor => McpTab::Templates,
            McpTab::Response => McpTab::Editor,
            McpTab::History => McpTab::Response,
        }
    }
}

/// MCP Testing Interface context shared across all MCP components
#[derive(Clone)]
pub struct McpContext {
    inner: Arc<RwLock<McpState>>,
}

impl McpContext {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(McpState::default())),
        }
    }

    // --- Server Management ---

    /// Get list of available MCP servers
    pub fn servers(&self) -> Vec<McpServerConfig> {
        self.inner.read().unwrap().servers.clone()
    }

    /// Set list of available MCP servers
    pub fn set_servers(&self, servers: Vec<McpServerConfig>) {
        self.inner.write().unwrap().servers = servers;
    }

    /// Get selected server index
    pub fn selected_server_index(&self) -> usize {
        self.inner.read().unwrap().selected_server_index
    }

    /// Set selected server index
    pub fn set_selected_server_index(&self, index: usize) {
        let mut state = self.inner.write().unwrap();
        if index < state.servers.len() {
            state.selected_server_index = index;
        }
    }

    /// Get selected server
    pub fn selected_server(&self) -> Option<McpServerConfig> {
        let state = self.inner.read().unwrap();
        state.servers.get(state.selected_server_index).cloned()
    }

    /// Select next server
    pub fn next_server(&self) {
        let mut state = self.inner.write().unwrap();
        if !state.servers.is_empty() {
            state.selected_server_index = (state.selected_server_index + 1) % state.servers.len();
        }
    }

    /// Select previous server
    pub fn previous_server(&self) {
        let mut state = self.inner.write().unwrap();
        if !state.servers.is_empty() {
            if state.selected_server_index == 0 {
                state.selected_server_index = state.servers.len() - 1;
            } else {
                state.selected_server_index -= 1;
            }
        }
    }

    // --- Template Management ---

    /// Get all loaded templates
    pub fn templates(&self) -> Vec<TemplateInfo> {
        self.inner.read().unwrap().templates.clone()
    }

    /// Set loaded templates
    pub fn set_templates(&self, templates: Vec<TemplateInfo>) {
        self.inner.write().unwrap().templates = templates;
    }

    /// Get selected template index
    pub fn selected_template_index(&self) -> usize {
        self.inner.read().unwrap().selected_template_index
    }

    /// Set selected template index
    pub fn set_selected_template_index(&self, index: usize) {
        let mut state = self.inner.write().unwrap();
        let filtered = Self::filter_templates_internal(&state);
        if index < filtered.len() {
            state.selected_template_index = index;
        }
    }

    /// Get selected template
    pub fn selected_template(&self) -> Option<TemplateInfo> {
        let state = self.inner.read().unwrap();
        let filtered = Self::filter_templates_internal(&state);
        filtered.get(state.selected_template_index).cloned()
    }

    /// Select next template
    pub fn next_template(&self) {
        let mut state = self.inner.write().unwrap();
        let filtered = Self::filter_templates_internal(&state);
        if !filtered.is_empty() {
            state.selected_template_index = (state.selected_template_index + 1) % filtered.len();
        }
    }

    /// Select previous template
    pub fn previous_template(&self) {
        let mut state = self.inner.write().unwrap();
        let filtered = Self::filter_templates_internal(&state);
        if !filtered.is_empty() {
            if state.selected_template_index == 0 {
                state.selected_template_index = filtered.len() - 1;
            } else {
                state.selected_template_index -= 1;
            }
        }
    }

    /// Get template search filter
    pub fn template_filter(&self) -> String {
        self.inner.read().unwrap().template_filter.clone()
    }

    /// Set template search filter
    pub fn set_template_filter(&self, filter: String) {
        let mut state = self.inner.write().unwrap();
        state.template_filter = filter;
        // Reset selection when filter changes
        state.selected_template_index = 0;
    }

    /// Get filtered templates
    pub fn filtered_templates(&self) -> Vec<TemplateInfo> {
        let state = self.inner.read().unwrap();
        Self::filter_templates_internal(&state)
    }

    /// Internal helper to filter templates based on current filter
    fn filter_templates_internal(state: &McpState) -> Vec<TemplateInfo> {
        if state.template_filter.is_empty() {
            state.templates.clone()
        } else {
            let filter_lower = state.template_filter.to_lowercase();
            state
                .templates
                .iter()
                .filter(|t| {
                    t.template_name.to_lowercase().contains(&filter_lower)
                        || t.server_name.to_lowercase().contains(&filter_lower)
                        || t.tool_name.to_lowercase().contains(&filter_lower)
                        || t.description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&filter_lower))
                            .unwrap_or(false)
                })
                .cloned()
                .collect()
        }
    }

    // --- Request Editor ---

    /// Get current request JSON content
    pub fn request_json(&self) -> String {
        self.inner.read().unwrap().current_request_json.clone()
    }

    /// Set current request JSON content
    pub fn set_request_json(&self, json: String) {
        self.inner.write().unwrap().current_request_json = json;
    }

    /// Get JSON validation error
    pub fn json_error(&self) -> Option<String> {
        self.inner.read().unwrap().json_error.clone()
    }

    /// Set JSON validation error
    pub fn set_json_error(&self, error: Option<String>) {
        self.inner.write().unwrap().json_error = error;
    }

    // --- Response Viewer ---

    /// Get current response
    pub fn response(&self) -> Option<String> {
        self.inner.read().unwrap().current_response.clone()
    }

    /// Set current response
    pub fn set_response(&self, response: Option<String>) {
        self.inner.write().unwrap().current_response = response;
    }

    /// Get response metadata
    pub fn response_metadata(&self) -> Option<ResponseMetadata> {
        self.inner.read().unwrap().response_metadata.clone()
    }

    /// Set response metadata
    pub fn set_response_metadata(&self, metadata: Option<ResponseMetadata>) {
        self.inner.write().unwrap().response_metadata = metadata;
    }

    // --- History Management ---

    /// Get request history
    pub fn history(&self) -> Vec<McpHistoryEntry> {
        self.inner.read().unwrap().history.clone()
    }

    /// Add entry to history
    pub fn add_history_entry(&self, entry: McpHistoryEntry) {
        self.inner.write().unwrap().history.insert(0, entry);
    }

    /// Clear all history
    pub fn clear_history(&self) {
        self.inner.write().unwrap().history.clear();
    }

    /// Delete history entry by ID
    pub fn delete_history_entry(&self, id: &str) {
        self.inner
            .write()
            .unwrap()
            .history
            .retain(|e| e.id != id);
    }

    /// Get history query
    pub fn history_query(&self) -> HistoryQuery {
        self.inner.read().unwrap().history_query.clone()
    }

    /// Set history query
    pub fn set_history_query(&self, query: HistoryQuery) {
        self.inner.write().unwrap().history_query = query;
    }

    /// Get paginated history based on current query
    pub fn paginated_history(&self) -> HistoryPage {
        let state = self.inner.read().unwrap();
        let query = &state.history_query;

        // Filter history entries
        let filtered: Vec<McpHistoryEntry> = state
            .history
            .iter()
            .filter(|e| query.matches(e))
            .cloned()
            .collect();

        let total_count = filtered.len();

        // Calculate pagination
        let start = query.page * query.page_size;
        let end = (start + query.page_size).min(total_count);

        let entries = if start < total_count {
            filtered[start..end].to_vec()
        } else {
            Vec::new()
        };

        HistoryPage::new(entries, total_count, query.page, query.page_size)
    }

    // --- Tab Management ---

    /// Get active tab
    pub fn active_tab(&self) -> McpTab {
        self.inner.read().unwrap().active_tab
    }

    /// Set active tab
    pub fn set_active_tab(&self, tab: McpTab) {
        self.inner.write().unwrap().active_tab = tab;
    }

    /// Switch to next tab
    pub fn next_tab(&self) {
        let mut state = self.inner.write().unwrap();
        state.active_tab = state.active_tab.next();
    }

    /// Switch to previous tab
    pub fn previous_tab(&self) {
        let mut state = self.inner.write().unwrap();
        state.active_tab = state.active_tab.previous();
    }

    // --- Request Status ---

    /// Check if request is in progress
    pub fn request_in_progress(&self) -> bool {
        self.inner.read().unwrap().request_in_progress
    }

    /// Set request in progress status
    pub fn set_request_in_progress(&self, in_progress: bool) {
        self.inner.write().unwrap().request_in_progress = in_progress;
    }
}

impl Default for McpContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Response metadata for display
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseMetadata {
    pub status: String,
    pub duration_ms: u64,
    pub timestamp: String,
}

/// Internal MCP state
#[derive(Debug, Clone)]
struct McpState {
    // Server selection
    servers: Vec<McpServerConfig>,
    selected_server_index: usize,

    // Template browsing
    templates: Vec<TemplateInfo>,
    selected_template_index: usize,
    template_filter: String,

    // Request editor
    current_request_json: String,
    json_error: Option<String>,

    // Response viewer
    current_response: Option<String>,
    response_metadata: Option<ResponseMetadata>,

    // History
    history: Vec<McpHistoryEntry>,
    history_query: HistoryQuery,

    // Tab navigation
    active_tab: McpTab,

    // Request status
    request_in_progress: bool,
}

impl Default for McpState {
    fn default() -> Self {
        Self {
            servers: McpServerConfig::all_servers(),
            selected_server_index: 0,
            templates: Vec::new(),
            selected_template_index: 0,
            template_filter: String::new(),
            current_request_json: String::new(),
            json_error: None,
            current_response: None,
            response_metadata: None,
            history: Vec::new(),
            history_query: HistoryQuery::default(),
            active_tab: McpTab::Templates,
            request_in_progress: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tab_display_names() {
        assert_eq!(McpTab::Templates.display_name(), "Templates");
        assert_eq!(McpTab::Editor.display_name(), "Request Editor");
        assert_eq!(McpTab::Response.display_name(), "Response");
        assert_eq!(McpTab::History.display_name(), "History");
    }

    #[test]
    fn test_mcp_tab_numbers() {
        assert_eq!(McpTab::Templates.tab_number(), 1);
        assert_eq!(McpTab::Editor.tab_number(), 2);
        assert_eq!(McpTab::Response.tab_number(), 3);
        assert_eq!(McpTab::History.tab_number(), 4);
    }

    #[test]
    fn test_mcp_tab_from_number() {
        assert_eq!(McpTab::from_number(1), Some(McpTab::Templates));
        assert_eq!(McpTab::from_number(2), Some(McpTab::Editor));
        assert_eq!(McpTab::from_number(3), Some(McpTab::Response));
        assert_eq!(McpTab::from_number(4), Some(McpTab::History));
        assert_eq!(McpTab::from_number(0), None);
        assert_eq!(McpTab::from_number(5), None);
    }

    #[test]
    fn test_mcp_tab_navigation() {
        assert_eq!(McpTab::Templates.next(), McpTab::Editor);
        assert_eq!(McpTab::Editor.next(), McpTab::Response);
        assert_eq!(McpTab::Response.next(), McpTab::History);
        assert_eq!(McpTab::History.next(), McpTab::Templates);

        assert_eq!(McpTab::Templates.previous(), McpTab::History);
        assert_eq!(McpTab::Editor.previous(), McpTab::Templates);
        assert_eq!(McpTab::Response.previous(), McpTab::Editor);
        assert_eq!(McpTab::History.previous(), McpTab::Response);
    }

    #[test]
    fn test_mcp_context_server_management() {
        let ctx = McpContext::new();

        // Check default servers loaded
        let servers = ctx.servers();
        assert_eq!(servers.len(), 5);

        // Check server selection
        assert_eq!(ctx.selected_server_index(), 0);

        ctx.next_server();
        assert_eq!(ctx.selected_server_index(), 1);

        ctx.previous_server();
        assert_eq!(ctx.selected_server_index(), 0);
    }

    #[test]
    fn test_mcp_context_template_management() {
        let ctx = McpContext::new();

        // Set test templates
        let templates = vec![
            TemplateInfo {
                server_name: "orchestrator".to_string(),
                template_name: "test1".to_string(),
                file_path: "/path/to/test1.json".to_string(),
                description: Some("Test template 1".to_string()),
                tool_name: "test_tool".to_string(),
            },
            TemplateInfo {
                server_name: "story-generator".to_string(),
                template_name: "test2".to_string(),
                file_path: "/path/to/test2.json".to_string(),
                description: Some("Test template 2".to_string()),
                tool_name: "another_tool".to_string(),
            },
        ];

        ctx.set_templates(templates.clone());
        assert_eq!(ctx.templates().len(), 2);

        // Test template selection
        assert_eq!(ctx.selected_template_index(), 0);

        ctx.next_template();
        assert_eq!(ctx.selected_template_index(), 1);

        ctx.previous_template();
        assert_eq!(ctx.selected_template_index(), 0);

        // Test filtering
        ctx.set_template_filter("story".to_string());
        let filtered = ctx.filtered_templates();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].server_name, "story-generator");
    }

    #[test]
    fn test_mcp_context_request_editor() {
        let ctx = McpContext::new();

        // Test request JSON management
        let json = r#"{"test": "value"}"#.to_string();
        ctx.set_request_json(json.clone());
        assert_eq!(ctx.request_json(), json);

        // Test error management
        assert_eq!(ctx.json_error(), None);

        ctx.set_json_error(Some("Invalid JSON".to_string()));
        assert_eq!(ctx.json_error(), Some("Invalid JSON".to_string()));
    }

    #[test]
    fn test_mcp_context_response_viewer() {
        let ctx = McpContext::new();

        // Test response management
        assert_eq!(ctx.response(), None);

        let response = "Test response".to_string();
        ctx.set_response(Some(response.clone()));
        assert_eq!(ctx.response(), Some(response));

        // Test metadata
        let metadata = ResponseMetadata {
            status: "success".to_string(),
            duration_ms: 123,
            timestamp: "2025-11-02T12:00:00Z".to_string(),
        };

        ctx.set_response_metadata(Some(metadata.clone()));
        assert!(ctx.response_metadata().is_some());
    }

    #[test]
    fn test_mcp_context_tab_navigation() {
        let ctx = McpContext::new();

        // Check default tab
        assert_eq!(ctx.active_tab(), McpTab::Templates);

        // Test tab navigation
        ctx.next_tab();
        assert_eq!(ctx.active_tab(), McpTab::Editor);

        ctx.next_tab();
        assert_eq!(ctx.active_tab(), McpTab::Response);

        ctx.previous_tab();
        assert_eq!(ctx.active_tab(), McpTab::Editor);

        // Test direct tab setting
        ctx.set_active_tab(McpTab::History);
        assert_eq!(ctx.active_tab(), McpTab::History);
    }

    #[test]
    fn test_mcp_context_request_status() {
        let ctx = McpContext::new();

        assert!(!ctx.request_in_progress());

        ctx.set_request_in_progress(true);
        assert!(ctx.request_in_progress());

        ctx.set_request_in_progress(false);
        assert!(!ctx.request_in_progress());
    }
}
