//! Constants for shared-types-generated
//!
//! CONSTANTS FIRST PRINCIPLE:
//! These constants are used by the generated types and preset logic.
//! They are duplicated from shared-types/constants.rs to avoid circular dependencies.

// ============================================================================
// STORY STRUCTURE PRESET NODE COUNTS
// ============================================================================

/// Node count for "guided" story structure preset
pub const PRESET_GUIDED_NODE_COUNT: i64 = 12;

/// Node count for "adventure" story structure preset
pub const PRESET_ADVENTURE_NODE_COUNT: i64 = 16;

/// Node count for "epic" story structure preset
pub const PRESET_EPIC_NODE_COUNT: i64 = 24;

/// Node count for "choose_your_path" story structure preset
pub const PRESET_CHOOSE_YOUR_PATH_NODE_COUNT: i64 = 16;
