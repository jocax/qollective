// ABOUTME: Tests for the direct typify code generator
// ABOUTME: Focused on testing error handling and basic functionality

use super::direct_typify::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_from_file_with_invalid_file_returns_error() {
        // ARRANGE: Create a generator
        let generator = DirectTypifyGenerator::new();

        // ACT: Try to generate from non-existent file
        let result = generator.generate_from_file("/non/existent/file.json");

        // ASSERT: Should return an error
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, DirectTypifyError::Io(_)));
    }
}
