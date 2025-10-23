/// Utility functions for tenant-scoped storage keys
///
/// Provides helpers to generate tenant-specific storage keys for bookmarks and settings,
/// ensuring data isolation between tenants while maintaining backward compatibility.

/// Base key for global bookmarks
const BOOKMARKS_BASE_KEY: &str = "bookmarks";

/// Base key for global preferences
const PREFERENCES_BASE_KEY: &str = "preferences";

/// Generate a tenant-scoped bookmark storage key
///
/// # Arguments
/// * `tenant_id` - Optional tenant identifier
///
/// # Returns
/// * `"bookmarks"` - For global/no-tenant bookmarks
/// * `"bookmarks.{tenant_id}"` - For tenant-specific bookmarks
pub fn get_bookmark_key(tenant_id: Option<&str>) -> String {
    match tenant_id {
        Some(id) if !id.is_empty() => format!("{}.{}", BOOKMARKS_BASE_KEY, id),
        _ => BOOKMARKS_BASE_KEY.to_string(),
    }
}

/// Generate a tenant-scoped preferences storage key
///
/// # Arguments
/// * `tenant_id` - Optional tenant identifier
///
/// # Returns
/// * `"preferences"` - For global/default preferences
/// * `"preferences.{tenant_id}"` - For tenant-specific preferences
pub fn get_preferences_key(tenant_id: Option<&str>) -> String {
    match tenant_id {
        Some(id) if !id.is_empty() => format!("{}.{}", PREFERENCES_BASE_KEY, id),
        _ => PREFERENCES_BASE_KEY.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_key_no_tenant() {
        assert_eq!(get_bookmark_key(None), "bookmarks");
    }

    #[test]
    fn test_bookmark_key_empty_tenant() {
        assert_eq!(get_bookmark_key(Some("")), "bookmarks");
    }

    #[test]
    fn test_bookmark_key_with_tenant() {
        assert_eq!(get_bookmark_key(Some("tenant-123")), "bookmarks.tenant-123");
    }

    #[test]
    fn test_preferences_key_no_tenant() {
        assert_eq!(get_preferences_key(None), "preferences");
    }

    #[test]
    fn test_preferences_key_empty_tenant() {
        assert_eq!(get_preferences_key(Some("")), "preferences");
    }

    #[test]
    fn test_preferences_key_with_tenant() {
        assert_eq!(get_preferences_key(Some("tenant-456")), "preferences.tenant-456");
    }
}
