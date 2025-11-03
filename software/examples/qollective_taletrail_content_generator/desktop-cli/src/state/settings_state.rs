use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use crate::config::Config;
use crate::error::Result;

/// Settings state context for managing application configuration
#[derive(Clone)]
pub struct SettingsContext {
    inner: Arc<RwLock<SettingsState>>,
}

/// Internal settings state
struct SettingsState {
    config: Config,
    editing: EditingConfig,
    validation_errors: ValidationErrors,
    is_dirty: bool,
    current_section: SettingsSection,
    active_tab: SettingsTab,
}

/// Editable configuration fields (as strings for UI editing)
#[derive(Clone, Debug)]
pub struct EditingConfig {
    // NATS fields
    pub nats_url: String,
    pub nats_timeout: String,
    pub tls_cert_path: String,
    pub nkey_path: String,

    // Directory fields
    pub trails_dir: String,
    pub templates_dir: String,
    pub execution_logs_dir: String,

    // UI fields
    pub color_theme: String,
    pub auto_scroll: bool,
    pub display_mode: Option<String>,
}

/// Validation errors for each field
#[derive(Clone, Default, Debug)]
pub struct ValidationErrors {
    pub nats_url: Option<String>,
    pub nats_timeout: Option<String>,
    pub tls_cert_path: Option<String>,
    pub nkey_path: Option<String>,
    pub trails_dir: Option<String>,
    pub templates_dir: Option<String>,
    pub execution_logs_dir: Option<String>,
    pub color_theme: Option<String>,
}

/// Settings section tabs (now supports Overview and Config tabs)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SettingsSection {
    NatsConnection,
    Directories,
    UiPreferences,
}

/// Settings view tabs (Overview vs Config)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SettingsTab {
    Overview,
    Config,
}

impl SettingsTab {
    /// Get tab number (1-2)
    pub fn tab_number(&self) -> usize {
        match self {
            SettingsTab::Overview => 1,
            SettingsTab::Config => 2,
        }
    }

    /// Get tab from number (1-2)
    pub fn from_number(number: usize) -> Option<Self> {
        match number {
            1 => Some(SettingsTab::Overview),
            2 => Some(SettingsTab::Config),
            _ => None,
        }
    }

    /// Get display name for the tab
    pub fn display_name(&self) -> &'static str {
        match self {
            SettingsTab::Overview => "Overview",
            SettingsTab::Config => "Config",
        }
    }
}

/// Field identifiers for updates
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SettingsField {
    NatsUrl,
    NatsTimeout,
    TlsCertPath,
    NkeyPath,
    TrailsDir,
    TemplatesDir,
    ExecutionLogsDir,
    ColorTheme,
    AutoScroll,
}

impl SettingsContext {
    /// Create a new settings context with loaded configuration
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        let editing = EditingConfig::from_config(&config);

