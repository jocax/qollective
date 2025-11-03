/// Integration tests for MCP Testing Interface
///
/// Tests covering the full workflow from template selection to request/response handling

#[cfg(test)]
mod integration_tests {
    use crate::components::text_editor::TextEditorState;
    use crate::models::history::{HistoryQuery, HistoryStatus, McpHistoryEntry};
    use crate::models::mcp::{McpServerConfig, TemplateInfo};
    use crate::state::{McpContext, McpTab, ResponseMetadata};
    use crate::utils::json_validator;
    use rmcp::model::{CallToolRequest, CallToolResult, CallToolRequestParam};

    /// Test 1: Server selection workflow
    #[test]
    fn test_server_selection_workflow() {
        let ctx = McpContext::new();

        // Verify default servers are loaded
        let servers = ctx.servers();
        assert_eq!(servers.len(), 5, "Should have 5 default MCP servers");

        // Verify server names
        let server_names: Vec<String> = servers.iter().map(|s| s.name.clone()).collect();
        assert!(server_names.contains(&"orchestrator".to_string()));
        assert!(server_names.contains(&"story-generator".to_string()));
        assert!(server_names.contains(&"quality-control".to_string()));
        assert!(server_names.contains(&"constraint-enforcer".to_string()));
        assert!(server_names.contains(&"prompt-helper".to_string()));

        // Test server navigation
        assert_eq!(ctx.selected_server_index(), 0);

        ctx.next_server();
        assert_eq!(ctx.selected_server_index(), 1);

        ctx.previous_server();
        assert_eq!(ctx.selected_server_index(), 0);

        // Test getting selected server
        let selected = ctx.selected_server();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "orchestrator");

