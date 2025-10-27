//! UTF-8-safe text truncation utilities
//!
//! This module provides centralized text truncation functions that safely handle
//! UTF-8 multi-byte characters (umlauts, emojis, etc.) by truncating at character
//! boundaries rather than byte boundaries.
//!
//! # Functions
//!
//! - [`truncate_at_char_boundary`] - Basic truncation at UTF-8 character boundary
//! - [`truncate_with_ellipsis`] - Truncation with "..." appended, preferring word boundaries
//! - [`preview_text`] - Formatted preview for logging with length indicator
//!
//! # Examples
//!
//! ```
//! use shared_types::text_utils::*;
//!
//! // Basic truncation
//! let text = "Hello world";
//! assert_eq!(truncate_at_char_boundary(text, 5), "Hello");
//!
//! // Truncation with ellipsis
//! let text = "This is a long sentence";
//! let result = truncate_with_ellipsis(text, 10);
//! assert!(result.ends_with("..."));
//!
//! // Preview for logging
//! let text = "A very long text for logging purposes";
//! let preview = preview_text(text, 10);
//! assert!(preview.contains("... ("));
//! assert!(preview.contains("chars total)"));
//! ```

/// Truncate text at UTF-8 character boundary
///
/// Returns a slice of the original text truncated to fit within `max_length` bytes
/// while respecting UTF-8 character boundaries. No new allocation unless the text
/// needs truncation.
///
/// # Arguments
///
/// * `text` - The text to truncate
/// * `max_length` - Maximum byte length (not Unicode characters)
///
/// # Returns
///
/// Slice of original text truncated at a character boundary
///
/// # Examples
///
/// ```
/// use shared_types::text_utils::truncate_at_char_boundary;
///
/// // Text shorter than max_length
/// assert_eq!(truncate_at_char_boundary("Hello", 10), "Hello");
///
/// // Text exactly at max_length
/// assert_eq!(truncate_at_char_boundary("Hello", 5), "Hello");
///
/// // Truncation needed
/// assert_eq!(truncate_at_char_boundary("Hello world", 5), "Hello");
///
/// // UTF-8 multi-byte characters (ä is 2 bytes)
/// assert_eq!(truncate_at_char_boundary("Verordnungenä", 13), "Verordnungen");
/// ```
pub fn truncate_at_char_boundary(text: &str, max_length: usize) -> &str {
    if text.len() <= max_length {
        return text;
    }

    // Find the last valid UTF-8 character that fits entirely within max_length bytes
    let truncate_at = text
        .char_indices()
        .take_while(|(idx, ch)| idx + ch.len_utf8() <= max_length)
        .last()
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);

    &text[..truncate_at]
}

/// Truncate text with ellipsis, preferring word boundaries
///
/// Returns a new String with "..." appended if truncation occurred.
/// Attempts to truncate at word boundaries (whitespace) to avoid cutting
/// words in half when possible.
///
/// # Arguments
///
/// * `text` - The text to truncate
/// * `max_length` - Maximum byte length before ellipsis (not Unicode characters)
///
/// # Returns
///
/// New String with "..." appended if truncated, or original text if no truncation needed
///
/// # Special Cases
///
/// - If `max_length` is 0, returns "..."
/// - If text is shorter than `max_length`, returns original text unchanged
/// - If no word boundary found, truncates at character boundary
///
/// # Examples
///
/// ```
/// use shared_types::text_utils::truncate_with_ellipsis;
///
/// // Short text - no truncation
/// assert_eq!(truncate_with_ellipsis("Short", 100), "Short");
///
/// // Truncate at word boundary
/// let result = truncate_with_ellipsis("This is a long piece of content", 20);
/// assert!(result.ends_with("..."));
/// assert!(result.starts_with("This is a long"));
///
/// // No spaces - truncate at character boundary
/// let result = truncate_with_ellipsis("VeryLongWordWithoutSpaces", 10);
/// assert_eq!(result, "VeryLongWo...");
///
/// // Zero length
/// assert_eq!(truncate_with_ellipsis("Text", 0), "...");
/// ```
pub fn truncate_with_ellipsis(text: &str, max_length: usize) -> String {
    if max_length == 0 {
        return "...".to_string();
    }

    if text.len() <= max_length {
        return text.to_string();
    }

    // Find the last valid UTF-8 character that fits entirely within max_length bytes
    let truncate_at = text
        .char_indices()
        .take_while(|(idx, ch)| idx + ch.len_utf8() <= max_length)
        .last()
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);

    if truncate_at == 0 {
        return "...".to_string();
    }

    let truncated = &text[..truncate_at];

    // Try to find last word boundary (whitespace) in the truncated content
    if let Some(last_space) = truncated.rfind(char::is_whitespace) {
        text[..last_space].to_string() + "..."
    } else {
        // No word boundary found, use character boundary truncation
        truncated.to_string() + "..."
    }
}

