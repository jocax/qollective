use nuxtor_lib::models::{Bookmark, UserPreferences};

#[cfg(test)]
mod bookmark_tests {
    use super::*;

    fn create_test_bookmark() -> Bookmark {
        Bookmark {
            trail_id: "test-trail-123".to_string(),
            trail_title: "Test Ocean Adventure".to_string(),
            file_path: "/path/to/trail.json".to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            user_note: "Test bookmark note".to_string(),
            tenant_id: None,
        }
    }

    #[test]
    fn test_bookmark_creation() {
        let bookmark = create_test_bookmark();
        assert_eq!(bookmark.trail_id, "test-trail-123");
        assert_eq!(bookmark.trail_title, "Test Ocean Adventure");
        assert_eq!(bookmark.file_path, "/path/to/trail.json");
        assert!(!bookmark.timestamp.is_empty());
    }

    #[test]
    fn test_bookmark_serialization() {
        let bookmark = create_test_bookmark();
        let json = serde_json::to_string(&bookmark).expect("Failed to serialize");
        assert!(json.contains("test-trail-123"));
        assert!(json.contains("Test Ocean Adventure"));
    }

    #[test]
    fn test_bookmark_deserialization() {
        let json = r#"{
            "trail_id": "test-trail-123",
            "trail_title": "Test Ocean Adventure",
            "file_path": "/path/to/trail.json",
            "timestamp": "2025-01-01T00:00:00Z",
            "user_note": "Test note"
        }"#;

        let bookmark: Bookmark = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(bookmark.trail_id, "test-trail-123");
        assert_eq!(bookmark.trail_title, "Test Ocean Adventure");
        assert_eq!(bookmark.file_path, "/path/to/trail.json");
    }

    #[test]
    fn test_bookmarks_vec_operations() {
        let mut bookmarks = Vec::new();
        let bookmark1 = create_test_bookmark();

        // Test adding bookmark
        bookmarks.push(bookmark1.clone());
        assert_eq!(bookmarks.len(), 1);

        // Test duplicate detection
        let exists = bookmarks.iter().any(|b| b.trail_id == "test-trail-123");
        assert!(exists);

        // Test removing bookmark
        bookmarks.retain(|b| b.trail_id != "test-trail-123");
        assert_eq!(bookmarks.len(), 0);
    }

    #[test]
    fn test_multiple_bookmarks() {
        let mut bookmarks = Vec::new();

        for i in 0..5 {
            bookmarks.push(Bookmark {
                trail_id: format!("trail-{}", i),
                trail_title: format!("Trail {}", i),
                file_path: format!("/path/trail-{}.json", i),
                timestamp: "2025-01-01T00:00:00Z".to_string(),
                user_note: String::new(),
                tenant_id: None,
            });
        }

        assert_eq!(bookmarks.len(), 5);

        // Test finding specific bookmark
        let found = bookmarks.iter().find(|b| b.trail_id == "trail-2");
        assert!(found.is_some());
        assert_eq!(found.unwrap().trail_title, "Trail 2");
    }

    #[test]
    fn test_user_preferences_default() {
        let prefs = UserPreferences::default();
        assert!(prefs.auto_validate);
        assert_eq!(prefs.directory_path, "");
    }
}
