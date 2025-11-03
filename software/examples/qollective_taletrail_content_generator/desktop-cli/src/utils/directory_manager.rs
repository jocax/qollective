/// Directory management utilities
///
/// Provides directory scanning, filtering, and pagination support

use crate::error::AppError;
use smol::fs;
use smol::stream::StreamExt;
use std::path::{Path, PathBuf};

/// Scan a directory and return all file paths, optionally filtered
///
/// # Arguments
/// * `path` - Directory path to scan
/// * `filter` - Optional filter function to apply to paths
///
/// # Returns
/// * `Ok(Vec<PathBuf>)` - List of file paths matching filter
/// * `Err(AppError)` - If directory cannot be read
pub async fn scan_directory<F>(path: &Path, filter: Option<F>) -> Result<Vec<PathBuf>, AppError>
where
    F: Fn(&Path) -> bool,
{
    if !path.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Directory does not exist: {}", path.display()),
        )));
    }

    if !path.is_dir() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a directory: {}", path.display()),
        )));
    }

    let mut entries = fs::read_dir(path).await?;
    let mut paths = Vec::new();

    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let entry_path = entry.path();

        // Apply filter if provided
        if let Some(ref filter_fn) = filter {
            if !filter_fn(&entry_path) {
                continue;
            }
        }

        paths.push(entry_path);
    }

    // Sort paths for consistent ordering
    paths.sort();

    Ok(paths)
}

/// Paginate a list of items
///
/// # Arguments
/// * `items` - List of items to paginate
/// * `page` - Page number (0-indexed)
/// * `page_size` - Number of items per page
///
/// # Returns
/// * `Vec<T>` - Items for the requested page
pub fn paginate<T: Clone>(items: Vec<T>, page: usize, page_size: usize) -> Vec<T> {
    let start = page * page_size;
    let end = (start + page_size).min(items.len());

    if start >= items.len() {
        return Vec::new();
    }

    items[start..end].to_vec()
}

/// Calculate total number of pages for a given item count and page size
///
/// # Arguments
/// * `total_items` - Total number of items
/// * `page_size` - Number of items per page
///
/// # Returns
/// * `usize` - Total number of pages (at least 1)
pub fn calculate_total_pages(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 {
        return 1;
    }
    (total_items + page_size - 1) / page_size
}

/// Filter paths by file extension
///
/// # Arguments
/// * `path` - Path to check
/// * `extension` - Extension to match (e.g., "json")
///
/// # Returns
/// * `bool` - True if path has the specified extension
pub fn filter_by_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s == extension)
        .unwrap_or(false)
}

/// Filter paths to only include files (not directories)
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `bool` - True if path is a file
pub fn filter_files_only(path: &Path) -> bool {
    path.is_file()
}

/// Filter paths to only include directories
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `bool` - True if path is a directory
pub fn filter_directories_only(path: &Path) -> bool {
    path.is_dir()
}
