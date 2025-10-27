//! Markdown response parser for LLM-generated content
//!
//! Provides utilities to extract structured sections from Markdown-formatted
//! LLM responses using standard ## headers.

use regex::Regex;

/// Markdown response parser for extracting sections by header
pub struct MarkdownResponseParser;

impl MarkdownResponseParser {
    /// Extract section content by header name
    ///
    /// # Arguments
    /// * `markdown` - The full Markdown response
    /// * `header` - Header name to find (without ## prefix)
    ///
    /// # Returns
    /// Section content if found, None otherwise
    ///
    /// # Example
    /// ```
    /// use shared_types::MarkdownResponseParser;
    /// let md = "## System Prompt\nYou are helpful\n## User Prompt\nGenerate story";
    /// let system = MarkdownResponseParser::extract_section(md, "System Prompt");
    /// assert_eq!(system, Some("You are helpful".to_string()));
    /// ```
    pub fn extract_section(markdown: &str, header: &str) -> Option<String> {
        // Find the header line
        let header_marker = format!("## {}", header);
        let start_idx = markdown.find(&header_marker)?;

        // Find the start of content (after the newline following the header)
        let content_start = markdown[start_idx..].find('\n')? + start_idx + 1;

        // Find the next ## header or end of string
        let content_slice = &markdown[content_start..];
        let end_idx = content_slice.find("\n##")
            .unwrap_or(content_slice.len());

        let content = &content_slice[..end_idx];
        Some(content.trim().to_string())
    }

    /// Extract numbered list items from a section
    ///
    /// # Arguments
    /// * `section` - Section content containing numbered list
    ///
    /// # Returns
    /// Vector of list item texts (without numbers)
    ///
    /// # Example
    /// ```
    /// use shared_types::MarkdownResponseParser;
    /// let section = "1. First choice\n2. Second choice\n3. Third choice";
    /// let items = MarkdownResponseParser::extract_numbered_list(section);
    /// assert_eq!(items, vec!["First choice", "Second choice", "Third choice"]);
    /// ```
    pub fn extract_numbered_list(section: &str) -> Vec<String> {
        // Pattern: 1. Item text\n2. Item text\n3. Item text
        let re = Regex::new(r"(?m)^\d+\.\s*(.+)$").unwrap();
        re.captures_iter(section)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().trim().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_section_simple() {
        let md = "## System Prompt\nYou are helpful\n## User Prompt\nGenerate story";
        let system = MarkdownResponseParser::extract_section(md, "System Prompt");
        assert_eq!(system, Some("You are helpful".to_string()));
    }

    #[test]
    fn test_extract_section_multiline() {
        let md = "## Narrative\nLine 1\nLine 2\nLine 3\n## Choices\nChoice text";
        let narrative = MarkdownResponseParser::extract_section(md, "Narrative");
        assert_eq!(narrative, Some("Line 1\nLine 2\nLine 3".to_string()));
    }

    #[test]
    fn test_extract_section_missing() {
        let md = "## System Prompt\nContent here";
        let result = MarkdownResponseParser::extract_section(md, "User Prompt");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_numbered_list() {
        let section = "1. First choice\n2. Second choice\n3. Third choice";
        let items = MarkdownResponseParser::extract_numbered_list(section);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "First choice");
        assert_eq!(items[1], "Second choice");
        assert_eq!(items[2], "Third choice");
    }

    #[test]
    fn test_extract_numbered_list_with_extra_text() {
        let section = "Here are the choices:\n1. Go left\n2. Go right\n3. Stay here\nEnd of list";
        let items = MarkdownResponseParser::extract_numbered_list(section);
        assert_eq!(items.len(), 3);
    }
}
