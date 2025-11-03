pub const APP_NAME: &str = "TaleTrail Desktop CLI";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// Configuration
pub const DEFAULT_CONFIG_FILE: &str = "config.toml";
pub const DEFAULT_NATS_URL: &str = "nats://localhost:4222";
pub const DEFAULT_NATS_TIMEOUT_SECS: u64 = 30;

// Directory defaults
pub const DEFAULT_TRAILS_DIR: &str = "./trails";
pub const DEFAULT_TEMPLATES_DIR: &str = "./templates";
pub const DEFAULT_EXECUTION_LOGS_DIR: &str = "./execution_logs";
pub const DEFAULT_BOOKMARKS_FILE: &str = "./bookmarks.json";
pub const DEFAULT_HISTORY_FILE: &str = "./request_history.json";

// UI constants
pub const MIN_TERMINAL_WIDTH: u16 = 80;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;
pub const STATUS_BAR_HEIGHT: u16 = 1;
pub const MENU_HEIGHT: u16 = 3;

// NATS subjects
pub const NATS_MCP_SUBJECT_PREFIX: &str = "mcp";
pub const NATS_TALETRAIL_SUBJECT_PREFIX: &str = "taletrail";

// MCP servers
pub const MCP_SERVERS: [&str; 5] = [
    "orchestrator",
    "story-generator",
    "quality-control",
    "constraint-enforcer",
    "prompt-helper",
];

// Pagination
pub const DEFAULT_PAGE_SIZE: usize = 20;
pub const MAX_MESSAGE_BUFFER_SIZE: usize = 1000;

// Color themes
pub const COLOR_THEME_DARK: &str = "dark";
pub const COLOR_THEME_LIGHT: &str = "light";
pub const DEFAULT_COLOR_THEME: &str = COLOR_THEME_DARK;
