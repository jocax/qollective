// ABOUTME: Basic type generation tests
// ABOUTME: Tests simple schemas for enums, structs, and validation

use crate::test_helpers::{assert_integer_type, assert_type_exists, verify_compilation};
use qollective_tools_lib::codegen::DirectTypifyGenerator;

#[cfg(test)]
mod test_numeric_validation {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/basic/test_numeric_validation.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_numeric_validation.json: {:?}",
            result.err()
        );

        let generated_code = result.unwrap();
        assert!(
            !generated_code.is_empty(),
            "Generated code should not be empty"
        );
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/basic/test_numeric_validation.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        let compilation_result = verify_compilation(&generated_code);
        assert!(
            compilation_result.is_ok(),
            "Generated code should compile: {}",
            compilation_result.unwrap_err()
        );
    }

    #[test]
    fn test_expected_types() {
        let schema_path = "schemas/basic/test_numeric_validation.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "Percentage");
        assert_type_exists(&generated_code, "Port");
        assert_type_exists(&generated_code, "SmallCount");
        assert_type_exists(&generated_code, "Temperature");
    }

    #[test]
    fn test_u8_selected_for_percentage() {
        let schema_path = "schemas/basic/test_numeric_validation.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_integer_type(&generated_code, "Percentage", "value", "u8");
    }

    #[test]
    fn test_u16_selected_for_port() {
        let schema_path = "schemas/basic/test_numeric_validation.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_integer_type(&generated_code, "Port", "number", "u16");
    }

    #[test]
    fn test_u8_selected_for_count() {
        let schema_path = "schemas/basic/test_numeric_validation.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_integer_type(&generated_code, "SmallCount", "count", "u8");
    }

    #[test]
    fn test_i16_selected_for_temperature() {
        let schema_path = "schemas/basic/test_numeric_validation.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_integer_type(&generated_code, "Temperature", "celsius", "i16");
    }
}

#[cfg(test)]
mod test_simple_enum {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/basic/test_simple_enum.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_simple_enum.json"
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/basic/test_simple_enum.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        let compilation_result = verify_compilation(&generated_code);
        assert!(
            compilation_result.is_ok(),
            "Generated code should compile: {}",
            compilation_result.unwrap_err()
        );
    }

    #[test]
    fn test_expected_types() {
        let schema_path = "schemas/basic/test_simple_enum.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "Status");
        assert!(generated_code.contains("active"));
        assert!(generated_code.contains("inactive"));
        assert!(generated_code.contains("pending"));
    }
}

#[cfg(test)]
mod test_nested_types {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/basic/test_nested_types.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_nested_types.json"
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/basic/test_nested_types.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        let compilation_result = verify_compilation(&generated_code);
        assert!(
            compilation_result.is_ok(),
            "Generated code should compile: {}",
            compilation_result.unwrap_err()
        );
    }

    #[test]
    fn test_expected_types() {
        let schema_path = "schemas/basic/test_nested_types.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "Status");
        assert_type_exists(&generated_code, "Metadata");
        assert_type_exists(&generated_code, "Item");
    }

    #[test]
    fn test_nested_refs_resolved() {
        let schema_path = "schemas/basic/test_nested_types.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        // Item should reference Status and Metadata (typify uses spaces around colons)
        assert!(
            generated_code.contains("status : Status"),
            "Item should have field 'status : Status'"
        );
        assert!(
            generated_code.contains("metadata : :: std :: option :: Option < Metadata >"),
            "Item should have optional field 'metadata : Option<Metadata>'"
        );
    }
}

#[cfg(test)]
mod test_numeric_enum {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/basic/test_numeric_enum.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_numeric_enum.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/basic/test_numeric_enum.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        let compilation_result = verify_compilation(&generated_code);
        assert!(
            compilation_result.is_ok(),
            "Generated code should compile: {}",
            compilation_result.unwrap_err()
        );
    }

    #[test]
    fn test_expected_types() {
        let schema_path = "schemas/basic/test_numeric_enum.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "AgeRange");
        // Check for special enum variant handling - typify may handle numeric prefixes
        assert!(
            generated_code.contains("X05")
                || generated_code.contains("_0_5")
                || generated_code.contains("0-5")
                || generated_code.contains("X0_5"),
            "Should handle '0-5' enum value with some naming strategy"
        );
    }
}

#[cfg(test)]
mod test_simple_struct {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/basic/test_simple_struct.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_simple_struct.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/basic/test_simple_struct.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        let compilation_result = verify_compilation(&generated_code);
        assert!(
            compilation_result.is_ok(),
            "Generated code should compile: {}",
            compilation_result.unwrap_err()
        );
    }

    #[test]
    fn test_expected_types() {
        let schema_path = "schemas/basic/test_simple_struct.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "Person");
        assert!(generated_code.contains("name"), "Should have 'name' field");
        assert!(generated_code.contains("age"), "Should have 'age' field");
    }

    #[test]
    fn test_primitive_types_present() {
        let schema_path = "schemas/basic/test_simple_struct.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        // Should have string field
        assert!(
            generated_code.contains("String")
                || generated_code.contains(":: std :: string :: String"),
            "Should have String type for name"
        );
        // Should have boolean field
        assert!(
            generated_code.contains("bool"),
            "Should have bool type for active"
        );
        // Should have numeric fields
        assert!(
            generated_code.contains("i64")
                || generated_code.contains("u8")
                || generated_code.contains("i32"),
            "Should have integer type for age"
        );
    }
}