        // Test wrap-around navigation
        ctx.set_selected_server_index(4);
        ctx.next_server();
        assert_eq!(ctx.selected_server_index(), 0);
    }

    /// Test 2: Template loading and filtering workflow
    #[test]
    fn test_template_loading_and_filtering() {
        let ctx = McpContext::new();

        // Create test templates
        let templates = vec![
            TemplateInfo {
                server_name: "orchestrator".to_string(),
                template_name: "start_workflow".to_string(),
                file_path: "/path/to/start_workflow.json".to_string(),
                description: Some("Start a new workflow".to_string()),
                tool_name: "start_workflow".to_string(),
            },
            TemplateInfo {
                server_name: "story-generator".to_string(),
                template_name: "generate_scene".to_string(),
                file_path: "/path/to/generate_scene.json".to_string(),
                description: Some("Generate a story scene".to_string()),
                tool_name: "generate_scene".to_string(),
            },
            TemplateInfo {
                server_name: "quality-control".to_string(),
                template_name: "validate_content".to_string(),
                file_path: "/path/to/validate_content.json".to_string(),
                description: Some("Validate generated content".to_string()),
                tool_name: "validate_content".to_string(),
            },
        ];

        ctx.set_templates(templates.clone());

        // Test all templates loaded
        assert_eq!(ctx.templates().len(), 3);

        // Test filtering by server name
        ctx.set_template_filter("story".to_string());
        let filtered = ctx.filtered_templates();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].server_name, "story-generator");

        // Test filtering by tool name
        ctx.set_template_filter("validate".to_string());
        let filtered = ctx.filtered_templates();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].tool_name, "validate_content");

        // Test case-insensitive filtering
        ctx.set_template_filter("WORKFLOW".to_string());
        let filtered = ctx.filtered_templates();
        assert_eq!(filtered.len(), 1);

        // Test clearing filter
        ctx.set_template_filter(String::new());
        let filtered = ctx.filtered_templates();
        assert_eq!(filtered.len(), 3);

        // Test template navigation
        assert_eq!(ctx.selected_template_index(), 0);
        ctx.next_template();
        assert_eq!(ctx.selected_template_index(), 1);
        ctx.previous_template();
        assert_eq!(ctx.selected_template_index(), 0);
    }

    /// Test 3: Request JSON editing and validation
    #[test]
    fn test_request_editing_and_validation() {
        let ctx = McpContext::new();

        // Test setting valid JSON
        let valid_json = r#"{
            "meta": {
                "tenant": "1",
                "request_id": "123"
            },
            "payload": {
                "tool_call": {
                    "params": {
                        "name": "test_tool",
                        "arguments": {}
                    }
                }
            }
        }"#;

        ctx.set_request_json(valid_json.to_string());
        assert_eq!(ctx.request_json(), valid_json);

        // Validate JSON
        let validation_result = json_validator::validate_json(&ctx.request_json());
        assert!(validation_result.is_ok());

        // Test setting invalid JSON
        let invalid_json = r#"{"invalid": json}"#;
        ctx.set_request_json(invalid_json.to_string());

        let validation_result = json_validator::validate_json(&ctx.request_json());
        assert!(validation_result.is_err());

        // Test error tracking
        ctx.set_json_error(Some("Invalid JSON syntax".to_string()));
        assert_eq!(
            ctx.json_error(),
            Some("Invalid JSON syntax".to_string())
        );

        ctx.set_json_error(None);
        assert_eq!(ctx.json_error(), None);
    }

    /// Test 4: Response display workflow
    #[test]
    fn test_response_display_workflow() {
        let ctx = McpContext::new();

        // Initially no response
        assert_eq!(ctx.response(), None);
        assert_eq!(ctx.response_metadata(), None);

        // Simulate request in progress
        ctx.set_request_in_progress(true);
        assert!(ctx.request_in_progress());

        // Set response
        let response_json = r#"{
            "meta": {
                "request_id": "123",
                "timestamp": "2025-11-02T12:00:00Z"
            },
            "payload": {
                "tool_response": {
                    "content": [
                        {"type": "text", "text": "Success response"}
                    ],
                    "is_error": false
                }
            }
        }"#;

        ctx.set_response(Some(response_json.to_string()));
        assert!(ctx.response().is_some());

        // Set metadata
        let metadata = ResponseMetadata {
            status: "success".to_string(),
            duration_ms: 250,
            timestamp: "2025-11-02T12:00:00Z".to_string(),
        };

        ctx.set_response_metadata(Some(metadata.clone()));
        let loaded_meta = ctx.response_metadata();
        assert!(loaded_meta.is_some());
        assert_eq!(loaded_meta.unwrap().duration_ms, 250);

        // Clear request in progress
        ctx.set_request_in_progress(false);
        assert!(!ctx.request_in_progress());
    }

    /// Test 5: Request history persistence and pagination
    #[test]
    fn test_request_history_persistence() {
        let ctx = McpContext::new();

        // Create test history entries
        fn create_entry(
            server: &str,
            tool: &str,
            status: HistoryStatus,
        ) -> McpHistoryEntry {
            let params = CallToolRequestParam {
                name: tool.to_string().into(),
                arguments: None,
            };

            let request = CallToolRequest::new(params);

            let response = if status == HistoryStatus::Error {
                CallToolResult::error(vec![])
            } else {
                CallToolResult::success(vec![])
            };

            McpHistoryEntry::new(
                server.to_string(),
                tool.to_string(),
                1,
                status,
                100,
                request,
                response,
            )
        }

        // Add multiple entries
        for i in 0..25 {
            ctx.add_history_entry(create_entry(
                "orchestrator",
                &format!("tool_{}", i),
                HistoryStatus::Success,
            ));
        }

        // Test pagination
        let query = HistoryQuery::default();
        ctx.set_history_query(query);

        let page = ctx.paginated_history();
        assert_eq!(page.entries.len(), 20); // Default page size
        assert_eq!(page.total_count, 25);
        assert_eq!(page.total_pages, 2);
        assert!(page.has_next());
        assert!(!page.has_previous());

        // Get second page
        let query = HistoryQuery::default().with_page(1);
        ctx.set_history_query(query);

        let page = ctx.paginated_history();
        assert_eq!(page.entries.len(), 5);
        assert!(page.has_previous());
        assert!(!page.has_next());

        // Test filtering
        ctx.add_history_entry(create_entry(
            "story-generator",
            "generate_scene",
            HistoryStatus::Error,
        ));

        let query = HistoryQuery::default()
            .with_server("story-generator".to_string());
        ctx.set_history_query(query);

        let page = ctx.paginated_history();
        assert_eq!(page.total_count, 1);

        // Test deletion
        let initial_count = ctx.history().len();
        let first_entry_id = ctx.history()[0].id.clone();

        ctx.delete_history_entry(&first_entry_id);
        assert_eq!(ctx.history().len(), initial_count - 1);

        // Test clear all
        ctx.clear_history();
        assert_eq!(ctx.history().len(), 0);
    }

    /// Test 6: Tab navigation and auto-switching
    #[test]
    fn test_tab_navigation_and_auto_switching() {
        let ctx = McpContext::new();

        // Default tab is Templates
        assert_eq!(ctx.active_tab(), McpTab::Templates);

        // Navigate through tabs
        ctx.next_tab();
        assert_eq!(ctx.active_tab(), McpTab::Editor);

        ctx.next_tab();
        assert_eq!(ctx.active_tab(), McpTab::Response);

        ctx.next_tab();
        assert_eq!(ctx.active_tab(), McpTab::History);

        // Wrap around
        ctx.next_tab();
        assert_eq!(ctx.active_tab(), McpTab::Templates);

        // Navigate backwards
        ctx.previous_tab();
        assert_eq!(ctx.active_tab(), McpTab::History);

        // Direct tab selection
        ctx.set_active_tab(McpTab::Editor);
        assert_eq!(ctx.active_tab(), McpTab::Editor);

        // Test tab from number
        let tab = McpTab::from_number(3);
        assert_eq!(tab, Some(McpTab::Response));

        ctx.set_active_tab(tab.unwrap());
        assert_eq!(ctx.active_tab(), McpTab::Response);
    }

    /// Test 7: End-to-end template selection to request preparation
    #[test]
    fn test_end_to_end_template_to_request() {
        let ctx = McpContext::new();

        // 1. Load templates
        let templates = vec![TemplateInfo {
            server_name: "orchestrator".to_string(),
            template_name: "start_workflow".to_string(),
            file_path: "/path/to/start_workflow.json".to_string(),
            description: Some("Start workflow".to_string()),
            tool_name: "start_workflow".to_string(),
        }];

        ctx.set_templates(templates);

        // 2. Select template (simulate user selecting first template)
        let selected_template = ctx.selected_template();
        assert!(selected_template.is_some());

        let template = selected_template.unwrap();
        assert_eq!(template.tool_name, "start_workflow");

        // 3. Load template JSON into editor (simulated)
        let template_json = r#"{
            "meta": {
                "tenant": "1"
            },
            "payload": {
                "tool_call": {
                    "params": {
                        "name": "start_workflow",
                        "arguments": {
                            "workflow_id": "test-123"
                        }
                    }
                }
            }
        }"#;

        ctx.set_request_json(template_json.to_string());

        // 4. Validate JSON
        let validation = json_validator::validate_json(&ctx.request_json());
        assert!(validation.is_ok());

        // 5. Auto-switch to Editor tab
        ctx.set_active_tab(McpTab::Editor);
        assert_eq!(ctx.active_tab(), McpTab::Editor);

        // 6. Prepare to send (simulate)
        ctx.set_request_in_progress(true);

        // 7. Auto-switch to Response tab after sending
        ctx.set_active_tab(McpTab::Response);
        assert_eq!(ctx.active_tab(), McpTab::Response);

        // 8. Receive response (simulate)
        let response = r#"{"status": "success"}"#;
        ctx.set_response(Some(response.to_string()));
        ctx.set_request_in_progress(false);

        // Verify final state
        assert!(ctx.response().is_some());
        assert!(!ctx.request_in_progress());
    }

    /// Test 8: History replay workflow
    #[test]
    fn test_history_replay_workflow() {
        let ctx = McpContext::new();

        // Create and add a history entry
        let mut args_map = serde_json::Map::new();
        args_map.insert("param".to_string(), serde_json::json!("value"));

        let params = CallToolRequestParam {
            name: "test_tool".to_string().into(),
            arguments: Some(args_map),
        };

        let request = CallToolRequest::new(params);
        let response = CallToolResult::success(vec![]);

        let entry = McpHistoryEntry::new(
            "orchestrator".to_string(),
            "test_tool".to_string(),
            1,
            HistoryStatus::Success,
            150,
            request.clone(),
            response,
        );

        ctx.add_history_entry(entry.clone());

        // Verify entry is in history
        assert_eq!(ctx.history().len(), 1);

        // Simulate replay: extract request from history and load into editor
        let historical_request = &ctx.history()[0].request;
        assert_eq!(
            historical_request.params.name,
            request.params.name
        );

        // Convert request to JSON for editor
        let request_json = serde_json::to_string_pretty(&historical_request).unwrap();
        ctx.set_request_json(request_json);

        // Auto-switch to editor
        ctx.set_active_tab(McpTab::Editor);
        assert_eq!(ctx.active_tab(), McpTab::Editor);

        // Verify JSON is valid
        let validation = json_validator::validate_json(&ctx.request_json());
        assert!(validation.is_ok());
    }
}
