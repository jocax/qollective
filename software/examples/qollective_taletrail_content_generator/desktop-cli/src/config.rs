use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::error::{AppError, Result};
use crate::constants::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub nats: NatsConfig,
    pub directories: DirectoriesConfig,
    pub ui: UiConfig,
    #[serde(default = "KeyBindings::default_for_platform")]
    pub keyboard: KeyBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    #[serde(default = "default_nats_url")]
    pub url: String,

    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    #[serde(default)]
    pub tls_cert_path: Option<PathBuf>,

    #[serde(default)]
    pub nkey_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoriesConfig {
    #[serde(default = "default_trails_dir")]
    pub trails_dir: PathBuf,

    #[serde(default = "default_templates_dir")]
    pub templates_dir: PathBuf,

    #[serde(default = "default_execution_logs_dir")]
    pub execution_logs_dir: PathBuf,

    #[serde(default = "default_bookmarks_file")]
    pub bookmarks_file: PathBuf,

    #[serde(default = "default_history_file")]
    pub history_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_color_theme")]
    pub color_theme: String,

    #[serde(default = "default_auto_scroll")]
    pub auto_scroll: bool,

    #[serde(default = "default_page_size")]
    pub page_size: usize,

    #[serde(default)]
    pub display_mode: Option<String>,
}

// Default value functions
fn default_nats_url() -> String {
    std::env::var("NATS_URL").unwrap_or_else(|_| DEFAULT_NATS_URL.to_string())
}

