use iocraft::prelude::*;

use crate::{
    components::{create_help_content, Menu, Modal, Navbar, StatusBar},
    config::Config,
    keyboard::{Action, KeyMapper},
    layout::{LayoutConfig, LayoutMode},
    state::{AppContext, View as AppView, SearchContext},
    views::{LogsView, PlaceholderView, SearchView},
};

/// Main application component
#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    // Get system context for exiting the render loop
    let mut system = hooks.use_context_mut::<SystemContext>();

    // Create application context using use_ref for non-reactive data
    let app_ctx = hooks.use_ref(|| {
        let ctx = AppContext::new();
        // Add some initial debug logs for demonstration
        ctx.add_debug_log("Application initialized".to_string());
        ctx.add_debug_log(format!("Environment: {}", ctx.environment().name()));
        ctx.add_debug_log("Loading configuration...".to_string());
        ctx.add_debug_log("Ready for user input".to_string());
        ctx
    });

    // Create search context
    let search_ctx = hooks.use_ref(|| SearchContext::new());

    // Load keyboard mapper once with use_memo
    let key_mapper = hooks.use_ref(|| {
        let config = Config::load().unwrap_or_default();
        KeyMapper::from_config(config.keyboard).expect("Failed to create keyboard mapper")
    });

    // Reactive state using use_state
    let current_view = hooks.use_state(|| AppView::Menu);
    let help_visible = hooks.use_state(|| false);
    let should_quit = hooks.use_state(|| false);
    let manual_display_mode = hooks.use_state(|| None::<LayoutMode>);
    let layout_config = hooks.use_state(|| LayoutConfig::default());

    // Force re-render trigger for debug state changes
    let debug_state_version = hooks.use_state(|| 0u32);

    // Local state for menu navigation (used in Classic mode)
    let selected_menu_index = hooks.use_state(|| 0usize);

    // Detect terminal size and update layout config
    let terminal_size = hooks.use_terminal_size();
    hooks.use_effect({
        let mut config = layout_config;
        let manual_mode = manual_display_mode.get();

        move || {
            let (width, height) = terminal_size;
            app_ctx.read().update_terminal_size(width as usize, height as usize);
            app_ctx.read().add_debug_log(format!("Terminal size: {}×{}", width, height));

            // Update layout config based on manual mode or terminal size
            if let Some(mode) = manual_mode {
                let (w, h) = match mode {
                    LayoutMode::Classic => (80, 24),
                    LayoutMode::Modern => (120, 30),
                    LayoutMode::FullHD => (240, 60),
                    LayoutMode::FourK => (480, 120),
                };
                config.set(LayoutConfig::from_terminal_size(w, h));
            } else {
                config.set(LayoutConfig::from_terminal_size(width as usize, height as usize));
            }
        }
    }, (terminal_size, manual_display_mode.get()));

    // Set up keyboard event handling with KeyMapper
    hooks.use_terminal_events({
        let app_context = app_ctx.clone();
        let mut mapper = key_mapper.clone();
        let mut menu_index = selected_menu_index;
        let mut view = current_view;
        let mut help = help_visible;
        let mut quit = should_quit;
        let mut manual_mode = manual_display_mode;
        let mut config = layout_config;
        let mut debug_version = debug_state_version;

        move |event| {
            if let TerminalEvent::Key(key_event) = event {
                // Debug logging - log every keypress if debug mode
                if app_context.read().is_debug_mode() {
                    app_context.read().add_debug_log(format!(
                        "KEY: {:?} | MODS: {:?} | VIEW: {:?}",
                        key_event.code, key_event.modifiers, view.get()
                    ));
                }

                // Use KeyMapper to get action
                if let Some(action) = mapper.write().handle_event(&key_event) {
                    // Log matched action in debug mode
                    if app_context.read().is_debug_mode() {
                        app_context.read().add_debug_log(format!("ACTION: {:?}", action));
                    }

                    // Dispatch action
                    match action {
                        Action::Quit => {
                            quit.set(true);
                            return;
                        }
                        Action::Help => {
                            help.set(!help.get());
                            return;
                        }
                        Action::DebugMode => {
                            app_context.read().toggle_debug_mode();
                            let state = if app_context.read().is_debug_mode() {
                                "enabled"
                            } else {
                                "disabled"
                            };
                            app_context
                                .read()
                                .add_debug_log(format!("Debug mode {}", state));
                            debug_version.set(debug_version.get().wrapping_add(1));
                            return;
                        }
                        Action::DebugConsole => {
                            app_context.read().toggle_debug_console();
                            debug_version.set(debug_version.get().wrapping_add(1));
                            return;
                        }
                        Action::ThemeToggle => {
                            app_context.read().toggle_theme();
                            let theme = app_context.read().theme_mode();
                            app_context
                                .read()
                                .add_debug_log(format!("Theme toggled to: {:?}", theme));
                            debug_version.set(debug_version.get().wrapping_add(1));
                            return;
                        }
                        Action::DisplayModeCycle => {
                            // Cycle display mode logic
                            let next_mode = match manual_mode.get() {
                                None => Some(LayoutMode::Classic),
                                Some(LayoutMode::Classic) => Some(LayoutMode::Modern),
                                Some(LayoutMode::Modern) => Some(LayoutMode::FullHD),
                                Some(LayoutMode::FullHD) => Some(LayoutMode::FourK),
                                Some(LayoutMode::FourK) => None,
                            };
                            manual_mode.set(next_mode);

                            if let Some(mode) = next_mode {
                                let (w, h) = match mode {
                                    LayoutMode::Classic => (80, 24),
                                    LayoutMode::Modern => (120, 30),
                                    LayoutMode::FullHD => (240, 60),
                                    LayoutMode::FourK => (480, 120),
                                };
                                config.set(LayoutConfig::from_terminal_size(w, h));
                                app_context
                                    .read()
                                    .add_debug_log(format!("Display mode: {}", mode.display_name()));
                            } else {
                                app_context
                                    .read()
                                    .add_debug_log("Display mode: Auto".to_string());
                            }
                            return;
                        }
                        Action::GoToView(target_view) => {
                            view.set(target_view);
                            app_context.read().add_debug_log(format!(
                                "Switched to: {}",
                                target_view.display_name()
                            ));
                            return;
                        }
                        Action::MenuNavigateUp => {
                            if view.get() == AppView::Menu && menu_index.get() > 0 {
                                menu_index.set(menu_index.get() - 1);
                            }
                            return;
                        }
                        Action::MenuNavigateDown => {
                            if view.get() == AppView::Menu && menu_index.get() < 6 {
                                menu_index.set(menu_index.get() + 1);
                            }
                            return;
                        }
                        Action::MenuSelect => {
                            if view.get() == AppView::Menu {
                                match menu_index.get() {
                                    0 => view.set(AppView::McpTester),
                                    1 => view.set(AppView::TrailViewer),
                                    2 => view.set(AppView::NatsMonitor),
                                    3 => view.set(AppView::StoryGenerator),
                                    4 => view.set(AppView::Search),
                                    5 => view.set(AppView::Settings),
                                    6 => quit.set(true),
                                    _ => {}
                                }
                            }
                            return;
                        }
                        Action::MenuBack => {
                            if help.get() {
                                help.set(false);
                            } else if view.get() != AppView::Menu {
                                view.set(AppView::Menu);
                            }
                            return;
                        }
                    }
                }
            }
        }
    });

    // Just use the current view - let user control what they see
    let effective_view = current_view.get();

    // Check if user wants to quit and exit the render loop
    if should_quit.get() {
        system.exit();
    }

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
                layout_config: layout_config.get()
            )
        }.into(),
        AppView::Settings => element! {
            PlaceholderView(view: AppView::Settings)
        }.into(),
        AppView::Logs => element! {
            LogsView(
                layout_config: layout_config.get(),
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
                layout_config: layout_config.get(),
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
                layout_config: layout_config.get(),
                app_context: Some(app_ctx.read().clone()),
                manual_display_mode: manual_display_mode.get(),
            )

            // Help modal overlay
            Modal(
                title: "Help - TaleTrail Desktop CLI".to_string(),
                content: help_content,
                visible: help_visible.get(),
            )
        }
    }
}
