use serde::{Deserialize, Serialize};

/// User preferences for the TaleTrail CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Path to trails directory
    pub trails_directory: String,

    /// Path to templates directory (symlinked from desktop app)
    pub templates_directory: String,

    /// Path to execution logs directory
    pub logs_directory: String,

    /// NATS connection URL
    pub nats_url: String,

    /// NATS connection timeout in seconds
    pub nats_timeout: u64,

    /// Optional TLS certificate path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_cert_path: Option<String>,

    /// Optional NKey path for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nkey_path: Option<String>,

    /// Auto-scroll behavior in NATS monitor
    pub auto_scroll: bool,

    /// Color theme preference
    pub theme: Theme,

    /// Default tenant ID for requests
    #[serde(default)]
    pub default_tenant_id: String,

    /// Auto-validate JSON in request editor
    pub auto_validate_json: bool,

    /// Number of entries per page in history view
    pub history_page_size: usize,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            trails_directory: "taletrail-data".to_string(),
            templates_directory: "./templates".to_string(),
            logs_directory: "./logs".to_string(),
            nats_url: "nats://localhost:4222".to_string(),
            nats_timeout: 30,
            tls_cert_path: None,
            nkey_path: None,
            auto_scroll: true,
            theme: Theme::System,
            default_tenant_id: "1".to_string(),
            auto_validate_json: true,
            history_page_size: 20,
        }
    }
}

/// View mode for trail display
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViewMode {
    /// Linear sequential reading mode
    Linear,
    /// Interactive choice navigation mode
    Interactive,
    /// DAG visualization mode
    DAG,
    /// Execution trace timeline view
    ExecutionTrace,
}

/// Theme preference for UI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Theme {
    /// Light color scheme
    Light,
    /// Dark color scheme
    Dark,
    /// Use system theme preference
    System,
}

/// User bookmark for a trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Unique trail identifier
    pub trail_id: String,

    /// Trail title for display
    pub trail_title: String,

    /// Full file path to the trail
    pub file_path: String,

    /// Timestamp when bookmark was created
    pub timestamp: String,

    /// Optional user note about this bookmark
    #[serde(default)]
    pub user_note: String,

    /// Optional tenant ID for multi-tenancy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl Bookmark {
    /// Create a new bookmark
    pub fn new(trail_id: String, trail_title: String, file_path: String) -> Self {
        Self {
            trail_id,
            trail_title,
            file_path,
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_note: String::new(),
            tenant_id: None,
        }
    }

    /// Add a note to the bookmark
    pub fn with_note(mut self, note: String) -> Self {
        self.user_note = note;
        self
    }

    /// Set the tenant ID
    pub fn with_tenant(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }
}

/// Collection of bookmarks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkCollection {
    pub bookmarks: Vec<Bookmark>,
}

impl BookmarkCollection {
    /// Create a new empty bookmark collection
    pub fn new() -> Self {
        Self {
            bookmarks: Vec::new(),
        }
    }

    /// Add a bookmark
    pub fn add(&mut self, bookmark: Bookmark) {
        self.bookmarks.push(bookmark);
    }

    /// Remove a bookmark by trail ID
    pub fn remove(&mut self, trail_id: &str) -> bool {
        if let Some(pos) = self.bookmarks.iter().position(|b| b.trail_id == trail_id) {
            self.bookmarks.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if a trail is bookmarked
    pub fn contains(&self, trail_id: &str) -> bool {
        self.bookmarks.iter().any(|b| b.trail_id == trail_id)
    }

    /// Get a bookmark by trail ID
    pub fn get(&self, trail_id: &str) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| b.trail_id == trail_id)
    }

    /// Get all bookmarks
    pub fn all(&self) -> &[Bookmark] {
        &self.bookmarks
    }

    /// Toggle a bookmark (add if not present, remove if present)
    pub fn toggle(&mut self, bookmark: Bookmark) -> bool {
        if self.remove(&bookmark.trail_id) {
            false // Removed
        } else {
            self.add(bookmark);
            true // Added
        }
    }
}
