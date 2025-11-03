/// Trail Viewer Module
///
/// Provides UI for browsing, filtering, and viewing trails

pub mod detail_view;
pub mod filter_panel;
pub mod list_view;

#[cfg(test)]
mod tests;

pub use detail_view::{TrailDetailView, TrailDetailViewProps};
pub use filter_panel::{
    FilterPanel, FilterPanelProps, apply_age_group_filter, apply_language_filter,
    apply_status_filter, AGE_GROUPS, LANGUAGES, STATUSES,
};
pub use list_view::{TrailListView, TrailListViewProps};

use crate::layout::LayoutConfig;
use crate::state::trail_state::{TrailContext, TrailViewMode};
use iocraft::prelude::*;

/// Render the main content area based on view mode
fn render_main_content(ctx: &TrailContext, view_mode: TrailViewMode, layout: LayoutConfig) -> AnyElement<'static> {
    match view_mode {
        TrailViewMode::List => {
            element! {
                TrailListView(
                    trail_context: ctx.clone(),
                    layout_config: Some(layout)
                )
            }
            .into_any()
        }
        TrailViewMode::Detail => {
            render_detail_or_placeholder(ctx)
        }
    }
}

/// Render detail view or placeholder if no trail selected
fn render_detail_or_placeholder(ctx: &TrailContext) -> AnyElement<'static> {
    match ctx.selected_trail() {
        Some(trail) => {
            element! {
                TrailDetailView(
                    trail: trail,
                    full_data: None  // TODO: Load full data asynchronously
                )
            }
            .into_any()
        }
        None => {
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Red,
                    padding: 2
                ) {
                    Text(
                        content: "No trail selected",
                        color: Color::Red
                    )
                }
            }
            .into_any()
        }
    }
}

/// Props for main Trail Viewer component
#[derive(Props)]
pub struct TrailViewerProps {
    pub trail_context: TrailContext,
    pub show_filter_panel: bool,
    pub layout_config: LayoutConfig,
}

impl Default for TrailViewerProps {
    fn default() -> Self {
        Self {
            trail_context: TrailContext::new(),
            show_filter_panel: false,
            layout_config: LayoutConfig::default(),
        }
    }
}

/// Main Trail Viewer Component
///
/// Coordinates between list view and detail view based on current view mode.
/// Handles view switching and displays filter panel when requested.
#[component]
pub fn TrailViewer(
    _hooks: Hooks,
    props: &TrailViewerProps,
) -> impl Into<AnyElement<'static>> {
    let ctx = &props.trail_context;
    let view_mode = ctx.view_mode();

    let mut elements: Vec<AnyElement> = Vec::new();

    // Title bar
    elements.push(
        element! {
            View(
                border_style: BorderStyle::Double,
                border_color: Color::Cyan,
                padding: 1,
                margin_bottom: 1
            ) {
                Text(
                    content: format!("Trail Viewer - {}", view_mode.display_name()),
                    color: Color::Cyan,
                    weight: Weight::Bold
                )
            }
        }
        .into_any(),
    );

    // Filter panel (conditional)
    elements.push(
        element! {
            FilterPanel(
                trail_context: ctx.clone(),
                is_visible: props.show_filter_panel
            )
        }
        .into_any(),
    );

    // Main content - switch between list and detail views
    elements.push(render_main_content(ctx, view_mode, props.layout_config));

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(elements.into_iter())
        }
    }
    .into_any()
}
