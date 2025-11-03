/// Trail Viewer State Management
///
/// Manages state for the Trail Viewer including loaded trails list,
/// filters, search, bookmarks, and view mode

use crate::models::preferences::{Bookmark, BookmarkCollection};
use crate::models::trail::TrailListItem;
use std::sync::{Arc, RwLock};

/// Represents the different view modes in the Trail Viewer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrailViewMode {
    /// List view showing all trails
    List,
    /// Detail view showing a single trail
    Detail,
}

impl TrailViewMode {
    /// Get the display name for the view mode
    pub fn display_name(&self) -> &'static str {
        match self {
            TrailViewMode::List => "Trail List",
            TrailViewMode::Detail => "Trail Details",
        }
    }
}

/// Filters for trail list
#[derive(Debug, Clone, Default)]
pub struct TrailFilters {
    /// Filter by age group (e.g., "6-8", "9-11")
    pub age_group: Option<String>,

    /// Filter by language (e.g., "en", "de")
    pub language: Option<String>,

    /// Filter by status (e.g., "Completed", "InProgress", "Failed")
    pub status: Option<String>,

    /// Filter by tenant ID
    pub tenant_id: Option<String>,
}

impl TrailFilters {
    /// Create a new empty filter set
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any filters are active
    pub fn is_active(&self) -> bool {
        self.age_group.is_some()
            || self.language.is_some()
            || self.status.is_some()
            || self.tenant_id.is_some()
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.age_group = None;
        self.language = None;
        self.status = None;
        self.tenant_id = None;
    }
}

/// Trail Viewer context shared across all trail viewer components
#[derive(Clone)]
pub struct TrailContext {
    inner: Arc<RwLock<TrailState>>,
}

