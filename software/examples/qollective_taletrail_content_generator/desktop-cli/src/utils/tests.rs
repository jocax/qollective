/// Integration tests for utility functions
///
/// Tests cover:
/// - Trail file loading
/// - Template loading
/// - JSON validation
/// - Directory scanning
/// - Bookmark management

use super::*;
use crate::models::TemplateInfo;
use smol::block_on;
use std::fs;
use tempfile::TempDir;

/// Test trail file loading from directory
#[test]
fn test_load_trails_from_directory() {
    block_on(async {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Create a valid trail file
        let valid_content = r#"{
            "meta": {
                "request_id": "test-uuid-123",
                "timestamp": "2025-10-22T12:00:00Z",
                "tenant": "1",
                "version": "1.0"
            },
            "payload": {
                "tool_response": {
                    "content": [{
                        "type": "text",
                        "text": "{\"generation_response\":{\"request_id\":\"test-uuid-123\",\"progress_percentage\":100,\"status\":\"completed\",\"trail\":{\"title\":\"Test Trail\",\"description\":\"A test trail\",\"is_public\":false,\"status\":\"DRAFT\",\"tags\":[],\"metadata\":{\"generation_params\":{\"age_group\":\"8-10\",\"theme\":\"Adventure\",\"language\":\"en\",\"node_count\":5},\"start_node_id\":\"node1\"}}}}"
                    }],
                    "isError": false
                }
            }
        }"#;

        fs::write(
            temp_dir.path().join("response_test.json"),
            valid_content,
        )
        .unwrap();

        // Test loading trails
        let result = load_trails_from_directory(temp_path).await;
        assert!(result.is_ok(), "Should load trails successfully");

        let trails = result.unwrap();
        assert_eq!(trails.len(), 1, "Should find one trail");
        assert_eq!(trails[0].title, "Test Trail");
        assert_eq!(trails[0].theme, "Adventure");
    });
}

/// Test template loading and parsing
#[test]
fn test_load_templates() {
    block_on(async {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create orchestrator directory and template
        let orchestrator_dir = base_path.join("orchestrator");
        fs::create_dir(&orchestrator_dir).unwrap();

        let template_content = r#"{
            "subject": "mcp.orchestrator.request",
            "envelope": {
                "meta": {
                    "request_id": "550e8400-e29b-41d4-a716-446655440000",
                    "tenant": "1",
                    "tracing": {
                        "trace_id": "test-trace",
                        "operation_name": "orchestrate_generation"
                    }
                },
                "payload": {
                    "tool_call": {
                        "method": "tools/call",
                        "params": {
                            "name": "orchestrate_generation",
                            "arguments": {
                                "generation_request": {
                                    "theme": "Test Theme",
                                    "age_group": "9-11",
                                    "language": "en"
                                }
                            }
                        }
                    }
                }
            }
        }"#;

        fs::write(
            orchestrator_dir.join("test_template.json"),
            template_content,
        )
        .unwrap();

        // Test loading templates
        let base_str = base_path.to_string_lossy().to_string();
        let result = load_all_templates(&base_str).await;
        assert!(result.is_ok(), "Should load templates successfully");

        let templates = result.unwrap();
        assert_eq!(templates.len(), 1, "Should find one template");
        assert_eq!(templates[0].server_name, "orchestrator");
        assert_eq!(templates[0].tool_name, "orchestrate_generation");
    });
}

/// Test JSON validation with valid and invalid JSON
#[test]
fn test_json_validation() {
    // Valid JSON
    let valid_json = r#"{"name": "test", "value": 123}"#;
    let result = json_validator::validate_json(valid_json);
    assert!(result.is_ok(), "Should validate valid JSON");

    // Invalid JSON
    let invalid_json = r#"{"name": "test", invalid}"#;
    let result = json_validator::validate_json(invalid_json);
    assert!(result.is_err(), "Should reject invalid JSON");

    let error = result.unwrap_err();
    assert!(error.contains("line"), "Error should include line number");
    assert!(error.contains("column"), "Error should include column number");
}

