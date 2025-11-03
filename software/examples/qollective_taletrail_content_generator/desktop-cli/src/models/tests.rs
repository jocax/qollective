use super::*;

// Test 1: Trail JSON deserialization
#[test]
fn test_trail_list_item_deserialization() {
    let json = r#"{
        "id": "trail-123",
        "file_path": "/path/to/trail.json",
        "title": "Space Adventure",
        "description": "An exciting space journey",
        "theme": "space exploration",
        "age_group": "9-11",
        "language": "en",
        "tags": ["science", "adventure"],
        "status": "completed",
        "generated_at": "2025-10-22T10:00:00Z",
        "node_count": 15,
        "tenant_id": "1"
    }"#;

    let item: TrailListItem = serde_json::from_str(json).expect("Failed to deserialize TrailListItem");

    assert_eq!(item.id, "trail-123");
    assert_eq!(item.title, "Space Adventure");
    assert_eq!(item.age_group, "9-11");
    assert_eq!(item.language, "en");
    assert_eq!(item.node_count, 15);
    assert_eq!(item.tags, vec!["science", "adventure"]);
    assert_eq!(item.tenant_id, Some("1".to_string()));
}

// Test 2: Trail envelope structure
#[test]
fn test_response_envelope_structure() {
    let envelope = ResponseEnvelope {
        meta: EnvelopeMeta {
            request_id: "req-456".to_string(),
            timestamp: "2025-10-22T10:00:00Z".to_string(),
            tenant: "1".to_string(),
            version: "1.0".to_string(),
        },
        payload: EnvelopePayload {
            tool_response: ToolResponse {
                content: vec![ContentItem {
                    content_type: "text".to_string(),
                    text: "Story content here".to_string(),
                }],
                is_error: false,
            },
        },
    };

    // Serialize and deserialize
    let json = serde_json::to_string(&envelope).expect("Failed to serialize");
    let deserialized: ResponseEnvelope = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.meta.request_id, "req-456");
    assert_eq!(deserialized.payload.tool_response.content.len(), 1);
    assert!(!deserialized.payload.tool_response.is_error);
}

// Test 3: MCP server configuration
#[test]
fn test_mcp_server_config() {
    let servers = McpServerConfig::all_servers();

    // Should have 5 predefined servers
    assert_eq!(servers.len(), 5);

    // Check orchestrator exists
    let orchestrator = McpServerConfig::get_server("orchestrator");
    assert!(orchestrator.is_some());
    let orch = orchestrator.unwrap();
    assert_eq!(orch.name, "orchestrator");
    assert_eq!(orch.subject, "mcp.orchestrator.>");
    assert!(orch.description.is_some());

    // Test server availability toggle
    let mut server = McpServerConfig::new("test".to_string(), "test.>".to_string());
    assert!(!server.available);
    server.set_available(true);
    assert!(server.available);
}

// Test 4: User preferences with defaults
#[test]
fn test_user_preferences_defaults() {
    let prefs = UserPreferences::default();

    assert_eq!(prefs.trails_directory, "taletrail-data");
    assert_eq!(prefs.nats_url, "nats://localhost:4222");
    assert_eq!(prefs.nats_timeout, 30);
    assert!(prefs.auto_scroll);
    assert!(prefs.auto_validate_json);
    assert_eq!(prefs.history_page_size, 20);
    assert_eq!(prefs.theme, Theme::System);
}

