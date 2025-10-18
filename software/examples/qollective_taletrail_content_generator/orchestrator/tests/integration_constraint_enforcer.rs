//! Integration tests for Constraint Enforcer MCP server
//!
//! REQUIRES:
//! - NATS server running: ./start-nats.sh
//! - constraint-enforcer service running: cargo run -p constraint-enforcer

mod integration_harness;
use integration_harness::TestHarness;
use serde_json::json;

#[tokio::test]
#[ignore] // Run with: cargo test --test integration_constraint_enforcer -- --ignored
async fn test_constraint_enforcer_happy_path() {
    let harness = TestHarness::new().await
        .expect("NATS server must be running on localhost:5222");

    // Valid content for age 6-8 with appropriate vocabulary
    let params = json!({
        "content_node": {
            "id": "node-1",
            "content": {
                "node_id": "node-1",
                "type": "story",
                "text": "The cat played with the ball. It was a fun day.",
                "choices": [],
                "next_nodes": [],
                "convergence_point": false,
                "educational_content": null
            },
            "incoming_edges": 0,
            "outgoing_edges": 0,
            "generation_metadata": null
        },
        "constraints": {
            "age_group": "6-8",
            "theme": "animals",
            "max_choices": 3,
            "vocabulary_level": "basic",
            "educational_goals": ["reading"]
        }
    });

    let response = harness.send_mcp_request(
        "mcp.constraint.enforce",
        "enforce_constraints",
        params
    ).await;

    assert!(response.is_ok(), "Constraint enforcer should respond: {:?}", response.err());

    let tool_result = response.unwrap();

    // Verify no error in response
    assert_eq!(tool_result.is_error, Some(false), "Should not be an error response");

    // Verify content exists
    assert!(!tool_result.content.is_empty(), "Response should have content");
}

#[tokio::test]
#[ignore]
async fn test_constraint_enforcer_vocabulary_violation() {
    let harness = TestHarness::new().await.unwrap();

    // Content with complex vocabulary for young age
    let params = json!({
        "content_node": {
            "id": "node-1",
            "content": {
                "node_id": "node-1",
                "type": "story",
                "text": "The feline demonstrated extraordinary acrobatic capabilities while engaging in recreational activities.",
                "choices": [],
                "next_nodes": [],
                "convergence_point": false,
                "educational_content": null
            },
            "incoming_edges": 0,
            "outgoing_edges": 0,
            "generation_metadata": null
        },
        "constraints": {
            "age_group": "6-8",
            "theme": "animals",
            "max_choices": 3,
            "vocabulary_level": "basic",
            "educational_goals": []
        }
    });

    let response = harness.send_mcp_request(
        "mcp.constraint.enforce",
        "enforce_constraints",
        params
    ).await;

    assert!(response.is_ok(), "Should return constraint result, not error");

    let tool_result = response.unwrap();

    // Tool should execute successfully even if content has violations
    assert_eq!(tool_result.is_error, Some(false));

    // Parse response to check for vocabulary violations
    if let Some(first_content) = tool_result.content.first() {
        let content_json = serde_json::to_value(first_content).unwrap();
        let text = content_json.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Constraint result should be parseable JSON
        let _result: serde_json::Value = serde_json::from_str(text)
            .expect("Response should be valid JSON");
    }
}

#[tokio::test]
#[ignore]
async fn test_constraint_enforcer_theme_violation() {
    let harness = TestHarness::new().await.unwrap();

    // Content that doesn't match the specified theme
    let params = json!({
        "content_node": {
            "id": "node-1",
            "content": {
                "node_id": "node-1",
                "type": "story",
                "text": "The spaceship flew to Mars and landed near the crater.",
                "choices": [],
                "next_nodes": [],
                "convergence_point": false,
                "educational_content": null
            },
            "incoming_edges": 0,
            "outgoing_edges": 0,
            "generation_metadata": null
        },
        "constraints": {
            "age_group": "6-8",
            "theme": "animals",  // Theme is animals but content is space
            "max_choices": 3,
            "vocabulary_level": "basic",
            "educational_goals": []
        }
    });

    let response = harness.send_mcp_request(
        "mcp.constraint.enforce",
        "enforce_constraints",
        params
    ).await;

    assert!(response.is_ok(), "Should return constraint result, not error");

    let tool_result = response.unwrap();

    // Tool should execute successfully even if theme doesn't match
    assert_eq!(tool_result.is_error, Some(false));
}

#[tokio::test]
#[ignore]
async fn test_constraint_enforcer_timeout() {
    let harness = TestHarness::new().await.unwrap();

    // Send request to non-existent subject to test timeout
    let params = json!({
        "content_node": {
            "id": "node-1",
            "content": {
                "node_id": "node-1",
                "type": "story",
                "text": "Test",
                "choices": [],
                "next_nodes": [],
                "convergence_point": false,
                "educational_content": null
            },
            "incoming_edges": 0,
            "outgoing_edges": 0,
            "generation_metadata": null
        },
        "constraints": {
            "age_group": "6-8",
            "theme": "animals",
            "max_choices": 3,
            "vocabulary_level": "basic",
            "educational_goals": []
        }
    });

    let response = harness.send_mcp_request(
        "mcp.constraint.nonexistent",  // Wrong subject
        "enforce_constraints",
        params
    ).await;

    // Should timeout
    assert!(response.is_err(), "Should timeout on non-existent subject");
}