/// Test directory scanning with filtering
#[test]
fn test_directory_scanning() {
    block_on(async {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(temp_path.join("test1.json"), "{}").unwrap();
        fs::write(temp_path.join("test2.txt"), "text").unwrap();
        fs::create_dir(temp_path.join("subdir")).unwrap();

        // Test scanning without filter
        let result = directory_manager::scan_directory(temp_path, None::<fn(&std::path::Path) -> bool>).await;
        assert!(result.is_ok(), "Should scan directory successfully");

        let paths = result.unwrap();
        assert_eq!(paths.len(), 3, "Should find 3 entries");

        // Test scanning with JSON filter
        let result = directory_manager::scan_directory(
            temp_path,
            Some(|p: &std::path::Path| directory_manager::filter_by_extension(p, "json")),
        )
        .await;
        assert!(result.is_ok(), "Should scan with filter");

        let json_files = result.unwrap();
        assert_eq!(json_files.len(), 1, "Should find 1 JSON file");
    });
}

/// Test bookmark management
#[test]
fn test_bookmark_management() {
    block_on(async {
        let temp_dir = TempDir::new().unwrap();
        let bookmark_file = temp_dir.path().join("bookmarks.json");

        // Test creating and saving bookmarks
        let mut bookmarks = BookmarkCollection::new();
        assert_eq!(bookmarks.count(), 0);

        bookmarks.add("trail-1");
        bookmarks.add("trail-2");
        assert_eq!(bookmarks.count(), 2);

        let result = save_bookmarks(&bookmark_file, &bookmarks).await;
        assert!(result.is_ok(), "Should save bookmarks successfully");

        // Test loading bookmarks
        let loaded = load_bookmarks(&bookmark_file).await.unwrap();
        assert_eq!(loaded.count(), 2);
        assert!(loaded.is_bookmarked("trail-1"));
        assert!(loaded.is_bookmarked("trail-2"));
    });
}

/// Test pagination functionality
#[test]
fn test_pagination() {
    let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Page 0, size 3
    let page = directory_manager::paginate(items.clone(), 0, 3);
    assert_eq!(page, vec![1, 2, 3]);

    // Page 1, size 3
    let page = directory_manager::paginate(items.clone(), 1, 3);
    assert_eq!(page, vec![4, 5, 6]);

    // Last page
    let page = directory_manager::paginate(items.clone(), 3, 3);
    assert_eq!(page, vec![10]);

    // Out of bounds
    let page = directory_manager::paginate(items.clone(), 10, 3);
    assert_eq!(page, Vec::<i32>::new());

    // Test total pages calculation
    let total = directory_manager::calculate_total_pages(10, 3);
    assert_eq!(total, 4);
}

/// Test template grouping by server
#[test]
fn test_template_grouping() {
    let templates = vec![
        TemplateInfo {
            server_name: "orchestrator".to_string(),
            template_name: "test1".to_string(),
            file_path: "/path/test1.json".to_string(),
            description: None,
            tool_name: "tool1".to_string(),
        },
        TemplateInfo {
            server_name: "orchestrator".to_string(),
            template_name: "test2".to_string(),
            file_path: "/path/test2.json".to_string(),
            description: None,
            tool_name: "tool2".to_string(),
        },
        TemplateInfo {
            server_name: "story-generator".to_string(),
            template_name: "story1".to_string(),
            file_path: "/path/story1.json".to_string(),
            description: None,
            tool_name: "tool3".to_string(),
        },
    ];

    let grouped = group_templates_by_server(templates);

    assert_eq!(grouped.len(), 2);
    assert_eq!(grouped.get("orchestrator").unwrap().len(), 2);
    assert_eq!(grouped.get("story-generator").unwrap().len(), 1);
}

/// Test JSON pretty printing and minification
#[test]
fn test_json_formatting() {
    let json = r#"{"name":"test","value":123}"#;

    // Test pretty printing
    let pretty = json_validator::pretty_print_json(json).unwrap();
    assert!(pretty.contains('\n'));
    assert!(pretty.contains("  "));

    // Test minification
    let minified_input = r#"{
        "name": "test",
        "value": 123
    }"#;
    let minified = json_validator::minify_json(minified_input).unwrap();
    assert!(!minified.contains('\n'));
    assert!(!minified.contains("  "));
}
