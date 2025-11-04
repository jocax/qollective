// ABOUTME: Intelligent integer type selection based on JSON Schema min/max constraints
// ABOUTME: Selects optimal Rust integer types (u8-u128, i8-i128) based on value ranges

use crate::codegen::types::RustType;
use crate::schema::ir::Schema;

/// Select the optimal Rust integer type based on schema constraints
///
/// This function analyzes the minimum and maximum constraints in a JSON Schema
/// and selects the smallest Rust integer type that can safely represent the full range.
///
/// # Type Selection Strategy
///
/// 1. **Unsigned types** (u8, u16, u32, u64, u128) are preferred when minimum >= 0
/// 2. **Signed types** (i8, i16, i32, i64, i128) are used when minimum < 0
/// 3. The **smallest type** that fits the range is selected for optimal memory usage
/// 4. **Fallback to i64** when constraints are missing or ambiguous
/// 5. **Exclusive bounds** are converted to inclusive bounds before type selection
///
/// # Examples
///
/// ```ignore
/// // Range [0, 255] -> u8
/// let schema = Schema { minimum: Some(0.0), maximum: Some(255.0), .. };
/// assert_eq!(select_optimal_integer_type(&schema), RustType::U8);
///
/// // Range [-128, 127] -> i8
/// let schema = Schema { minimum: Some(-128.0), maximum: Some(127.0), .. };
/// assert_eq!(select_optimal_integer_type(&schema), RustType::I8);
///
/// // Range [0, 65535] -> u16
/// let schema = Schema { minimum: Some(0.0), maximum: Some(65535.0), .. };
/// assert_eq!(select_optimal_integer_type(&schema), RustType::U16);
///
/// // No constraints -> i64 (default)
/// let schema = Schema::default();
/// assert_eq!(select_optimal_integer_type(&schema), RustType::I64);
/// ```
pub fn select_optimal_integer_type(schema: &Schema) -> RustType {
    // Extract and normalize bounds (handling exclusive bounds)
    let (min, max) = extract_bounds(schema);

    // If we don't have both bounds, fall back to i64
    let Some(min_val) = min else {
        return RustType::I64;
    };
    let Some(max_val) = max else {
        return RustType::I64;
    };

    // Decide between signed and unsigned based on minimum value
    if min_val >= 0.0 {
        select_unsigned_type(min_val as u128, max_val as u128)
    } else {
        select_signed_type(min_val as i128, max_val as i128)
    }
}

/// Extract minimum and maximum bounds from schema, handling exclusive bounds
///
/// Exclusive bounds are converted to inclusive bounds:
/// - exclusive_minimum: n -> minimum: n + 1
/// - exclusive_maximum: n -> maximum: n - 1
fn extract_bounds(schema: &Schema) -> (Option<f64>, Option<f64>) {
    let min = match (schema.minimum, schema.exclusive_minimum) {
        // Prefer explicit minimum
        (Some(min), _) => Some(min),
        // Convert exclusive_minimum to inclusive by adding 1
        (None, Some(exclusive_min)) => Some(exclusive_min + 1.0),
        (None, None) => None,
    };

    let max = match (schema.maximum, schema.exclusive_maximum) {
        // Prefer explicit maximum
        (Some(max), _) => Some(max),
        // Convert exclusive_maximum to inclusive by subtracting 1
        (None, Some(exclusive_max)) => Some(exclusive_max - 1.0),
        (None, None) => None,
    };

    (min, max)
}

/// Select the smallest unsigned integer type that can represent the range [min, max]
fn select_unsigned_type(min: u128, max: u128) -> RustType {
    // Check if values are within valid range (both non-negative)
    if min > max {
        return RustType::I64; // Invalid range, fall back to i64
    }

    // Select smallest type that fits the maximum value
    // We use the max value as the determining factor since min >= 0
    match max {
        0..=255 => RustType::U8,
        256..=65_535 => RustType::U16,
        65_536..=4_294_967_295 => RustType::U32,
        4_294_967_296..=18_446_744_073_709_551_615 => RustType::U64,
        _ => RustType::U128,
    }
}

/// Select the smallest signed integer type that can represent the range [min, max]
fn select_signed_type(min: i128, max: i128) -> RustType {
    // Check if values are within valid range
    if min > max {
        return RustType::I64; // Invalid range, fall back to i64
    }

    // Check each signed integer type's bounds
    if fits_in_i8(min, max) {
        RustType::I8
    } else if fits_in_i16(min, max) {
        RustType::I16
    } else if fits_in_i32(min, max) {
        RustType::I32
    } else if fits_in_i64(min, max) {
        RustType::I64
    } else {
        RustType::I128
    }
}

