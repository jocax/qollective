use serde::{Deserialize, Serialize};

/// User preferences for the TaleTrail viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub default_view_mode: ViewMode,
    pub theme: Theme,
    pub directory_path: String,
    pub auto_validate: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        UserPreferences {
            default_view_mode: ViewMode::Linear,
            theme: Theme::System,
            directory_path: String::new(),
            auto_validate: true,
        }
    }
}

/// View mode for trail display
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViewMode {
    Linear,
    Interactive,
    DAG,
    ExecutionTrace,
}

/// Theme preference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// User bookmark for a trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub trail_id: String,
    pub trail_title: String,
    pub file_path: String,
    pub timestamp: String,
    pub user_note: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}