// Test 5: Bookmark collection operations
#[test]
fn test_bookmark_collection_operations() {
    let mut collection = BookmarkCollection::new();

    // Initially empty
    assert_eq!(collection.all().len(), 0);
    assert!(!collection.contains("trail-1"));

    // Add bookmark
    let bookmark1 = Bookmark::new(
        "trail-1".to_string(),
        "Space Adventure".to_string(),
        "/path/to/trail1.json".to_string(),
    );
    collection.add(bookmark1);

    // Check contains
    assert!(collection.contains("trail-1"));
    assert_eq!(collection.all().len(), 1);

    // Get bookmark
    let retrieved = collection.get("trail-1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().trail_title, "Space Adventure");

    // Toggle (remove)
    let bookmark_copy = Bookmark::new(
        "trail-1".to_string(),
        "Space Adventure".to_string(),
        "/path/to/trail1.json".to_string(),
    );
    let removed = collection.toggle(bookmark_copy);
    assert!(!removed); // Returns false when removed
    assert!(!collection.contains("trail-1"));

    // Toggle (add)
    let bookmark2 = Bookmark::new(
        "trail-2".to_string(),
        "Medieval Quest".to_string(),
        "/path/to/trail2.json".to_string(),
    )
    .with_note("Great story!".to_string())
    .with_tenant("1".to_string());

    let added = collection.toggle(bookmark2);
    assert!(added); // Returns true when added
    assert!(collection.contains("trail-2"));

    let retrieved = collection.get("trail-2").unwrap();
    assert_eq!(retrieved.user_note, "Great story!");
    assert_eq!(retrieved.tenant_id, Some("1".to_string()));
}

// Test 6: Generation event creation
#[test]
fn test_generation_event_creation() {
    let event = GenerationEvent::new(
        "generation_started".to_string(),
        "tenant-123".to_string(),
        "req-456".to_string(),
        "story-generator".to_string(),
        "in_progress".to_string(),
    );

    assert_eq!(event.event_type, "generation_started");
    assert_eq!(event.tenant_id, "tenant-123");
    assert_eq!(event.request_id, "req-456");
    assert_eq!(event.service_phase, "story-generator");
    assert_eq!(event.status, "in_progress");
    assert!(event.progress.is_none());
    assert!(event.error_message.is_none());

    // Test with progress
    let event_with_progress = event.with_progress(0.5);
    assert_eq!(event_with_progress.progress, Some(0.5));

    // Test with error
    let event_with_error = GenerationEvent::new(
        "generation_failed".to_string(),
        "tenant-123".to_string(),
        "req-789".to_string(),
        "story-generator".to_string(),
        "failed".to_string(),
    )
    .with_error("Timeout occurred".to_string());

    assert_eq!(event_with_error.status, "failed");
    assert_eq!(
        event_with_error.error_message,
        Some("Timeout occurred".to_string())
    );
}

// Test 7: History query operations (without rmcp dependencies)
#[test]
fn test_history_query_operations() {
    // Test default query
    let query = HistoryQuery::default();
    assert_eq!(query.page, 0);
    assert_eq!(query.page_size, 20);
    assert!(query.server_filter.is_none());
    assert!(query.status_filter.is_none());
    assert!(query.search_term.is_none());

    // Test query builder
    let query = HistoryQuery::new()
        .with_page(1)
        .with_page_size(10)
        .with_server("orchestrator".to_string())
        .with_status(HistoryStatus::Success)
        .with_search("story".to_string());

    assert_eq!(query.page, 1);
    assert_eq!(query.page_size, 10);
    assert_eq!(query.server_filter, Some("orchestrator".to_string()));
    assert_eq!(query.status_filter, Some(HistoryStatus::Success));
    assert_eq!(query.search_term, Some("story".to_string()));

    // Test status display
    assert_eq!(HistoryStatus::Success.to_string(), "success");
    assert_eq!(HistoryStatus::Error.to_string(), "error");
    assert_eq!(HistoryStatus::Timeout.to_string(), "timeout");
}

// Test 8: History page pagination logic (without creating actual entries)
#[test]
fn test_history_page_pagination() {
    // Test page with 10 entries on first page of 25 total
    let page1 = HistoryPage::new(vec![], 25, 0, 10);

    assert_eq!(page1.total_count, 25);
    assert_eq!(page1.page, 0);
    assert_eq!(page1.page_size, 10);
    assert_eq!(page1.total_pages, 3); // 25 / 10 = 3 pages
    assert!(page1.has_next());
    assert!(!page1.has_previous());

    // Test middle page
    let page2 = HistoryPage::new(vec![], 25, 1, 10);
    assert!(page2.has_next());
    assert!(page2.has_previous());

    // Test last page
    let page3 = HistoryPage::new(vec![], 25, 2, 10);
    assert!(!page3.has_next());
    assert!(page3.has_previous());

    // Empty page
    let empty_page = HistoryPage::new(vec![], 0, 0, 10);
    assert_eq!(empty_page.range_display(), "0 of 0");
    assert_eq!(empty_page.total_pages, 0);
    assert!(!empty_page.has_next());
    assert!(!empty_page.has_previous());

    // Test edge case: exact multiple of page size
    let exact_page = HistoryPage::new(vec![], 20, 0, 10);
    assert_eq!(exact_page.total_pages, 2);

    // Test single item
    let single_page = HistoryPage::new(vec![], 1, 0, 10);
    assert_eq!(single_page.total_pages, 1);
    assert!(!single_page.has_next());
}
