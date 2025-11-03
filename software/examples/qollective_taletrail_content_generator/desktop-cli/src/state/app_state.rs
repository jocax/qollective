use std::sync::{Arc, RwLock};

use crate::environment::Environment;
use crate::layout::{LayoutConfig, LayoutMode};

/// Represents the different views in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Menu,
    McpTester,
    TrailViewer,
    NatsMonitor,
    StoryGenerator,
    Search,
    Settings,
    Logs,
}

impl View {
    /// Get the display name for the view
    pub fn display_name(&self) -> &'static str {
        match self {
            View::Menu => "Main Menu",
            View::McpTester => "MCP Tester",
            View::TrailViewer => "Trail Viewer",
            View::NatsMonitor => "NATS Monitor",
            View::StoryGenerator => "Story Generator",
            View::Search => "Search & Comparison",
            View::Settings => "Settings",
            View::Logs => "Application Logs",
        }
    }

    /// Get the menu option number (1-8)
    pub fn menu_index(&self) -> Option<usize> {
        match self {
            View::Menu => None,
            View::McpTester => Some(1),
            View::TrailViewer => Some(2),
            View::NatsMonitor => Some(3),
            View::StoryGenerator => Some(4),
            View::Search => Some(5),
            View::Settings => Some(6),
            View::Logs => Some(7),
        }
    }

    /// Get view from menu index (1-8, where 8 is Quit)
    pub fn from_menu_index(index: usize) -> Option<Self> {
        match index {
            1 => Some(View::McpTester),
            2 => Some(View::TrailViewer),
            3 => Some(View::NatsMonitor),
            4 => Some(View::StoryGenerator),
            5 => Some(View::Search),
            6 => Some(View::Settings),
            7 => Some(View::Logs),
            _ => None,
        }
    }
}