impl TrailContext {
    /// Create a new Trail context
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(TrailState::default())),
        }
    }

    // --- Trail Management ---

    /// Get all loaded trails
    pub fn trails(&self) -> Vec<TrailListItem> {
        self.inner.read().unwrap().trails.clone()
    }

    /// Set trails list
    pub fn set_trails(&self, trails: Vec<TrailListItem>) {
        let mut state = self.inner.write().unwrap();
        state.trails = trails;

        // Recompute filtered trails
        state.update_filtered_trails();

        // Reset selection if out of bounds
        if state.selected_index >= state.filtered_trails.len() && !state.filtered_trails.is_empty() {
            state.selected_index = state.filtered_trails.len() - 1;
        } else if state.filtered_trails.is_empty() {
            state.selected_index = 0;
        }
    }

    /// Get filtered trails based on current filters and search
    pub fn filtered_trails(&self) -> Vec<TrailListItem> {
        self.inner.read().unwrap().filtered_trails.clone()
    }

    // --- Selection Management ---

    /// Get selected trail index in filtered list
    pub fn selected_index(&self) -> usize {
        self.inner.read().unwrap().selected_index
    }

    /// Set selected trail index
    pub fn set_selected_index(&self, index: usize) {
        let mut state = self.inner.write().unwrap();
        if index < state.filtered_trails.len() {
            state.selected_index = index;
        }
    }

    /// Get selected trail
    pub fn selected_trail(&self) -> Option<TrailListItem> {
        let state = self.inner.read().unwrap();
        state.filtered_trails.get(state.selected_index).cloned()
    }

    /// Select next trail
    pub fn next_trail(&self) {
        let mut state = self.inner.write().unwrap();
        if !state.filtered_trails.is_empty() {
            state.selected_index = (state.selected_index + 1) % state.filtered_trails.len();
        }
    }

    /// Select previous trail
    pub fn previous_trail(&self) {
        let mut state = self.inner.write().unwrap();
        if !state.filtered_trails.is_empty() {
            if state.selected_index == 0 {
                state.selected_index = state.filtered_trails.len() - 1;
            } else {
                state.selected_index -= 1;
            }
        }
    }

    // --- Filter Management ---

    /// Get current filters
    pub fn filters(&self) -> TrailFilters {
        self.inner.read().unwrap().filters.clone()
    }

    /// Set filters
    pub fn set_filters(&self, filters: TrailFilters) {
        let mut state = self.inner.write().unwrap();
        state.filters = filters;
        state.update_filtered_trails();

        // Reset selection
        state.selected_index = 0;
    }

    /// Clear all filters
    pub fn clear_filters(&self) {
        let mut state = self.inner.write().unwrap();
        state.filters.clear();
        state.update_filtered_trails();
        state.selected_index = 0;
    }

    // --- Search Management ---

    /// Get search query
    pub fn search_query(&self) -> String {
        self.inner.read().unwrap().search_query.clone()
    }

    /// Set search query
    pub fn set_search_query(&self, query: String) {
        let mut state = self.inner.write().unwrap();
        state.search_query = query;
        state.update_filtered_trails();

        // Reset selection
        state.selected_index = 0;
    }

    /// Clear search query
    pub fn clear_search(&self) {
        self.set_search_query(String::new());
    }

    // --- Bookmark Management ---

    /// Get bookmarks collection
    pub fn bookmarks(&self) -> BookmarkCollection {
        self.inner.read().unwrap().bookmarks.clone()
    }

    /// Set bookmarks collection
    pub fn set_bookmarks(&self, bookmarks: BookmarkCollection) {
        self.inner.write().unwrap().bookmarks = bookmarks;
    }

    /// Check if a trail is bookmarked
    pub fn is_bookmarked(&self, trail_id: &str) -> bool {
        self.inner.read().unwrap().bookmarks.contains(trail_id)
    }

    /// Toggle bookmark for a trail
    pub fn toggle_bookmark(&self, bookmark: Bookmark) {
        let mut state = self.inner.write().unwrap();
        state.bookmarks.toggle(bookmark);
    }

    /// Filter to show only bookmarked trails
    pub fn show_bookmarks_only(&self) -> bool {
        self.inner.read().unwrap().show_bookmarks_only
    }

    /// Set bookmarks-only filter
    pub fn set_show_bookmarks_only(&self, show_only: bool) {
        let mut state = self.inner.write().unwrap();
        state.show_bookmarks_only = show_only;
        state.update_filtered_trails();
        state.selected_index = 0;
    }

    // --- View Mode Management ---

    /// Get current view mode
    pub fn view_mode(&self) -> TrailViewMode {
        self.inner.read().unwrap().view_mode
    }

    /// Set view mode
    pub fn set_view_mode(&self, mode: TrailViewMode) {
        self.inner.write().unwrap().view_mode = mode;
    }

    /// Switch to list view
    pub fn show_list(&self) {
        self.set_view_mode(TrailViewMode::List);
    }

    /// Switch to detail view
    pub fn show_detail(&self) {
        self.set_view_mode(TrailViewMode::Detail);
    }
}

impl Default for TrailContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal trail state
#[derive(Debug, Clone)]
struct TrailState {
    /// All loaded trails
    trails: Vec<TrailListItem>,

    /// Filtered trails based on current filters and search
    filtered_trails: Vec<TrailListItem>,

    /// Selected trail index in filtered list
    selected_index: usize,

    /// Active filters
    filters: TrailFilters,

    /// Search query
    search_query: String,

    /// Bookmarks collection
    bookmarks: BookmarkCollection,

    /// Show only bookmarked trails
    show_bookmarks_only: bool,

    /// Current view mode
    view_mode: TrailViewMode,
}

impl Default for TrailState {
    fn default() -> Self {
        Self {
            trails: Vec::new(),
            filtered_trails: Vec::new(),
            selected_index: 0,
            filters: TrailFilters::default(),
            search_query: String::new(),
            bookmarks: BookmarkCollection::new(),
            show_bookmarks_only: false,
            view_mode: TrailViewMode::List,
        }
    }
}

