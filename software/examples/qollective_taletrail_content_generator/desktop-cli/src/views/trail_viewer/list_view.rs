/// Trail List View Component
///
/// Displays trails in a scrollable list with pagination support

use crate::components::{CardGrid, List};
use crate::layout::LayoutConfig;
use crate::models::trail::TrailListItem;
use crate::state::trail_state::TrailContext;
use iocraft::prelude::*;

/// Props for the Trail List View
#[derive(Props)]
pub struct TrailListViewProps {
    pub trail_context: TrailContext,
    pub layout_config: Option<LayoutConfig>,
}

impl Default for TrailListViewProps {
    fn default() -> Self {
        Self {
            trail_context: TrailContext::new(),
            layout_config: None,
        }
    }
}

/// Trail List View Component
///
/// Displays a scrollable list of trails with:
/// - Trail title, language, age group, status, date
/// - Bookmark indicator
/// - Selection highlighting
/// - Pagination for large collections
#[component]
pub fn TrailListView(
    _hooks: Hooks,
    props: &TrailListViewProps,
) -> impl Into<AnyElement<'static>> {
    let ctx = &props.trail_context;
    let layout = props.layout_config.unwrap_or_else(|| LayoutConfig::default());

    let filtered_trails = ctx.filtered_trails();
    let selected_idx = ctx.selected_index();
    let _bookmarks = ctx.bookmarks();

    // Check if we have trails to display
    if filtered_trails.is_empty() {
        return element! {
            View(
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Round,
                border_color: Color::Yellow,
                padding: 2,
            ) {
                Text(
                    content: "No trails found",
                    color: Color::Grey,
                    weight: Weight::Bold
                )
                Text(
                    content: "Try adjusting your filters or search query",
                    color: Color::Grey
                )
            }
        }
        .into_any();
    }

    // Desktop/Large mode: Use card grid
    if layout.grid_columns() > 1 {
        let render_fn: Option<fn(&TrailListItem, bool) -> Vec<String>> = Some(render_trail_card);

        element! {
            View(flex_direction: FlexDirection::Column) {
                // Header
                View(
                    border_style: BorderStyle::Round,
                    border_color: Color::Cyan,
                    padding: 1,
                    margin_bottom: 1
                ) {
                    Text(
                        content: format!("Trail List ({} trails) - Grid View", filtered_trails.len()),
                        color: Color::Cyan,
                        weight: Weight::Bold
                    )
                }

                // Card grid
                CardGrid::<TrailListItem>(
                    items: filtered_trails.clone(),
                    columns: layout.grid_columns(),
                    render_card: render_fn,
                    selected_index: selected_idx,
                    layout_config: layout
                )

                // Help text
                View(margin_top: 1, padding: 1) {
                    Text(
                        content: "↑/↓/←/→: Navigate | Enter: View Details | b: Bookmark | Esc: Back",
                        color: Color::Grey
                    )
                }
            }
        }
        .into_any()
    }
    // Medium/Small mode: Use list
    else {
        element! {
            View(flex_direction: FlexDirection::Column) {
                // Header
                View(
                    border_style: BorderStyle::Round,
                    border_color: Color::Cyan,
                    padding: 1,
                    margin_bottom: 1
                ) {
                    Text(
                        content: format!("Trail List ({} trails)", filtered_trails.len()),
                        color: Color::Cyan,
                        weight: Weight::Bold
                    )
                }

                // Trail list
                List::<TrailListItem>(
                    items: filtered_trails.clone(),
                    selected_index: selected_idx,
                    render_item: None,  // Use default rendering
                    visible_rows: layout.list_visible_rows(),
                    show_pagination: true
                )

                // Help text
                View(margin_top: 1, padding: 1) {
                    Text(
                        content: "↑/↓: Navigate | Enter: View Details | b: Bookmark | f: Filters | /: Search | Esc: Back",
                        color: Color::Grey
                    )
                }
            }
        }
        .into_any()
    }
}

/// Render a trail as a card (for grid view)
///
/// Returns a vector of strings representing the card lines
fn render_trail_card(trail: &TrailListItem, selected: bool) -> Vec<String> {
    // Format the date (take first 10 chars for YYYY-MM-DD)
    let date = if trail.generated_at.len() >= 10 {
        &trail.generated_at[..10]
    } else {
        &trail.generated_at
    };

    // Truncate title if too long (max 30 chars for card)
    let title = if trail.title.len() > 30 {
        format!("{}...", &trail.title[..27])
    } else {
        trail.title.clone()
    };

    vec![
        format!("{}", if selected { "▶ " } else { "" }),
        title,
        format!(""),
        format!("Age: {}", trail.age_group),
        format!("Lang: {}", trail.language),
        format!("Theme: {}", trail.theme),
        format!("Status: {}", trail.status),
        format!(""),
        format!("Date: {}", date),
        format!("Nodes: {}", trail.node_count),
        format!(""),
        if selected { "[View] [Delete]".to_string() } else { "".to_string() },
    ]
}

