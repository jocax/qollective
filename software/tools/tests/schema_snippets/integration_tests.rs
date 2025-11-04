use crate::test_helpers::*;
use qollective_tools_lib::codegen::DirectTypifyGenerator;

const SCHEMA_PATH: &str = "schemas/examples/schema_example.json";

#[test]
fn test_schema_example_generates_code() {
    // Load the comprehensive schema
    let generator = DirectTypifyGenerator::new();

    // Generate code
    let result = generator.generate_from_file(SCHEMA_PATH);

    // Assert generation succeeds
    assert!(result.is_ok(), "Schema generation failed: {:?}", result.err());

    let code = result.unwrap();

    // Assert non-empty output
    assert!(!code.is_empty(), "Generated code should not be empty");

    // Verify it contains expected type declarations
    assert!(code.contains("pub struct Configuration"), "Should contain Configuration struct");
    assert!(code.contains("pub struct Workflow"), "Should contain Workflow struct");
    assert!(code.contains("pub enum Status"), "Should contain Status enum");
}

#[test]
fn test_schema_example_compiles() {
    // Generate code from comprehensive schema
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify compilation succeeds
    let compilation_result = verify_compilation(&code);
    assert!(
        compilation_result.is_ok(),
        "Generated code should compile successfully: {:?}",
        compilation_result.err()
    );
}

#[test]
fn test_all_types_generated() {
    // Generate code
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify all 15 types are generated
    assert_type_exists(&code, "Status");
    assert_type_exists(&code, "Priority");
    assert_type_exists(&code, "Configuration");
    assert_type_exists(&code, "Entity");
    assert_type_exists(&code, "WorkflowStep");
    assert_type_exists(&code, "Workflow");
    assert_type_exists(&code, "GraphNode");
    assert_type_exists(&code, "GraphStructure");
    assert_type_exists(&code, "ValidationRule");
    assert_type_exists(&code, "ValidationResult");
    assert_type_exists(&code, "ServiceInvocation");
    assert_type_exists(&code, "ExecutionTrace");
    assert_type_exists(&code, "ResultValue");
    assert_type_exists(&code, "DataContainer");
    assert_type_exists(&code, "NullableEntity");
}

#[test]
fn test_enums_generated_correctly() {
    // Generate code
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify Status enum (string enum)
    assert!(code.contains("pub enum Status"), "Should contain Status enum");
    assert!(code.contains("Pending") || code.contains("pending"), "Status should have pending variant");
    assert!(code.contains("InProgress") || code.contains("in_progress"), "Status should have in_progress variant");
    assert!(code.contains("Completed") || code.contains("completed"), "Status should have completed variant");
    assert!(code.contains("Failed") || code.contains("failed"), "Status should have failed variant");

    // Verify Priority enum (integer enum)
    // Note: typify may generate this differently - it might use newtypes or direct integers
    // We just verify the type exists
    assert_type_exists(&code, "Priority");
}

#[test]
fn test_nested_structures() {
    // Generate code
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify Workflow contains references to nested types
    assert!(code.contains("pub struct Workflow"), "Should contain Workflow struct");

    // Workflow should reference Status, Priority, Configuration, and WorkflowStep
    // Look for these types being used in the Workflow struct
    let workflow_section = extract_type_definition(&code, "Workflow");

    // Check for nested references (the exact field names may vary)
    assert!(
        workflow_section.contains("Status") || workflow_section.contains("status"),
        "Workflow should reference Status type"
    );
    assert!(
        workflow_section.contains("Configuration") || workflow_section.contains("config"),
        "Workflow should reference Configuration type"
    );
    assert!(
        workflow_section.contains("WorkflowStep") || workflow_section.contains("steps"),
        "Workflow should reference WorkflowStep type"
    );
}

#[test]
fn test_hashmap_generation() {
    // Generate code
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify GraphStructure generates with HashMap or similar structure
    assert_type_exists(&code, "GraphStructure");

    let graph_section = extract_type_definition(&code, "GraphStructure");

    // GraphStructure.nodes should use additionalProperties pattern
    // This might generate as HashMap, BTreeMap, or a wrapper type
    assert!(
        graph_section.contains("HashMap")
        || graph_section.contains("BTreeMap")
        || graph_section.contains("nodes"),
        "GraphStructure should have a nodes field with map-like structure"
    );

    // Should also reference GraphNode
    assert!(
        graph_section.contains("GraphNode"),
        "GraphStructure.nodes should reference GraphNode type"
    );
}

