//! Strategic Integration Tests for Critical Workflows
//!
//! These tests cover end-to-end scenarios and edge cases not fully covered
//! by feature-specific tests. Focus is on integration points and critical paths.

#[cfg(test)]
mod strategic_integration_tests {
    use crate::config::Config;
    use crate::state::{SettingsContext, SettingsField, TrailContext, MonitorContext, McpContext};
    use crate::models::trail::TrailListItem;
    use crate::nats::monitoring::NatsMessage;
    use std::path::PathBuf;

    /// Test 1: Settings persistence cycle (load -> modify -> verify)
    #[test]
    fn test_settings_persistence_cycle() {
        let ctx = SettingsContext::new();

        // Get initial config
        let initial = ctx.get_editing_config();
        assert!(!initial.nats_url.is_empty());

        // Modify settings
        ctx.update_field(SettingsField::NatsUrl, "nats://modified:4222".to_string());
        ctx.update_field(SettingsField::NatsTimeout, "120".to_string());

        // Verify dirty flag
        assert!(ctx.is_dirty());

        // Get modified config
        let modified = ctx.get_editing_config();
        assert_eq!(modified.nats_url, "nats://modified:4222");
        assert_eq!(modified.nats_timeout, "120");

        // Validate all fields
        assert!(ctx.validate_all());
    }

    /// Test 2: Config validation with invalid values
    #[test]
    fn test_config_validation_errors() {
        let ctx = SettingsContext::new();

        // Invalid URL
        ctx.update_field(SettingsField::NatsUrl, "not-a-valid-url".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.nats_url.is_some());

        // Invalid timeout (too large)
        ctx.update_field(SettingsField::NatsTimeout, "500".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.nats_timeout.is_some());

        // Invalid timeout (non-numeric)
        ctx.update_field(SettingsField::NatsTimeout, "abc".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.nats_timeout.is_some());

        // Empty required field
        ctx.update_field(SettingsField::NatsUrl, "".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.nats_url.is_some());

        // Valid values should clear errors
        ctx.update_field(SettingsField::NatsUrl, "nats://localhost:4222".to_string());
        ctx.update_field(SettingsField::NatsTimeout, "30".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.nats_url.is_none());
        assert!(errors.nats_timeout.is_none());
    }

    /// Test 3: Trail viewer with empty dataset
    #[test]
    fn test_trail_viewer_empty_dataset() {
        let ctx = TrailContext::new();

        // Initially empty
        assert_eq!(ctx.trails().len(), 0);
        assert_eq!(ctx.filtered_trails().len(), 0);

        // Navigation should be safe with empty data
        ctx.next_trail();
        assert_eq!(ctx.selected_index(), 0);

        ctx.previous_trail();
        assert_eq!(ctx.selected_index(), 0);

        // Selected trail should be None
        assert!(ctx.selected_trail().is_none());

        // Filtering empty dataset should work
        ctx.set_search_query("test".to_string());
        assert_eq!(ctx.filtered_trails().len(), 0);

        // Bookmarking with no trails should be safe
        let bookmark_count_before = ctx.bookmarks().bookmarks.len();
        // Can't toggle bookmark on non-existent trail, but shouldn't crash
        assert_eq!(ctx.bookmarks().bookmarks.len(), bookmark_count_before);
    }

