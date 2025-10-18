//! Contract Tests: Orchestrator → Quality Control
//!
//! These tests validate that parameter schemas match between
//! orchestrator calls and quality-control handlers.

use shared_types::contract_tests::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared_types::{ContentNode, AgeGroup, ValidationResult, CorrectionSuggestion, CorrectionCapability};

// ============================================================================
// Quality Control Parameter Types (mirrored from quality-control crate)
// ============================================================================

/// Request parameters for validate_content tool (quality-control side)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct QcValidateContentParams {
    pub content_node: ContentNode,
    pub age_group: AgeGroup,
    #[serde(default)]
    pub educational_goals: Vec<String>,
}

/// Response for validate_content tool (quality-control side)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct QcValidateContentResponse {
    pub node_id: String,
    pub validation_result: ValidationResult,
}

// ============================================================================
// Contract Tests
// ============================================================================

#[test]
fn test_validate_content_params_schema() {
    // Validate that QcValidateContentParams has a well-formed schema
    validate_tool_contract::<QcValidateContentParams>()
        .expect("QcValidateContentParams schema should be valid");
}

#[test]
fn test_validate_content_response_schema() {
    // Validate that QcValidateContentResponse has a well-formed schema
    validate_tool_contract::<QcValidateContentResponse>()
        .expect("QcValidateContentResponse schema should be valid");
}

#[test]
fn test_validate_content_params_roundtrip() {
    // Create sample content node
    let content_node = ContentNode {
        id: "test-node-1".to_string(),
        depth: 1,
        content: Some("This is a test story for young children.".to_string()),
        choices: vec![
            "Continue the adventure".to_string(),
            "Return home".to_string(),
        ],
        educational_content: Some("Learning about courage and decision-making.".to_string()),
    };

    let params = QcValidateContentParams {
        content_node,
        age_group: AgeGroup::_6To8,
        educational_goals: vec!["reading comprehension".to_string()],
    };

    test_roundtrip_serialization(params)
        .expect("ValidateContentParams roundtrip should succeed");
}

#[test]
fn test_validate_content_response_roundtrip() {
    let response = QcValidateContentResponse {
        node_id: "test-node-1".to_string(),
        validation_result: ValidationResult {
            is_valid: true,
            age_appropriate_score: 95.0,
            safety_issues: vec![],
            corrections: vec![],
            educational_value_score: 90.0,
            correction_capability: CorrectionCapability::CanFixLocally,
        },
    };

    test_roundtrip_serialization(response)
        .expect("ValidateContentResponse roundtrip should succeed");
}

#[test]
fn test_validate_content_with_optional_fields() {
    // Test with empty educational_goals
    let content_node = ContentNode {
        id: "test-node-2".to_string(),
        depth: 2,
        content: Some("Another test".to_string()),
        choices: vec![],
        educational_content: None,
    };

    let params = QcValidateContentParams {
        content_node,
        age_group: AgeGroup::_9To11,
        educational_goals: vec![], // Empty vector
    };

    test_roundtrip_serialization(params)
        .expect("ValidateContentParams with empty goals should roundtrip");
}

#[test]
fn test_validation_result_with_issues() {
    // Test with issues and corrections
    let response = QcValidateContentResponse {
        node_id: "test-node-3".to_string(),
        validation_result: ValidationResult {
            is_valid: false,
            age_appropriate_score: 60.0,
            safety_issues: vec![
                "Sentence too complex for age group".to_string(),
                "Contains inappropriate vocabulary".to_string(),
            ],
            corrections: vec![
                CorrectionSuggestion {
                    issue: "Sentence too complex".to_string(),
                    severity: "high".to_string(),
                    suggestion: "Simplify sentence structure".to_string(),
                    field: "content".to_string(),
                },
                CorrectionSuggestion {
                    issue: "Inappropriate vocabulary".to_string(),
                    severity: "medium".to_string(),
                    suggestion: "Replace with age-appropriate words".to_string(),
                    field: "content".to_string(),
                },
            ],
            educational_value_score: 75.0,
            correction_capability: CorrectionCapability::NeedsRevision,
        },
    };

    test_roundtrip_serialization(response)
        .expect("ValidationResult with issues should roundtrip");
}
