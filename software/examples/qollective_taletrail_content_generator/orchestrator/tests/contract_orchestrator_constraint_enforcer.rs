//! Contract Tests: Orchestrator → Constraint Enforcer
//!
//! These tests validate that parameter schemas match between
//! orchestrator calls and constraint-enforcer handlers.

use shared_types::contract_tests::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared_types::{ContentNode, GenerationRequest, ConstraintResult, AgeGroup};

// ============================================================================
// Constraint Enforcer Parameter Types (mirrored from constraint-enforcer crate)
// ============================================================================

/// Request parameters for enforce_constraints tool (constraint-enforcer side)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CeEnforceConstraintsParams {
    pub content_node: ContentNode,
    pub generation_request: GenerationRequest,
}

/// Response for enforce_constraints tool (constraint-enforcer side)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CeEnforceConstraintsResponse {
    pub node_id: String,
    pub constraint_result: ConstraintResult,
}

// ============================================================================
// Contract Tests
// ============================================================================

#[test]
fn test_enforce_constraints_params_schema() {
    // Validate that CeEnforceConstraintsParams has a well-formed schema
    validate_tool_contract::<CeEnforceConstraintsParams>()
        .expect("CeEnforceConstraintsParams schema should be valid");
}

#[test]
fn test_enforce_constraints_response_schema() {
    // Validate that CeEnforceConstraintsResponse has a well-formed schema
    validate_tool_contract::<CeEnforceConstraintsResponse>()
        .expect("CeEnforceConstraintsResponse schema should be valid");
}

#[test]
fn test_enforce_constraints_params_roundtrip() {
    // Create sample content node
    let content_node = ContentNode {
        id: "test-node-1".to_string(),
        depth: 1,
        content: Some("This is a test story about space exploration.".to_string()),
        choices: vec![
            "Explore the alien planet".to_string(),
            "Return to the spaceship".to_string(),
        ],
        educational_content: Some("Learning about planets and decision-making.".to_string()),
    };

    // Create generation request
    let generation_request = GenerationRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        node_count: 10,
        theme: Some("Space Adventure".to_string()),
        age_group: AgeGroup::_9To11,
        language: "en".to_string(),
        convergence_points: None,
        prompt_packages: None,
        tenant_id: Some("test-tenant".to_string()),
        user_id: Some("test-user".to_string()),
    };

    let params = CeEnforceConstraintsParams {
        content_node,
        generation_request,
    };

    test_roundtrip_serialization(params)
        .expect("EnforceConstraintsParams roundtrip should succeed");
}

#[test]
fn test_enforce_constraints_response_roundtrip() {
    let response = CeEnforceConstraintsResponse {
        node_id: "test-node-1".to_string(),
        constraint_result: ConstraintResult {
            is_valid: true,
            constraint_violations: vec![],
            theme_adherence_score: 95.0,
            language_match: true,
        },
    };

    test_roundtrip_serialization(response)
        .expect("EnforceConstraintsResponse roundtrip should succeed");
}

#[test]
fn test_enforce_constraints_with_violations() {
    let response = CeEnforceConstraintsResponse {
        node_id: "test-node-2".to_string(),
        constraint_result: ConstraintResult {
            is_valid: false,
            constraint_violations: vec![
                "Content does not match theme 'Space Adventure'".to_string(),
                "Language mismatch: expected 'en', found 'de'".to_string(),
            ],
            theme_adherence_score: 45.0,
            language_match: false,
        },
    };

    test_roundtrip_serialization(response)
        .expect("ConstraintResult with violations should roundtrip");
}

#[test]
fn test_generation_request_with_optional_fields() {
    let content_node = ContentNode {
        id: "test-node-3".to_string(),
        depth: 3,
        content: None, // No content yet
        choices: vec![],
        educational_content: None,
    };

    let generation_request = GenerationRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        node_count: 5,
        theme: None, // No theme
        age_group: AgeGroup::_12To14,
        language: "de".to_string(),
        convergence_points: Some(2),
        prompt_packages: None,
        tenant_id: None,
        user_id: None,
    };

    let params = CeEnforceConstraintsParams {
        content_node,
        generation_request,
    };

    test_roundtrip_serialization(params)
        .expect("EnforceConstraintsParams with optional fields should roundtrip");
}
