//! Tests for negotiation protocol module

use orchestrator::{OrchestratorConfig, negotiation::*};
use shared_types::*;
use shared_types_llm::{LlmConfig, ProviderConfig, parameters::{ProviderType, SystemPromptStyle}};
use std::collections::HashMap;

// Helper function to create test LlmConfig
fn create_test_llm_config() -> LlmConfig {
    LlmConfig {
        provider: ProviderConfig {
            provider_type: ProviderType::LmStudio,
            url: "http://localhost:1234/v1".to_string(),
            api_key: None,
            default_model: "test-model".to_string(),
            models: HashMap::new(),
            use_default_model_fallback: true,
            max_tokens: 4096,
            temperature: 0.7,
            timeout_secs: 60,
            system_prompt_style: SystemPromptStyle::Native,
        },
        tenants: HashMap::new(),
    }
}

// Helper function to create test config
fn create_test_config() -> OrchestratorConfig {
    OrchestratorConfig {
        service: orchestrator::config::ServiceConfig {
            name: "test-orchestrator".to_string(),
            version: "0.1.0".to_string(),
            description: "Test Orchestrator".to_string(),
        },
        nats: orchestrator::config::NatsConfig {
            url: "nats://localhost:4222".to_string(),
            subject: "test.subject".to_string(),
            queue_group: "test-group".to_string(),
            tls: orchestrator::config::TlsConfig {
                ca_cert: "./certs/ca.pem".to_string(),
                client_cert: "./certs/client-cert.pem".to_string(),
                client_key: "./certs/client-key.pem".to_string(),
            },
        },
        llm: create_test_llm_config(),
        pipeline: orchestrator::config::PipelineConfig {
            generation_timeout_secs: 60,
            validation_timeout_secs: 10,
            retry_max_attempts: 3,
            retry_base_delay_secs: 1,
            retry_max_delay_secs: 30,
        },
        batch: orchestrator::config::BatchConfig {
            size_min: 4,
            size_max: 6,
            concurrent_batches: 3,
            concurrent_batches_max: 5,
        },
        dag: orchestrator::config::DagConfig {
            default_node_count: 16,
            convergence_point_ratio: 0.25,
            max_depth: 10,
        },
        negotiation: orchestrator::config::NegotiationConfig {
            max_rounds: 3,
        },
    }
}

// Helper function to create test ContentNode
fn create_test_node(id: &str) -> ContentNode {
    ContentNode {
        id: id.to_string(),
        content: Content {
            text: format!("Test content for {}", id),
            r#type: "scene".to_string(),
            node_id: id.to_string(),
            next_nodes: vec![],
            choices: vec![],
            convergence_point: false,
            educational_content: None,
        },
        incoming_edges: 0,
        outgoing_edges: 0,
        generation_metadata: None,
    }
}

// Helper function to create passing ValidationResult
fn create_passing_validation_result() -> ValidationResult {
    ValidationResult {
        is_valid: true,
        age_appropriate_score: 0.95,
        educational_value_score: 0.90,
        safety_issues: vec![],
        corrections: vec![],
        correction_capability: CorrectionCapability::CanFixLocally,
    }
}

// Helper function to create failing ValidationResult
fn create_failing_validation_result(capability: CorrectionCapability, score: f64) -> ValidationResult {
    ValidationResult {
        is_valid: false,
        age_appropriate_score: score,
        educational_value_score: score,
        safety_issues: vec!["Test safety issue".to_string()],
        corrections: vec![CorrectionSuggestion {
            issue: "Test issue".to_string(),
            severity: "high".to_string(),
            suggestion: "Test correction suggestion".to_string(),
            field: "content".to_string(),
        }],
        correction_capability: capability,
    }
}

// Helper function to create passing ConstraintResult
fn create_passing_constraint_result() -> ConstraintResult {
    ConstraintResult {
        required_elements_present: true,
        theme_consistency_score: 0.95,
        vocabulary_violations: vec![],
        missing_elements: vec![],
        corrections: vec![],
        correction_capability: CorrectionCapability::CanFixLocally,
    }
}

// Helper function to create failing ConstraintResult
fn create_failing_constraint_result(capability: CorrectionCapability, violations_count: usize) -> ConstraintResult {
    ConstraintResult {
        required_elements_present: false,
        theme_consistency_score: 0.50,
        vocabulary_violations: (0..violations_count).map(|i| {
            VocabularyViolation {
                word: format!("word{}", i),
                current_level: VocabularyLevel::Advanced,
                target_level: VocabularyLevel::Basic,
                suggestions: vec![format!("suggestion{}", i)],
                node_id: "test-node".to_string(),
            }
        }).collect(),
        missing_elements: vec!["element1".to_string()],
        corrections: vec![CorrectionSuggestion {
            issue: "Missing elements".to_string(),
            severity: "medium".to_string(),
            suggestion: "Add missing elements".to_string(),
            field: "content".to_string(),
        }],
        correction_capability: capability,
    }
}