/// Application context shared across all components
#[derive(Clone)]
pub struct AppContext {
    inner: Arc<RwLock<AppState>>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(AppState::default())),
        }
    }

    /// Get the current view
    pub fn current_view(&self) -> View {
        self.inner.read().unwrap().current_view
    }

    /// Set the current view
    pub fn set_current_view(&self, view: View) {
        self.inner.write().unwrap().current_view = view;
    }

    /// Get NATS connection status
    pub fn nats_connected(&self) -> bool {
        self.inner.read().unwrap().nats_connected
    }

    /// Set NATS connection status
    pub fn set_nats_connected(&self, connected: bool) {
        self.inner.write().unwrap().nats_connected = connected;
    }

    /// Get active requests count
    pub fn active_requests(&self) -> usize {
        self.inner.read().unwrap().active_requests
    }

    /// Set active requests count
    pub fn set_active_requests(&self, count: usize) {
        self.inner.write().unwrap().active_requests = count;
    }

    /// Increment active requests count
    pub fn increment_requests(&self) {
        self.inner.write().unwrap().active_requests += 1;
    }

    /// Decrement active requests count
    pub fn decrement_requests(&self) {
        let mut state = self.inner.write().unwrap();
        if state.active_requests > 0 {
            state.active_requests -= 1;
        }
    }

    /// Check if help modal is visible
    pub fn help_visible(&self) -> bool {
        self.inner.read().unwrap().help_visible
    }

    /// Set help modal visibility
    pub fn set_help_visible(&self, visible: bool) {
        self.inner.write().unwrap().help_visible = visible;
    }

    /// Toggle help modal
    pub fn toggle_help(&self) {
        let mut state = self.inner.write().unwrap();
        state.help_visible = !state.help_visible;
    }

    /// Check if application should quit
    pub fn should_quit(&self) -> bool {
        self.inner.read().unwrap().should_quit
    }

    /// Set quit flag
    pub fn set_quit(&self, quit: bool) {
        self.inner.write().unwrap().should_quit = quit;
    }

    /// Get current terminal width
    pub fn terminal_width(&self) -> usize {
        self.inner.read().unwrap().terminal_width
    }

    /// Get current terminal height
    pub fn terminal_height(&self) -> usize {
        self.inner.read().unwrap().terminal_height
    }

    /// Get current layout configuration
    pub fn layout_config(&self) -> LayoutConfig {
        self.inner.read().unwrap().layout_config
    }

    /// Update terminal size and recalculate layout configuration
    /// Respects manual_display_mode if set (manual mode takes precedence)
    pub fn update_terminal_size(&self, width: usize, height: usize) {
        let mut state = self.inner.write().unwrap();
        state.terminal_width = width;
        state.terminal_height = height;

        // Only update layout if not in manual mode
        if state.manual_display_mode.is_none() {
            state.layout_config = LayoutConfig::from_terminal_size(width, height);
        }
    }

    /// Toggle debug console expansion
    pub fn toggle_debug_console(&self) {
        let mut state = self.inner.write().unwrap();
        state.debug_console_expanded = !state.debug_console_expanded;
    }

    /// Check if debug console is expanded
    pub fn is_debug_console_expanded(&self) -> bool {
        self.inner.read().unwrap().debug_console_expanded
    }

    /// Add a debug log message
    pub fn add_debug_log(&self, message: String) {
        let mut state = self.inner.write().unwrap();

        // Add timestamp
        let timestamp = chrono::Utc::now().format("%H:%M:%S");
        let log_entry = format!("[{}] {}", timestamp, message);

        state.debug_logs.push(log_entry);

        // Keep only last 100 logs
        if state.debug_logs.len() > 100 {
            state.debug_logs.remove(0);
        }
    }

    /// Get debug logs
    pub fn get_debug_logs(&self) -> Vec<String> {
        self.inner.read().unwrap().debug_logs.clone()
    }

    /// Clear debug logs
    pub fn clear_debug_logs(&self) {
        let mut state = self.inner.write().unwrap();
        state.debug_logs.clear();
    }

    /// Toggle debug mode
    pub fn toggle_debug_mode(&self) {
        let mut state = self.inner.write().unwrap();
        state.debug_mode = !state.debug_mode;
    }

    /// Check if debug mode is enabled
    pub fn is_debug_mode(&self) -> bool {
        self.inner.read().unwrap().debug_mode
    }

    /// Set debug mode
    pub fn set_debug_mode(&self, enabled: bool) {
        self.inner.write().unwrap().debug_mode = enabled;
    }

    /// Get the detected environment
    pub fn environment(&self) -> Environment {
        self.inner.read().unwrap().environment
    }

    /// Get the manual display mode override (if set)
    pub fn manual_display_mode(&self) -> Option<LayoutMode> {
        self.inner.read().unwrap().manual_display_mode
    }

    /// Set the manual display mode override
    pub fn set_manual_display_mode(&self, mode: Option<LayoutMode>) {
        let mut state = self.inner.write().unwrap();
        state.manual_display_mode = mode;

        // If manual mode is set, update layout config immediately
        if let Some(layout_mode) = mode {
            // Use specific dimensions for each mode
            let (width, height) = match layout_mode {
                LayoutMode::Classic => (80, 24),
                LayoutMode::Modern => (120, 30),
                LayoutMode::FullHD => (240, 60),
                LayoutMode::FourK => (480, 120),
            };
            state.layout_config = LayoutConfig::from_terminal_size(width, height);
        } else {
            // Auto mode: use actual terminal size
            state.layout_config = LayoutConfig::from_terminal_size(state.terminal_width, state.terminal_height);
        }
    }

    /// Cycle to next display mode (for Ctrl+L hotkey)
    pub fn cycle_display_mode(&self) {
        let mut state = self.inner.write().unwrap();

        state.manual_display_mode = match state.manual_display_mode {
            None => Some(LayoutMode::Classic),
            Some(LayoutMode::Classic) => Some(LayoutMode::Modern),
            Some(LayoutMode::Modern) => Some(LayoutMode::FullHD),
            Some(LayoutMode::FullHD) => Some(LayoutMode::FourK),
            Some(LayoutMode::FourK) => None, // Back to Auto
        };

        // Update layout config
        if let Some(layout_mode) = state.manual_display_mode {
            let (width, height) = match layout_mode {
                LayoutMode::Classic => (80, 24),
                LayoutMode::Modern => (120, 30),
                LayoutMode::FullHD => (240, 60),
                LayoutMode::FourK => (480, 120),
            };
            state.layout_config = LayoutConfig::from_terminal_size(width, height);
        } else {
            // Auto mode: use actual terminal size
            state.layout_config = LayoutConfig::from_terminal_size(state.terminal_width, state.terminal_height);
        }
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_view: View,
    pub nats_connected: bool,
    pub active_requests: usize,
    pub help_visible: bool,
    pub should_quit: bool,
    pub terminal_width: usize,
    pub terminal_height: usize,
    pub layout_config: LayoutConfig,
    pub debug_console_expanded: bool,
    pub debug_logs: Vec<String>,
    pub debug_mode: bool,
    pub environment: Environment,
    pub manual_display_mode: Option<LayoutMode>,
}

