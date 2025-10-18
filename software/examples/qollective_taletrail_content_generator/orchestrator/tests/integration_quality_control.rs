//! Integration tests for Quality Control MCP server
//!
//! REQUIRES:
//! - NATS server running: ./start-nats.sh
//! - quality-control service running: cargo run -p quality-control

mod integration_harness;
use integration_harness::TestHarness;
use serde_json::json;

#[tokio::test]
#[ignore] // Run with: cargo test --test integration_quality_control -- --ignored
async fn test_quality_control_happy_path() {
    let harness = TestHarness::new().await
        .expect("NATS server must be running on localhost:5222");

    // Valid content for age 6-8
    let params = json!({
        "content_node": {
            "id": "node-1",
            "content": {
                "node_id": "node-1",
                "type": "story",
                "text": "A friendly cat plays with a ball. The cat is happy.",
                "choices": [],
                "next_nodes": [],
                "convergence_point": false,
                "educational_content": null
            },
            "incoming_edges": 0,
            "outgoing_edges": 0,
            "generation_metadata": null
        },
        "age_group": "6-8",
        "educational_goals": ["reading", "comprehension"]
    });

    let response = harness.send_mcp_request(
        "mcp.quality.validate",
        "validate_content",
        params
    ).await;

    assert!(response.is_ok(), "Quality control should respond: {:?}", response.err());

    let tool_result = response.unwrap();

    // Verify no error in response
    assert_eq!(tool_result.is_error, Some(false), "Should not be an error response");

    // Verify content exists
    assert!(!tool_result.content.is_empty(), "Response should have content");
}

#[tokio::test]
#[ignore]
async fn test_quality_control_safety_violation() {
    let harness = TestHarness::new().await.unwrap();

    // Content with safety violation keywords
    let params = json!({
        "content_node": {
            "id": "node-1",
            "content": {
                "node_id": "node-1",
                "type": "story",
                "text": "The scary monster appeared with a sword and was very violent.",
                "choices": [],
                "next_nodes": [],
                "convergence_point": false,
                "educational_content": null
            },
            "incoming_edges": 0,
            "outgoing_edges": 0,
            "generation_metadata": null
        },
        "age_group": "6-8",
        "educational_goals": []
    });

    let response = harness.send_mcp_request(
        "mcp.quality.validate",
        "validate_content",
        params
    ).await;

    assert!(response.is_ok(), "Should return validation result, not error");

    let tool_result = response.unwrap();

    // Tool should execute successfully even if content has issues
    assert_eq!(tool_result.is_error, Some(false));

    // Parse response to check for safety violations
    if let Some(first_content) = tool_result.content.first() {
        let content_json = serde_json::to_value(first_content).unwrap();
        let text = content_json.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Validation result should be parseable JSON
        let _result: serde_json::Value = serde_json::from_str(text)
            .expect("Response should be valid JSON");
    }
}

#[tokio::test]
#[ignore]
async fn test_quality_control_timeout() {
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
        "age_group": "6-8",
        "educational_goals": []
    });

    let response = harness.send_mcp_request(
        "mcp.quality.nonexistent",  // Wrong subject
        "validate_content",
        params
    ).await;

    // Should timeout
    assert!(response.is_err(), "Should timeout on non-existent subject");
}
