/// Request History Panel Component
///
/// Displays paginated history of MCP requests with filtering and replay functionality

use crate::components::list::List;
use crate::models::history::{HistoryQuery, HistoryStatus, McpHistoryEntry};
use crate::state::McpContext;
use iocraft::prelude::*;

/// Props for HistoryPanel component
#[derive(Props)]
pub struct HistoryPanelProps {
    pub mcp_context: McpContext,
}

impl Default for HistoryPanelProps {
    fn default() -> Self {
        Self {
            mcp_context: McpContext::new(),
        }
    }
}

/// History panel component with pagination and filtering
#[component]
pub fn HistoryPanel(_hooks: Hooks, props: &HistoryPanelProps) -> impl Into<AnyElement<'static>> {
    let page = props.mcp_context.paginated_history();
    let query = props.mcp_context.history_query();

    let mut elements: Vec<AnyElement> = Vec::new();

    // Header
    elements.push(
        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Cyan,
                padding: 1,
                margin_bottom: 1,
            ) {
                Text(
                    content: "Request History",
                    color: Color::Cyan,
                    weight: Weight::Bold,
                )
            }
        }
        .into_any(),
    );

    // Filter info
    let filter_text = build_filter_text(&query);
    if !filter_text.is_empty() {
        elements.push(
            element! {
                View(margin_bottom: 1) {
                    Text(
                        content: format!("Filters: {}", filter_text),
                        color: Color::Yellow,
                    )
                }
            }
            .into_any(),
        );
    }

    // Pagination info
    elements.push(
        element! {
            View(margin_bottom: 1) {
                Text(
                    content: page.range_display(),
                    color: Color::Grey,
                )
            }
        }
        .into_any(),
    );

    // History list
    if page.entries.is_empty() {
        elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Grey,
                    padding: 2,
                ) {
                    Text(
                        content: "No history entries found. Send a request to populate history.",
                        color: Color::Grey,
                    )
                }
            }
            .into_any(),
        );
    } else {
        // Render history items with List component
        fn render_history_item(entry: &McpHistoryEntry, is_selected: bool) -> String {
            let prefix = if is_selected { "> " } else { "  " };
            let status_icon = match entry.status {
                HistoryStatus::Success => "✓",
                HistoryStatus::Error => "✗",
                HistoryStatus::Timeout => "⏱",
            };

            format!(
                "{}{} [{}] {} - {} ({}ms)",
                prefix,
                status_icon,
                entry.short_timestamp(),
                entry.server_name,
                entry.tool_name,
                entry.duration_ms
            )
        }

        elements.push(
            element! {
                List::<McpHistoryEntry>(
                    items: page.entries.clone(),
                    selected_index: 0usize,
                    render_item: Some(render_history_item as fn(&McpHistoryEntry, bool) -> String),
                    visible_rows: 15usize,
                    show_pagination: false,
                )
            }
            .into_any(),
        );
    }

    // Pagination controls
    if page.total_pages > 1 {
        let pagination_text = format!(
            "Page {} of {} | {}/{}",
            page.page + 1,
            page.total_pages,
            if page.has_previous() { "←" } else { " " },
            if page.has_next() { "→" } else { " " }
        );

        elements.push(
            element! {
                View(margin_top: 1) {
                    Text(content: pagination_text, color: Color::Cyan)
                }
            }
            .into_any(),
        );
    }

    // Help text
    elements.push(
        element! {
            View(margin_top: 1) {
                Text(
                    content: "↑/↓: Navigate | Enter: Replay | D: Delete | C: Clear All | F: Filter",
                    color: Color::Grey,
                )
            }
        }
        .into_any(),
    );

    element! {
        View(
            flex_direction: FlexDirection::Column,
        ) {
            #(elements.into_iter())
        }
    }
    .into_any()
}

/// Build filter description text
fn build_filter_text(query: &HistoryQuery) -> String {
    let mut filters = Vec::new();

    if let Some(ref server) = query.server_filter {
        filters.push(format!("Server: {}", server));
    }

    if let Some(ref status) = query.status_filter {
        filters.push(format!("Status: {}", status));
    }

    if let Some(ref term) = query.search_term {
        filters.push(format!("Search: {}", term));
    }

    filters.join(", ")
}

/// Save history to JSON file
pub async fn save_history(
    history: &[McpHistoryEntry],
    file_path: &str,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(history)
        .map_err(|e| format!("Failed to serialize history: {}", e))?;

    smol::fs::write(file_path, json)
        .await
        .map_err(|e| format!("Failed to write history file: {}", e))?;

    Ok(())
}

