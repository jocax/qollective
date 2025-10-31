/// MCP Request History Commands
///
/// Provides Tauri commands for saving and retrieving MCP request/response history
/// using tauri-plugin-store for persistence.
use crate::constants::history;
use crate::models::mcp_history::{HistoryPage, HistoryQuery, HistoryStatus, McpHistoryEntry};
use chrono::Utc;
use rmcp::model::{CallToolRequest, CallToolResult};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

/// Save an MCP request/response to history
///
/// Creates a history entry from the request/response and saves it to the Tauri store.
/// Automatically determines status from the response (Success/Error).
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `server_name` - Name of the MCP server (e.g., "orchestrator")
/// * `tenant_id` - Tenant ID for multi-tenancy support
/// * `duration_ms` - Duration of the request in milliseconds
/// * `request` - Original CallToolRequest
/// * `response` - CallToolResult from the server
///
/// # Returns
/// * `Ok(String)` - The generated entry ID (UUID)
/// * `Err(String)` - Error message if saving fails
#[tauri::command]
pub async fn save_request_to_history(
    app: AppHandle,
    server_name: String,
    tenant_id: i32,
    duration_ms: u64,
    request: CallToolRequest,
    response: CallToolResult,
) -> Result<String, String> {
    // Generate UUID for entry
    let entry_id = Uuid::new_v4().to_string();

    // Extract tool name from request
    let tool_name = request.params.name.to_string();

    // Determine status from response
    let status = if response.is_error.unwrap_or(false) {
        HistoryStatus::Error
    } else {
        HistoryStatus::Success
    };

    // Create history entry
    let entry = McpHistoryEntry {
        id: entry_id.clone(),
        timestamp: Utc::now(),
        server_name,
        tool_name,
        tenant_id,
        status,
        duration_ms,
        request,
        response,
    };

    // Get or create store
    let store = app
        .store(history::STORE_FILE)
        .map_err(|e| format!("Failed to access store: {}", e))?;

    // Load existing history
    let mut entries: Vec<McpHistoryEntry> = store
        .get(history::STORE_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // Add new entry at the beginning (most recent first)
    entries.insert(0, entry);

    // Save back to store
    store.set(history::STORE_KEY, serde_json::to_value(&entries).unwrap());

    store
        .save()
        .map_err(|e| format!("Failed to persist store: {}", e))?;

    Ok(entry_id)
}

/// Load request history with filtering and pagination
///
/// Loads history from the Tauri store and applies filters, search, and pagination.
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `query` - Query parameters (page, filters, search term)
///
/// # Returns
/// * `Ok(HistoryPage)` - Paginated history results
/// * `Err(String)` - Error message if loading fails
#[tauri::command]
pub async fn load_request_history(
    app: AppHandle,
    query: HistoryQuery,
) -> Result<HistoryPage, String> {
    // Validate page size
    let page_size = query
        .page_size
        .min(history::MAX_PAGE_SIZE)
        .max(history::MIN_PAGE_SIZE);

    // Get store
    let store = app
        .store(history::STORE_FILE)
        .map_err(|e| format!("Failed to access store: {}", e))?;

    // Load all entries
    let mut entries: Vec<McpHistoryEntry> = store
        .get(history::STORE_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // Apply filters
    if let Some(ref server_filter) = query.server_filter {
        entries.retain(|e| e.server_name == *server_filter);
    }

    if let Some(ref status_filter) = query.status_filter {
        entries.retain(|e| e.status == *status_filter);
    }

    if let Some(ref search_term) = query.search_term {
        let search_lower = search_term.to_lowercase();
        entries.retain(|e| e.tool_name.to_lowercase().contains(&search_lower));
    }

    // Calculate pagination
    let total_count = entries.len();
    let total_pages = (total_count + page_size - 1) / page_size;
    let page = query.page.min(total_pages.saturating_sub(1));

    // Extract page
    let start = page * page_size;
    let end = (start + page_size).min(total_count);
    let page_entries = entries[start..end].to_vec();

    Ok(HistoryPage {
        entries: page_entries,
        total_count,
        page,
        page_size,
        total_pages,
    })
}

/// Delete a specific history entry by ID
///
/// Removes the entry with the specified ID from the history store.
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `entry_id` - UUID of the entry to delete
///
/// # Returns
/// * `Ok(())` - Entry deleted successfully
/// * `Err(String)` - Error message if deletion fails
#[tauri::command]
pub async fn delete_history_entry(app: AppHandle, entry_id: String) -> Result<(), String> {
    // Get store
    let store = app
        .store(history::STORE_FILE)
        .map_err(|e| format!("Failed to access store: {}", e))?;

    // Load all entries
    let mut entries: Vec<McpHistoryEntry> = store
        .get(history::STORE_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // Find and remove entry
    let original_len = entries.len();
    entries.retain(|e| e.id != entry_id);

    if entries.len() == original_len {
        return Err(format!("Entry not found: {}", entry_id));
    }

    // Save back to store
    store.set(history::STORE_KEY, serde_json::to_value(&entries).unwrap());

    store
        .save()
        .map_err(|e| format!("Failed to persist store: {}", e))?;

    Ok(())
}

/// Clear all request history
///
/// Removes all history entries from the store.
///
/// # Arguments
/// * `app` - Tauri application handle
///
/// # Returns
/// * `Ok(())` - History cleared successfully
/// * `Err(String)` - Error message if clearing fails
#[tauri::command]
pub async fn clear_request_history(app: AppHandle) -> Result<(), String> {
    // Get store
    let store = app
        .store(history::STORE_FILE)
        .map_err(|e| format!("Failed to access store: {}", e))?;

    // Clear history by setting empty array
    let empty_vec: Vec<McpHistoryEntry> = vec![];
    store.set(
        history::STORE_KEY,
        serde_json::to_value(&empty_vec).unwrap(),
    );

    store
        .save()
        .map_err(|e| format!("Failed to persist store: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::{CallToolRequestMethod, CallToolRequestParam, Content};
    use serde_json::json;

    /// Helper to create a test CallToolRequest
    fn create_test_request(tool_name: &str) -> CallToolRequest {
        CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: tool_name.to_string().into(),
                arguments: Some(
                    json!({
                        "theme": "Test Theme",
                        "age_group": "9-11"
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            },
            extensions: Default::default(),
        }
    }

    /// Helper to create a test CallToolResult
    fn create_test_response(is_error: bool) -> CallToolResult {
        let message = if is_error {
            "Error occurred"
        } else {
            "Success"
        };

        CallToolResult {
            content: vec![Content::text(message)],
            structured_content: None,
            is_error: Some(is_error),
            meta: None,
        }
    }

    #[tokio::test]
    async fn test_history_entry_creation() {
        let request = create_test_request("orchestrate_generation");
        let response = create_test_response(false);

        let entry = McpHistoryEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            server_name: "orchestrator".to_string(),
            tool_name: "orchestrate_generation".to_string(),
            tenant_id: 42,
            status: HistoryStatus::Success,
            duration_ms: 1500,
            request: request.clone(),
            response: response.clone(),
        };

        assert_eq!(entry.server_name, "orchestrator");
        assert_eq!(entry.tool_name, "orchestrate_generation");
        assert_eq!(entry.tenant_id, 42);
        assert_eq!(entry.status, HistoryStatus::Success);
        assert_eq!(entry.duration_ms, 1500);
        assert!(!entry.id.is_empty());
    }

    #[tokio::test]
    async fn test_status_determination_success() {
        let response = create_test_response(false);
        let status = if response.is_error.unwrap_or(false) {
            HistoryStatus::Error
        } else {
            HistoryStatus::Success
        };

        assert_eq!(status, HistoryStatus::Success);
    }

    #[tokio::test]
    async fn test_status_determination_error() {
        let response = create_test_response(true);
        let status = if response.is_error.unwrap_or(false) {
            HistoryStatus::Error
        } else {
            HistoryStatus::Success
        };

        assert_eq!(status, HistoryStatus::Error);
    }

    #[tokio::test]
    async fn test_status_determination_missing() {
        let response = CallToolResult {
            content: vec![Content::text("Unknown")],
            structured_content: None,
            is_error: None,
            meta: None,
        };

        let status = if response.is_error.unwrap_or(false) {
            HistoryStatus::Error
        } else {
            HistoryStatus::Success
        };

        // Missing is_error should default to Success
        assert_eq!(status, HistoryStatus::Success);
    }

    #[tokio::test]
    async fn test_history_query_defaults() {
        let query = HistoryQuery::default();

        assert_eq!(query.page, 0);
        assert_eq!(query.page_size, history::DEFAULT_PAGE_SIZE);
        assert!(query.server_filter.is_none());
        assert!(query.status_filter.is_none());
        assert!(query.search_term.is_none());
    }

    #[tokio::test]
    async fn test_history_pagination_calculation() {
        let total_items = 125;
        let page_size = 50;
        let total_pages = (total_items + page_size - 1) / page_size;

        assert_eq!(total_pages, 3);

        // Test page boundaries
        let page_0_start = 0 * page_size;
        let page_0_end = (page_0_start + page_size).min(total_items);
        assert_eq!(page_0_start, 0);
        assert_eq!(page_0_end, 50);

        let page_1_start = 1 * page_size;
        let page_1_end = (page_1_start + page_size).min(total_items);
        assert_eq!(page_1_start, 50);
        assert_eq!(page_1_end, 100);

        let page_2_start = 2 * page_size;
        let page_2_end = (page_2_start + page_size).min(total_items);
        assert_eq!(page_2_start, 100);
        assert_eq!(page_2_end, 125);
    }

    #[tokio::test]
    async fn test_history_filtering_logic_server() {
        let mut entries = vec![
            McpHistoryEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                server_name: "orchestrator".to_string(),
                tool_name: "orchestrate_generation".to_string(),
                tenant_id: 1,
                status: HistoryStatus::Success,
                duration_ms: 1000,
                request: create_test_request("orchestrate_generation"),
                response: create_test_response(false),
            },
            McpHistoryEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                server_name: "story-generator".to_string(),
                tool_name: "generate_structure".to_string(),
                tenant_id: 1,
                status: HistoryStatus::Success,
                duration_ms: 2000,
                request: create_test_request("generate_structure"),
                response: create_test_response(false),
            },
        ];

        // Filter by orchestrator
        let server_filter = "orchestrator".to_string();
        entries.retain(|e| e.server_name == server_filter);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].server_name, "orchestrator");
    }

    #[tokio::test]
    async fn test_history_filtering_logic_status() {
        let mut entries = vec![
            McpHistoryEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                server_name: "orchestrator".to_string(),
                tool_name: "orchestrate_generation".to_string(),
                tenant_id: 1,
                status: HistoryStatus::Success,
                duration_ms: 1000,
                request: create_test_request("orchestrate_generation"),
                response: create_test_response(false),
            },
            McpHistoryEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                server_name: "story-generator".to_string(),
                tool_name: "generate_structure".to_string(),
                tenant_id: 1,
                status: HistoryStatus::Error,
                duration_ms: 2000,
                request: create_test_request("generate_structure"),
                response: create_test_response(true),
            },
        ];

        // Filter by error status
        let status_filter = HistoryStatus::Error;
        entries.retain(|e| e.status == status_filter);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].status, HistoryStatus::Error);
    }

    #[tokio::test]
    async fn test_history_search_logic() {
        let mut entries = vec![
            McpHistoryEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                server_name: "orchestrator".to_string(),
                tool_name: "orchestrate_generation".to_string(),
                tenant_id: 1,
                status: HistoryStatus::Success,
                duration_ms: 1000,
                request: create_test_request("orchestrate_generation"),
                response: create_test_response(false),
            },
            McpHistoryEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                server_name: "story-generator".to_string(),
                tool_name: "generate_structure".to_string(),
                tenant_id: 1,
                status: HistoryStatus::Success,
                duration_ms: 2000,
                request: create_test_request("generate_structure"),
                response: create_test_response(false),
            },
        ];

        // Search for "orchestrate"
        let search_term = "orchestrate".to_string();
        let search_lower = search_term.to_lowercase();
        entries.retain(|e| e.tool_name.to_lowercase().contains(&search_lower));

        assert_eq!(entries.len(), 1);
        assert!(entries[0].tool_name.contains("orchestrate"));
    }

    #[tokio::test]
    async fn test_history_page_size_validation() {
        // Test max boundary
        let oversized = 500;
        let validated = oversized
            .min(history::MAX_PAGE_SIZE)
            .max(history::MIN_PAGE_SIZE);
        assert_eq!(validated, history::MAX_PAGE_SIZE);

        // Test min boundary
        let undersized = 5;
        let validated = undersized
            .min(history::MAX_PAGE_SIZE)
            .max(history::MIN_PAGE_SIZE);
        assert_eq!(validated, history::MIN_PAGE_SIZE);

        // Test normal value
        let normal = 50;
        let validated = normal
            .min(history::MAX_PAGE_SIZE)
            .max(history::MIN_PAGE_SIZE);
        assert_eq!(validated, 50);
    }

    #[tokio::test]
    async fn test_uuid_generation() {
        let id1 = Uuid::new_v4().to_string();
        let id2 = Uuid::new_v4().to_string();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID v4 string length
        assert_eq!(id2.len(), 36);
    }

    #[tokio::test]
    async fn test_tool_name_extraction() {
        let request = create_test_request("orchestrate_generation");
        let tool_name = request.params.name.to_string();

        assert_eq!(tool_name, "orchestrate_generation");
    }

    #[tokio::test]
    async fn test_history_entry_serialization() {
        let entry = McpHistoryEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            server_name: "orchestrator".to_string(),
            tool_name: "orchestrate_generation".to_string(),
            tenant_id: 42,
            status: HistoryStatus::Success,
            duration_ms: 1500,
            request: create_test_request("orchestrate_generation"),
            response: create_test_response(false),
        };

        // Test serialization
        let serialized = serde_json::to_value(&entry).expect("Failed to serialize");
        assert!(serialized.is_object());

        // Test deserialization
        let deserialized: McpHistoryEntry =
            serde_json::from_value(serialized).expect("Failed to deserialize");
        assert_eq!(deserialized.id, entry.id);
        assert_eq!(deserialized.server_name, entry.server_name);
        assert_eq!(deserialized.tool_name, entry.tool_name);
        assert_eq!(deserialized.status, entry.status);
    }

    #[tokio::test]
    async fn test_history_status_serialization() {
        // Test all status variants
        let success = HistoryStatus::Success;
        let error = HistoryStatus::Error;
        let timeout = HistoryStatus::Timeout;

        let success_json = serde_json::to_value(&success).unwrap();
        let error_json = serde_json::to_value(&error).unwrap();
        let timeout_json = serde_json::to_value(&timeout).unwrap();

        assert_eq!(success_json, json!("success"));
        assert_eq!(error_json, json!("error"));
        assert_eq!(timeout_json, json!("timeout"));

        // Test deserialization
        let success_back: HistoryStatus = serde_json::from_value(success_json).unwrap();
        assert_eq!(success_back, HistoryStatus::Success);
    }

    // Integration tests with Tauri would require a test AppHandle
    // These are unit tests for the business logic
}
