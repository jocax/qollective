/// NATS Monitoring Interface State Management
///
/// Manages state for the NATS Monitoring UI including message buffer,
/// filtering, connection diagnostics, and message detail view

use crate::nats::monitoring::{MonitoringDiagnostics, NatsMessage};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};

/// NATS Monitoring context shared across all monitoring components
#[derive(Clone)]
pub struct MonitorContext {
    inner: Arc<RwLock<MonitorState>>,
}

impl MonitorContext {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MonitorState::default())),
        }
    }

    // --- Message Buffer Management ---

    /// Add a message to the buffer (circular buffer, max capacity)
    pub fn add_message(&self, msg: NatsMessage) {
        let mut state = self.inner.write().unwrap();

        // Circular buffer: remove oldest if at capacity
        if state.messages.len() >= state.max_messages {
            state.messages.pop_front();
        }

        state.messages.push_back(msg);

        // Reapply filters to update filtered messages
        Self::apply_filters_internal(&mut state);

        // Update auto-scroll to latest if enabled
        if state.auto_scroll && !state.filtered_messages.is_empty() {
            state.selected_index = state.filtered_messages.len() - 1;
        }
    }

    /// Get all messages (unfiltered)
    pub fn messages(&self) -> Vec<NatsMessage> {
        self.inner.read().unwrap().messages.iter().cloned().collect()
    }

    /// Get filtered messages based on current filters
    pub fn filtered_messages(&self) -> Vec<NatsMessage> {
        self.inner.read().unwrap().filtered_messages.clone()
    }

    /// Get count of total messages in buffer
    pub fn message_count(&self) -> usize {
        self.inner.read().unwrap().messages.len()
    }

    /// Get count of filtered messages
    pub fn filtered_count(&self) -> usize {
        self.inner.read().unwrap().filtered_messages.len()
    }

    /// Clear all messages from buffer
    pub fn clear_messages(&self) {
        let mut state = self.inner.write().unwrap();
        state.messages.clear();
        state.filtered_messages.clear();
        state.selected_index = 0;
        state.show_detail = false;
    }

    // --- Filter Management ---

    /// Get current filters
    pub fn filters(&self) -> MessageFilters {
        self.inner.read().unwrap().filters.clone()
    }

    /// Set filters and reapply
    pub fn set_filters(&self, filters: MessageFilters) {
        let mut state = self.inner.write().unwrap();
        state.filters = filters;
        Self::apply_filters_internal(&mut state);

        // Reset selection to first item if filters changed
        state.selected_index = 0;
    }

    /// Set endpoint filter
    pub fn set_endpoint_filter(&self, endpoint: Option<String>) {
        let mut state = self.inner.write().unwrap();
        state.filters.endpoint = endpoint;
        Self::apply_filters_internal(&mut state);
        state.selected_index = 0;
    }

    /// Set message type filter
    pub fn set_message_type_filter(&self, message_type: Option<String>) {
        let mut state = self.inner.write().unwrap();
        state.filters.message_type = message_type;
        Self::apply_filters_internal(&mut state);
        state.selected_index = 0;
    }

    /// Set search query filter
    pub fn set_search_query(&self, query: Option<String>) {
        let mut state = self.inner.write().unwrap();
        state.filters.search_query = query;
        Self::apply_filters_internal(&mut state);
        state.selected_index = 0;
    }

    /// Clear all filters
    pub fn clear_filters(&self) {
        let mut state = self.inner.write().unwrap();
        state.filters = MessageFilters::default();
        Self::apply_filters_internal(&mut state);
        state.selected_index = 0;
    }

    /// Internal helper to apply filters
    fn apply_filters_internal(state: &mut MonitorState) {
        state.filtered_messages = state
            .messages
            .iter()
            .filter(|msg| {
                // Apply endpoint filter
                if let Some(ref endpoint) = state.filters.endpoint {
                    if !msg.endpoint.eq_ignore_ascii_case(endpoint) {
                        return false;
                    }
                }

                // Apply message type filter
                if let Some(ref msg_type) = state.filters.message_type {
                    if !msg.message_type.eq_ignore_ascii_case(msg_type) {
                        return false;
                    }
                }

                // Apply text search (searches in subject and payload)
                if let Some(ref search) = state.filters.search_query {
                    let search_lower = search.to_lowercase();
                    if !msg.subject.to_lowercase().contains(&search_lower)
                        && !msg.payload.to_lowercase().contains(&search_lower)
                    {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();
    }

    // --- Selection Management ---

    /// Get selected message index
    pub fn selected_index(&self) -> usize {
        self.inner.read().unwrap().selected_index
    }

    /// Set selected message index
    pub fn set_selected_index(&self, index: usize) {
        let mut state = self.inner.write().unwrap();
        if index < state.filtered_messages.len() {
            state.selected_index = index;
        }
    }

    /// Get selected message
    pub fn selected_message(&self) -> Option<NatsMessage> {
        let state = self.inner.read().unwrap();
        state.filtered_messages.get(state.selected_index).cloned()
    }

    /// Select next message
    pub fn next_message(&self) {
        let mut state = self.inner.write().unwrap();
        if !state.filtered_messages.is_empty() {
            state.selected_index = (state.selected_index + 1).min(state.filtered_messages.len() - 1);

            // Disable auto-scroll when manually navigating
            state.auto_scroll = false;
        }
    }

    /// Select previous message
    pub fn previous_message(&self) {
        let mut state = self.inner.write().unwrap();
        if state.selected_index > 0 {
            state.selected_index -= 1;

            // Disable auto-scroll when manually navigating
            state.auto_scroll = false;
        }
    }

    // --- Detail View Management ---

    /// Check if detail view is shown
    pub fn show_detail(&self) -> bool {
        self.inner.read().unwrap().show_detail
    }

    /// Set detail view visibility
    pub fn set_show_detail(&self, show: bool) {
        self.inner.write().unwrap().show_detail = show;
    }

    /// Toggle detail view
    pub fn toggle_detail(&self) {
        let mut state = self.inner.write().unwrap();
        state.show_detail = !state.show_detail;
    }

    // --- Auto-scroll Management ---

    /// Check if auto-scroll is enabled
    pub fn auto_scroll(&self) -> bool {
        self.inner.read().unwrap().auto_scroll
    }

    /// Set auto-scroll state
    pub fn set_auto_scroll(&self, enabled: bool) {
        self.inner.write().unwrap().auto_scroll = enabled;
    }

    /// Toggle auto-scroll
    pub fn toggle_auto_scroll(&self) {
        let mut state = self.inner.write().unwrap();
        state.auto_scroll = !state.auto_scroll;
    }

    // --- Diagnostics Management ---

    /// Get connection diagnostics
    pub fn diagnostics(&self) -> MonitoringDiagnostics {
        self.inner.read().unwrap().diagnostics.clone()
    }

    /// Update diagnostics
    pub fn update_diagnostics(&self, diagnostics: MonitoringDiagnostics) {
        self.inner.write().unwrap().diagnostics = diagnostics;
    }

    // --- Connection Status ---

    /// Check if connected to NATS
    pub fn is_connected(&self) -> bool {
        self.inner.read().unwrap().diagnostics.is_connected
    }

    /// Set connection status
    pub fn set_connected(&self, connected: bool) {
        self.inner.write().unwrap().diagnostics.is_connected = connected;
    }
}

impl Default for MonitorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Message filters for the monitoring interface
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MessageFilters {
    /// Filter by endpoint name (e.g., "orchestrator", "story-generator")
    pub endpoint: Option<String>,
    /// Filter by message type (e.g., "Request", "Response", "Event")
    pub message_type: Option<String>,
    /// Wildcard text search across subject and payload
    pub search_query: Option<String>,
}

impl MessageFilters {
    /// Check if any filters are active
    pub fn is_active(&self) -> bool {
        self.endpoint.is_some() || self.message_type.is_some() || self.search_query.is_some()
    }

    /// Get a description of active filters
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref endpoint) = self.endpoint {
            parts.push(format!("Endpoint: {}", endpoint));
        }

        if let Some(ref msg_type) = self.message_type {
            parts.push(format!("Type: {}", msg_type));
        }

        if let Some(ref query) = self.search_query {
            parts.push(format!("Search: {}", query));
        }

        if parts.is_empty() {
            "No filters".to_string()
        } else {
            parts.join(" | ")
        }
    }
}

/// Internal monitoring state
#[derive(Debug, Clone)]
struct MonitorState {
    /// Circular buffer of messages (max 1000 by default)
    messages: VecDeque<NatsMessage>,
    /// Maximum number of messages to buffer
    max_messages: usize,
    /// Filtered messages based on current filters
    filtered_messages: Vec<NatsMessage>,
    /// Current message filters
    filters: MessageFilters,
    /// Selected message index (in filtered list)
    selected_index: usize,
    /// Whether to show detail view for selected message
    show_detail: bool,
    /// Auto-scroll to latest message
    auto_scroll: bool,
    /// Connection diagnostics
    diagnostics: MonitoringDiagnostics,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            messages: VecDeque::with_capacity(1000),
            max_messages: 1000,
            filtered_messages: Vec::new(),
            filters: MessageFilters::default(),
            selected_index: 0,
            show_detail: false,
            auto_scroll: true,
            diagnostics: MonitoringDiagnostics::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test message
    fn create_test_message(subject: &str, payload: &str) -> NatsMessage {
        NatsMessage::new(subject.to_string(), payload.as_bytes().to_vec())
    }

    #[test]
    fn test_monitor_context_initialization() {
        let ctx = MonitorContext::new();

        assert_eq!(ctx.message_count(), 0);
        assert_eq!(ctx.filtered_count(), 0);
        assert!(ctx.auto_scroll());
        assert!(!ctx.show_detail());
        assert_eq!(ctx.selected_index(), 0);
    }

    #[test]
    fn test_add_message() {
        let ctx = MonitorContext::new();

        let msg1 = create_test_message("mcp.orchestrator.request", r#"{"request_id": "req-1"}"#);
        ctx.add_message(msg1);

        assert_eq!(ctx.message_count(), 1);
        assert_eq!(ctx.filtered_count(), 1);
    }

    #[test]
    fn test_circular_buffer() {
        let ctx = MonitorContext::new();

        // Add 1001 messages (exceeds max of 1000)
        for i in 0..1001 {
            let msg = create_test_message(
                "mcp.test.request",
                &format!(r#"{{"id": "msg-{:04}"}}"#, i),
            );
            ctx.add_message(msg);
        }

        // Should cap at 1000
        assert_eq!(ctx.message_count(), 1000);

        // First message should be msg-0001 (msg-0000 was evicted)
        let messages = ctx.filtered_messages();
        assert!(messages[0].payload.contains("msg-0001"));
    }

    #[test]
    fn test_endpoint_filter() {
        let ctx = MonitorContext::new();

        ctx.add_message(create_test_message("mcp.orchestrator.request", "{}"));
        ctx.add_message(create_test_message("mcp.story-generator.request", "{}"));
        ctx.add_message(create_test_message("taletrail.generation.events", "{}"));

        assert_eq!(ctx.filtered_count(), 3);

        // Filter by orchestrator
        ctx.set_endpoint_filter(Some("orchestrator".to_string()));
        assert_eq!(ctx.filtered_count(), 1);

        // Clear filter
        ctx.clear_filters();
        assert_eq!(ctx.filtered_count(), 3);
    }

    #[test]
    fn test_message_type_filter() {
        let ctx = MonitorContext::new();

        ctx.add_message(create_test_message("mcp.orchestrator.request", "{}"));
        ctx.add_message(create_test_message("mcp.orchestrator.response", "{}"));
        ctx.add_message(create_test_message("taletrail.generation.events", "{}"));

        assert_eq!(ctx.filtered_count(), 3);

        // Filter by Request type
        ctx.set_message_type_filter(Some("Request".to_string()));
        assert_eq!(ctx.filtered_count(), 1);

        // Filter by Event type
        ctx.set_message_type_filter(Some("Event".to_string()));
        assert_eq!(ctx.filtered_count(), 1);
    }

    #[test]
    fn test_search_query_filter() {
        let ctx = MonitorContext::new();

        ctx.add_message(create_test_message("mcp.orchestrator.request", r#"{"tool": "test-tool"}"#));
        ctx.add_message(create_test_message("mcp.story-generator.request", r#"{"tool": "another-tool"}"#));

        assert_eq!(ctx.filtered_count(), 2);

        // Search for "test-tool"
        ctx.set_search_query(Some("test-tool".to_string()));
        assert_eq!(ctx.filtered_count(), 1);

        // Search for "story"
        ctx.set_search_query(Some("story".to_string()));
        assert_eq!(ctx.filtered_count(), 1);
    }

    #[test]
    fn test_combined_filters() {
        let ctx = MonitorContext::new();

        ctx.add_message(create_test_message("mcp.orchestrator.request", r#"{"id": "req-1"}"#));
        ctx.add_message(create_test_message("mcp.orchestrator.response", r#"{"id": "res-1"}"#));
        ctx.add_message(create_test_message("mcp.story-generator.request", r#"{"id": "req-2"}"#));

        // Filter by endpoint AND message type
        let filters = MessageFilters {
            endpoint: Some("orchestrator".to_string()),
            message_type: Some("Request".to_string()),
            search_query: None,
        };
        ctx.set_filters(filters);

        assert_eq!(ctx.filtered_count(), 1);
        let msg = ctx.filtered_messages()[0].clone();
        assert_eq!(msg.endpoint, "orchestrator");
        assert_eq!(msg.message_type, "Request");
    }

    #[test]
    fn test_message_selection() {
        let ctx = MonitorContext::new();

        // Disable auto-scroll for manual selection testing
        ctx.set_auto_scroll(false);

        ctx.add_message(create_test_message("mcp.orchestrator.request", r#"{"id": "1"}"#));
        ctx.add_message(create_test_message("mcp.story-generator.request", r#"{"id": "2"}"#));
        ctx.add_message(create_test_message("taletrail.generation.events", r#"{"id": "3"}"#));

        // Initial selection
        assert_eq!(ctx.selected_index(), 0);

        // Select next
        ctx.next_message();
        assert_eq!(ctx.selected_index(), 1);

        // Select next again
        ctx.next_message();
        assert_eq!(ctx.selected_index(), 2);

        // Try to go beyond last (should cap at last)
        ctx.next_message();
        assert_eq!(ctx.selected_index(), 2);

        // Select previous
        ctx.previous_message();
        assert_eq!(ctx.selected_index(), 1);
    }

    #[test]
    fn test_auto_scroll() {
        let ctx = MonitorContext::new();

        // Auto-scroll enabled by default
        assert!(ctx.auto_scroll());

        // Add messages - should auto-scroll to latest
        ctx.add_message(create_test_message("mcp.orchestrator.request", "{}"));
        ctx.add_message(create_test_message("mcp.story-generator.request", "{}"));

        // Selected should be last message (index 1)
        assert_eq!(ctx.selected_index(), 1);

        // Navigate up - should disable auto-scroll
        ctx.previous_message();
        assert!(!ctx.auto_scroll());
        assert_eq!(ctx.selected_index(), 0);

        // Add another message - selection should not change
        ctx.add_message(create_test_message("taletrail.generation.events", "{}"));
        assert_eq!(ctx.selected_index(), 0);

        // Toggle auto-scroll back on
        ctx.toggle_auto_scroll();
        assert!(ctx.auto_scroll());

        // Add message - should jump to latest
        ctx.add_message(create_test_message("mcp.test.request", "{}"));
        assert_eq!(ctx.selected_index(), 3); // Last message
    }

    #[test]
    fn test_detail_view() {
        let ctx = MonitorContext::new();

        assert!(!ctx.show_detail());

        ctx.set_show_detail(true);
        assert!(ctx.show_detail());

        ctx.toggle_detail();
        assert!(!ctx.show_detail());
    }

    #[test]
    fn test_clear_messages() {
        let ctx = MonitorContext::new();

        ctx.add_message(create_test_message("mcp.orchestrator.request", "{}"));
        ctx.add_message(create_test_message("mcp.story-generator.request", "{}"));

        assert_eq!(ctx.message_count(), 2);

        ctx.clear_messages();
        assert_eq!(ctx.message_count(), 0);
        assert_eq!(ctx.filtered_count(), 0);
        assert_eq!(ctx.selected_index(), 0);
        assert!(!ctx.show_detail());
    }

    #[test]
    fn test_diagnostics() {
        let ctx = MonitorContext::new();

        let diag = ctx.diagnostics();
        assert!(diag.is_connected);
        assert_eq!(diag.messages_received, 0);

        // Update diagnostics
        let mut new_diag = MonitoringDiagnostics::new();
        new_diag.messages_received = 100;
        new_diag.is_connected = false;

        ctx.update_diagnostics(new_diag);

        let updated = ctx.diagnostics();
        assert!(!updated.is_connected);
        assert_eq!(updated.messages_received, 100);
    }

    #[test]
    fn test_filter_description() {
        let filters = MessageFilters::default();
        assert_eq!(filters.description(), "No filters");
        assert!(!filters.is_active());

        let filters = MessageFilters {
            endpoint: Some("orchestrator".to_string()),
            message_type: None,
            search_query: None,
        };
        assert_eq!(filters.description(), "Endpoint: orchestrator");
        assert!(filters.is_active());

        let filters = MessageFilters {
            endpoint: Some("orchestrator".to_string()),
            message_type: Some("Request".to_string()),
            search_query: Some("test".to_string()),
        };
        assert_eq!(filters.description(), "Endpoint: orchestrator | Type: Request | Search: test");
        assert!(filters.is_active());
    }
}