    /// Test 4: NATS monitoring with rapid message influx (stress test)
    #[test]
    fn test_nats_monitoring_rapid_message_influx() {
        let ctx = MonitorContext::new();

        // Simulate rapid message arrival (200 messages)
        for i in 0..200 {
            let msg = NatsMessage::new(
                format!("mcp.orchestrator.request.{}", i % 5),
                format!(r#"{{"request_id": "req-{:04}", "tool": "test"}}"#, i).as_bytes().to_vec(),
            );
            ctx.add_message(msg);
        }

        // Verify messages are buffered
        assert_eq!(ctx.message_count(), 200);

        // Add 900 more messages to test circular buffer limit
        for i in 200..1100 {
            let msg = NatsMessage::new(
                "mcp.story-generator.request".to_string(),
                format!(r#"{{"id": "msg-{:04}"}}"#, i).as_bytes().to_vec(),
            );
            ctx.add_message(msg);
        }

        // Should be capped at 1000
        assert_eq!(ctx.message_count(), 1000);

        // Filtered count should also respect buffer size
        assert_eq!(ctx.filtered_count(), 1000);

        // Apply filter - should work even with max buffer
        ctx.set_endpoint_filter(Some("story-generator".to_string()));
        let filtered_count = ctx.filtered_count();

        // Most messages should be from story-generator (900 out of 1000)
        assert!(filtered_count > 850);
    }

    /// Test 5: MCP tester with JSON validation edge cases
    #[test]
    fn test_mcp_json_validation_edge_cases() {
        let ctx = McpContext::new();

        // Empty JSON
        ctx.set_request_json("".to_string());
        assert_eq!(ctx.request_json(), "");

        // Whitespace only
        ctx.set_request_json("   \n\t  ".to_string());
        assert_eq!(ctx.request_json().trim(), "");

        // Valid empty object
        ctx.set_request_json("{}".to_string());
        assert_eq!(ctx.request_json(), "{}");

        // Valid empty array
        ctx.set_request_json("[]".to_string());
        assert_eq!(ctx.request_json(), "[]");

        // Nested complex structure
        let complex_json = r#"{
            "meta": {
                "tenant": "1",
                "request_id": "test-123",
                "trace": {
                    "parent_id": "parent-456",
                    "span_id": "span-789"
                }
            },
            "payload": {
                "tool_call": {
                    "params": {
                        "name": "complex_tool",
                        "arguments": {
                            "nested": {
                                "deeply": {
                                    "value": 42
                                }
                            },
                            "array": [1, 2, 3, 4, 5]
                        }
                    }
                }
            }
        }"#;

        ctx.set_request_json(complex_json.to_string());
        assert!(ctx.request_json().contains("complex_tool"));

        // Malformed JSON (missing closing brace)
        ctx.set_request_json(r#"{"key": "value""#.to_string());
        let json = ctx.request_json();
        assert!(json.contains("key"));
    }

    /// Test 6: Trail filtering with multiple criteria and edge cases
    #[test]
    fn test_trail_filtering_comprehensive() {
        let ctx = TrailContext::new();

        // Create diverse trail dataset
        let trails = vec![
            TrailListItem {
                id: "1".to_string(),
                file_path: "/trails/1.json".to_string(),
                title: "Dragon Quest".to_string(),
                description: "A brave knight faces a dragon".to_string(),
                theme: "Fantasy".to_string(),
                age_group: "6-8".to_string(),
                language: "en".to_string(),
                tags: vec!["dragon".to_string(), "adventure".to_string()],
                status: "Completed".to_string(),
                generated_at: "2025-11-01T10:00:00Z".to_string(),
                node_count: 15,
                tenant_id: Some("tenant1".to_string()),
            },
            TrailListItem {
                id: "2".to_string(),
                file_path: "/trails/2.json".to_string(),
                title: "Space Adventure".to_string(),
                description: "Exploring the cosmos".to_string(),
                theme: "Science Fiction".to_string(),
                age_group: "9-11".to_string(),
                language: "en".to_string(),
                tags: vec!["space".to_string(), "adventure".to_string()],
                status: "Completed".to_string(),
                generated_at: "2025-11-01T11:00:00Z".to_string(),
                node_count: 20,
                tenant_id: Some("tenant1".to_string()),
            },
            TrailListItem {
                id: "3".to_string(),
                file_path: "/trails/3.json".to_string(),
                title: "Drache Abenteuer".to_string(),
                description: "Ein mutiger Ritter kämpft gegen einen Drachen".to_string(),
                theme: "Fantasy".to_string(),
                age_group: "6-8".to_string(),
                language: "de".to_string(),
                tags: vec!["drache".to_string(), "abenteuer".to_string()],
                status: "InProgress".to_string(),
                generated_at: "2025-11-01T12:00:00Z".to_string(),
                node_count: 8,
                tenant_id: Some("tenant2".to_string()),
            },
            TrailListItem {
                id: "4".to_string(),
                file_path: "/trails/4.json".to_string(),
                title: "Mystery Manor".to_string(),
                description: "Solve the mystery of the haunted manor".to_string(),
                theme: "Mystery".to_string(),
                age_group: "12-14".to_string(),
                language: "en".to_string(),
                tags: vec!["mystery".to_string(), "puzzle".to_string()],
                status: "Failed".to_string(),
                generated_at: "2025-11-01T13:00:00Z".to_string(),
                node_count: 0,
                tenant_id: Some("tenant1".to_string()),
            },
        ];

        ctx.set_trails(trails.clone());
        assert_eq!(ctx.trails().len(), 4);

        // Test: No filters = all trails
        assert_eq!(ctx.filtered_trails().len(), 4);

        // Test: Age group only
        let mut filters = crate::state::trail_state::TrailFilters::default();
        filters.age_group = Some("6-8".to_string());
        ctx.set_filters(filters);
        assert_eq!(ctx.filtered_trails().len(), 2);

        // Test: Language only
        let mut filters = crate::state::trail_state::TrailFilters::default();
        filters.language = Some("de".to_string());
        ctx.set_filters(filters);
        assert_eq!(ctx.filtered_trails().len(), 1);

        // Test: Status only
        let mut filters = crate::state::trail_state::TrailFilters::default();
        filters.status = Some("Completed".to_string());
        ctx.set_filters(filters);
        assert_eq!(ctx.filtered_trails().len(), 2);

        // Test: Age + Language
        let mut filters = crate::state::trail_state::TrailFilters::default();
        filters.age_group = Some("6-8".to_string());
        filters.language = Some("en".to_string());
        ctx.set_filters(filters);
        assert_eq!(ctx.filtered_trails().len(), 1); // Only trail 1

        // Test: All filters + search
        let mut filters = crate::state::trail_state::TrailFilters::default();
        filters.age_group = Some("6-8".to_string());
        filters.language = Some("en".to_string());
        filters.status = Some("Completed".to_string());
        ctx.set_filters(filters);
        ctx.set_search_query("dragon".to_string());
        assert_eq!(ctx.filtered_trails().len(), 1); // Only trail 1

        // Test: Search with no results
        ctx.set_search_query("nonexistent_term_xyz".to_string());
        assert_eq!(ctx.filtered_trails().len(), 0);

        // Test: Case-insensitive search
        ctx.set_filters(crate::state::trail_state::TrailFilters::default());
        ctx.set_search_query("DRAGON".to_string());
        assert_eq!(ctx.filtered_trails().len(), 1); // Should find "Dragon Quest"

        // Test: Clear all filters
        ctx.set_filters(crate::state::trail_state::TrailFilters::default());
        ctx.set_search_query("".to_string());
        assert_eq!(ctx.filtered_trails().len(), 4);
    }

    /// Test 7: Cross-feature workflow - MCP request affects monitoring
    #[test]
    fn test_cross_feature_mcp_to_monitoring_workflow() {
        let mcp_ctx = McpContext::new();
        let monitor_ctx = MonitorContext::new();

        // Step 1: Prepare MCP request
        let request_json = r#"{
            "meta": {
                "tenant": "1",
                "request_id": "cross-test-123"
            },
            "payload": {
                "tool_call": {
                    "params": {
                        "name": "generate_story",
                        "arguments": {"theme": "space"}
                    }
                }
            }
        }"#;

        mcp_ctx.set_request_json(request_json.to_string());

        // Step 2: Simulate sending request (would publish to NATS)
        mcp_ctx.set_request_in_progress(true);

        // Step 3: Simulate NATS monitoring receiving the request message
        let nats_msg = NatsMessage::new(
            "mcp.story-generator.request".to_string(),
            request_json.as_bytes().to_vec(),
        );
        monitor_ctx.add_message(nats_msg);

        // Step 4: Verify message appears in monitoring
        assert_eq!(monitor_ctx.message_count(), 1);
        let selected = monitor_ctx.selected_message();
        assert!(selected.is_some());

        let msg = selected.unwrap();
        assert_eq!(msg.endpoint, "story-generator");
        assert_eq!(msg.request_id, Some("cross-test-123".to_string()));

        // Step 5: Simulate response
        let response_json = r#"{
            "meta": {
                "request_id": "cross-test-123"
            },
            "payload": {
                "tool_response": {
                    "content": [{"type": "text", "text": "Story generated"}],
                    "is_error": false
                }
            }
        }"#;

        // Step 6: Update MCP context with response
        mcp_ctx.set_response(Some(response_json.to_string()));
        mcp_ctx.set_request_in_progress(false);

        // Step 7: Add response to monitoring
        let response_msg = NatsMessage::new(
            "mcp.story-generator.response".to_string(),
            response_json.as_bytes().to_vec(),
        );
        monitor_ctx.add_message(response_msg);

        // Step 8: Filter monitoring to show request/response pair
        monitor_ctx.set_endpoint_filter(Some("story-generator".to_string()));
        assert_eq!(monitor_ctx.filtered_count(), 2);

        // Verify both request and response are in monitoring
        let messages = monitor_ctx.filtered_messages();
        assert_eq!(messages.len(), 2);
        assert!(messages.iter().any(|m| m.message_type == "Request"));
        assert!(messages.iter().any(|m| m.message_type == "Response"));
    }

