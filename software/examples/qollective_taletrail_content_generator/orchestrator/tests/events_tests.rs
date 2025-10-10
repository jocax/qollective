//! Tests for Pipeline Event Publishing
//!
//! This module tests the NATS-based event publishing system for pipeline progress monitoring.
//! Tests follow TDD principles: written before implementation.

use orchestrator::events::PipelineEvent;
use shared_types::constants::MCP_EVENTS_PREFIX;
use serde_json;

// ============================================================================
// Event Serialization Tests
// ============================================================================

#[test]
fn test_prompts_generated_event_serialization() {
    let event = PipelineEvent::PromptsGenerated {
        request_id: "req-123".to_string(),
        duration_ms: 1500,
        fallback_count: 2,
        services: vec![
            "prompt_helper".to_string(),
            "story_generator".to_string(),
            "quality_control".to_string(),
        ],
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize PromptsGenerated event");

    // Verify it contains type discriminator
    assert!(json.contains(r#""type":"prompts_generated"#), "Missing type discriminator");
    assert!(json.contains(r#""request_id":"req-123"#), "Missing request_id");
    assert!(json.contains(r#""duration_ms":1500"#), "Missing duration_ms");
    assert!(json.contains(r#""fallback_count":2"#), "Missing fallback_count");
    assert!(json.contains(r#""services""#), "Missing services array");
}

#[test]
fn test_structure_created_event_serialization() {
    let event = PipelineEvent::StructureCreated {
        request_id: "req-456".to_string(),
        node_count: 16,
        convergence_points: 3,
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize StructureCreated event");

    assert!(json.contains(r#""type":"structure_created"#), "Missing type discriminator");
    assert!(json.contains(r#""request_id":"req-456"#), "Missing request_id");
    assert!(json.contains(r#""node_count":16"#), "Missing node_count");
    assert!(json.contains(r#""convergence_points":3"#), "Missing convergence_points");
}

#[test]
fn test_batch_started_event_serialization() {
    let event = PipelineEvent::BatchStarted {
        request_id: "req-789".to_string(),
        batch_id: 1,
        node_count: 5,
        nodes: vec!["node-1".to_string(), "node-2".to_string(), "node-3".to_string()],
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize BatchStarted event");

    assert!(json.contains(r#""type":"batch_started"#), "Missing type discriminator");
    assert!(json.contains(r#""batch_id":1"#), "Missing batch_id");
    assert!(json.contains(r#""node_count":5"#), "Missing node_count");
    assert!(json.contains(r#""nodes""#), "Missing nodes array");
}

#[test]
fn test_batch_completed_event_serialization() {
    let event = PipelineEvent::BatchCompleted {
        request_id: "req-abc".to_string(),
        batch_id: 2,
        success: true,
        duration_ms: 3500,
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize BatchCompleted event");

    assert!(json.contains(r#""type":"batch_completed"#), "Missing type discriminator");
    assert!(json.contains(r#""success":true"#), "Missing success field");
    assert!(json.contains(r#""duration_ms":3500"#), "Missing duration_ms");
}

#[test]
fn test_validation_started_event_serialization() {
    let event = PipelineEvent::ValidationStarted {
        request_id: "req-def".to_string(),
        batch_id: 3,
        validator: "quality_control".to_string(),
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize ValidationStarted event");

    assert!(json.contains(r#""type":"validation_started"#), "Missing type discriminator");
    assert!(json.contains(r#""validator":"quality_control"#), "Missing validator");
}

#[test]
fn test_negotiation_round_event_serialization() {
    let event = PipelineEvent::NegotiationRound {
        request_id: "req-ghi".to_string(),
        round: 1,
        issues_count: 5,
        corrections_applied: 3,
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize NegotiationRound event");

    assert!(json.contains(r#""type":"negotiation_round"#), "Missing type discriminator");
    assert!(json.contains(r#""round":1"#), "Missing round");
    assert!(json.contains(r#""issues_count":5"#), "Missing issues_count");
    assert!(json.contains(r#""corrections_applied":3"#), "Missing corrections_applied");
}

#[test]
fn test_complete_event_serialization() {
    let event = PipelineEvent::Complete {
        request_id: "req-jkl".to_string(),
        total_duration_ms: 15000,
        total_nodes: 16,
        total_validations: 32,
        negotiation_rounds: 2,
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize Complete event");

    assert!(json.contains(r#""type":"complete"#), "Missing type discriminator");
    assert!(json.contains(r#""total_duration_ms":15000"#), "Missing total_duration_ms");
    assert!(json.contains(r#""total_nodes":16"#), "Missing total_nodes");
    assert!(json.contains(r#""total_validations":32"#), "Missing total_validations");
    assert!(json.contains(r#""negotiation_rounds":2"#), "Missing negotiation_rounds");
}

#[test]
fn test_failed_event_serialization() {
    let event = PipelineEvent::Failed {
        request_id: "req-mno".to_string(),
        phase: "Phase 2: Content Generation".to_string(),
        error: "LLM service unavailable".to_string(),
        duration_ms: 5000,
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize Failed event");

    assert!(json.contains(r#""type":"failed"#), "Missing type discriminator");
    assert!(json.contains(r#""phase":"Phase 2: Content Generation"#), "Missing phase");
    assert!(json.contains(r#""error":"LLM service unavailable"#), "Missing error");
}

// ============================================================================
// Event Deserialization Tests
// ============================================================================

#[test]
fn test_prompts_generated_deserialization() {
    let json = r#"{
        "type": "prompts_generated",
        "request_id": "req-123",
        "duration_ms": 1500,
        "fallback_count": 2,
        "services": ["prompt_helper", "story_generator"]
    }"#;

    let event: PipelineEvent = serde_json::from_str(json)
        .expect("Failed to deserialize PromptsGenerated event");

    match event {
        PipelineEvent::PromptsGenerated {
            request_id,
            duration_ms,
            fallback_count,
            services,
        } => {
            assert_eq!(request_id, "req-123");
            assert_eq!(duration_ms, 1500);
            assert_eq!(fallback_count, 2);
            assert_eq!(services.len(), 2);
        }
        _ => panic!("Wrong event variant deserialized"),
    }
}

#[test]
fn test_complete_deserialization() {
    let json = r#"{
        "type": "complete",
        "request_id": "req-999",
        "total_duration_ms": 20000,
        "total_nodes": 20,
        "total_validations": 40,
        "negotiation_rounds": 1
    }"#;

    let event: PipelineEvent = serde_json::from_str(json)
        .expect("Failed to deserialize Complete event");

    match event {
        PipelineEvent::Complete {
            request_id,
            total_duration_ms,
            total_nodes,
            total_validations,
            negotiation_rounds,
        } => {
            assert_eq!(request_id, "req-999");
            assert_eq!(total_duration_ms, 20000);
            assert_eq!(total_nodes, 20);
            assert_eq!(total_validations, 40);
            assert_eq!(negotiation_rounds, 1);
        }
        _ => panic!("Wrong event variant deserialized"),
    }
}

// ============================================================================
// EventPublisher Tests
// ============================================================================

#[tokio::test]
async fn test_event_publisher_subject_construction() {
    // This test will use a mock NATS client once implementation is ready
    // For now, just test that we can create the publisher structure

    // We'll need to create a mock NATS client for this test
    // The actual implementation will come after we implement EventPublisher

    // Placeholder: This will be expanded once EventPublisher is implemented
    // Testing that subject prefix is correctly used
    let expected_subject = format!("{}.pipeline", MCP_EVENTS_PREFIX);
    assert_eq!(expected_subject, "mcp.events.pipeline");
}

#[test]
fn test_event_roundtrip_serialization() {
    // Test that events can be serialized and deserialized without data loss
    let original = PipelineEvent::NegotiationRound {
        request_id: "req-roundtrip".to_string(),
        round: 5,
        issues_count: 10,
        corrections_applied: 8,
    };

    let json = serde_json::to_string(&original)
        .expect("Failed to serialize event");

    let deserialized: PipelineEvent = serde_json::from_str(&json)
        .expect("Failed to deserialize event");

    // Compare using JSON representation since PipelineEvent doesn't implement PartialEq
    let original_json = serde_json::to_value(&original).unwrap();
    let deserialized_json = serde_json::to_value(&deserialized).unwrap();

    assert_eq!(original_json, deserialized_json, "Event data lost in roundtrip");
}

#[test]
fn test_all_event_variants_have_request_id() {
    // Ensure all events include request_id for traceability
    let events = vec![
        PipelineEvent::PromptsGenerated {
            request_id: "req-1".to_string(),
            duration_ms: 100,
            fallback_count: 0,
            services: vec![],
        },
        PipelineEvent::StructureCreated {
            request_id: "req-2".to_string(),
            node_count: 10,
            convergence_points: 2,
        },
        PipelineEvent::BatchStarted {
            request_id: "req-3".to_string(),
            batch_id: 1,
            node_count: 5,
            nodes: vec![],
        },
        PipelineEvent::BatchCompleted {
            request_id: "req-4".to_string(),
            batch_id: 1,
            success: true,
            duration_ms: 200,
        },
        PipelineEvent::ValidationStarted {
            request_id: "req-5".to_string(),
            batch_id: 1,
            validator: "test".to_string(),
        },
        PipelineEvent::NegotiationRound {
            request_id: "req-6".to_string(),
            round: 1,
            issues_count: 3,
            corrections_applied: 2,
        },
        PipelineEvent::Complete {
            request_id: "req-7".to_string(),
            total_duration_ms: 1000,
            total_nodes: 10,
            total_validations: 20,
            negotiation_rounds: 1,
        },
        PipelineEvent::Failed {
            request_id: "req-8".to_string(),
            phase: "test".to_string(),
            error: "test error".to_string(),
            duration_ms: 500,
        },
    ];

    for event in events {
        let json = serde_json::to_string(&event).expect("Failed to serialize");
        assert!(json.contains(r#""request_id""#), "Event missing request_id: {:?}", event);
    }
}