#[tokio::test]
async fn test_negotiator_creation() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);

    // Verify negotiator was created successfully
    // max_rounds should be set from config
    assert_eq!(negotiator.max_rounds(), 3);
}

#[tokio::test]
async fn test_no_issues_returns_none() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);
    let node = create_test_node("node-1");

    // All passing results
    let quality_results = vec![create_passing_validation_result()];
    let constraint_results = vec![create_passing_constraint_result()];

    let result = negotiator.negotiate_improvements(
        &node,
        quality_results,
        constraint_results
    ).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "Should return None when all validations pass");
}

#[tokio::test]
async fn test_can_fix_locally_creates_plan() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);
    let node = create_test_node("node-1");

    // Quality validation fails with CanFixLocally
    let quality_results = vec![
        create_failing_validation_result(CorrectionCapability::CanFixLocally, 0.60)
    ];
    let constraint_results = vec![create_passing_constraint_result()];

    let result = negotiator.negotiate_improvements(
        &node,
        quality_results,
        constraint_results
    ).await;

    assert!(result.is_ok());
    let plan = result.unwrap();
    assert!(plan.is_some(), "Should return a correction plan");

    let plan = plan.unwrap();
    // CanFixLocally should result in regeneration request (for now)
    assert!(plan.regenerate_nodes.contains(&"node-1".to_string()) ||
            !plan.local_fixes.is_empty(),
            "Should have corrections for CanFixLocally capability");
}

#[tokio::test]
async fn test_needs_revision_creates_plan() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);
    let node = create_test_node("node-1");

    // Quality validation fails with NeedsRevision
    let quality_results = vec![
        create_failing_validation_result(CorrectionCapability::NeedsRevision, 0.40)
    ];
    let constraint_results = vec![create_passing_constraint_result()];

    let result = negotiator.negotiate_improvements(
        &node,
        quality_results,
        constraint_results
    ).await;

    assert!(result.is_ok());
    let plan = result.unwrap();
    assert!(plan.is_some(), "Should return a correction plan");

    let plan = plan.unwrap();
    // NeedsRevision should result in regeneration
    assert!(plan.regenerate_nodes.contains(&"node-1".to_string()),
            "Should mark node for regeneration with NeedsRevision");
}

#[tokio::test]
async fn test_no_fix_possible_critical_returns_error() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);
    let node = create_test_node("node-1");

    // Critical quality issue with NoFixPossible (score < 0.5 = Critical)
    let quality_results = vec![
        create_failing_validation_result(CorrectionCapability::NoFixPossible, 0.30)
    ];
    let constraint_results = vec![create_passing_constraint_result()];

    let result = negotiator.negotiate_improvements(
        &node,
        quality_results,
        constraint_results
    ).await;

    assert!(result.is_err(), "Should return error for critical NoFixPossible issue");

    let err = result.unwrap_err();
    assert!(matches!(err, TaleTrailError::ValidationError(_)),
            "Should be a ValidationError");
}

#[tokio::test]
async fn test_no_fix_possible_warning_skips() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);
    let node = create_test_node("node-1");

    // Warning level issue with NoFixPossible (score >= 0.5, < 0.7 = Warning)
    let quality_results = vec![
        create_failing_validation_result(CorrectionCapability::NoFixPossible, 0.60)
    ];
    let constraint_results = vec![create_passing_constraint_result()];

    let result = negotiator.negotiate_improvements(
        &node,
        quality_results,
        constraint_results
    ).await;

    assert!(result.is_ok());
    let plan = result.unwrap();
    assert!(plan.is_some(), "Should return a correction plan");

    let plan = plan.unwrap();
    // Non-critical NoFixPossible should be skipped
    assert!(plan.skipped_nodes.contains(&"node-1".to_string()),
            "Should skip node with non-critical NoFixPossible issue");
}

#[tokio::test]
async fn test_severity_determination_from_score() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);
    let node = create_test_node("node-1");

    // Test Critical severity (score < 0.5)
    let critical_result = create_failing_validation_result(CorrectionCapability::NeedsRevision, 0.30);
    let result = negotiator.negotiate_improvements(
        &node,
        vec![critical_result],
        vec![]
    ).await;

    // Should create plan for critical issue
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());

    // Test Warning severity (0.5 <= score < 0.7)
    let node2 = create_test_node("node-2");
    let warning_result = create_failing_validation_result(CorrectionCapability::NeedsRevision, 0.60);
    let result = negotiator.negotiate_improvements(
        &node2,
        vec![warning_result],
        vec![]
    ).await;

    // Should create plan for warning issue
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());

    // Test Info severity (score >= 0.7)
    let node3 = create_test_node("node-3");
    let info_result = create_failing_validation_result(CorrectionCapability::NeedsRevision, 0.75);
    let result = negotiator.negotiate_improvements(
        &node3,
        vec![info_result],
        vec![]
    ).await;

    // Should still create plan for info level issue
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[tokio::test]
async fn test_constraint_severity_determination() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);
    let node = create_test_node("node-1");

    // Test Critical severity (> 5 violations)
    let critical_constraint = create_failing_constraint_result(CorrectionCapability::NeedsRevision, 6);
    let result = negotiator.negotiate_improvements(
        &node,
        vec![],
        vec![critical_constraint]
    ).await;

    assert!(result.is_ok());
    let plan = result.unwrap();
    assert!(plan.is_some());

    // Test Warning severity (1-5 violations)
    let node2 = create_test_node("node-2");
    let warning_constraint = create_failing_constraint_result(CorrectionCapability::NeedsRevision, 3);
    let result = negotiator.negotiate_improvements(
        &node2,
        vec![],
        vec![warning_constraint]
    ).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[tokio::test]
