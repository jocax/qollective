use crate::models::UserPreferences;
use crate::utils::get_preferences_key;
use tauri_plugin_store::StoreExt;
use serde_json::json;

/// Save user preferences to persistent storage
///
/// Supports tenant isolation - preferences for different tenants are stored separately
#[tauri::command]
pub async fn save_preferences(
    app: tauri::AppHandle,
    preferences: UserPreferences,
    tenant_id: Option<String>,
) -> Result<(), String> {
    let store = app.store("settings.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    // Get tenant-scoped storage key
    let storage_key = get_preferences_key(tenant_id.as_deref());

    store.set(&storage_key, json!(preferences));
    store.save().map_err(|e| format!("Failed to save preferences: {}", e))?;

    Ok(())
}

/// Load user preferences from persistent storage
///
/// Supports tenant isolation - retrieves preferences for specified tenant
#[tauri::command]
pub async fn load_preferences(
    app: tauri::AppHandle,
    tenant_id: Option<String>,
) -> Result<UserPreferences, String> {
    let store = app.store("settings.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    // Get tenant-scoped storage key
    let storage_key = get_preferences_key(tenant_id.as_deref());

    let preferences: UserPreferences = store
        .get(&storage_key)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(UserPreferences::default);

    Ok(preferences)
}

/// Load the application configuration file (config.toml)
///
/// Returns the raw TOML content from the config.toml file located in the src-tauri directory
#[tauri::command]
pub async fn load_config_toml() -> Result<String, String> {
    // The config.toml file is in the same directory as Cargo.toml (src-tauri/)
    let config_path = std::path::PathBuf::from("config.toml");

    std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config.toml: {}", e))
}
