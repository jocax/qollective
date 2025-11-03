use iocraft::prelude::*;

use crate::{
    components::{create_help_content, Menu, Modal, Navbar, StatusBar},
    layout::LayoutMode,
    state::{AppContext, View as AppView, SearchContext},
    views::{LogsView, PlaceholderView, SearchView},
};

/// Main application component
#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    // Create application context using use_ref for persistence
    let app_ctx = hooks.use_ref(|| {
        let ctx = AppContext::new();
        // Add some initial debug logs for demonstration
        ctx.add_debug_log("Application initialized".to_string());
        ctx.add_debug_log(format!("Environment: {}", ctx.environment().name()));
        ctx.add_debug_log("Loading configuration...".to_string());
        ctx.add_debug_log("Ready for user input".to_string());
        ctx
    });

    // Detect terminal size and update layout config
    let terminal_size = hooks.use_terminal_size();
    hooks.use_effect(move || {
        let (width, height) = terminal_size;
        app_ctx.read().update_terminal_size(width as usize, height as usize);
        app_ctx.read().add_debug_log(format!("Terminal size: {}×{}", width, height));
    }, terminal_size);

    // Create search context
    let search_ctx = hooks.use_ref(|| SearchContext::new());

    // Local state for menu navigation (used in Classic mode)
    let selected_menu_index = hooks.use_state(|| 0usize);

    // Get current state from app_ctx
    let current_view = app_ctx.read().current_view();
    let help_visible = app_ctx.read().help_visible();
    let layout_config = app_ctx.read().layout_config();

    // TODO: Set up keyboard event handling for navbar navigation
    // This will be implemented with proper event channels in the next iteration
    // For now, we'll rely on individual component keyboard handlers

    // Determine which view to show based on layout mode
    // In Classic mode, show Menu if current_view is Menu
    // In Modern/FullHD/FourK mode, skip Menu and show McpTester by default
    let effective_view = if layout_config.layout_mode == LayoutMode::Classic {
        current_view
    } else {
        // In desktop modes, if we're on Menu, switch to McpTester
        if current_view == AppView::Menu {
            AppView::McpTester
        } else {
            current_view
        }
    };

    // Render the appropriate view based on effective_view
    let main_content: AnyElement = match effective_view {
        AppView::Menu => element! {
            Menu(selected_index: selected_menu_index.get())
        }.into(),
        AppView::McpTester => element! {
            PlaceholderView(view: AppView::McpTester)
        }.into(),
        AppView::TrailViewer => element! {
            PlaceholderView(view: AppView::TrailViewer)
        }.into(),
        AppView::NatsMonitor => element! {
            PlaceholderView(view: AppView::NatsMonitor)
        }.into(),
        AppView::StoryGenerator => element! {
            PlaceholderView(view: AppView::StoryGenerator)
        }.into(),
        AppView::Search => element! {
            SearchView(
                search_context: search_ctx.read().clone(),
                layout_config: layout_config
            )
        }.into(),
        AppView::Settings => element! {
            PlaceholderView(view: AppView::Settings)
        }.into(),
        AppView::Logs => element! {
            LogsView(
                layout_config: layout_config,
                app_context: app_ctx.read().clone()
            )
        }.into(),
    };

    let help_content = create_help_content();

    element! {
        View(flex_direction: FlexDirection::Column, height: 100pct) {
            // Top: Navbar (responsive - hidden in Compact mode)
            Navbar(
                current_view: effective_view,
                layout_config: layout_config,
            )

            // Middle: Main content area (flex grow to fill)
            View(flex_grow: 1.0) {
                #(main_content)
            }

            // Bottom: Status bar (responsive height)
            StatusBar(
                help_hint: None,
                nats_connected: app_ctx.read().nats_connected(),
                active_requests: app_ctx.read().active_requests(),
                view_name: effective_view.display_name().to_string(),
                layout_config: layout_config,
                app_context: Some(app_ctx.read().clone()),
            )

            // Help modal overlay
            Modal(
                title: "Help - TaleTrail Desktop CLI".to_string(),
                content: help_content,
                visible: help_visible,
            )
        }
    }
}

