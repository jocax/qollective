/// Filter Panel Component
///
/// Provides UI for filtering trails by age group, language, status, and tenant

use crate::components::{Select, TextInput};
use crate::state::trail_state::{TrailContext, TrailFilters};
use iocraft::prelude::*;

/// Available age groups for filtering
pub const AGE_GROUPS: &[&str] = &["All", "6-8", "9-11", "12-14", "15-17", "18+"];

/// Available languages for filtering
pub const LANGUAGES: &[&str] = &["All", "en", "de"];

/// Available statuses for filtering
pub const STATUSES: &[&str] = &["All", "Completed", "InProgress", "Failed"];

/// Props for Filter Panel
#[derive(Props)]
pub struct FilterPanelProps {
    pub trail_context: TrailContext,
    pub is_visible: bool,
}

impl Default for FilterPanelProps {
    fn default() -> Self {
        Self {
            trail_context: TrailContext::new(),
            is_visible: false,
        }
    }
}

/// Filter Panel Component
///
/// Displays filter controls for:
/// - Age group
/// - Language
/// - Status
/// - Text search
/// - Bookmark filter toggle
#[component]
pub fn FilterPanel(
    _hooks: Hooks,
    props: &FilterPanelProps,
) -> impl Into<AnyElement<'static>> {
    if !props.is_visible {
        return element! {
            View {}
        }
        .into_any();
    }

    let ctx = &props.trail_context;
    let filters = ctx.filters();
    let search_query = ctx.search_query();
    let show_bookmarks_only = ctx.show_bookmarks_only();

    // Determine selected indices for dropdowns
    let age_group_idx = filters
        .age_group
        .as_ref()
        .and_then(|ag| AGE_GROUPS.iter().position(|&x| x == ag))
        .unwrap_or(0);

    let language_idx = filters
        .language
        .as_ref()
        .and_then(|lang| LANGUAGES.iter().position(|&x| x == lang))
        .unwrap_or(0);

    let status_idx = filters
        .status
        .as_ref()
        .and_then(|st| STATUSES.iter().position(|&x| x == st))
        .unwrap_or(0);

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Double,
            border_color: Color::Magenta,
            padding: 2,
            margin_bottom: 2
        ) {
            // Title
            View(margin_bottom: 2) {
                Text(
                    content: "Trail Filters",
                    color: Color::Magenta,
                    weight: Weight::Bold
                )
            }

            // Search input
            View(margin_bottom: 1) {
                TextInput(
                    label: "Search (title, description, theme)".to_string(),
                    value: search_query.clone(),
                    error: None,
                    placeholder: Some("Type to search...".to_string()),
                    is_focused: false
                )
            }

            // Age Group filter
            View(margin_bottom: 1) {
                Select(
                    label: "Age Group".to_string(),
                    options: AGE_GROUPS.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
                    selected_index: age_group_idx,
                    is_focused: false,
                    is_expanded: false
                )
            }

            // Language filter
            View(margin_bottom: 1) {
                Select(
                    label: "Language".to_string(),
                    options: LANGUAGES.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
                    selected_index: language_idx,
                    is_focused: false,
                    is_expanded: false
                )
            }

            // Status filter
            View(margin_bottom: 1) {
                Select(
                    label: "Status".to_string(),
                    options: STATUSES.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
                    selected_index: status_idx,
                    is_focused: false,
                    is_expanded: false
                )
            }

            // Bookmark filter toggle
            View(margin_bottom: 1) {
                Text(
                    content: if show_bookmarks_only {
                        "[X] Show only bookmarked trails"
                    } else {
                        "[ ] Show only bookmarked trails"
                    },
                    color: if show_bookmarks_only { Color::Yellow } else { Color::White }
                )
            }

            // Active filters summary
            View(
                margin_top: 2,
                padding_top: 1,
                border_style: BorderStyle::Single,
                border_color: Color::DarkGrey
            ) {
                Text(
                    content: build_active_filters_summary(&filters, &search_query, show_bookmarks_only),
                    color: Color::DarkGrey
                )
            }

            // Help text
            View(margin_top: 2) {
                Text(
                    content: "c: Clear Filters | Esc: Close Panel",
                    color: Color::DarkGrey
                )
            }
        }
    }
    .into_any()
}