async fn test_max_rounds_limit() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);

    // Create nodes with persistent failures
    let nodes = vec![
        create_test_node("node-1"),
        create_test_node("node-2"),
    ];

    // Create failing results that would trigger regeneration
    let quality_results = vec![
        vec![create_failing_validation_result(CorrectionCapability::NeedsRevision, 0.40)],
        vec![create_failing_validation_result(CorrectionCapability::NeedsRevision, 0.40)],
    ];

    let constraint_results = vec![
        vec![create_failing_constraint_result(CorrectionCapability::NeedsRevision, 3)],
        vec![create_failing_constraint_result(CorrectionCapability::NeedsRevision, 3)],
    ];

    let result = negotiator.execute_negotiation_rounds(
        &nodes,
        quality_results,
        constraint_results
    ).await;

    assert!(result.is_ok());
    let rounds = result.unwrap();

    // Should stop at max_rounds (3)
    assert!(rounds.len() <= 3, "Should not exceed max_rounds");

    // Last round should be marked as unsuccessful if max rounds reached
    if rounds.len() == 3 {
        assert!(!rounds.last().unwrap().success,
                "Last round should be unsuccessful if max rounds reached");
    }
}

#[tokio::test]
async fn test_successful_round_stops_negotiation() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);

    // Create nodes
    let nodes = vec![create_test_node("node-1")];

    // All passing results (successful validation)
    let quality_results = vec![
        vec![create_passing_validation_result()],
    ];

    let constraint_results = vec![
        vec![create_passing_constraint_result()],
    ];

    let result = negotiator.execute_negotiation_rounds(
        &nodes,
        quality_results,
        constraint_results
    ).await;

    assert!(result.is_ok());
    let rounds = result.unwrap();

    // Should only have 1 round since validation passed
    assert_eq!(rounds.len(), 1, "Should stop after first successful round");
    assert!(rounds[0].success, "First round should be successful");
}

#[tokio::test]
async fn test_multiple_issues_same_node() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);
    let node = create_test_node("node-1");

    // Both quality and constraint failures
    let quality_results = vec![
        create_failing_validation_result(CorrectionCapability::NeedsRevision, 0.40)
    ];
    let constraint_results = vec![
        create_failing_constraint_result(CorrectionCapability::NeedsRevision, 3)
    ];

    let result = negotiator.negotiate_improvements(
        &node,
        quality_results,
        constraint_results
    ).await;

    assert!(result.is_ok());
    let plan = result.unwrap();
    assert!(plan.is_some());

    let plan = plan.unwrap();
    // Should only have node once in regenerate list
    assert_eq!(plan.regenerate_nodes.iter().filter(|n| *n == "node-1").count(), 1,
               "Node should only appear once in regeneration list");
}

#[tokio::test]
async fn test_mixed_capabilities_prioritizes_critical() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);
    let node = create_test_node("node-1");

    // Mix of different capabilities and severities
    let quality_results = vec![
        create_failing_validation_result(CorrectionCapability::CanFixLocally, 0.60),
    ];
    let constraint_results = vec![
        create_failing_constraint_result(CorrectionCapability::NeedsRevision, 6), // Critical
    ];

    let result = negotiator.negotiate_improvements(
        &node,
        quality_results,
        constraint_results
    ).await;

    assert!(result.is_ok());
    let plan = result.unwrap();
    assert!(plan.is_some());

    let plan = plan.unwrap();
    // Should handle both issues
    assert!(!plan.regenerate_nodes.is_empty() || !plan.local_fixes.is_empty(),
            "Should create corrections for mixed capability issues");
}

#[tokio::test]
async fn test_round_tracking() {
    let config = create_test_config();
    let negotiator = Negotiator::new(&config);

    let nodes = vec![create_test_node("node-1")];

    // Create failing results
    let quality_results = vec![
        vec![create_failing_validation_result(CorrectionCapability::NeedsRevision, 0.40)],
    ];
    let constraint_results = vec![
        vec![create_failing_constraint_result(CorrectionCapability::NeedsRevision, 3)],
    ];

    let result = negotiator.execute_negotiation_rounds(
        &nodes,
        quality_results,
        constraint_results
    ).await;

    assert!(result.is_ok());
    let rounds = result.unwrap();

    // Verify round metadata
    for (i, round) in rounds.iter().enumerate() {
        assert_eq!(round.iteration, (i + 1) as u32,
                   "Round iteration should be 1-based and sequential");
    }
}