    /// Test 8: Large dataset performance (1000+ trails)
    #[test]
    fn test_large_dataset_performance() {
        let ctx = TrailContext::new();

        // Create 1500 trails
        let mut trails = Vec::new();
        for i in 0..1500 {
            let age_group = match i % 5 {
                0 => "6-8",
                1 => "9-11",
                2 => "12-14",
                3 => "15-17",
                _ => "18+",
            };

            let language = if i % 2 == 0 { "en" } else { "de" };
            let status = match i % 3 {
                0 => "Completed",
                1 => "InProgress",
                _ => "Failed",
            };

            trails.push(TrailListItem {
                id: format!("trail-{:04}", i),
                file_path: format!("/trails/trail-{:04}.json", i),
                title: format!("Trail Number {}", i),
                description: format!("Description for trail {}", i),
                theme: "Generated".to_string(),
                age_group: age_group.to_string(),
                language: language.to_string(),
                tags: vec!["test".to_string()],
                status: status.to_string(),
                generated_at: "2025-11-02T00:00:00Z".to_string(),
                node_count: 10,
                tenant_id: Some("perf-test".to_string()),
            });
        }

        // Load all trails - should be fast
        let start = std::time::Instant::now();
        ctx.set_trails(trails);
        let load_duration = start.elapsed();

        assert_eq!(ctx.trails().len(), 1500);
        assert!(load_duration.as_millis() < 100, "Loading 1500 trails took too long: {:?}", load_duration);

        // Filtering should be fast
        let start = std::time::Instant::now();
        let mut filters = crate::state::trail_state::TrailFilters::default();
        filters.language = Some("en".to_string());
        filters.age_group = Some("9-11".to_string());
        ctx.set_filters(filters);
        let filter_duration = start.elapsed();

        let filtered = ctx.filtered_trails();
        assert!(filtered.len() > 0);
        assert!(filter_duration.as_millis() < 50, "Filtering took too long: {:?}", filter_duration);

        // Search should be fast (clear filters first to test search alone)
        ctx.clear_filters();
        let start = std::time::Instant::now();
        ctx.set_search_query("500".to_string());
        let search_duration = start.elapsed();

        let searched = ctx.filtered_trails();
        assert!(searched.len() > 0, "Search for '500' should find trails");
        assert!(search_duration.as_millis() < 50, "Search took too long: {:?}", search_duration);

        // Navigation should be instant
        let start = std::time::Instant::now();
        for _ in 0..100 {
            ctx.next_trail();
        }
        let nav_duration = start.elapsed();
        assert!(nav_duration.as_millis() < 10, "Navigation took too long: {:?}", nav_duration);
    }