/// Wrapper function for render_item that doesn't capture bookmarks in closure
fn render_trail_item_wrapper(trail: &TrailListItem, selected: bool) -> String {
    // Simple rendering without bookmark indicator since we can't capture context
    let selection_indicator = if selected { ">" } else { " " };

    // Format the date (take first 10 chars for YYYY-MM-DD)
    let date = if trail.generated_at.len() >= 10 {
        &trail.generated_at[..10]
    } else {
        &trail.generated_at
    };

    // Truncate title if too long (max 40 chars)
    let title = if trail.title.len() > 40 {
        format!("{}...", &trail.title[..37])
    } else {
        trail.title.clone()
    };

    format!(
        "{} {:40} | {} ({:5}) [{:12}] {}",
        selection_indicator,
        title,
        trail.language,
        trail.age_group,
        trail.status,
        date
    )
}

/// Render a single trail item in the list
///
/// Format: [★] Title - Language (Age Group) [Status] Date
/// Where ★ indicates a bookmarked trail
fn render_trail_item(
    trail: &TrailListItem,
    selected: bool,
    bookmarks: &crate::models::preferences::BookmarkCollection,
) -> String {
    let bookmark_indicator = if bookmarks.contains(&trail.id) {
        "★"
    } else {
        " "
    };

    let selection_indicator = if selected { ">" } else { " " };

    // Format the date (take first 10 chars for YYYY-MM-DD)
    let date = if trail.generated_at.len() >= 10 {
        &trail.generated_at[..10]
    } else {
        &trail.generated_at
    };

    // Truncate title if too long (max 40 chars)
    let title = if trail.title.len() > 40 {
        format!("{}...", &trail.title[..37])
    } else {
        trail.title.clone()
    };

    format!(
        "{} {} {:40} | {} ({:5}) [{:12}] {}",
        selection_indicator,
        bookmark_indicator,
        title,
        trail.language,
        trail.age_group,
        trail.status,
        date
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::preferences::{Bookmark, BookmarkCollection};

    #[test]
    fn test_render_trail_item() {
        let trail = TrailListItem {
            id: "test-123".to_string(),
            file_path: "/test/path.json".to_string(),
            title: "Test Trail".to_string(),
            description: "Description".to_string(),
            theme: "Adventure".to_string(),
            age_group: "6-8".to_string(),
            language: "en".to_string(),
            tags: vec![],
            status: "Completed".to_string(),
            generated_at: "2025-11-02T12:00:00Z".to_string(),
            node_count: 10,
            tenant_id: None,
        };

        let bookmarks = BookmarkCollection::new();

        // Test non-selected, non-bookmarked
        let rendered = render_trail_item(&trail, false, &bookmarks);
        assert!(rendered.contains("Test Trail"));
        assert!(rendered.contains("en"));
        assert!(rendered.contains("6-8"));
        assert!(rendered.contains("Completed"));
        assert!(rendered.contains("2025-11-02"));

        // Test with bookmark
        let mut bookmarks_with = BookmarkCollection::new();
        bookmarks_with.add(Bookmark::new(
            trail.id.clone(),
            trail.title.clone(),
            trail.file_path.clone(),
        ));

        let rendered_bookmarked = render_trail_item(&trail, false, &bookmarks_with);
        assert!(rendered_bookmarked.contains("★"));
    }

    #[test]
    fn test_render_trail_item_truncates_long_title() {
        let trail = TrailListItem {
            id: "test-123".to_string(),
            file_path: "/test/path.json".to_string(),
            title: "This is a very long trail title that should be truncated to fit within the display limit".to_string(),
            description: "Description".to_string(),
            theme: "Adventure".to_string(),
            age_group: "6-8".to_string(),
            language: "en".to_string(),
            tags: vec![],
            status: "Completed".to_string(),
            generated_at: "2025-11-02T12:00:00Z".to_string(),
            node_count: 10,
            tenant_id: None,
        };

        let bookmarks = BookmarkCollection::new();
        let rendered = render_trail_item(&trail, false, &bookmarks);

        // Should contain ellipsis
        assert!(rendered.contains("..."));
    }
}