#[cfg(test)]
mod test_struct_with_required_optional {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/basic/test_struct_with_required_optional.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_struct_with_required_optional.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/basic/test_struct_with_required_optional.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        let compilation_result = verify_compilation(&generated_code);
        assert!(
            compilation_result.is_ok(),
            "Generated code should compile: {}",
            compilation_result.unwrap_err()
        );
    }

    #[test]
    fn test_expected_types() {
        let schema_path = "schemas/basic/test_struct_with_required_optional.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "Configuration");
        assert!(generated_code.contains("id"), "Should have 'id' field");
        assert!(generated_code.contains("name"), "Should have 'name' field");
        assert!(
            generated_code.contains("description"),
            "Should have 'description' field"
        );
    }

    #[test]
    fn test_optional_fields_generate_option() {
        let schema_path = "schemas/basic/test_struct_with_required_optional.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        // Optional fields should be Option<T>
        assert!(
            generated_code.contains("description : :: std :: option :: Option")
                || generated_code.contains("description: Option<")
                || generated_code.contains("description : Option"),
            "Optional field 'description' should be Option<T>"
        );

        assert!(
            generated_code.contains("enabled : :: std :: option :: Option")
                || generated_code.contains("enabled: Option<")
                || generated_code.contains("enabled : Option"),
            "Optional field 'enabled' should be Option<T>"
        );
    }
}

#[cfg(test)]
mod test_string_validation {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/basic/test_string_validation.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_string_validation.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/basic/test_string_validation.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        let compilation_result = verify_compilation(&generated_code);
        assert!(
            compilation_result.is_ok(),
            "Generated code should compile: {}",
            compilation_result.unwrap_err()
        );
    }

    #[test]
    fn test_expected_types() {
        let schema_path = "schemas/basic/test_string_validation.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "ValidatedText");
        assert!(
            generated_code.contains("short_code"),
            "Should have 'short_code' field"
        );
        assert!(
            generated_code.contains("description"),
            "Should have 'description' field"
        );
    }
}

#[cfg(test)]
mod test_uuid_fields {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/basic/test_uuid_fields.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_uuid_fields.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/basic/test_uuid_fields.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        let compilation_result = verify_compilation(&generated_code);
        assert!(
            compilation_result.is_ok(),
            "Generated code should compile: {}",
            compilation_result.unwrap_err()
        );
    }

    #[test]
    fn test_expected_types() {
        let schema_path = "schemas/basic/test_uuid_fields.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "Entity");
        assert!(generated_code.contains("id"), "Should have 'id' field");
        assert!(
            generated_code.contains("parent_id"),
            "Should have 'parent_id' field"
        );
    }

    #[test]
    fn test_uuid_type_generated() {
        let schema_path = "schemas/basic/test_uuid_fields.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert!(
            generated_code.contains("uuid :: Uuid")
                || generated_code.contains(":: uuid :: Uuid")
                || generated_code.contains("Uuid"),
            "UUID fields should use uuid::Uuid type or String"
        );
    }
}

#[cfg(test)]
mod test_datetime_fields {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/basic/test_datetime_fields.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_datetime_fields.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/basic/test_datetime_fields.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        let compilation_result = verify_compilation(&generated_code);
        assert!(
            compilation_result.is_ok(),
            "Generated code should compile: {}",
            compilation_result.unwrap_err()
        );
    }

    #[test]
    fn test_expected_types() {
        let schema_path = "schemas/basic/test_datetime_fields.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "Event");
        assert!(
            generated_code.contains("occurred_at"),
            "Should have 'occurred_at' field"
        );
        assert!(
            generated_code.contains("expires_at"),
            "Should have 'expires_at' field"
        );
    }

    #[test]
    fn test_datetime_type_generated() {
        let schema_path = "schemas/basic/test_datetime_fields.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert!(
            generated_code.contains("chrono ::")
                || generated_code.contains(":: chrono ::")
                || generated_code.contains("DateTime")
                || generated_code.contains("String"), // Typify may use String for date-time
            "DateTime fields should use chrono types or String"
        );
    }
}

#[cfg(test)]
mod test_array_fields {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/basic/test_array_fields.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_array_fields.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/basic/test_array_fields.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        let compilation_result = verify_compilation(&generated_code);
        assert!(
            compilation_result.is_ok(),
            "Generated code should compile: {}",
            compilation_result.unwrap_err()
        );
    }

    #[test]
    fn test_expected_types() {
        let schema_path = "schemas/basic/test_array_fields.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "TaggedItem");
        assert!(generated_code.contains("tags"), "Should have 'tags' field");
        assert!(
            generated_code.contains("scores"),
            "Should have 'scores' field"
        );
    }

    #[test]
    fn test_array_types_present() {
        let schema_path = "schemas/basic/test_array_fields.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert!(
            generated_code.contains("Vec") || generated_code.contains(":: std :: vec :: Vec"),
            "Should use Vec for array types"
        );
    }
}