/// Handle keyboard events
/// Returns new selected_menu_index if it changed
fn handle_key_event(
    app_ctx: &AppContext,
    selected_menu_index: usize,
    key_event: KeyEvent,
    current_view: AppView,
) -> Option<usize> {
    use KeyCode::*;
    use KeyModifiers;

    // Global hotkeys (work in any view)
    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
        match key_event.code {
            Char('q') | Char('Q') => {
                app_ctx.set_quit(true);
                return None;
            }
            Char('h') | Char('H') => {
                app_ctx.toggle_help();
                return None;
            }
            Char('d') | Char('D') => {
                app_ctx.toggle_debug_console();
                return None;
            }
            Char('1') => {
                app_ctx.set_current_view(AppView::McpTester);
                return None;
            }
            Char('2') => {
                app_ctx.set_current_view(AppView::TrailViewer);
                return None;
            }
            Char('3') => {
                app_ctx.set_current_view(AppView::NatsMonitor);
                return None;
            }
            Char('4') => {
                app_ctx.set_current_view(AppView::StoryGenerator);
                return None;
            }
            Char('5') => {
                app_ctx.set_current_view(AppView::Search);
                return None;
            }
            Char('6') => {
                app_ctx.set_current_view(AppView::Settings);
                return None;
            }
            Char('7') => {
                app_ctx.set_current_view(AppView::Logs);
                return None;
            }
            Char('l') | Char('L') => {
                // Ctrl+L: Cycle display mode
                app_ctx.cycle_display_mode();
                let mode_name = app_ctx.layout_config().layout_mode.display_name();
                app_ctx.add_debug_log(format!("Display mode: {}", mode_name));
                return None;
            }
            _ => {}
        }
    }

    // F1 for help
    if let F(1) = key_event.code {
        app_ctx.toggle_help();
        return None;
    }

    // F12 for debug mode toggle
    if let F(12) = key_event.code {
        app_ctx.toggle_debug_mode();
        let state = if app_ctx.is_debug_mode() { "enabled" } else { "disabled" };
        app_ctx.add_debug_log(format!("Debug mode {}", state));
        return None;
    }

    // ESC to close modal or go back
    if let Esc = key_event.code {
        if app_ctx.help_visible() {
            app_ctx.set_help_visible(false);
        } else if current_view != AppView::Menu {
            app_ctx.set_current_view(AppView::Menu);
        }
        return None;
    }

    // View-specific keyboard handling
    match current_view {
        AppView::Menu => handle_menu_keys(app_ctx, selected_menu_index, key_event),
        _ => {
            // Other views will handle their own keys in future task groups
            // For now, just allow ESC to go back (handled above)
            None
        }
    }
}

/// Handle keyboard events specific to the menu view
/// Returns new selected_menu_index if it changed
fn handle_menu_keys(
    app_ctx: &AppContext,
    selected_menu_index: usize,
    key_event: KeyEvent,
) -> Option<usize> {
    use KeyCode::*;

    match key_event.code {
        // Arrow keys for navigation
        Up | Char('k') | Char('K') => {
            if selected_menu_index > 0 {
                Some(selected_menu_index - 1)
            } else {
                None
            }
        }
        Down | Char('j') | Char('J') => {
            if selected_menu_index < 6 {
                // 7 items, 0-6 index
                Some(selected_menu_index + 1)
            } else {
                None
            }
        }
        // Number keys for direct selection
        Char('1') => Some(0),
        Char('2') => Some(1),
        Char('3') => Some(2),
        Char('4') => Some(3),
        Char('5') => Some(4),
        Char('6') => Some(5),
        Char('7') => Some(6),
        // Enter to confirm selection
        Enter => {
            handle_menu_selection(app_ctx, selected_menu_index);
            None
        }
        _ => None,
    }
}

/// Handle menu item selection
fn handle_menu_selection(app_ctx: &AppContext, selected_index: usize) {
    match selected_index {
        0 => app_ctx.set_current_view(AppView::McpTester),
        1 => app_ctx.set_current_view(AppView::TrailViewer),
        2 => app_ctx.set_current_view(AppView::NatsMonitor),
        3 => app_ctx.set_current_view(AppView::StoryGenerator),
        4 => app_ctx.set_current_view(AppView::Search),
        5 => app_ctx.set_current_view(AppView::Settings),
        6 => app_ctx.set_quit(true), // Quit
        _ => {}
    }
}