impl Default for AppState {
    fn default() -> Self {
        // Detect environment and determine debug mode
        let environment = Environment::detect();
        let debug_mode = environment.is_ide();

        // Start with a reasonable default size (Modern mode)
        // This will be updated immediately on startup with actual terminal size
        let terminal_width = 120;
        let terminal_height = 30;
        let layout_config = LayoutConfig::from_terminal_size(terminal_width, terminal_height);

        Self {
            current_view: View::Menu,
            nats_connected: false,
            active_requests: 0,
            help_visible: false,
            should_quit: false,
            terminal_width,
            terminal_height,
            layout_config,
            debug_console_expanded: false,
            debug_logs: vec![],
            debug_mode,
            environment,
            manual_display_mode: None, // Start in Auto mode
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_display_names() {
        assert_eq!(View::Menu.display_name(), "Main Menu");
        assert_eq!(View::McpTester.display_name(), "MCP Tester");
        assert_eq!(View::TrailViewer.display_name(), "Trail Viewer");
        assert_eq!(View::NatsMonitor.display_name(), "NATS Monitor");
        assert_eq!(View::StoryGenerator.display_name(), "Story Generator");
        assert_eq!(View::Search.display_name(), "Search & Comparison");
        assert_eq!(View::Settings.display_name(), "Settings");
        assert_eq!(View::Logs.display_name(), "Application Logs");
    }

    #[test]
    fn test_view_menu_index() {
        assert_eq!(View::Menu.menu_index(), None);
        assert_eq!(View::McpTester.menu_index(), Some(1));
        assert_eq!(View::TrailViewer.menu_index(), Some(2));
        assert_eq!(View::NatsMonitor.menu_index(), Some(3));
        assert_eq!(View::StoryGenerator.menu_index(), Some(4));
        assert_eq!(View::Search.menu_index(), Some(5));
        assert_eq!(View::Settings.menu_index(), Some(6));
        assert_eq!(View::Logs.menu_index(), Some(7));
    }

    #[test]
    fn test_view_from_menu_index() {
        assert_eq!(View::from_menu_index(1), Some(View::McpTester));
        assert_eq!(View::from_menu_index(2), Some(View::TrailViewer));
        assert_eq!(View::from_menu_index(3), Some(View::NatsMonitor));
        assert_eq!(View::from_menu_index(4), Some(View::StoryGenerator));
        assert_eq!(View::from_menu_index(5), Some(View::Search));
        assert_eq!(View::from_menu_index(6), Some(View::Settings));
        assert_eq!(View::from_menu_index(7), Some(View::Logs));
        assert_eq!(View::from_menu_index(8), None); // Quit action, not a view
        assert_eq!(View::from_menu_index(0), None);
        assert_eq!(View::from_menu_index(99), None);
    }

    #[test]
    fn test_app_context_view_management() {
        let ctx = AppContext::new();
        assert_eq!(ctx.current_view(), View::Menu);

        ctx.set_current_view(View::McpTester);
        assert_eq!(ctx.current_view(), View::McpTester);

        ctx.set_current_view(View::Settings);
        assert_eq!(ctx.current_view(), View::Settings);
    }

    #[test]
    fn test_app_context_nats_status() {
        let ctx = AppContext::new();
        assert_eq!(ctx.nats_connected(), false);

        ctx.set_nats_connected(true);
        assert!(ctx.nats_connected());

        ctx.set_nats_connected(false);
        assert!(!ctx.nats_connected());
    }

    #[test]
    fn test_app_context_active_requests() {
        let ctx = AppContext::new();
        assert_eq!(ctx.active_requests(), 0);

        ctx.set_active_requests(5);
        assert_eq!(ctx.active_requests(), 5);

        ctx.increment_requests();
        assert_eq!(ctx.active_requests(), 6);

        ctx.decrement_requests();
        assert_eq!(ctx.active_requests(), 5);

        // Test decrement doesn't go below 0
        ctx.set_active_requests(0);
        ctx.decrement_requests();
        assert_eq!(ctx.active_requests(), 0);
    }

    #[test]
    fn test_app_context_help_modal() {
        let ctx = AppContext::new();
        assert!(!ctx.help_visible());

        ctx.set_help_visible(true);
        assert!(ctx.help_visible());

        ctx.toggle_help();
        assert!(!ctx.help_visible());

        ctx.toggle_help();
        assert!(ctx.help_visible());
    }

    #[test]
    fn test_app_context_quit_flag() {
        let ctx = AppContext::new();
        assert!(!ctx.should_quit());

        ctx.set_quit(true);
        assert!(ctx.should_quit());

        ctx.set_quit(false);
        assert!(!ctx.should_quit());
    }
}