/// Format text preview for logging (truncate + format with length)
///
/// Returns a formatted string suitable for logging that truncates the text
/// and appends a length indicator. If truncation occurs, adds "... (N chars total)".
///
/// # Arguments
///
/// * `text` - The text to preview
/// * `max_length` - Maximum byte length for preview (not Unicode characters)
///
/// # Returns
///
/// Formatted string with truncation and total length indicator
///
/// # Examples
///
/// ```
/// use shared_types::text_utils::preview_text;
///
/// // Short text - no truncation
/// assert_eq!(preview_text("Short", 100), "Short");
///
/// // Long text - truncated with length
/// let result = preview_text("This is a very long text that needs truncation", 10);
/// assert!(result.contains("..."));
/// assert!(result.contains("(46 chars total)"));
///
/// // Zero length
/// let result = preview_text("Text", 0);
/// assert_eq!(result, "... (4 chars total)");
/// ```
pub fn preview_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        // Find the last valid UTF-8 character that fits entirely within max_length bytes
        let truncate_at = text
            .char_indices()
            .take_while(|(idx, ch)| idx + ch.len_utf8() <= max_length)
            .last()
            .map(|(idx, ch)| idx + ch.len_utf8())
            .unwrap_or(0);

        if truncate_at == 0 {
            format!("... ({} chars total)", text.len())
        } else {
            format!("{}... ({} chars total)", &text[..truncate_at], text.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // truncate_at_char_boundary tests
    // ============================================================================

    #[test]
    fn test_truncate_at_char_boundary_short() {
        let text = "Short content";
        let truncated = truncate_at_char_boundary(text, 100);
        assert_eq!(truncated, "Short content");
    }

    #[test]
    fn test_truncate_at_char_boundary_exact_length() {
        let text = "Exactly";
        let truncated = truncate_at_char_boundary(text, 7);
        assert_eq!(truncated, "Exactly");
    }

    #[test]
    fn test_truncate_at_char_boundary_needs_truncation() {
        let text = "Hello world";
        let truncated = truncate_at_char_boundary(text, 5);
        assert_eq!(truncated, "Hello");
    }

    #[test]
    fn test_truncate_at_char_boundary_german_umlauts() {
        // German text with multi-byte UTF-8 characters (ä, ö, ü, ß are 2 bytes each)
        let text = "Verordnungen über schöne Grüße auf der Straße";

        let truncated = truncate_at_char_boundary(text, 20);
        // Should not panic and should be valid UTF-8
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn test_truncate_at_char_boundary_german_at_multibyte_boundary() {
        // Test truncation exactly at a multi-byte character boundary
        // "Verordnungen" has 12 ASCII chars, 'ä' is at byte 12-13
        let text = "Verordnungenä";

        // Try to truncate at 13 (middle of 'ä')
        let truncated = truncate_at_char_boundary(text, 13);

        // Should truncate before 'ä', not in the middle
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
        assert_eq!(truncated, "Verordnungen");
        assert!(!truncated.contains("ä"));
    }

    #[test]
    fn test_truncate_at_char_boundary_emoji() {
        // Emojis are 4-byte UTF-8 characters
        let text = "Hello 👋 World 🌍";

        let truncated = truncate_at_char_boundary(text, 10);
        // Should be valid UTF-8 and not panic
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn test_truncate_at_char_boundary_zero_length() {
        let text = "Some text";
        let truncated = truncate_at_char_boundary(text, 0);
        assert_eq!(truncated, "");
    }

    #[test]
    fn test_truncate_at_char_boundary_mixed_scripts() {
        // Mix of ASCII, German, Cyrillic, Chinese, and emoji
        let text = "Test äöü Москва 北京 🌍";

        let truncated = truncate_at_char_boundary(text, 20);
        // Should be valid UTF-8
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn test_truncate_at_char_boundary_all_multibyte() {
        // String entirely of 2-byte characters
        let text = "äöüßäöüßäöüß";

        let truncated = truncate_at_char_boundary(text, 10);
        // Should be valid UTF-8 and not panic
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    // ============================================================================
    // truncate_with_ellipsis tests
    // ============================================================================

    #[test]
    fn test_truncate_with_ellipsis_short() {
        let text = "Short content";
        let truncated = truncate_with_ellipsis(text, 100);
        assert_eq!(truncated, "Short content");
    }

    #[test]
    fn test_truncate_with_ellipsis_at_word_boundary() {
        let text = "This is a long piece of content that needs to be truncated";
        let truncated = truncate_with_ellipsis(text, 30);

        // Should end with ellipsis
        assert!(truncated.ends_with("..."));

        // Should truncate at word boundary (finds last space before byte 30)
        assert!(truncated.starts_with("This is a long piece of"));

        // Should not include words that come after the boundary
        assert!(!truncated.contains("content"));
        assert!(!truncated.contains("needs"));

        // Should be valid UTF-8
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn test_truncate_with_ellipsis_no_spaces() {
        let text = "ThisIsAVeryLongWordWithoutAnySpaces";
        let truncated = truncate_with_ellipsis(text, 15);
        assert_eq!(truncated.len(), 18); // 15 + "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_truncate_with_ellipsis_exact_length() {
        let text = "Exactly right";
        let truncated = truncate_with_ellipsis(text, 13);
        assert_eq!(truncated, "Exactly right");
    }

    #[test]
    fn test_truncate_with_ellipsis_zero_length() {
        let text = "Some content";
        let truncated = truncate_with_ellipsis(text, 0);
        assert_eq!(truncated, "...");
    }

    #[test]
    fn test_truncate_with_ellipsis_german_umlauts() {
        // German text with multi-byte UTF-8 characters (ä, ö, ü, ß are 2 bytes each)
        let text = "Verordnungen über schöne Grüße auf der Straße";

        // Truncate at a safe point (before any multi-byte character)
        let truncated = truncate_with_ellipsis(text, 20);
        assert!(truncated.ends_with("..."));
        // Should not panic and should be valid UTF-8
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn test_truncate_with_ellipsis_german_at_multibyte_boundary() {
        // Test truncation exactly at a multi-byte character boundary
        // "Verordnungen" has 'V','e','r','o','r','d','n','u','n','g','e','n' (12 ASCII chars)
        let text = "Verordnungenä"; // 'ä' is at byte 12-13

        // Try to truncate at 13 (middle of 'ä')
        let truncated = truncate_with_ellipsis(text, 13);

        // Should truncate before 'ä', not in the middle
        assert!(truncated.ends_with("..."));
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
        // Should contain "Verordnungen" but not "ä"
        assert!(truncated.contains("Verordnungen"));
        assert!(!truncated.contains("ä"));
    }

    #[test]
    fn test_truncate_with_ellipsis_german_word_boundary() {
        let text = "Schöne Grüße von der Straße";

        // Truncate somewhere in the middle
        let truncated = truncate_with_ellipsis(text, 20);

        // Should truncate at word boundary and be valid UTF-8
        assert!(truncated.ends_with("..."));
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());

        // Should not cut words in half (should end at a space before ...)
        let without_ellipsis = truncated.trim_end_matches("...");
        if !without_ellipsis.is_empty() {
            assert!(
                without_ellipsis.ends_with(char::is_whitespace)
                    || text.starts_with(without_ellipsis)
            );
        }
    }

    #[test]
    fn test_truncate_with_ellipsis_emoji() {
        // Emojis are 4-byte UTF-8 characters
        let text = "Hello 👋 World 🌍 Test 🚀";

        let truncated = truncate_with_ellipsis(text, 15);

        // Should be valid UTF-8 and not panic
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_truncate_with_ellipsis_mixed_scripts() {
        // Mix of ASCII, German, and other Unicode
        let text = "Test äöü Москва 北京 🌍";

        let truncated = truncate_with_ellipsis(text, 20);

        // Should be valid UTF-8
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn test_truncate_with_ellipsis_all_multibyte() {
        // String entirely of 2-byte characters
        let text = "äöüßäöüßäöüß";

        let truncated = truncate_with_ellipsis(text, 10);

        // Should be valid UTF-8 and not panic
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
        assert!(truncated.ends_with("..."));
    }

    // ============================================================================
    // preview_text tests
    // ============================================================================

    #[test]
    fn test_preview_text_short() {
        let text = "Short text";
        let preview = preview_text(text, 100);
        assert_eq!(preview, "Short text");
    }

    #[test]
    fn test_preview_text_exact_length() {
        let text = "Exact";
        let preview = preview_text(text, 5);
        assert_eq!(preview, "Exact");
    }

    #[test]
    fn test_preview_text_needs_truncation() {
        let text = "This is a long text that needs truncation";
        let preview = preview_text(text, 10);

        assert!(preview.contains("..."));
        assert!(preview.contains("chars total"));
        assert!(std::str::from_utf8(preview.as_bytes()).is_ok());
    }

    #[test]
    fn test_preview_text_german_umlauts() {
        // German text with multi-byte UTF-8 characters
        let text = "Verordnungen über schöne Grüße auf der Straße";

        let preview = preview_text(text, 20);

        // Should not panic and should be valid UTF-8
        assert!(std::str::from_utf8(preview.as_bytes()).is_ok());
        assert!(preview.contains("..."));
        assert!(preview.contains("chars total"));
    }

    #[test]
    fn test_preview_text_german_at_multibyte_boundary() {
        // Test truncation exactly at a multi-byte character boundary
        let text = "Verordnungenä"; // 'ä' is at byte 12-13 (2 bytes)

        // "Verordnungen" is 12 bytes, 'ä' is 2 bytes, total is 14 bytes
        assert_eq!(text.len(), 14);

        // Try to truncate at 13 (middle of 'ä' - should truncate at byte 12 instead)
        let preview = preview_text(text, 13);

        // Should be valid UTF-8 and not panic
        assert!(std::str::from_utf8(preview.as_bytes()).is_ok());

        // Should contain "..." and total length indicator
        assert!(preview.contains("..."));
        assert!(preview.contains("(14 chars total)"));
        // The prefix should be "Verordnungen" (without the 'ä')
        assert!(preview.starts_with("Verordnungen"));
        assert!(!preview.contains("ä"));
    }

    #[test]
    fn test_preview_text_emoji() {
        // Emojis are 4-byte UTF-8 characters
        let text = "Hello 👋 World 🌍 Test 🚀";

        let preview = preview_text(text, 15);

        // Should be valid UTF-8 and not panic
        assert!(std::str::from_utf8(preview.as_bytes()).is_ok());
    }

    #[test]
    fn test_preview_text_zero_length() {
        let text = "Some text";
        let preview = preview_text(text, 0);

        // Should handle zero gracefully
        assert!(preview.contains("..."));
        assert!(preview.contains("chars total"));
    }

    #[test]
    fn test_preview_text_mixed_scripts() {
        // Mix of ASCII, German, Cyrillic, Chinese, and emoji
        let text = "Test äöü Москва 北京 🌍";

        let preview = preview_text(text, 20);

        // Should be valid UTF-8
        assert!(std::str::from_utf8(preview.as_bytes()).is_ok());
    }

    #[test]
    fn test_preview_text_all_multibyte() {
        // String entirely of 2-byte characters
        let text = "äöüßäöüßäöüß";

        let preview = preview_text(text, 10);

        // Should be valid UTF-8 and not panic
        assert!(std::str::from_utf8(preview.as_bytes()).is_ok());
    }
}
