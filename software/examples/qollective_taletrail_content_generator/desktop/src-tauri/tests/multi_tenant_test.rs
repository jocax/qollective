/// Multi-tenant integration tests for TaleTrail Desktop Viewer
///
/// Tests verify:
/// - Tenant ID extraction from trail metadata
/// - Tenant filtering in load_trails_from_directory
/// - Data isolation between tenants for bookmarks
/// - Data isolation between tenants for settings
/// - Empty tenant handling (trails without tenant_id)

use std::fs;
use std::path::{Path, PathBuf};

// Import types from the main crate
use nuxtor_lib::{
    models::{TrailListItem, Bookmark},
    utils::{FileLoader, get_bookmark_key, get_preferences_key},
};

/// Helper to create test trail files with different tenants
fn create_test_trail_file(dir: &Path, filename: &str, tenant_id: &str, title: &str) -> PathBuf {
    // Build the inner JSON (generation response)
    let inner_json = serde_json::json!({
        "status": "completed",
        "trail": {
            "title": title,
            "description": "Test trail",
            "metadata": {
                "generation_params": {
                    "age_group": "8-10",
                    "theme": "Adventure",
                    "language": "en",
                    "node_count": 5
                },
                "start_node_id": "node1"
            },
            "dag": {
                "nodes": {},
                "edges": []
            }
        }
    });

    // Build the envelope
    let envelope = serde_json::json!({
        "meta": {
            "request_id": format!("test-uuid-{}", filename),
            "timestamp": "2025-10-22T12:00:00Z",
            "tenant": tenant_id,
            "version": "1.0"
        },
        "payload": {
            "tool_response": {
                "content": [{
                    "type": "text",
                    "text": inner_json.to_string()
                }],
                "isError": false
            }
        }
    });

    let file_path = dir.join(filename);
    fs::write(&file_path, serde_json::to_string_pretty(&envelope).unwrap()).unwrap();
    file_path
}

/// Helper to create test trail file without tenant
fn create_test_trail_file_no_tenant(dir: &Path, filename: &str, title: &str) -> PathBuf {
    // Build the inner JSON (generation response)
    let inner_json = serde_json::json!({
        "status": "completed",
        "trail": {
            "title": title,
            "description": "Test trail",
            "metadata": {
                "generation_params": {
                    "age_group": "8-10",
                    "theme": "Adventure",
                    "language": "en",
                    "node_count": 5
                },
                "start_node_id": "node1"
            },
            "dag": {
                "nodes": {},
                "edges": []
            }
        }
    });

    // Build the envelope with empty tenant
    let envelope = serde_json::json!({
        "meta": {
            "request_id": format!("test-uuid-{}", filename),
            "timestamp": "2025-10-22T12:00:00Z",
            "tenant": "",
            "version": "1.0"
        },
        "payload": {
            "tool_response": {
                "content": [{
                    "type": "text",
                    "text": inner_json.to_string()
                }],
                "isError": false
            }
        }
    });

    let file_path = dir.join(filename);
    fs::write(&file_path, serde_json::to_string_pretty(&envelope).unwrap()).unwrap();
    file_path
}

#[test]
fn test_tenant_id_extraction_from_trail_metadata() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create trail files with different tenants
    create_test_trail_file(temp_dir.path(), "response_tenant1.json", "tenant-123", "Trail for Tenant 123");
    create_test_trail_file(temp_dir.path(), "response_tenant2.json", "tenant-456", "Trail for Tenant 456");
    create_test_trail_file_no_tenant(temp_dir.path(), "response_no_tenant.json", "Trail without Tenant");

    // Load trails
    let trails = FileLoader::load_trails_from_directory(temp_dir.path().to_str().unwrap())
        .expect("Failed to load trails");

    assert_eq!(trails.len(), 3, "Should load all three trail files");

    // Check tenant_id extraction
    let tenant1_trail = trails.iter().find(|t| t.title == "Trail for Tenant 123").unwrap();
    assert_eq!(tenant1_trail.tenant_id, Some("tenant-123".to_string()));

    let tenant2_trail = trails.iter().find(|t| t.title == "Trail for Tenant 456").unwrap();
    assert_eq!(tenant2_trail.tenant_id, Some("tenant-456".to_string()));

    let no_tenant_trail = trails.iter().find(|t| t.title == "Trail without Tenant").unwrap();
    assert_eq!(no_tenant_trail.tenant_id, None, "Empty tenant string should result in None");
}