/// Load history from JSON file
pub async fn load_history(file_path: &str) -> Result<Vec<McpHistoryEntry>, String> {
    // Check if file exists
    if !std::path::Path::new(file_path).exists() {
        return Ok(Vec::new());
    }

    let content = smol::fs::read_to_string(file_path)
        .await
        .map_err(|e| format!("Failed to read history file: {}", e))?;

    let history: Vec<McpHistoryEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse history file: {}", e))?;

    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::history::{HistoryQuery, HistoryStatus, McpHistoryEntry};
    use chrono::Utc;
    use rmcp::model::{CallToolRequest, CallToolRequestParam, CallToolResult};

    fn create_test_entry(
        server_name: &str,
        tool_name: &str,
        status: HistoryStatus,
    ) -> McpHistoryEntry {
        let params = CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments: None,
        };

        let request = CallToolRequest::new(params);

        let response = if status == HistoryStatus::Error {
            CallToolResult::error(vec![])
        } else {
            CallToolResult::success(vec![])
        };

        McpHistoryEntry::new(
            server_name.to_string(),
            tool_name.to_string(),
            1,
            status,
            100,
            request,
            response,
        )
    }

    #[test]
    fn test_history_panel_pagination() {
        let ctx = McpContext::new();

        // Create test history entries
        let mut entries = Vec::new();
        for i in 0..50 {
            entries.push(create_test_entry(
                "orchestrator",
                &format!("tool_{}", i),
                HistoryStatus::Success,
            ));
        }

        // Add entries to context
        for entry in entries {
            ctx.add_history_entry(entry);
        }

        // Get first page (default page size is 20)
        let query = HistoryQuery::default();
        ctx.set_history_query(query);

        let page = ctx.paginated_history();
        assert_eq!(page.entries.len(), 20);
        assert_eq!(page.total_count, 50);
        assert_eq!(page.page, 0);
        assert_eq!(page.total_pages, 3);
        assert!(page.has_next());
        assert!(!page.has_previous());

        // Get second page
        let query = HistoryQuery::default().with_page(1);
        ctx.set_history_query(query);

        let page = ctx.paginated_history();
        assert_eq!(page.entries.len(), 20);
        assert_eq!(page.page, 1);
        assert!(page.has_next());
        assert!(page.has_previous());

        // Get last page
        let query = HistoryQuery::default().with_page(2);
        ctx.set_history_query(query);

        let page = ctx.paginated_history();
        assert_eq!(page.entries.len(), 10); // Remaining entries
        assert_eq!(page.page, 2);
        assert!(!page.has_next());
        assert!(page.has_previous());
    }

    #[test]
    fn test_history_filtering() {
        let ctx = McpContext::new();

        // Create diverse history
        ctx.add_history_entry(create_test_entry(
            "orchestrator",
            "start_workflow",
            HistoryStatus::Success,
        ));
        ctx.add_history_entry(create_test_entry(
            "story-generator",
            "generate_scene",
            HistoryStatus::Success,
        ));
        ctx.add_history_entry(create_test_entry(
            "orchestrator",
            "validate_input",
            HistoryStatus::Error,
        ));

        // Filter by server
        let query = HistoryQuery::default().with_server("orchestrator".to_string());
        ctx.set_history_query(query);

        let page = ctx.paginated_history();
        assert_eq!(page.total_count, 2);

        // Filter by status
        let query = HistoryQuery::default().with_status(HistoryStatus::Error);
        ctx.set_history_query(query);

        let page = ctx.paginated_history();
        assert_eq!(page.total_count, 1);

        // Filter by search term
        let query = HistoryQuery::default().with_search("workflow".to_string());
        ctx.set_history_query(query);

        let page = ctx.paginated_history();
        assert_eq!(page.total_count, 1);
    }

    #[test]
    fn test_history_management() {
        let ctx = McpContext::new();

        // Add entries
        let entry1 = create_test_entry("orchestrator", "tool1", HistoryStatus::Success);
        let entry2 = create_test_entry("story-generator", "tool2", HistoryStatus::Error);

        let id1 = entry1.id.clone();

        ctx.add_history_entry(entry1);
        ctx.add_history_entry(entry2);

        assert_eq!(ctx.history().len(), 2);

        // Delete specific entry
        ctx.delete_history_entry(&id1);
        assert_eq!(ctx.history().len(), 1);

        // Clear all
        ctx.clear_history();
        assert_eq!(ctx.history().len(), 0);
    }

    #[test]
    fn test_build_filter_text() {
        // No filters
        let query = HistoryQuery::default();
        assert_eq!(build_filter_text(&query), "");

        // Server filter only
        let query = HistoryQuery::default().with_server("orchestrator".to_string());
        assert_eq!(build_filter_text(&query), "Server: orchestrator");

        // Multiple filters
        let query = HistoryQuery::default()
            .with_server("orchestrator".to_string())
            .with_status(HistoryStatus::Success)
            .with_search("test".to_string());

        let text = build_filter_text(&query);
        assert!(text.contains("Server: orchestrator"));
        assert!(text.contains("Status: success"));
        assert!(text.contains("Search: test"));
    }

    #[test]
    fn test_page_range_display() {
        let ctx = McpContext::new();

        // Empty history
        let page = ctx.paginated_history();
        assert_eq!(page.range_display(), "0 of 0");

        // Add some entries
        for i in 0..5 {
            ctx.add_history_entry(create_test_entry(
                "orchestrator",
                &format!("tool_{}", i),
                HistoryStatus::Success,
            ));
        }

        let page = ctx.paginated_history();
        assert_eq!(page.range_display(), "1-5 of 5");

        // Test with more entries than page size
        for i in 5..25 {
            ctx.add_history_entry(create_test_entry(
                "orchestrator",
                &format!("tool_{}", i),
                HistoryStatus::Success,
            ));
        }

        let page = ctx.paginated_history();
        assert_eq!(page.range_display(), "1-20 of 25");
    }

    // Note: Async tests for save/load history are omitted here
    // They would require smol::test macro or a test runtime
    // These functions are tested indirectly through the app runtime
}
