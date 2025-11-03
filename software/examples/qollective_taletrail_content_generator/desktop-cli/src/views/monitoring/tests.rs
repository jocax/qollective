/// Integration tests for NATS Monitoring Interface
///
/// Tests cover message subscription, filtering, diagnostics, and detail view

use crate::nats::monitoring::NatsMessage;
use crate::state::{MessageFilters, MonitorContext};

/// Helper to create a test message
fn create_test_message(subject: &str, payload: &str) -> NatsMessage {
    NatsMessage::new(subject.to_string(), payload.as_bytes().to_vec())
}

/// Test 1: Message subscription and buffering
#[test]
fn test_message_subscription_and_buffer() {
    let ctx = MonitorContext::new();

    // Initially empty
    assert_eq!(ctx.message_count(), 0);
    assert_eq!(ctx.filtered_count(), 0);

    // Add messages
    ctx.add_message(create_test_message(
        "mcp.orchestrator.request",
        r#"{"request_id": "req-1", "tool": "test"}"#,
    ));

    ctx.add_message(create_test_message(
        "mcp.story-generator.request",
        r#"{"request_id": "req-2", "tool": "generate"}"#,
    ));

    ctx.add_message(create_test_message(
        "taletrail.generation.events",
        r#"{"event": "started", "request_id": "req-1"}"#,
    ));

    // Verify buffering
    assert_eq!(ctx.message_count(), 3);
    assert_eq!(ctx.filtered_count(), 3);

    // Verify circular buffer (add 1001 messages, should cap at 1000)
    for i in 0..1001 {
        ctx.add_message(create_test_message(
            "mcp.test.request",
            &format!(r#"{{"id": "msg-{:04}"}}"#, i),
        ));
    }

    assert_eq!(ctx.message_count(), 1000); // Capped at max
}

/// Test 2: Message filtering by endpoint
#[test]
fn test_message_filtering_by_endpoint() {
    let ctx = MonitorContext::new();

    // Add messages from different endpoints
    ctx.add_message(create_test_message("mcp.orchestrator.request", "{}"));
    ctx.add_message(create_test_message("mcp.orchestrator.response", "{}"));
    ctx.add_message(create_test_message("mcp.story-generator.request", "{}"));
    ctx.add_message(create_test_message("mcp.quality-control.request", "{}"));
    ctx.add_message(create_test_message("taletrail.generation.events", "{}"));

    assert_eq!(ctx.filtered_count(), 5);

    // Filter by orchestrator endpoint
    ctx.set_endpoint_filter(Some("orchestrator".to_string()));
    assert_eq!(ctx.filtered_count(), 2); // Request and response

    // Filter by story-generator
    ctx.set_endpoint_filter(Some("story-generator".to_string()));
    assert_eq!(ctx.filtered_count(), 1);

    // Clear filter
    ctx.clear_filters();
    assert_eq!(ctx.filtered_count(), 5);
}

/// Test 3: Message filtering by type
#[test]
fn test_message_filtering_by_type() {
    let ctx = MonitorContext::new();

    // Add messages of different types
    ctx.add_message(create_test_message("mcp.orchestrator.request", "{}"));
    ctx.add_message(create_test_message("mcp.orchestrator.response", "{}"));
    ctx.add_message(create_test_message("mcp.story-generator.request", "{}"));
    ctx.add_message(create_test_message("taletrail.generation.events", "{}"));

    assert_eq!(ctx.filtered_count(), 4);

    // Filter by Request type
    ctx.set_message_type_filter(Some("Request".to_string()));
    assert_eq!(ctx.filtered_count(), 2);

    // Filter by Response type
    ctx.set_message_type_filter(Some("Response".to_string()));
    assert_eq!(ctx.filtered_count(), 1);

    // Filter by Event type
    ctx.set_message_type_filter(Some("Event".to_string()));
    assert_eq!(ctx.filtered_count(), 1);
}

/// Test 4: Message filtering by search query
#[test]
fn test_message_filtering_by_search() {
    let ctx = MonitorContext::new();

    ctx.add_message(create_test_message(
        "mcp.orchestrator.request",
        r#"{"tool": "test-tool-alpha"}"#,
    ));
    ctx.add_message(create_test_message(
        "mcp.story-generator.request",
        r#"{"tool": "test-tool-beta"}"#,
    ));
    ctx.add_message(create_test_message(
        "mcp.quality-control.request",
        r#"{"tool": "another-tool"}"#,
    ));

    assert_eq!(ctx.filtered_count(), 3);

    // Search for "test-tool"
    ctx.set_search_query(Some("test-tool".to_string()));
    assert_eq!(ctx.filtered_count(), 2);

    // Search for "alpha"
    ctx.set_search_query(Some("alpha".to_string()));
    assert_eq!(ctx.filtered_count(), 1);

    // Search in subject
    ctx.set_search_query(Some("story-generator".to_string()));
    assert_eq!(ctx.filtered_count(), 1);
}

/// Test 5: Combined filters
#[test]
fn test_combined_filters() {
    let ctx = MonitorContext::new();

    ctx.add_message(create_test_message(
        "mcp.orchestrator.request",
        r#"{"id": "req-1", "data": "test-alpha"}"#,
    ));
    ctx.add_message(create_test_message(
        "mcp.orchestrator.response",
        r#"{"id": "res-1", "data": "test-beta"}"#,
    ));
    ctx.add_message(create_test_message(
        "mcp.story-generator.request",
        r#"{"id": "req-2", "data": "test-alpha"}"#,
    ));

    // Filter by endpoint AND message type
    let filters = MessageFilters {
        endpoint: Some("orchestrator".to_string()),
        message_type: Some("Request".to_string()),
        search_query: None,
    };
    ctx.set_filters(filters);
    assert_eq!(ctx.filtered_count(), 1);

    // Add search query to narrow further
    let filters = MessageFilters {
        endpoint: Some("orchestrator".to_string()),
        message_type: Some("Request".to_string()),
        search_query: Some("test-alpha".to_string()),
    };
    ctx.set_filters(filters);
    assert_eq!(ctx.filtered_count(), 1);

    // Search that doesn't match
    let filters = MessageFilters {
        endpoint: Some("orchestrator".to_string()),
        message_type: Some("Request".to_string()),
        search_query: Some("nonexistent".to_string()),
    };
    ctx.set_filters(filters);
    assert_eq!(ctx.filtered_count(), 0);
}

/// Test 6: Connection diagnostics tracking
#[test]
fn test_connection_diagnostics() {
    let ctx = MonitorContext::new();

    // Initial diagnostics
    let diag = ctx.diagnostics();
    assert!(diag.is_connected);
    assert_eq!(diag.messages_received, 0);
    assert_eq!(diag.messages_buffered, 0);
    assert!(diag.last_message_timestamp.is_none());

    // Simulate updating diagnostics (would be done by NatsMonitor)
    let mut updated_diag = ctx.diagnostics();
    updated_diag.messages_received = 50;
    updated_diag.messages_buffered = 50;
    updated_diag.last_message_timestamp = Some(chrono::Utc::now().to_rfc3339());

    ctx.update_diagnostics(updated_diag);

    let diag = ctx.diagnostics();
    assert_eq!(diag.messages_received, 50);
    assert_eq!(diag.messages_buffered, 50);
    assert!(diag.last_message_timestamp.is_some());

    // Test connection status changes
    ctx.set_connected(false);
    assert!(!ctx.is_connected());

    ctx.set_connected(true);
    assert!(ctx.is_connected());
}

/// Test 7: Message detail expansion
#[test]
fn test_message_detail_view() {
    let ctx = MonitorContext::new();

    // Initially detail view is closed
    assert!(!ctx.show_detail());

    // Disable auto-scroll to ensure predictable selection
    ctx.set_auto_scroll(false);

    // Add messages and select one
    ctx.add_message(create_test_message(
        "mcp.orchestrator.request",
        r#"{"request_id": "req-1", "tool": "test", "params": {"key": "value"}}"#,
    ));
    ctx.add_message(create_test_message(
        "mcp.story-generator.request",
        r#"{"request_id": "req-2"}"#,
    ));

    // Get selected message (default is first)
    let selected = ctx.selected_message();
    assert!(selected.is_some());
    let msg = selected.unwrap();
    assert_eq!(msg.endpoint, "orchestrator");
    assert_eq!(msg.request_id, Some("req-1".to_string()));

    // Open detail view
    ctx.set_show_detail(true);
    assert!(ctx.show_detail());

    // Toggle detail view
    ctx.toggle_detail();
    assert!(!ctx.show_detail());
}

/// Test 8: Auto-scroll behavior
#[test]
fn test_auto_scroll_behavior() {
    let ctx = MonitorContext::new();

    // Auto-scroll enabled by default
    assert!(ctx.auto_scroll());

    // Add messages - selection should jump to latest
    ctx.add_message(create_test_message("mcp.test.request", r#"{"id": "1"}"#));
    assert_eq!(ctx.selected_index(), 0);

    ctx.add_message(create_test_message("mcp.test.request", r#"{"id": "2"}"#));
    assert_eq!(ctx.selected_index(), 1); // Jumped to latest

    ctx.add_message(create_test_message("mcp.test.request", r#"{"id": "3"}"#));
    assert_eq!(ctx.selected_index(), 2); // Jumped to latest

    // Manual navigation should disable auto-scroll
    ctx.previous_message();
    assert!(!ctx.auto_scroll()); // Auto-scroll disabled
    assert_eq!(ctx.selected_index(), 1);

    // Adding message shouldn't change selection now
    ctx.add_message(create_test_message("mcp.test.request", r#"{"id": "4"}"#));
    assert_eq!(ctx.selected_index(), 1); // Stayed at same position

    // Re-enable auto-scroll
    ctx.toggle_auto_scroll();
    assert!(ctx.auto_scroll());

    // Next message should jump to latest
    ctx.add_message(create_test_message("mcp.test.request", r#"{"id": "5"}"#));
    assert_eq!(ctx.selected_index(), 4); // Jumped to latest (index 4)
}