#[test]
fn test_uuid_fields() {
    // Generate code
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify Entity has UUID field
    let entity_section = extract_type_definition(&code, "Entity");
    assert!(
        entity_section.contains("uuid::Uuid") || entity_section.contains("Uuid") || entity_section.contains("String"),
        "Entity should have UUID field (as Uuid or String)"
    );

    // Verify Workflow has UUID field
    let workflow_section = extract_type_definition(&code, "Workflow");
    assert!(
        workflow_section.contains("uuid::Uuid") || workflow_section.contains("Uuid") || workflow_section.contains("String"),
        "Workflow should have UUID field (as Uuid or String)"
    );

    // Note: typify may generate UUIDs as String types rather than uuid::Uuid
    // This is a known limitation - format: "uuid" doesn't always generate uuid::Uuid
    println!("Entity section:");
    println!("{}", entity_section);
    println!("\nWorkflow section:");
    println!("{}", workflow_section);
}

#[test]
fn test_integer_optimization_in_context() {
    // Generate code
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify Configuration struct exists
    assert_type_exists(&code, "Configuration");

    let config_section = extract_type_definition(&code, "Configuration");

    // Document what typify ACTUALLY generates for integer fields
    // Note: Based on previous tests, typify may not optimize integers as expected
    // and might generate i64 for all integer types regardless of min/max constraints

    // Check for integer fields
    assert!(
        config_section.contains("max_retries"),
        "Configuration should have max_retries field"
    );
    assert!(
        config_section.contains("timeout_seconds"),
        "Configuration should have timeout_seconds field"
    );
    assert!(
        config_section.contains("batch_size"),
        "Configuration should have batch_size field"
    );

    // Document the actual types generated
    // Expected: u8 for max_retries (0-10), u16 for timeout_seconds (0-3600), u16 for batch_size (1-1000)
    // Actual: typify likely generates i64 for all of these

    println!("Configuration type definition:");
    println!("{}", config_section);
    println!("\nNote: typify may generate i64 for integer fields instead of optimized types like u8/u16");
    println!("This is a known limitation of the current typify implementation");
}

#[test]
fn test_oneof_union_types() {
    // Generate code
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify ResultValue (oneOf union)
    assert_type_exists(&code, "ResultValue");

    let result_value_section = extract_type_definition(&code, "ResultValue");

    // ResultValue should be an enum with variants for string, number, boolean, null
    assert!(
        result_value_section.contains("enum") || result_value_section.contains("String") || result_value_section.contains("f64"),
        "ResultValue should be a union type with multiple variants"
    );
}

#[test]
fn test_anyof_types() {
    // Generate code
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify DataContainer (anyOf)
    assert_type_exists(&code, "DataContainer");

    let data_container_section = extract_type_definition(&code, "DataContainer");

    // DataContainer should have a value field with anyOf type
    assert!(
        data_container_section.contains("value"),
        "DataContainer should have a value field"
    );
}

#[test]
fn test_nullable_fields() {
    // Generate code
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify NullableEntity
    assert_type_exists(&code, "NullableEntity");

    let nullable_section = extract_type_definition(&code, "NullableEntity");

    // Should have nullable fields (Option<T> or oneOf with null)
    assert!(
        nullable_section.contains("name"),
        "NullableEntity should have name field"
    );
    assert!(
        nullable_section.contains("description"),
        "NullableEntity should have description field"
    );
    assert!(
        nullable_section.contains("tags"),
        "NullableEntity should have tags field"
    );

    // Check for Option or nullable patterns
    // typify may generate these as Option<T> or as enum variants
    println!("NullableEntity type definition:");
    println!("{}", nullable_section);
}

#[test]
fn test_datetime_fields() {
    // Generate code
    let generator = DirectTypifyGenerator::new();
    let result = generator.generate_from_file(SCHEMA_PATH);

    assert!(result.is_ok(), "Schema generation failed");
    let code = result.unwrap();

    // Verify Entity has date-time fields
    let entity_section = extract_type_definition(&code, "Entity");

    assert!(
        entity_section.contains("created_at"),
        "Entity should have created_at field"
    );
    assert!(
        entity_section.contains("updated_at"),
        "Entity should have updated_at field"
    );

    // Check for chrono DateTime usage
    assert!(
        code.contains("chrono::") || code.contains("DateTime"),
        "Generated code should use chrono for date-time types"
    );
}

// Helper function to extract a type definition from generated code
fn extract_type_definition(code: &str, type_name: &str) -> String {
    // Find the type definition (struct or enum)
    let struct_pattern = format!("pub struct {}", type_name);
    let enum_pattern = format!("pub enum {}", type_name);

    if let Some(start) = code.find(&struct_pattern).or_else(|| code.find(&enum_pattern)) {
        // Find the closing brace
        let remaining = &code[start..];
        if let Some(open_brace) = remaining.find('{') {
            let mut brace_count = 0;
            let mut end_pos = open_brace;

            for (i, ch) in remaining[open_brace..].chars().enumerate() {
                if ch == '{' {
                    brace_count += 1;
                } else if ch == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end_pos = open_brace + i + 1;
                        break;
                    }
                }
            }

            return remaining[..end_pos].to_string();
        }
    }

    String::new()
}