    /// Test 9: Error recovery - handling corrupted state
    #[test]
    fn test_error_recovery_corrupted_state() {
        let ctx = McpContext::new();

        // Set invalid JSON
        ctx.set_request_json("{ invalid json".to_string());
        ctx.set_json_error(Some("Syntax error at line 1".to_string()));

        // User corrects JSON
        let valid_json = r#"{"valid": "json"}"#;
        ctx.set_request_json(valid_json.to_string());
        ctx.set_json_error(None);

        // Error should be cleared
        assert_eq!(ctx.json_error(), None);

        // Request should be sendable
        assert_eq!(ctx.request_json(), valid_json);

        // Simulate failed request
        ctx.set_request_in_progress(true);
        ctx.set_response(Some(r#"{"error": "Connection failed"}"#.to_string()));
        ctx.set_request_in_progress(false);

        // User can retry - state should be clean
        ctx.set_response(None);
        ctx.set_request_in_progress(true);

        // New response
        ctx.set_response(Some(r#"{"success": true}"#.to_string()));
        ctx.set_request_in_progress(false);

        assert!(ctx.response().is_some());
        assert!(ctx.response().unwrap().contains("success"));
    }

    /// Test 10: Settings validation with boundary values
    #[test]
    fn test_settings_boundary_values() {
        let ctx = SettingsContext::new();

        // Minimum valid timeout
        ctx.update_field(SettingsField::NatsTimeout, "1".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.nats_timeout.is_none());

        // Maximum valid timeout
        ctx.update_field(SettingsField::NatsTimeout, "300".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.nats_timeout.is_none());

        // Zero timeout (invalid)
        ctx.update_field(SettingsField::NatsTimeout, "0".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.nats_timeout.is_some());

        // Above maximum (invalid)
        ctx.update_field(SettingsField::NatsTimeout, "301".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.nats_timeout.is_some());

        // Empty paths should be invalid for required directories
        ctx.update_field(SettingsField::TrailsDir, "".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.trails_dir.is_some());

        // Valid path
        ctx.update_field(SettingsField::TrailsDir, "/valid/path".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.trails_dir.is_none());

        // Optional paths can be empty
        ctx.update_field(SettingsField::TlsCertPath, "".to_string());
        let errors = ctx.get_validation_errors();
        assert!(errors.tls_cert_path.is_none()); // Should be valid (optional)
    }
}