impl TrailState {
    /// Update filtered trails based on current filters and search
    fn update_filtered_trails(&mut self) {
        self.filtered_trails = self
            .trails
            .iter()
            .filter(|trail| {
                // Apply age group filter
                if let Some(ref age_group) = self.filters.age_group {
                    if &trail.age_group != age_group {
                        return false;
                    }
                }

                // Apply language filter
                if let Some(ref language) = self.filters.language {
                    if &trail.language != language {
                        return false;
                    }
                }

                // Apply status filter
                if let Some(ref status) = self.filters.status {
                    if &trail.status != status {
                        return false;
                    }
                }

                // Apply tenant filter
                if let Some(ref tenant_id) = self.filters.tenant_id {
                    match &trail.tenant_id {
                        Some(tid) if tid == tenant_id => {},
                        _ => return false,
                    }
                }

                // Apply search query (case-insensitive across title, description, theme)
                if !self.search_query.is_empty() {
                    let query_lower = self.search_query.to_lowercase();
                    let matches = trail.title.to_lowercase().contains(&query_lower)
                        || trail.description.to_lowercase().contains(&query_lower)
                        || trail.theme.to_lowercase().contains(&query_lower);

                    if !matches {
                        return false;
                    }
                }

                // Apply bookmark filter
                if self.show_bookmarks_only {
                    if !self.bookmarks.contains(&trail.id) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_trail(id: &str, title: &str, language: &str, age_group: &str) -> TrailListItem {
        TrailListItem {
            id: id.to_string(),
            file_path: format!("/test/{}.json", id),
            title: title.to_string(),
            description: format!("Description for {}", title),
            theme: "Adventure".to_string(),
            age_group: age_group.to_string(),
            language: language.to_string(),
            tags: vec![],
            status: "Completed".to_string(),
            generated_at: "2025-11-02T12:00:00Z".to_string(),
            node_count: 10,
            tenant_id: None,
        }
    }

    #[test]
    fn test_trail_filters_is_active() {
        let mut filters = TrailFilters::new();
        assert!(!filters.is_active());

        filters.language = Some("en".to_string());
        assert!(filters.is_active());

        filters.clear();
        assert!(!filters.is_active());
    }

    #[test]
    fn test_trail_context_creation() {
        let ctx = TrailContext::new();
        assert_eq!(ctx.trails().len(), 0);
        assert_eq!(ctx.view_mode(), TrailViewMode::List);
    }

    #[test]
    fn test_trail_selection() {
        let ctx = TrailContext::new();

        let trails = vec![
            create_test_trail("1", "Trail 1", "en", "6-8"),
            create_test_trail("2", "Trail 2", "en", "9-11"),
            create_test_trail("3", "Trail 3", "en", "12-14"),
        ];

        ctx.set_trails(trails);

        // Test selection navigation
        assert_eq!(ctx.selected_index(), 0);

        ctx.next_trail();
        assert_eq!(ctx.selected_index(), 1);

        ctx.next_trail();
        assert_eq!(ctx.selected_index(), 2);

        ctx.next_trail(); // Wraps around
        assert_eq!(ctx.selected_index(), 0);

        ctx.previous_trail(); // Wraps around
        assert_eq!(ctx.selected_index(), 2);
    }

    #[test]
    fn test_filtering() {
        let ctx = TrailContext::new();

        let trails = vec![
            create_test_trail("1", "English Young", "en", "6-8"),
            create_test_trail("2", "German Young", "de", "6-8"),
            create_test_trail("3", "English Teen", "en", "12-14"),
        ];

        ctx.set_trails(trails);

        // Filter by language
        let mut filters = TrailFilters::new();
        filters.language = Some("en".to_string());
        ctx.set_filters(filters);

        let filtered = ctx.filtered_trails();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|t| t.language == "en"));
    }

    #[test]
    fn test_view_mode() {
        let ctx = TrailContext::new();

        assert_eq!(ctx.view_mode(), TrailViewMode::List);

        ctx.show_detail();
        assert_eq!(ctx.view_mode(), TrailViewMode::Detail);

        ctx.show_list();
        assert_eq!(ctx.view_mode(), TrailViewMode::List);
    }
}