#[test]
fn test_tenant_filtering_in_load_trails() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create multiple trails for different tenants
    create_test_trail_file(temp_dir.path(), "response_a1.json", "tenant-A", "Trail A1");
    create_test_trail_file(temp_dir.path(), "response_a2.json", "tenant-A", "Trail A2");
    create_test_trail_file(temp_dir.path(), "response_b1.json", "tenant-B", "Trail B1");
    create_test_trail_file_no_tenant(temp_dir.path(), "response_none.json", "Trail No Tenant");

    let trails = FileLoader::load_trails_from_directory(temp_dir.path().to_str().unwrap())
        .expect("Failed to load trails");

    // Verify all trails loaded
    assert_eq!(trails.len(), 4);

    // Filter for tenant-A
    let tenant_a_trails: Vec<&TrailListItem> = trails
        .iter()
        .filter(|t| t.tenant_id.as_deref() == Some("tenant-A"))
        .collect();
    assert_eq!(tenant_a_trails.len(), 2);
    assert!(tenant_a_trails.iter().any(|t| t.title == "Trail A1"));
    assert!(tenant_a_trails.iter().any(|t| t.title == "Trail A2"));

    // Filter for tenant-B
    let tenant_b_trails: Vec<&TrailListItem> = trails
        .iter()
        .filter(|t| t.tenant_id.as_deref() == Some("tenant-B"))
        .collect();
    assert_eq!(tenant_b_trails.len(), 1);
    assert_eq!(tenant_b_trails[0].title, "Trail B1");

    // Filter for no tenant
    let no_tenant_trails: Vec<&TrailListItem> = trails
        .iter()
        .filter(|t| t.tenant_id.is_none())
        .collect();
    assert_eq!(no_tenant_trails.len(), 1);
    assert_eq!(no_tenant_trails[0].title, "Trail No Tenant");
}

#[test]
fn test_bookmark_key_generation() {
    // Test bookmark key generation for tenant isolation
    assert_eq!(get_bookmark_key(None), "bookmarks");
    assert_eq!(get_bookmark_key(Some("")), "bookmarks");
    assert_eq!(get_bookmark_key(Some("tenant-123")), "bookmarks.tenant-123");
    assert_eq!(get_bookmark_key(Some("tenant-456")), "bookmarks.tenant-456");
}

#[test]
fn test_preferences_key_generation() {
    // Test preferences key generation for tenant isolation
    assert_eq!(get_preferences_key(None), "preferences");
    assert_eq!(get_preferences_key(Some("")), "preferences");
    assert_eq!(get_preferences_key(Some("tenant-123")), "preferences.tenant-123");
    assert_eq!(get_preferences_key(Some("tenant-456")), "preferences.tenant-456");
}

#[test]
fn test_bookmark_tenant_isolation_concept() {
    // This test demonstrates the concept of bookmark isolation
    // Actual Tauri store testing requires runtime environment

    let bookmark_tenant1 = Bookmark {
        trail_id: "trail-1".to_string(),
        trail_title: "Trail 1".to_string(),
        file_path: "/path/to/trail1.json".to_string(),
        timestamp: "2025-10-22T12:00:00Z".to_string(),
        user_note: "Great trail!".to_string(),
        tenant_id: Some("tenant-123".to_string()),
    };

    let bookmark_tenant2 = Bookmark {
        trail_id: "trail-2".to_string(),
        trail_title: "Trail 2".to_string(),
        file_path: "/path/to/trail2.json".to_string(),
        timestamp: "2025-10-22T12:00:00Z".to_string(),
        user_note: "Another trail!".to_string(),
        tenant_id: Some("tenant-456".to_string()),
    };

    // Verify that bookmarks have different tenant_ids
    assert_ne!(bookmark_tenant1.tenant_id, bookmark_tenant2.tenant_id);

    // Verify storage keys would be different
    let key1 = get_bookmark_key(bookmark_tenant1.tenant_id.as_deref());
    let key2 = get_bookmark_key(bookmark_tenant2.tenant_id.as_deref());
    assert_ne!(key1, key2);
    assert_eq!(key1, "bookmarks.tenant-123");
    assert_eq!(key2, "bookmarks.tenant-456");
}

