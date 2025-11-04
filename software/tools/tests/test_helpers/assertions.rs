// ABOUTME: Reusable assertion functions for testing generated code
// ABOUTME: Provides helpers for verifying types, derives, and field properties

/// Check if generated code contains a specific type
pub fn assert_type_exists(code: &str, type_name: &str) {
    assert!(
        code.contains(&format!("pub enum {}", type_name))
            || code.contains(&format!("pub struct {}", type_name)),
        "Generated code should contain type '{}'",
        type_name
    );
}

/// Check if a type has specific derive attributes
pub fn assert_has_derives(code: &str, type_name: &str, derives: &[&str]) {
    for derive in derives {
        assert!(
            code.contains(derive),
            "Type '{}' should have derive '{}'",
            type_name,
            derive
        );
    }
}

/// Verify that a specific integer type was selected for a field
pub fn assert_integer_type(code: &str, struct_name: &str, field_name: &str, expected_type: &str) {
    // Look for pattern: "pub field_name : expected_type" (typify generates with spaces)
    let pattern = format!("pub {} : {}", field_name, expected_type);
    assert!(
        code.contains(&pattern),
        "Struct '{}' should have field '{}' with type '{}', but pattern '{}' not found.\nGenerated code snippet: {}",
        struct_name, field_name, expected_type, pattern,
        code.chars().take(500).collect::<String>()
    );
}

/// Count occurrences of a derive attribute
pub fn count_derives(code: &str, derive_name: &str) -> usize {
    code.matches(derive_name).count()
}

/// Verify enum variants exist
pub fn assert_enum_variants(code: &str, enum_name: &str, variants: &[&str]) {
    for variant in variants {
        assert!(
            code.contains(variant),
            "Enum '{}' should have variant '{}'",
            enum_name,
            variant
        );
    }
}
