/// Bookmark management utilities
///
/// Provides bookmark persistence, loading, and management functionality

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use smol::fs;
use std::collections::HashSet;
use std::path::Path;

/// Collection of bookmarked trail IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkCollection {
    /// Set of trail IDs that are bookmarked
    pub bookmarks: HashSet<String>,
}

impl BookmarkCollection {
    /// Create a new empty bookmark collection
    pub fn new() -> Self {
        Self {
            bookmarks: HashSet::new(),
        }
    }

    /// Check if a trail is bookmarked
    ///
    /// # Arguments
    /// * `trail_id` - Trail ID to check
    ///
    /// # Returns
    /// * `bool` - True if trail is bookmarked
    pub fn is_bookmarked(&self, trail_id: &str) -> bool {
        self.bookmarks.contains(trail_id)
    }

    /// Add a bookmark
    ///
    /// # Arguments
    /// * `trail_id` - Trail ID to bookmark
    ///
    /// # Returns
    /// * `bool` - True if bookmark was added (wasn't already bookmarked)
    pub fn add(&mut self, trail_id: &str) -> bool {
        self.bookmarks.insert(trail_id.to_string())
    }

    /// Remove a bookmark
    ///
    /// # Arguments
    /// * `trail_id` - Trail ID to unbookmark
    ///
    /// # Returns
    /// * `bool` - True if bookmark was removed (was previously bookmarked)
    pub fn remove(&mut self, trail_id: &str) -> bool {
        self.bookmarks.remove(trail_id)
    }

    /// Toggle a bookmark
    ///
    /// # Arguments
    /// * `trail_id` - Trail ID to toggle
    ///
    /// # Returns
    /// * `bool` - True if bookmark is now active, false if removed
    pub fn toggle(&mut self, trail_id: &str) -> bool {
        if self.is_bookmarked(trail_id) {
            self.remove(trail_id);
            false
        } else {
            self.add(trail_id);
            true
        }
    }

    /// Get count of bookmarks
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Clear all bookmarks
    pub fn clear(&mut self) {
        self.bookmarks.clear();
    }
}

impl Default for BookmarkCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Load bookmarks from a JSON file
///
/// # Arguments
/// * `file_path` - Path to the bookmarks JSON file
///
/// # Returns
/// * `Ok(BookmarkCollection)` - Loaded bookmark collection
/// * `Err(AppError)` - If file cannot be read or parsed
pub async fn load_bookmarks(file_path: &Path) -> Result<BookmarkCollection, AppError> {
    // If file doesn't exist, return empty collection
    if !file_path.exists() {
        return Ok(BookmarkCollection::new());
    }

    let contents = fs::read_to_string(file_path).await?;

    // Handle empty file
    if contents.trim().is_empty() {
        return Ok(BookmarkCollection::new());
    }

    let collection: BookmarkCollection = serde_json::from_str(&contents)
        .map_err(|e| AppError::Serialization(e))?;

    Ok(collection)
}

/// Save bookmarks to a JSON file
///
/// # Arguments
/// * `file_path` - Path to the bookmarks JSON file
/// * `bookmarks` - Bookmark collection to save
///
/// # Returns
/// * `Ok(())` - Bookmarks saved successfully
/// * `Err(AppError)` - If file cannot be written
pub async fn save_bookmarks(
    file_path: &Path,
    bookmarks: &BookmarkCollection,
) -> Result<(), AppError> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    let json = serde_json::to_string_pretty(bookmarks)
        .map_err(|e| AppError::Serialization(e))?;

    fs::write(file_path, json).await?;

    Ok(())
}

/// Toggle a bookmark and save to file
///
/// # Arguments
/// * `file_path` - Path to the bookmarks JSON file
/// * `trail_id` - Trail ID to toggle
///
/// # Returns
/// * `Ok(bool)` - True if bookmark is now active, false if removed
/// * `Err(AppError)` - If file operations fail
pub async fn toggle_and_save(
    file_path: &Path,
    trail_id: &str,
) -> Result<bool, AppError> {
    let mut bookmarks = load_bookmarks(file_path).await?;
    let is_bookmarked = bookmarks.toggle(trail_id);
    save_bookmarks(file_path, &bookmarks).await?;
    Ok(is_bookmarked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_collection_new() {
        let collection = BookmarkCollection::new();
        assert_eq!(collection.count(), 0);
    }

    #[test]
    fn test_bookmark_add() {
        let mut collection = BookmarkCollection::new();
        assert!(collection.add("trail-1"));
        assert!(collection.is_bookmarked("trail-1"));
        assert_eq!(collection.count(), 1);
    }

    #[test]
    fn test_bookmark_add_duplicate() {
        let mut collection = BookmarkCollection::new();
        assert!(collection.add("trail-1"));
        assert!(!collection.add("trail-1")); // Should return false for duplicate
        assert_eq!(collection.count(), 1);
    }

    #[test]
    fn test_bookmark_remove() {
        let mut collection = BookmarkCollection::new();
        collection.add("trail-1");
        assert!(collection.remove("trail-1"));
        assert!(!collection.is_bookmarked("trail-1"));
        assert_eq!(collection.count(), 0);
    }

    #[test]
    fn test_bookmark_toggle() {
        let mut collection = BookmarkCollection::new();

        // Toggle on
        assert!(collection.toggle("trail-1"));
        assert!(collection.is_bookmarked("trail-1"));

        // Toggle off
        assert!(!collection.toggle("trail-1"));
        assert!(!collection.is_bookmarked("trail-1"));
    }

    #[test]
    fn test_bookmark_clear() {
        let mut collection = BookmarkCollection::new();
        collection.add("trail-1");
        collection.add("trail-2");
        assert_eq!(collection.count(), 2);

        collection.clear();
        assert_eq!(collection.count(), 0);
    }
}