#[test]
fn test_empty_tenant_handling() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create trails with empty tenant strings
    create_test_trail_file_no_tenant(temp_dir.path(), "response_empty1.json", "Trail Empty 1");
    create_test_trail_file_no_tenant(temp_dir.path(), "response_empty2.json", "Trail Empty 2");

    let trails = FileLoader::load_trails_from_directory(temp_dir.path().to_str().unwrap())
        .expect("Failed to load trails");

    assert_eq!(trails.len(), 2);

    // All should have None as tenant_id
    for trail in &trails {
        assert_eq!(trail.tenant_id, None, "Empty tenant string should be None");
    }
}

#[test]
fn test_mixed_tenant_and_no_tenant_trails() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create a mix of trails
    create_test_trail_file(temp_dir.path(), "response_with_tenant.json", "tenant-789", "Trail With Tenant");
    create_test_trail_file_no_tenant(temp_dir.path(), "response_without_tenant.json", "Trail Without Tenant");

    let trails = FileLoader::load_trails_from_directory(temp_dir.path().to_str().unwrap())
        .expect("Failed to load trails");

    assert_eq!(trails.len(), 2);

    // Check that one has tenant_id and one doesn't
    let with_tenant = trails.iter().find(|t| t.title == "Trail With Tenant").unwrap();
    let without_tenant = trails.iter().find(|t| t.title == "Trail Without Tenant").unwrap();

    assert_eq!(with_tenant.tenant_id, Some("tenant-789".to_string()));
    assert_eq!(without_tenant.tenant_id, None);
}

#[test]
fn test_backward_compatibility_with_existing_trails() {
    // Verify that trails without tenant_id field still load correctly
    let temp_dir = tempfile::tempdir().unwrap();

    // Create a trail file simulating old format (empty tenant)
    create_test_trail_file_no_tenant(temp_dir.path(), "response_legacy.json", "Legacy Trail");

    let trails = FileLoader::load_trails_from_directory(temp_dir.path().to_str().unwrap())
        .expect("Failed to load legacy trails");

    assert_eq!(trails.len(), 1);
    assert_eq!(trails[0].title, "Legacy Trail");
    assert_eq!(trails[0].tenant_id, None, "Legacy trails should have None tenant_id");
}

#[test]
fn test_storage_key_uniqueness() {
    // Verify that different tenants get different storage keys
    let tenants = vec!["tenant-1", "tenant-2", "tenant-3", "org-alpha", "org-beta"];

    let bookmark_keys: Vec<String> = tenants
        .iter()
        .map(|t| get_bookmark_key(Some(t)))
        .collect();

    let preferences_keys: Vec<String> = tenants
        .iter()
        .map(|t| get_preferences_key(Some(t)))
        .collect();

    // Check all bookmark keys are unique
    for i in 0..bookmark_keys.len() {
        for j in (i + 1)..bookmark_keys.len() {
            assert_ne!(bookmark_keys[i], bookmark_keys[j], "Bookmark keys must be unique");
        }
    }

    // Check all preferences keys are unique
    for i in 0..preferences_keys.len() {
        for j in (i + 1)..preferences_keys.len() {
            assert_ne!(preferences_keys[i], preferences_keys[j], "Preferences keys must be unique");
        }
    }
}

#[test]
fn test_global_storage_keys() {
    // Verify global (non-tenant) storage keys
    let global_bookmark_key = get_bookmark_key(None);
    let global_preferences_key = get_preferences_key(None);

    assert_eq!(global_bookmark_key, "bookmarks");
    assert_eq!(global_preferences_key, "preferences");

    // Verify these don't conflict with tenant-scoped keys
    let tenant_bookmark_key = get_bookmark_key(Some("bookmarks")); // Edge case: tenant named "bookmarks"
    let tenant_preferences_key = get_preferences_key(Some("preferences")); // Edge case: tenant named "preferences"

    assert_ne!(global_bookmark_key, tenant_bookmark_key);
    assert_ne!(global_preferences_key, tenant_preferences_key);
}