fn default_timeout() -> u64 {
    std::env::var("NATS_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_NATS_TIMEOUT_SECS)
}

fn default_trails_dir() -> PathBuf {
    std::env::var("TRAILS_DIR")
        .unwrap_or_else(|_| DEFAULT_TRAILS_DIR.to_string())
        .into()
}

fn default_templates_dir() -> PathBuf {
    std::env::var("TEMPLATES_DIR")
        .unwrap_or_else(|_| DEFAULT_TEMPLATES_DIR.to_string())
        .into()
}

fn default_execution_logs_dir() -> PathBuf {
    std::env::var("EXECUTION_LOGS_DIR")
        .unwrap_or_else(|_| DEFAULT_EXECUTION_LOGS_DIR.to_string())
        .into()
}

fn default_bookmarks_file() -> PathBuf {
    std::env::var("BOOKMARKS_FILE")
        .unwrap_or_else(|_| DEFAULT_BOOKMARKS_FILE.to_string())
        .into()
}

fn default_history_file() -> PathBuf {
    std::env::var("HISTORY_FILE")
        .unwrap_or_else(|_| DEFAULT_HISTORY_FILE.to_string())
        .into()
}

fn default_color_theme() -> String {
    std::env::var("COLOR_THEME").unwrap_or_else(|_| DEFAULT_COLOR_THEME.to_string())
}

fn default_auto_scroll() -> bool {
    std::env::var("AUTO_SCROLL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(true)
}

fn default_page_size() -> usize {
    std::env::var("PAGE_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PAGE_SIZE)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    #[serde(default = "default_platform")]
    pub platform: String, // "auto", "macos", "linux", "windows"

    #[serde(default = "default_sequence_timeout")]
    pub sequence_timeout_ms: Option<u64>,

    #[serde(default)]
    pub global: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub menu: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub navigation: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub platforms: HashMap<String, PlatformOverrides>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformOverrides {
    #[serde(default)]
    pub global: Option<HashMap<String, Vec<String>>>,

    #[serde(default)]
    pub navigation: Option<HashMap<String, Vec<String>>>,

    #[serde(default)]
    pub menu: Option<HashMap<String, Vec<String>>>,
}

fn default_platform() -> String {
    "auto".to_string()
}

fn default_sequence_timeout() -> Option<u64> {
    Some(500)
}

impl KeyBindings {
    /// Get platform-specific defaults
    pub fn default_for_platform() -> Self {
        #[cfg(target_os = "macos")]
        return Self::macos_defaults();

        #[cfg(target_os = "linux")]
        return Self::linux_defaults();

        #[cfg(target_os = "windows")]
        return Self::windows_defaults();

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return Self::base_defaults();
    }

    fn base_defaults() -> Self {
        let mut bindings = KeyBindings {
            platform: "auto".to_string(),
            sequence_timeout_ms: Some(500),
            global: HashMap::new(),
            menu: HashMap::new(),
            navigation: HashMap::new(),
            platforms: HashMap::new(),
        };

        // Global bindings (platform-neutral)
        bindings
            .global
            .insert("help".to_string(), vec!["f1".to_string()]);
        bindings
            .global
            .insert("debug_mode".to_string(), vec!["f12".to_string()]);
        bindings
            .global
            .insert("debug_console".to_string(), vec!["ctrl-d".to_string()]);
        bindings.global.insert(
            "display_mode_cycle".to_string(),
            vec!["ctrl-l".to_string()],
        );

        // Menu navigation
        bindings.menu.insert(
            "menu_navigate_up".to_string(),
            vec!["up".to_string(), "k".to_string()],
        );
        bindings.menu.insert(
            "menu_navigate_down".to_string(),
            vec!["down".to_string(), "j".to_string()],
        );
        bindings
            .menu
            .insert("menu_select".to_string(), vec!["enter".to_string()]);
        bindings
            .menu
            .insert("menu_back".to_string(), vec!["esc".to_string()]);

        bindings
    }

    fn macos_defaults() -> Self {
        let mut bindings = Self::base_defaults();

        // macOS: Option key (appears as Alt in terminal)
        bindings.global.insert(
            "help".to_string(),
            vec!["alt-h".to_string(), "f1".to_string()],
        );
        bindings
            .global
            .insert("quit".to_string(), vec!["alt-q".to_string()]);
        bindings
            .global
            .insert("theme_toggle".to_string(), vec!["alt-t".to_string()]);

        // View navigation with Option+1-7
        bindings.navigation.insert(
            "mcp_tester".to_string(),
            vec!["alt-1".to_string(), "f2".to_string()],
        );
        bindings.navigation.insert(
            "trail_viewer".to_string(),
            vec!["alt-2".to_string(), "f3".to_string()],
        );
        bindings.navigation.insert(
            "nats_monitor".to_string(),
            vec!["alt-3".to_string(), "f4".to_string()],
        );
        bindings.navigation.insert(
            "story_generator".to_string(),
            vec!["alt-4".to_string(), "f5".to_string()],
        );
        bindings.navigation.insert(
            "search".to_string(),
            vec!["alt-5".to_string(), "f6".to_string()],
        );
        bindings.navigation.insert(
            "settings".to_string(),
            vec!["alt-6".to_string(), "f7".to_string()],
        );
        bindings.navigation.insert(
            "logs".to_string(),
            vec!["alt-7".to_string(), "f8".to_string()],
        );

        bindings
    }

    fn linux_defaults() -> Self {
        let mut bindings = Self::base_defaults();

        // Linux: Ctrl-based
        bindings.global.insert(
            "help".to_string(),
            vec!["ctrl-h".to_string(), "f1".to_string()],
        );
        bindings
            .global
            .insert("quit".to_string(), vec!["ctrl-q".to_string()]);
        bindings
            .global
            .insert("theme_toggle".to_string(), vec!["ctrl-t".to_string()]);

        // View navigation with Ctrl+1-7
        bindings.navigation.insert(
            "mcp_tester".to_string(),
            vec!["ctrl-1".to_string(), "f2".to_string()],
        );
        bindings.navigation.insert(
            "trail_viewer".to_string(),
            vec!["ctrl-2".to_string(), "f3".to_string()],
        );
        bindings.navigation.insert(
            "nats_monitor".to_string(),
            vec!["ctrl-3".to_string(), "f4".to_string()],
        );
        bindings.navigation.insert(
            "story_generator".to_string(),
            vec!["ctrl-4".to_string(), "f5".to_string()],
        );
        bindings.navigation.insert(
            "search".to_string(),
            vec!["ctrl-5".to_string(), "f6".to_string()],
        );
        bindings.navigation.insert(
            "settings".to_string(),
            vec!["ctrl-6".to_string(), "f7".to_string()],
        );
        bindings
            .navigation
            .insert("logs".to_string(), vec!["ctrl-7".to_string(), "f8".to_string()]);

        bindings
    }

    fn windows_defaults() -> Self {
        // Similar to Linux for now
        Self::linux_defaults()
    }

    /// Apply platform-specific overrides
    pub fn with_platform_overrides(mut self) -> Self {
        let platform = std::env::consts::OS;

        if let Some(overrides) = self.platforms.get(platform).cloned() {
            self.merge_overrides(&overrides);
        }

        self
    }

    fn merge_overrides(&mut self, overrides: &PlatformOverrides) {
        if let Some(global) = &overrides.global {
            for (k, v) in global {
                self.global.insert(k.clone(), v.clone());
            }
        }

        if let Some(navigation) = &overrides.navigation {
            for (k, v) in navigation {
                self.navigation.insert(k.clone(), v.clone());
            }
        }

        if let Some(menu) = &overrides.menu {
            for (k, v) in menu {
                self.menu.insert(k.clone(), v.clone());
            }
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::default_for_platform()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            nats: NatsConfig::default(),
            directories: DirectoriesConfig::default(),
            ui: UiConfig::default(),
            keyboard: KeyBindings::default(),
        }
    }
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: default_nats_url(),
            timeout_secs: default_timeout(),
            tls_cert_path: std::env::var("NATS_TLS_CERT_PATH").ok().map(PathBuf::from),
            nkey_path: std::env::var("NATS_NKEY_PATH").ok().map(PathBuf::from),
        }
    }
}

impl Default for DirectoriesConfig {
    fn default() -> Self {
        Self {
            trails_dir: default_trails_dir(),
            templates_dir: default_templates_dir(),
            execution_logs_dir: default_execution_logs_dir(),
            bookmarks_file: default_bookmarks_file(),
            history_file: default_history_file(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            color_theme: default_color_theme(),
            auto_scroll: default_auto_scroll(),
            page_size: default_page_size(),
            display_mode: std::env::var("DISPLAY_MODE").ok(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| AppError::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Load configuration from default file or use defaults
    pub fn load() -> Result<Self> {
        let config_path = PathBuf::from(DEFAULT_CONFIG_FILE);

        if config_path.exists() {
            Self::load_from_file(config_path)
        } else {
            // Use default configuration with environment variable overrides
            Ok(Self::default())
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate NATS URL format
        if self.nats.url.is_empty() {
            return Err(AppError::Config("NATS URL cannot be empty".to_string()));
        }

        // Validate timeout is reasonable
        if self.nats.timeout_secs == 0 || self.nats.timeout_secs > 300 {
            return Err(AppError::Config(
                "NATS timeout must be between 1 and 300 seconds".to_string(),
            ));
        }

        // Validate TLS cert exists if specified
        if let Some(ref cert_path) = self.nats.tls_cert_path {
            if !cert_path.exists() {
                return Err(AppError::Config(format!(
                    "TLS certificate file not found: {}",
                    cert_path.display()
                )));
            }
        }

        // Validate NKey file exists if specified
        if let Some(ref nkey_path) = self.nats.nkey_path {
            if !nkey_path.exists() {
                return Err(AppError::Config(format!(
                    "NKey file not found: {}",
                    nkey_path.display()
                )));
            }
        }

        // Validate color theme
        if self.ui.color_theme != COLOR_THEME_DARK
            && self.ui.color_theme != COLOR_THEME_LIGHT
        {
            return Err(AppError::Config(format!(
                "Invalid color theme: {}. Must be 'dark' or 'light'",
                self.ui.color_theme
            )));
        }

        // Validate page size is reasonable
        if self.ui.page_size == 0 || self.ui.page_size > 1000 {
            return Err(AppError::Config(
                "Page size must be between 1 and 1000".to_string(),
            ));
        }

        // Validate keyboard bindings by attempting to create KeyMapper
        crate::keyboard::KeyMapper::from_config(self.keyboard.clone())
            .map_err(|e| AppError::Config(format!("Invalid keyboard bindings: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.nats.url, DEFAULT_NATS_URL);
        assert_eq!(config.nats.timeout_secs, DEFAULT_NATS_TIMEOUT_SECS);
        assert_eq!(config.ui.color_theme, DEFAULT_COLOR_THEME);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_timeout() {
        let mut config = Config::default();
        config.nats.timeout_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_color_theme() {
        let mut config = Config::default();
        config.ui.color_theme = "invalid".to_string();
        assert!(config.validate().is_err());
    }
}