        Self {
            inner: Arc::new(RwLock::new(SettingsState {
                config: config.clone(),
                editing,
                validation_errors: ValidationErrors::default(),
                is_dirty: false,
                current_section: SettingsSection::NatsConnection,
                active_tab: SettingsTab::Overview,
            })),
        }
    }

    /// Create a settings context from an existing config (for testing)
    pub fn from_config(config: Config) -> Self {
        let editing = EditingConfig::from_config(&config);

        Self {
            inner: Arc::new(RwLock::new(SettingsState {
                config: config.clone(),
                editing,
                validation_errors: ValidationErrors::default(),
                is_dirty: false,
                current_section: SettingsSection::NatsConnection,
                active_tab: SettingsTab::Overview,
            })),
        }
    }

    /// Update a field value
    pub fn update_field(&self, field: SettingsField, value: String) {
        let mut state = self.inner.write().unwrap();

        match field {
            SettingsField::NatsUrl => state.editing.nats_url = value,
            SettingsField::NatsTimeout => state.editing.nats_timeout = value,
            SettingsField::TlsCertPath => state.editing.tls_cert_path = value,
            SettingsField::NkeyPath => state.editing.nkey_path = value,
            SettingsField::TrailsDir => state.editing.trails_dir = value,
            SettingsField::TemplatesDir => state.editing.templates_dir = value,
            SettingsField::ExecutionLogsDir => state.editing.execution_logs_dir = value,
            SettingsField::ColorTheme => state.editing.color_theme = value,
            SettingsField::AutoScroll => {
                state.editing.auto_scroll = value.to_lowercase() == "true" || value == "1";
            }
        }

        state.is_dirty = true;
        Self::validate_field_internal(&mut state, field);
    }

    /// Toggle auto-scroll setting
    pub fn toggle_auto_scroll(&self) {
        let mut state = self.inner.write().unwrap();
        state.editing.auto_scroll = !state.editing.auto_scroll;
        state.is_dirty = true;
    }

    /// Cycle to next color theme
    pub fn cycle_theme(&self) {
        let mut state = self.inner.write().unwrap();
        state.editing.color_theme = match state.editing.color_theme.as_str() {
            "dark" => "light".to_string(),
            "light" => "dark".to_string(),
            _ => "dark".to_string(),
        };
        state.is_dirty = true;
        Self::validate_field_internal(&mut state, SettingsField::ColorTheme);
    }

    /// Internal validation function
    fn validate_field_internal(state: &mut SettingsState, field: SettingsField) {
        match field {
            SettingsField::NatsUrl => {
                state.validation_errors.nats_url = validate_nats_url(&state.editing.nats_url);
            }
            SettingsField::NatsTimeout => {
                state.validation_errors.nats_timeout = validate_timeout(&state.editing.nats_timeout);
            }
            SettingsField::TlsCertPath => {
                state.validation_errors.tls_cert_path =
                    validate_optional_file_path(&state.editing.tls_cert_path);
            }
            SettingsField::NkeyPath => {
                state.validation_errors.nkey_path =
                    validate_optional_file_path(&state.editing.nkey_path);
            }
            SettingsField::TrailsDir => {
                state.validation_errors.trails_dir =
                    validate_directory(&state.editing.trails_dir);
            }
            SettingsField::TemplatesDir => {
                state.validation_errors.templates_dir =
                    validate_directory(&state.editing.templates_dir);
            }
            SettingsField::ExecutionLogsDir => {
                state.validation_errors.execution_logs_dir =
                    validate_directory(&state.editing.execution_logs_dir);
            }
            SettingsField::ColorTheme => {
                state.validation_errors.color_theme =
                    validate_color_theme(&state.editing.color_theme);
            }
            SettingsField::AutoScroll => {
                // Auto-scroll is always valid (boolean)
            }
        }
    }

    /// Validate all fields
    pub fn validate_all(&self) -> bool {
        let mut state = self.inner.write().unwrap();

        Self::validate_field_internal(&mut state, SettingsField::NatsUrl);
        Self::validate_field_internal(&mut state, SettingsField::NatsTimeout);
        Self::validate_field_internal(&mut state, SettingsField::TlsCertPath);
        Self::validate_field_internal(&mut state, SettingsField::NkeyPath);
        Self::validate_field_internal(&mut state, SettingsField::TrailsDir);
        Self::validate_field_internal(&mut state, SettingsField::TemplatesDir);
        Self::validate_field_internal(&mut state, SettingsField::ExecutionLogsDir);
        Self::validate_field_internal(&mut state, SettingsField::ColorTheme);

        !Self::has_validation_errors_internal(&state)
    }

    /// Check if there are any validation errors
    fn has_validation_errors_internal(state: &SettingsState) -> bool {
        state.validation_errors.nats_url.is_some()
            || state.validation_errors.nats_timeout.is_some()
            || state.validation_errors.tls_cert_path.is_some()
            || state.validation_errors.nkey_path.is_some()
            || state.validation_errors.trails_dir.is_some()
            || state.validation_errors.templates_dir.is_some()
            || state.validation_errors.execution_logs_dir.is_some()
            || state.validation_errors.color_theme.is_some()
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        let mut state = self.inner.write().unwrap();

        // Validate all fields first
        Self::validate_field_internal(&mut state, SettingsField::NatsUrl);
        Self::validate_field_internal(&mut state, SettingsField::NatsTimeout);
        Self::validate_field_internal(&mut state, SettingsField::TlsCertPath);
        Self::validate_field_internal(&mut state, SettingsField::NkeyPath);
        Self::validate_field_internal(&mut state, SettingsField::TrailsDir);
        Self::validate_field_internal(&mut state, SettingsField::TemplatesDir);
        Self::validate_field_internal(&mut state, SettingsField::ExecutionLogsDir);
        Self::validate_field_internal(&mut state, SettingsField::ColorTheme);

        if Self::has_validation_errors_internal(&state) {
            return Err(crate::error::AppError::Config(
                "Fix validation errors before saving".to_string()
            ));
        }

        // Apply editing values to config
        state.config.nats.url = state.editing.nats_url.clone();
        state.config.nats.timeout_secs = state.editing.nats_timeout.parse().unwrap();
        state.config.nats.tls_cert_path = if state.editing.tls_cert_path.is_empty() {
            None
        } else {
            Some(PathBuf::from(&state.editing.tls_cert_path))
        };
        state.config.nats.nkey_path = if state.editing.nkey_path.is_empty() {
            None
        } else {
            Some(PathBuf::from(&state.editing.nkey_path))
        };
        state.config.directories.trails_dir = PathBuf::from(&state.editing.trails_dir);
        state.config.directories.templates_dir = PathBuf::from(&state.editing.templates_dir);
        state.config.directories.execution_logs_dir = PathBuf::from(&state.editing.execution_logs_dir);
        state.config.ui.color_theme = state.editing.color_theme.clone();
        state.config.ui.auto_scroll = state.editing.auto_scroll;
        state.config.ui.display_mode = state.editing.display_mode.clone();

        // Save to disk
        state.config.save_to_file(crate::constants::DEFAULT_CONFIG_FILE)?;

        state.is_dirty = false;
        Ok(())
    }

    /// Get current editing configuration
    pub fn get_editing_config(&self) -> EditingConfig {
        let state = self.inner.read().unwrap();
        state.editing.clone()
    }

    /// Get current validation errors
    pub fn get_validation_errors(&self) -> ValidationErrors {
        let state = self.inner.read().unwrap();
        state.validation_errors.clone()
    }

    /// Check if configuration has unsaved changes
    pub fn is_dirty(&self) -> bool {
        let state = self.inner.read().unwrap();
        state.is_dirty
    }

    /// Get current section
    pub fn get_current_section(&self) -> SettingsSection {
        let state = self.inner.read().unwrap();
        state.current_section
    }

    /// Set current section
    pub fn set_current_section(&self, section: SettingsSection) {
        let mut state = self.inner.write().unwrap();
        state.current_section = section;
    }

    /// Get active tab
    pub fn get_active_tab(&self) -> SettingsTab {
        let state = self.inner.read().unwrap();
        state.active_tab
    }

    /// Set active tab
    pub fn set_active_tab(&self, tab: SettingsTab) {
        let mut state = self.inner.write().unwrap();
        state.active_tab = tab;
    }

    /// Get underlying config (for testing)
    pub fn get_config(&self) -> Config {
        let state = self.inner.read().unwrap();
        state.config.clone()
    }

    /// Get config as TOML string
    pub fn get_config_toml(&self) -> String {
        let state = self.inner.read().unwrap();
        toml::to_string_pretty(&state.config).unwrap_or_else(|_| "# Error serializing config".to_string())
    }

    /// Update display mode in editing config (synced from AppContext)
    pub fn sync_display_mode_from_app(&self, mode: Option<String>) {
        let mut state = self.inner.write().unwrap();
        state.editing.display_mode = mode;
        state.is_dirty = true;
    }

    /// Get the display mode from editing config
    pub fn get_display_mode(&self) -> Option<String> {
        let state = self.inner.read().unwrap();
        state.editing.display_mode.clone()
    }
}

