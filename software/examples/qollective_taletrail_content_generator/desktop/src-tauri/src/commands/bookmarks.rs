use crate::models::Bookmark;
use crate::utils::get_bookmark_key;
use tauri_plugin_store::StoreExt;
use serde_json::json;

/// Add a bookmark for a trail
///
/// Supports tenant isolation - bookmarks with tenant_id are stored separately
#[tauri::command]
pub async fn add_bookmark(
    app: tauri::AppHandle,
    bookmark: Bookmark,
) -> Result<Vec<Bookmark>, String> {
    let store = app.store("bookmarks.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    // Get tenant-scoped storage key
    let storage_key = get_bookmark_key(bookmark.tenant_id.as_deref());

    let mut bookmarks: Vec<Bookmark> = store
        .get(&storage_key)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(Vec::new);

    // Check for duplicate bookmark by trail_id
    if !bookmarks.iter().any(|b| b.trail_id == bookmark.trail_id) {
        bookmarks.push(bookmark);
        store.set(&storage_key, json!(bookmarks));
        store.save().map_err(|e| format!("Failed to save bookmarks: {}", e))?;
    }

    Ok(bookmarks)
}

/// Remove a bookmark by trail_id
///
/// Supports tenant isolation - must specify tenant_id to remove from correct scope
#[tauri::command]
pub async fn remove_bookmark(
    app: tauri::AppHandle,
    trail_id: String,
    tenant_id: Option<String>,
) -> Result<Vec<Bookmark>, String> {
    let store = app.store("bookmarks.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    // Get tenant-scoped storage key
    let storage_key = get_bookmark_key(tenant_id.as_deref());

    let mut bookmarks: Vec<Bookmark> = store
        .get(&storage_key)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(Vec::new);

    bookmarks.retain(|b| b.trail_id != trail_id);

    store.set(&storage_key, json!(bookmarks));
    store.save().map_err(|e| format!("Failed to save bookmarks: {}", e))?;

    Ok(bookmarks)
}

/// Get all bookmarks
///
/// Supports tenant isolation - retrieves bookmarks for specified tenant
#[tauri::command]
pub async fn get_bookmarks(
    app: tauri::AppHandle,
    tenant_id: Option<String>,
) -> Result<Vec<Bookmark>, String> {
    let store = app.store("bookmarks.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    // Get tenant-scoped storage key
    let storage_key = get_bookmark_key(tenant_id.as_deref());

    let bookmarks: Vec<Bookmark> = store
        .get(&storage_key)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(Vec::new);

    Ok(bookmarks)
}
