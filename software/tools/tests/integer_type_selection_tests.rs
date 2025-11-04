// ABOUTME: Comprehensive tests for intelligent integer type selection
// ABOUTME: Tests all type boundaries, edge cases, and constraint handling

use qollective_tools_lib::codegen::integer_type_selection::select_optimal_integer_type;
use qollective_tools_lib::codegen::types::RustType;
use qollective_tools_lib::schema::ir::{Schema, SchemaType};

/// Helper function to create a schema with minimum and maximum bounds
fn schema_with_bounds(min: Option<f64>, max: Option<f64>) -> Schema {
    Schema {
        schema_type: SchemaType::Integer,
        minimum: min,
        maximum: max,
        ..Default::default()
    }
}

/// Helper function to create a schema with exclusive bounds
fn schema_with_exclusive_bounds(exclusive_min: Option<f64>, exclusive_max: Option<f64>) -> Schema {
    Schema {
        schema_type: SchemaType::Integer,
        exclusive_minimum: exclusive_min,
        exclusive_maximum: exclusive_max,
        ..Default::default()
    }
}

#[cfg(test)]
mod unsigned_type_tests {
    use super::*;

    #[test]
    fn test_u8_exact_range() {
        let schema = schema_with_bounds(Some(0.0), Some(255.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "Range [0, 255] should select u8"
        );
    }

    #[test]
    fn test_u8_subrange() {
        let schema = schema_with_bounds(Some(0.0), Some(100.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "Range [0, 100] should select u8"
        );
    }

    #[test]
    fn test_u8_partial_range() {
        let schema = schema_with_bounds(Some(10.0), Some(200.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "Range [10, 200] should select u8"
        );
    }

    #[test]
    fn test_u16_just_above_u8() {
        let schema = schema_with_bounds(Some(0.0), Some(256.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U16,
            "Range [0, 256] should select u16"
        );
    }

    #[test]
    fn test_u16_exact_range() {
        let schema = schema_with_bounds(Some(0.0), Some(65535.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U16,
            "Range [0, 65535] should select u16"
        );
    }

    #[test]
    fn test_u16_port_number_range() {
        let schema = schema_with_bounds(Some(0.0), Some(65000.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U16,
            "Port number range [0, 65000] should select u16"
        );
    }

    #[test]
    fn test_u32_just_above_u16() {
        let schema = schema_with_bounds(Some(0.0), Some(65536.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U32,
            "Range [0, 65536] should select u32"
        );
    }

    #[test]
    fn test_u32_exact_range() {
        let schema = schema_with_bounds(Some(0.0), Some(4294967295.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U32,
            "Range [0, 4294967295] should select u32"
        );
    }

    #[test]
    fn test_u64_just_above_u32() {
        let schema = schema_with_bounds(Some(0.0), Some(4294967296.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U64,
            "Range [0, 4294967296] should select u64"
        );
    }

    #[test]
    fn test_u64_large_value() {
        // Use a large value that fits in u64 but is less than u64::MAX
        // (f64 cannot represent u64::MAX exactly)
        let schema = schema_with_bounds(Some(0.0), Some(18_000_000_000_000_000_000.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U64,
            "Range [0, large u64 value] should select u64"
        );
    }

    #[test]
    fn test_u128_exceeding_u64() {
        let schema = schema_with_bounds(Some(0.0), Some(f64::MAX));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U128,
            "Range exceeding u64 should select u128"
        );
    }
}

#[cfg(test)]
mod signed_type_tests {
    use super::*;

    #[test]
    fn test_i8_exact_range() {
        let schema = schema_with_bounds(Some(-128.0), Some(127.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I8,
            "Range [-128, 127] should select i8"
        );
    }

    #[test]
    fn test_i8_subrange() {
        let schema = schema_with_bounds(Some(-50.0), Some(50.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I8,
            "Range [-50, 50] should select i8"
        );
    }

    #[test]
    fn test_i8_small_negative() {
        let schema = schema_with_bounds(Some(-100.0), Some(100.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I8,
            "Range [-100, 100] should select i8"
        );
    }

    #[test]
    fn test_i16_below_i8_min() {
        let schema = schema_with_bounds(Some(-129.0), Some(127.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I16,
            "Range [-129, 127] should select i16"
        );
    }

    #[test]
    fn test_i16_above_i8_max() {
        let schema = schema_with_bounds(Some(-128.0), Some(128.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I16,
            "Range [-128, 128] should select i16"
        );
    }

    #[test]
    fn test_i16_exact_range() {
        let schema = schema_with_bounds(Some(-32768.0), Some(32767.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I16,
            "Range [-32768, 32767] should select i16"
        );
    }

    #[test]
    fn test_i32_below_i16_min() {
        let schema = schema_with_bounds(Some(-32769.0), Some(32767.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I32,
            "Range [-32769, 32767] should select i32"
        );
    }

    #[test]
    fn test_i32_exact_range() {
        let schema = schema_with_bounds(Some(-2147483648.0), Some(2147483647.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I32,
            "Range [-2147483648, 2147483647] should select i32"
        );
    }

    #[test]
    fn test_i64_below_i32_min() {
        let schema = schema_with_bounds(Some(-2147483649.0), Some(2147483647.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I64,
            "Range [-2147483649, 2147483647] should select i64"
        );
    }

    #[test]
    fn test_i64_large_range() {
        // Use large values that fit in i64 but are less than i64 bounds
        // (f64 cannot represent i64::MIN/MAX exactly)
        let schema = schema_with_bounds(
            Some(-9_000_000_000_000_000_000.0),
            Some(9_000_000_000_000_000_000.0),
        );
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I64,
            "Range [large negative, large positive] should select i64"
        );
    }

    #[test]
    fn test_i128_exceeding_i64() {
        let schema = schema_with_bounds(Some(f64::MIN), Some(f64::MAX));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I128,
            "Range exceeding i64 should select i128"
        );
    }
}

#[cfg(test)]
mod type_preference_tests {
    use super::*;

    #[test]
    fn test_unsigned_preferred_when_min_zero() {
        let schema = schema_with_bounds(Some(0.0), Some(100.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "When min is 0, should prefer unsigned type"
        );
    }

    #[test]
    fn test_unsigned_preferred_when_min_positive() {
        let schema = schema_with_bounds(Some(5.0), Some(100.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "When min is positive, should prefer unsigned type"
        );
    }

    #[test]
    fn test_signed_required_when_min_negative() {
        let schema = schema_with_bounds(Some(-1.0), Some(100.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I8,
            "When min is negative, must use signed type"
        );
    }

    #[test]
    fn test_signed_for_small_negative() {
        let schema = schema_with_bounds(Some(-1.0), Some(1000.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I16,
            "Small negative value should use appropriate signed type"
        );
    }
}

#[cfg(test)]
mod exclusive_bounds_tests {
    use super::*;

    #[test]
    fn test_exclusive_minimum_conversion() {
        // exclusive_minimum: 0 becomes minimum: 1
        let schema = Schema {
            exclusive_minimum: Some(0.0),
            maximum: Some(255.0),
            ..Default::default()
        };
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "exclusive_minimum: 0 should become minimum: 1"
        );
    }

    #[test]
    fn test_exclusive_minimum_triggers_unsigned() {
        // exclusive_minimum: -1 becomes minimum: 0, should trigger unsigned
        let schema = Schema {
            exclusive_minimum: Some(-1.0),
            maximum: Some(100.0),
            ..Default::default()
        };
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "exclusive_minimum: -1 should become minimum: 0, triggering unsigned"
        );
    }

    #[test]
    fn test_exclusive_maximum_conversion() {
        // exclusive_maximum: 256 becomes maximum: 255
        let schema = Schema {
            minimum: Some(0.0),
            exclusive_maximum: Some(256.0),
            ..Default::default()
        };
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "exclusive_maximum: 256 should become maximum: 255"
        );
    }

    #[test]
    fn test_exclusive_maximum_i8_boundary() {
        // exclusive_maximum: 128 becomes maximum: 127
        let schema = Schema {
            minimum: Some(-128.0),
            exclusive_maximum: Some(128.0),
            ..Default::default()
        };
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I8,
            "exclusive_maximum: 128 should become maximum: 127"
        );
    }

    #[test]
    fn test_both_exclusive_bounds() {
        // exclusive_minimum: -1, exclusive_maximum: 256
        // Should become: minimum: 0, maximum: 255 -> u8
        let schema = schema_with_exclusive_bounds(Some(-1.0), Some(256.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "Both exclusive bounds should be converted correctly"
        );
    }

    #[test]
    fn test_exclusive_overrides_inclusive() {
        // When both are present, inclusive should take precedence
        let schema = Schema {
            minimum: Some(0.0),
            exclusive_minimum: Some(10.0),
            maximum: Some(255.0),
            ..Default::default()
        };
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "Inclusive minimum should take precedence over exclusive"
        );
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;

    #[test]
    fn test_no_bounds_fallback() {
        let schema = schema_with_bounds(None, None);
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I64,
            "No bounds should fall back to i64"
        );
    }

    #[test]
    fn test_only_minimum_fallback() {
        let schema = schema_with_bounds(Some(0.0), None);
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I64,
            "Only minimum should fall back to i64"
        );
    }

    #[test]
    fn test_only_maximum_fallback() {
        let schema = schema_with_bounds(None, Some(100.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I64,
            "Only maximum should fall back to i64"
        );
    }

    #[test]
    fn test_invalid_range_fallback() {
        // min > max should fall back to i64
        let schema = schema_with_bounds(Some(100.0), Some(50.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I64,
            "Invalid range (min > max) should fall back to i64"
        );
    }

    #[test]
    fn test_default_schema_fallback() {
        let schema = Schema::default();
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I64,
            "Default schema should fall back to i64"
        );
    }
}

#[cfg(test)]
mod boundary_value_tests {
    use super::*;

    #[test]
    fn test_u8_max_boundary() {
        // Exactly at u8 max
        let schema = schema_with_bounds(Some(0.0), Some(255.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U8);

        // One above u8 max
        let schema = schema_with_bounds(Some(0.0), Some(256.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U16);
    }

    #[test]
    fn test_u16_max_boundary() {
        // Exactly at u16 max
        let schema = schema_with_bounds(Some(0.0), Some(65535.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U16);

        // One above u16 max
        let schema = schema_with_bounds(Some(0.0), Some(65536.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U32);
    }

    #[test]
    fn test_u32_max_boundary() {
        // Exactly at u32 max
        let schema = schema_with_bounds(Some(0.0), Some(4294967295.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U32);

        // One above u32 max
        let schema = schema_with_bounds(Some(0.0), Some(4294967296.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U64);
    }

    #[test]
    fn test_i8_min_boundary() {
        // Exactly at i8 min/max
        let schema = schema_with_bounds(Some(-128.0), Some(127.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I8);

        // One below i8 min
        let schema = schema_with_bounds(Some(-129.0), Some(127.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I16);
    }

    #[test]
    fn test_i8_max_boundary() {
        // Exactly at i8 max
        let schema = schema_with_bounds(Some(-128.0), Some(127.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I8);

        // One above i8 max
        let schema = schema_with_bounds(Some(-128.0), Some(128.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I16);
    }

    #[test]
    fn test_i16_boundaries() {
        // Exactly at i16 min/max
        let schema = schema_with_bounds(Some(-32768.0), Some(32767.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I16);

        // One below i16 min
        let schema = schema_with_bounds(Some(-32769.0), Some(32767.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I32);

        // One above i16 max
        let schema = schema_with_bounds(Some(-32768.0), Some(32768.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I32);
    }

    #[test]
    fn test_i32_boundaries() {
        // Exactly at i32 min/max
        let schema = schema_with_bounds(Some(-2147483648.0), Some(2147483647.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I32);

        // One below i32 min
        let schema = schema_with_bounds(Some(-2147483649.0), Some(2147483647.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I64);

        // One above i32 max
        let schema = schema_with_bounds(Some(-2147483648.0), Some(2147483648.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I64);
    }
}

#[cfg(test)]
mod practical_use_cases {
    use super::*;

    #[test]
    fn test_http_status_code() {
        // HTTP status codes: [100, 599]
        let schema = schema_with_bounds(Some(100.0), Some(599.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U16,
            "HTTP status codes should use u16"
        );
    }

    #[test]
    fn test_percentage() {
        // Percentage: [0, 100]
        let schema = schema_with_bounds(Some(0.0), Some(100.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "Percentage values should use u8"
        );
    }

    #[test]
    fn test_port_number() {
        // Port numbers: [0, 65535]
        let schema = schema_with_bounds(Some(0.0), Some(65535.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U16,
            "Port numbers should use u16"
        );
    }

    #[test]
    fn test_age_field() {
        // Age: [0, 150]
        let schema = schema_with_bounds(Some(0.0), Some(150.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "Age field should use u8"
        );
    }

    #[test]
    fn test_temperature_celsius() {
        // Temperature in Celsius: [-273, 5778] (absolute zero to sun surface)
        let schema = schema_with_bounds(Some(-273.0), Some(5778.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I16,
            "Temperature range should use i16"
        );
    }

    #[test]
    fn test_unix_timestamp() {
        // Unix timestamp (seconds since epoch): typically positive, large values
        let schema = schema_with_bounds(Some(0.0), Some(4294967295.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U32,
            "32-bit Unix timestamp should use u32"
        );
    }

    #[test]
    fn test_byte_value() {
        // Byte value: [0, 255]
        let schema = schema_with_bounds(Some(0.0), Some(255.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "Byte value should use u8"
        );
    }

    #[test]
    fn test_rgba_color_component() {
        // RGBA color component: [0, 255]
        let schema = schema_with_bounds(Some(0.0), Some(255.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "Color component should use u8"
        );
    }

    #[test]
    fn test_priority_level() {
        // Priority level: [1, 10]
        let schema = schema_with_bounds(Some(1.0), Some(10.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::U8,
            "Priority level should use u8"
        );
    }

    #[test]
    fn test_rating_with_negative() {
        // Rating allowing negative: [-5, 5]
        let schema = schema_with_bounds(Some(-5.0), Some(5.0));
        assert_eq!(
            select_optimal_integer_type(&schema),
            RustType::I8,
            "Rating with negative values should use i8"
        );
    }
}