impl Default for SettingsContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EditingConfig {
    /// Create editing config from loaded config
    pub fn from_config(config: &Config) -> Self {
        Self {
            nats_url: config.nats.url.clone(),
            nats_timeout: config.nats.timeout_secs.to_string(),
            tls_cert_path: config.nats.tls_cert_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            nkey_path: config.nats.nkey_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            trails_dir: config.directories.trails_dir.display().to_string(),
            templates_dir: config.directories.templates_dir.display().to_string(),
            execution_logs_dir: config.directories.execution_logs_dir.display().to_string(),
            color_theme: config.ui.color_theme.clone(),
            auto_scroll: config.ui.auto_scroll,
            display_mode: config.ui.display_mode.clone(),
        }
    }
}

// Validation functions

/// Validate NATS URL format
fn validate_nats_url(url: &str) -> Option<String> {
    if url.trim().is_empty() {
        return Some("NATS URL is required".to_string());
    }

    if !url.starts_with("nats://") && !url.starts_with("tls://") {
        return Some("URL must start with nats:// or tls://".to_string());
    }

    None
}

/// Validate timeout value
fn validate_timeout(timeout: &str) -> Option<String> {
    if timeout.trim().is_empty() {
        return Some("Timeout is required".to_string());
    }

    match timeout.parse::<u64>() {
        Ok(val) if val > 0 && val <= 300 => None,
        Ok(_) => Some("Timeout must be between 1 and 300 seconds".to_string()),
        Err(_) => Some("Timeout must be a valid number".to_string()),
    }
}

