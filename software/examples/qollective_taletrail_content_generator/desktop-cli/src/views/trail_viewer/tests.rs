/// Trail Viewer Tests
///
/// Focused tests for trail loading, filtering, search, and bookmarks

#[cfg(test)]
mod trail_viewer_tests {
    use crate::state::trail_state::{TrailContext, TrailFilters, TrailViewMode};
    use crate::models::trail::TrailListItem;
    use crate::models::preferences::{Bookmark, BookmarkCollection};
    use std::sync::{Arc, RwLock};

    /// Helper to create a test trail item
    fn create_test_trail(
        id: &str,
        title: &str,
        language: &str,
        age_group: &str,
        status: &str,
        tenant_id: Option<&str>,
    ) -> TrailListItem {
        TrailListItem {
            id: id.to_string(),
            file_path: format!("/test/path/{}.json", id),
            title: title.to_string(),
            description: format!("Description for {}", title),
            theme: "Adventure".to_string(),
            age_group: age_group.to_string(),
            language: language.to_string(),
            tags: vec!["test".to_string()],
            status: status.to_string(),
            generated_at: "2025-11-02T12:00:00Z".to_string(),
            node_count: 10,
            tenant_id: tenant_id.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_trail_context_creation() {
        let ctx = TrailContext::new();

        // Verify initial state
        assert_eq!(ctx.trails().len(), 0);
        assert_eq!(ctx.selected_index(), 0);
        assert_eq!(ctx.view_mode(), TrailViewMode::List);
    }

    #[test]
    fn test_trail_loading() {
        let ctx = TrailContext::new();

        // Create test trails
        let trails = vec![
            create_test_trail("1", "Trail 1", "en", "6-8", "Completed", Some("tenant1")),
            create_test_trail("2", "Trail 2", "de", "9-11", "Completed", Some("tenant2")),
            create_test_trail("3", "Trail 3", "en", "12-14", "InProgress", Some("tenant1")),
        ];

        ctx.set_trails(trails.clone());

        // Verify trails loaded
        assert_eq!(ctx.trails().len(), 3);
        assert_eq!(ctx.filtered_trails().len(), 3);
    }

    #[test]
    fn test_filter_by_language() {
        let ctx = TrailContext::new();

        let trails = vec![
            create_test_trail("1", "English Trail", "en", "6-8", "Completed", None),
            create_test_trail("2", "German Trail", "de", "9-11", "Completed", None),
            create_test_trail("3", "Another English", "en", "12-14", "Completed", None),
        ];

        ctx.set_trails(trails);

        // Filter by language
        let mut filters = TrailFilters::default();
        filters.language = Some("en".to_string());
        ctx.set_filters(filters);

        let filtered = ctx.filtered_trails();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|t| t.language == "en"));
    }

    #[test]
    fn test_filter_by_age_group() {
        let ctx = TrailContext::new();

        let trails = vec![
            create_test_trail("1", "Young Trail", "en", "6-8", "Completed", None),
            create_test_trail("2", "Teen Trail", "en", "12-14", "Completed", None),
            create_test_trail("3", "Another Young", "en", "6-8", "Completed", None),
        ];

        ctx.set_trails(trails);

        // Filter by age group
        let mut filters = TrailFilters::default();
        filters.age_group = Some("6-8".to_string());
        ctx.set_filters(filters);

        let filtered = ctx.filtered_trails();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|t| t.age_group == "6-8"));
    }

    #[test]
    fn test_filter_by_status() {
        let ctx = TrailContext::new();

        let trails = vec![
            create_test_trail("1", "Done Trail", "en", "6-8", "Completed", None),
            create_test_trail("2", "Active Trail", "en", "9-11", "InProgress", None),
            create_test_trail("3", "Another Done", "en", "12-14", "Completed", None),
        ];

        ctx.set_trails(trails);

        // Filter by status
        let mut filters = TrailFilters::default();
        filters.status = Some("Completed".to_string());
        ctx.set_filters(filters);

        let filtered = ctx.filtered_trails();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|t| t.status == "Completed"));
    }

    #[test]
    fn test_text_search() {
        let ctx = TrailContext::new();

        let trails = vec![
            create_test_trail("1", "Dragon Adventure", "en", "6-8", "Completed", None),
            create_test_trail("2", "Space Quest", "en", "9-11", "Completed", None),
            create_test_trail("3", "Dragon Quest", "en", "12-14", "Completed", None),
        ];

        ctx.set_trails(trails);

        // Search for "dragon"
        ctx.set_search_query("dragon".to_string());

        let filtered = ctx.filtered_trails();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|t| t.title.to_lowercase().contains("dragon")));
    }

    #[test]
    fn test_combined_filters() {
        let ctx = TrailContext::new();

        let trails = vec![
            create_test_trail("1", "English Dragon", "en", "6-8", "Completed", Some("tenant1")),
            create_test_trail("2", "German Dragon", "de", "6-8", "Completed", Some("tenant1")),
            create_test_trail("3", "English Space", "en", "6-8", "InProgress", Some("tenant1")),
            create_test_trail("4", "English Dragon Teen", "en", "12-14", "Completed", Some("tenant1")),
        ];

        ctx.set_trails(trails);

        // Apply multiple filters: language=en, age_group=6-8, status=Completed, search=dragon
        let mut filters = TrailFilters::default();
        filters.language = Some("en".to_string());
        filters.age_group = Some("6-8".to_string());
        filters.status = Some("Completed".to_string());
        ctx.set_filters(filters);
        ctx.set_search_query("dragon".to_string());

        let filtered = ctx.filtered_trails();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "1");
    }

    #[test]
    fn test_bookmark_toggle() {
        let ctx = TrailContext::new();

        let trail = create_test_trail("1", "Test Trail", "en", "6-8", "Completed", None);

        // Initially not bookmarked
        assert!(!ctx.is_bookmarked(&trail.id));

        // Toggle bookmark on
        ctx.toggle_bookmark(Bookmark::new(
            trail.id.clone(),
            trail.title.clone(),
            trail.file_path.clone(),
        ));

        assert!(ctx.is_bookmarked(&trail.id));

        // Toggle bookmark off
        ctx.toggle_bookmark(Bookmark::new(
            trail.id.clone(),
            trail.title.clone(),
            trail.file_path.clone(),
        ));

        assert!(!ctx.is_bookmarked(&trail.id));
    }
}
