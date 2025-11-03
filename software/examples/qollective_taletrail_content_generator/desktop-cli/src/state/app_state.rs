use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

use crate::environment::Environment;

/// Theme mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Dark  // Default to Dark since most developers use dark terminals
    }
}

/// Represents the different views in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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

    /// Get current terminal width
    pub fn terminal_width(&self) -> usize {
        self.inner.read().unwrap().terminal_width
    }

    /// Get current terminal height
    pub fn terminal_height(&self) -> usize {
        self.inner.read().unwrap().terminal_height
    }

    /// Update terminal size
    pub fn update_terminal_size(&self, width: usize, height: usize) {
        let mut state = self.inner.write().unwrap();
        state.terminal_width = width;
        state.terminal_height = height;
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

    /// Get current theme mode
    pub fn theme_mode(&self) -> ThemeMode {
        self.inner.read().unwrap().theme_mode
    }

    /// Toggle theme mode between Dark and Light
    pub fn toggle_theme(&self) {
        let mut state = self.inner.write().unwrap();
        state.theme_mode = match state.theme_mode {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
    }

    /// Set theme mode
    pub fn set_theme(&self, mode: ThemeMode) {
        self.inner.write().unwrap().theme_mode = mode;
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
    pub nats_connected: bool,
    pub active_requests: usize,
    pub terminal_width: usize,
    pub terminal_height: usize,
    pub debug_console_expanded: bool,
    pub debug_logs: Vec<String>,
    pub debug_mode: bool,
    pub environment: Environment,
    pub theme_mode: ThemeMode,
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

        Self {
            nats_connected: false,
            active_requests: 0,
            terminal_width,
            terminal_height,
            debug_console_expanded: false,
            debug_logs: vec![],
            debug_mode,
            environment,
            theme_mode: ThemeMode::default(),
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
}