/// Validate optional file path (empty is OK, but if provided must exist)
fn validate_optional_file_path(path: &str) -> Option<String> {
    if path.trim().is_empty() {
        return None;  // Optional field
    }

    let p = PathBuf::from(path);
    if !p.exists() {
        return Some("File does not exist".to_string());
    }

    if !p.is_file() {
        return Some("Path is not a file".to_string());
    }

    None
}

/// Validate required directory path
fn validate_directory(path: &str) -> Option<String> {
    if path.trim().is_empty() {
        return Some("Directory path is required".to_string());
    }

    // Allow non-existent directories (they can be created)
    // Just validate it's a reasonable path
    let p = PathBuf::from(path);

    // If it exists, must be a directory
    if p.exists() && !p.is_dir() {
        return Some("Path exists but is not a directory".to_string());
    }

    None
}

/// Validate color theme
fn validate_color_theme(theme: &str) -> Option<String> {
    match theme {
        "dark" | "light" => None,
        _ => Some("Theme must be 'dark' or 'light'".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_nats_url() {
        assert!(validate_nats_url("nats://localhost:4222").is_none());
        assert!(validate_nats_url("tls://secure.example.com:4222").is_none());
        assert!(validate_nats_url("").is_some());
        assert!(validate_nats_url("http://invalid").is_some());
    }

    #[test]
    fn test_validate_timeout() {
        assert!(validate_timeout("30").is_none());
        assert!(validate_timeout("1").is_none());
        assert!(validate_timeout("300").is_none());
        assert!(validate_timeout("0").is_some());
        assert!(validate_timeout("301").is_some());
        assert!(validate_timeout("abc").is_some());
        assert!(validate_timeout("").is_some());
    }

    #[test]
    fn test_validate_color_theme() {
        assert!(validate_color_theme("dark").is_none());
        assert!(validate_color_theme("light").is_none());
        assert!(validate_color_theme("invalid").is_some());
        assert!(validate_color_theme("").is_some());
    }

    #[test]
    fn test_validate_optional_file_path() {
        // Empty should be valid (optional)
        assert!(validate_optional_file_path("").is_none());
        assert!(validate_optional_file_path("   ").is_none());
    }

    #[test]
    fn test_validate_directory() {
        // Empty should be invalid (required)
        assert!(validate_directory("").is_some());
        assert!(validate_directory("   ").is_some());

        // Non-existent paths are OK (can be created)
        assert!(validate_directory("/tmp/nonexistent_dir_for_test").is_none());
    }

    #[test]
    fn test_settings_context_creation() {
        let ctx = SettingsContext::new();
        let config = ctx.get_editing_config();

        assert!(!config.nats_url.is_empty());
        assert!(!config.nats_timeout.is_empty());
    }

    #[test]
    fn test_settings_field_update() {
        let ctx = SettingsContext::new();

        ctx.update_field(SettingsField::NatsUrl, "nats://test:4222".to_string());

        let config = ctx.get_editing_config();
        assert_eq!(config.nats_url, "nats://test:4222");
        assert!(ctx.is_dirty());
    }

    #[test]
    fn test_settings_validation() {
        let ctx = SettingsContext::new();

        // Set invalid URL
        ctx.update_field(SettingsField::NatsUrl, "invalid://url".to_string());

        let errors = ctx.get_validation_errors();
        assert!(errors.nats_url.is_some());
    }

    #[test]
    fn test_settings_toggle_auto_scroll() {
        let ctx = SettingsContext::new();

        let initial = ctx.get_editing_config().auto_scroll;
        ctx.toggle_auto_scroll();
        let toggled = ctx.get_editing_config().auto_scroll;

        assert_ne!(initial, toggled);
        assert!(ctx.is_dirty());
    }

    #[test]
    fn test_settings_cycle_theme() {
        let ctx = SettingsContext::new();

        // Assuming default is "dark"
        ctx.cycle_theme();
        let theme = ctx.get_editing_config().color_theme;
        assert_eq!(theme, "light");

        ctx.cycle_theme();
        let theme = ctx.get_editing_config().color_theme;
        assert_eq!(theme, "dark");
    }

    #[test]
    fn test_settings_save_validation() {
        let ctx = SettingsContext::new();

        // Set invalid timeout
        ctx.update_field(SettingsField::NatsTimeout, "999".to_string());

        // Save should fail
        let result = ctx.save();
        assert!(result.is_err());
    }

    #[test]
    fn test_settings_validate_all() {
        let ctx = SettingsContext::new();

        // Default config should be valid
        assert!(ctx.validate_all());

        // Make it invalid
        ctx.update_field(SettingsField::NatsUrl, "".to_string());
        assert!(!ctx.validate_all());
    }
}
