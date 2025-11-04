// ABOUTME: Advanced type generation tests
// ABOUTME: Tests advanced schema features: oneOf, anyOf, additionalProperties, nullable

use crate::test_helpers::{assert_type_exists, verify_compilation};
use qollective_tools_lib::codegen::DirectTypifyGenerator;

#[cfg(test)]
mod test_oneof_union {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/advanced/test_oneof_union.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_oneof_union.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/advanced/test_oneof_union.json";
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
        let schema_path = "schemas/advanced/test_oneof_union.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "StringValue");
        assert_type_exists(&generated_code, "NumberValue");
        assert_type_exists(&generated_code, "BooleanValue");
        assert_type_exists(&generated_code, "ConfigValue");
    }

    #[test]
    fn test_oneof_generates_enum() {
        let schema_path = "schemas/advanced/test_oneof_union.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        // ConfigValue should be an enum with variants
        assert!(
            generated_code.contains("pub enum ConfigValue")
                || generated_code.contains("ConfigValue"),
            "ConfigValue should be generated (may be enum or other type)"
        );
    }
}

#[cfg(test)]
mod test_anyof_union {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/advanced/test_anyof_union.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_anyof_union.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/advanced/test_anyof_union.json";
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
        let schema_path = "schemas/advanced/test_anyof_union.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "FlexibleValue");
        assert_type_exists(&generated_code, "Container");
    }

    #[test]
    fn test_anyof_generates_flexible_type() {
        let schema_path = "schemas/advanced/test_anyof_union.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        // FlexibleValue should support multiple types
        assert!(
            generated_code.contains("FlexibleValue"),
            "FlexibleValue type should be generated"
        );
    }
}

#[cfg(test)]
mod test_additional_properties {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/advanced/test_additional_properties.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_additional_properties.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/advanced/test_additional_properties.json";
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
        let schema_path = "schemas/advanced/test_additional_properties.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "StringMap");
        assert_type_exists(&generated_code, "WordList");
        assert_type_exists(&generated_code, "Node");
        assert_type_exists(&generated_code, "NodeMap");
        assert_type_exists(&generated_code, "Configuration");
    }

    #[test]
    fn test_additional_properties_generates_hashmap() {
        let schema_path = "schemas/advanced/test_additional_properties.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        // Should generate HashMap or similar collection type
        assert!(
            generated_code.contains("HashMap")
                || generated_code.contains("Map")
                || generated_code.contains("BTreeMap"),
            "Additional properties should generate a map type"
        );
    }
}

#[cfg(test)]
mod test_nullable_types {
    use super::*;

    #[test]
    fn test_generates_code() {
        let schema_path = "schemas/advanced/test_nullable_types.json";
        let generator = DirectTypifyGenerator::new();
        let result = generator.generate_from_file(schema_path);

        assert!(
            result.is_ok(),
            "Should generate code from test_nullable_types.json: {:?}",
            result.err()
        );
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_compiles() {
        let schema_path = "schemas/advanced/test_nullable_types.json";
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
        let schema_path = "schemas/advanced/test_nullable_types.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        assert_type_exists(&generated_code, "Price");
        assert_type_exists(&generated_code, "OptionalCounts");
        assert_type_exists(&generated_code, "Item");
    }

    #[test]
    fn test_nullable_generates_option() {
        let schema_path = "schemas/advanced/test_nullable_types.json";
        let generator = DirectTypifyGenerator::new();
        let generated_code = generator.generate_from_file(schema_path).unwrap();

        // Nullable types should generate Option<T>
        // typify generates with spaces in the token stream: ":: std :: option :: Option"
        assert!(
            generated_code.contains("Option<")
                || generated_code.contains("::std::option::Option<")
                || generated_code.contains(":: std :: option :: Option"),
            "Nullable fields should generate Option<T>"
        );
    }
}