/// Build a summary of active filters
fn build_active_filters_summary(
    filters: &TrailFilters,
    search_query: &str,
    show_bookmarks_only: bool,
) -> String {
    let mut active = Vec::new();

    if let Some(ref age_group) = filters.age_group {
        active.push(format!("Age: {}", age_group));
    }

    if let Some(ref language) = filters.language {
        active.push(format!("Lang: {}", language));
    }

    if let Some(ref status) = filters.status {
        active.push(format!("Status: {}", status));
    }

    if !search_query.is_empty() {
        active.push(format!("Search: \"{}\"", search_query));
    }

    if show_bookmarks_only {
        active.push("Bookmarks Only".to_string());
    }

    if active.is_empty() {
        "No active filters".to_string()
    } else {
        format!("Active: {}", active.join(" | "))
    }
}

/// Helper to apply filter from dropdown selection
pub fn apply_age_group_filter(ctx: &TrailContext, index: usize) {
    let mut filters = ctx.filters();

    if index == 0 {
        // "All" selected - clear filter
        filters.age_group = None;
    } else if let Some(age_group) = AGE_GROUPS.get(index) {
        filters.age_group = Some(age_group.to_string());
    }

    ctx.set_filters(filters);
}

/// Helper to apply language filter from dropdown selection
pub fn apply_language_filter(ctx: &TrailContext, index: usize) {
    let mut filters = ctx.filters();

    if index == 0 {
        // "All" selected - clear filter
        filters.language = None;
    } else if let Some(language) = LANGUAGES.get(index) {
        filters.language = Some(language.to_string());
    }

    ctx.set_filters(filters);
}

/// Helper to apply status filter from dropdown selection
pub fn apply_status_filter(ctx: &TrailContext, index: usize) {
    let mut filters = ctx.filters();

    if index == 0 {
        // "All" selected - clear filter
        filters.status = None;
    } else if let Some(status) = STATUSES.get(index) {
        filters.status = Some(status.to_string());
    }

    ctx.set_filters(filters);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_active_filters_summary_no_filters() {
        let filters = TrailFilters::default();
        let summary = build_active_filters_summary(&filters, "", false);
        assert_eq!(summary, "No active filters");
    }

    #[test]
    fn test_build_active_filters_summary_with_filters() {
        let mut filters = TrailFilters::default();
        filters.age_group = Some("6-8".to_string());
        filters.language = Some("en".to_string());

        let summary = build_active_filters_summary(&filters, "dragon", true);
        assert!(summary.contains("Age: 6-8"));
        assert!(summary.contains("Lang: en"));
        assert!(summary.contains("Search: \"dragon\""));
        assert!(summary.contains("Bookmarks Only"));
    }

    #[test]
    fn test_apply_age_group_filter() {
        let ctx = TrailContext::new();

        // Apply age group filter
        apply_age_group_filter(&ctx, 1); // "6-8"

        let filters = ctx.filters();
        assert_eq!(filters.age_group, Some("6-8".to_string()));

        // Clear filter by selecting "All"
        apply_age_group_filter(&ctx, 0);

        let filters = ctx.filters();
        assert_eq!(filters.age_group, None);
    }

    #[test]
    fn test_apply_language_filter() {
        let ctx = TrailContext::new();

        // Apply language filter
        apply_language_filter(&ctx, 1); // "en"

        let filters = ctx.filters();
        assert_eq!(filters.language, Some("en".to_string()));

        // Clear filter by selecting "All"
        apply_language_filter(&ctx, 0);

        let filters = ctx.filters();
        assert_eq!(filters.language, None);
    }

    #[test]
    fn test_apply_status_filter() {
        let ctx = TrailContext::new();

        // Apply status filter
        apply_status_filter(&ctx, 1); // "Completed"

        let filters = ctx.filters();
        assert_eq!(filters.status, Some("Completed".to_string()));

        // Clear filter by selecting "All"
        apply_status_filter(&ctx, 0);

        let filters = ctx.filters();
        assert_eq!(filters.status, None);
    }
}