/// Check if the range [min, max] fits within i8 bounds [-128, 127]
#[inline]
fn fits_in_i8(min: i128, max: i128) -> bool {
    min >= i8::MIN as i128 && max <= i8::MAX as i128
}

/// Check if the range [min, max] fits within i16 bounds [-32768, 32767]
#[inline]
fn fits_in_i16(min: i128, max: i128) -> bool {
    min >= i16::MIN as i128 && max <= i16::MAX as i128
}

/// Check if the range [min, max] fits within i32 bounds [-2147483648, 2147483647]
#[inline]
fn fits_in_i32(min: i128, max: i128) -> bool {
    min >= i32::MIN as i128 && max <= i32::MAX as i128
}

/// Check if the range [min, max] fits within i64 bounds
#[inline]
fn fits_in_i64(min: i128, max: i128) -> bool {
    min >= i64::MIN as i128 && max <= i64::MAX as i128
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ir::SchemaType;

    fn create_schema_with_bounds(min: Option<f64>, max: Option<f64>) -> Schema {
        Schema {
            schema_type: SchemaType::Integer,
            minimum: min,
            maximum: max,
            ..Default::default()
        }
    }

    fn create_schema_with_exclusive_bounds(
        exclusive_min: Option<f64>,
        exclusive_max: Option<f64>,
    ) -> Schema {
        Schema {
            schema_type: SchemaType::Integer,
            exclusive_minimum: exclusive_min,
            exclusive_maximum: exclusive_max,
            ..Default::default()
        }
    }

    #[test]
    fn test_u8_selection() {
        // Exact u8 range
        let schema = create_schema_with_bounds(Some(0.0), Some(255.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U8);

        // Subrange of u8
        let schema = create_schema_with_bounds(Some(0.0), Some(100.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U8);

        // Small positive range
        let schema = create_schema_with_bounds(Some(10.0), Some(200.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U8);
    }

    #[test]
    fn test_u16_selection() {
        // Just above u8 max
        let schema = create_schema_with_bounds(Some(0.0), Some(256.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U16);

        // Exact u16 range
        let schema = create_schema_with_bounds(Some(0.0), Some(65535.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U16);

        // Typical port number range
        let schema = create_schema_with_bounds(Some(0.0), Some(65000.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U16);
    }

    #[test]
    fn test_u32_selection() {
        // Just above u16 max
        let schema = create_schema_with_bounds(Some(0.0), Some(65536.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U32);

        // Exact u32 range
        let schema = create_schema_with_bounds(Some(0.0), Some(4294967295.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U32);
    }

    #[test]
    fn test_u64_selection() {
        // Just above u32 max
        let schema = create_schema_with_bounds(Some(0.0), Some(4294967296.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U64);

        // Large unsigned value (f64 cannot represent u64::MAX exactly)
        let schema = create_schema_with_bounds(Some(0.0), Some(18_000_000_000_000_000_000.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U64);
    }

    #[test]
    fn test_u128_selection() {
        // Value exceeding u64 max (represented as f64, which will be imprecise but that's ok)
        let schema = create_schema_with_bounds(Some(0.0), Some(f64::MAX));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U128);
    }

    #[test]
    fn test_i8_selection() {
        // Exact i8 range
        let schema = create_schema_with_bounds(Some(-128.0), Some(127.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I8);

        // Subrange of i8
        let schema = create_schema_with_bounds(Some(-50.0), Some(50.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I8);

        // Small negative range
        let schema = create_schema_with_bounds(Some(-100.0), Some(100.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I8);
    }

    #[test]
    fn test_i16_selection() {
        // Just below i8 min
        let schema = create_schema_with_bounds(Some(-129.0), Some(127.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I16);

        // Just above i8 max
        let schema = create_schema_with_bounds(Some(-128.0), Some(128.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I16);

        // Exact i16 range
        let schema = create_schema_with_bounds(Some(-32768.0), Some(32767.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I16);
    }

    #[test]
    fn test_i32_selection() {
        // Just below i16 min
        let schema = create_schema_with_bounds(Some(-32769.0), Some(32767.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I32);

        // Exact i32 range
        let schema = create_schema_with_bounds(Some(-2147483648.0), Some(2147483647.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I32);
    }

    #[test]
    fn test_i64_selection() {
        // Just below i32 min
        let schema = create_schema_with_bounds(Some(-2147483649.0), Some(2147483647.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I64);

        // Large negative to positive range (f64 cannot represent i64::MIN/MAX exactly)
        let schema = create_schema_with_bounds(
            Some(-9_000_000_000_000_000_000.0),
            Some(9_000_000_000_000_000_000.0),
        );
        assert_eq!(select_optimal_integer_type(&schema), RustType::I64);
    }

    #[test]
    fn test_i128_selection() {
        // Value exceeding i64 bounds
        let schema = create_schema_with_bounds(Some(f64::MIN), Some(f64::MAX));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I128);
    }

    #[test]
    fn test_unsigned_preference_when_min_is_zero() {
        // When min is 0, should prefer unsigned types
        let schema = create_schema_with_bounds(Some(0.0), Some(100.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U8);

        let schema = create_schema_with_bounds(Some(0.0), Some(1000.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U16);
    }

    #[test]
    fn test_signed_when_min_is_negative() {
        // Even a small negative value should trigger signed type
        let schema = create_schema_with_bounds(Some(-1.0), Some(100.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I8);

        let schema = create_schema_with_bounds(Some(-1.0), Some(1000.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I16);
    }

    #[test]
    fn test_exclusive_minimum() {
        // exclusive_minimum: 0 should become minimum: 1
        let schema = create_schema_with_exclusive_bounds(Some(0.0), None);
        let schema = Schema {
            maximum: Some(255.0),
            ..schema
        };
        assert_eq!(select_optimal_integer_type(&schema), RustType::U8);

        // exclusive_minimum: -1 should become minimum: 0, triggering unsigned
        let schema = create_schema_with_exclusive_bounds(Some(-1.0), None);
        let schema = Schema {
            maximum: Some(100.0),
            ..schema
        };
        assert_eq!(select_optimal_integer_type(&schema), RustType::U8);
    }

    #[test]
    fn test_exclusive_maximum() {
        // exclusive_maximum: 256 should become maximum: 255
        let schema = Schema {
            minimum: Some(0.0),
            exclusive_maximum: Some(256.0),
            ..Default::default()
        };
        assert_eq!(select_optimal_integer_type(&schema), RustType::U8);

        // exclusive_maximum: 128 should become maximum: 127
        let schema = Schema {
            minimum: Some(-128.0),
            exclusive_maximum: Some(128.0),
            ..Default::default()
        };
        assert_eq!(select_optimal_integer_type(&schema), RustType::I8);
    }

    #[test]
    fn test_missing_bounds_fallback() {
        // No bounds at all -> i64
        let schema = create_schema_with_bounds(None, None);
        assert_eq!(select_optimal_integer_type(&schema), RustType::I64);

        // Only minimum -> i64
        let schema = create_schema_with_bounds(Some(0.0), None);
        assert_eq!(select_optimal_integer_type(&schema), RustType::I64);

        // Only maximum -> i64
        let schema = create_schema_with_bounds(None, Some(100.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I64);
    }

    #[test]
    fn test_invalid_range_fallback() {
        // min > max should fall back to i64
        let schema = create_schema_with_bounds(Some(100.0), Some(50.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I64);
    }

    #[test]
    fn test_boundary_values() {
        // Test exact boundaries of each type

        // u8 boundary: [0, 255]
        let schema = create_schema_with_bounds(Some(0.0), Some(255.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U8);

        // u8 to u16 transition: [0, 256]
        let schema = create_schema_with_bounds(Some(0.0), Some(256.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U16);

        // i8 boundary: [-128, 127]
        let schema = create_schema_with_bounds(Some(-128.0), Some(127.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I8);

        // i8 to i16 transition: [-129, 127]
        let schema = create_schema_with_bounds(Some(-129.0), Some(127.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I16);

        // i8 to i16 transition: [-128, 128]
        let schema = create_schema_with_bounds(Some(-128.0), Some(128.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::I16);
    }

    #[test]
    fn test_both_exclusive_bounds() {
        // exclusive_minimum: -1, exclusive_maximum: 256
        // Should become: minimum: 0, maximum: 255 -> u8
        let schema = create_schema_with_exclusive_bounds(Some(-1.0), Some(256.0));
        assert_eq!(select_optimal_integer_type(&schema), RustType::U8);
    }
}
