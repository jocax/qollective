//! Integration tests for Story Generator MCP server
//!
//! REQUIRES:
//! - NATS server running: ./start-nats.sh
//! - story-generator service running: cargo run -p story-generator

mod integration_harness;
use integration_harness::TestHarness;
use serde_json::json;

#[tokio::test]
#[ignore] // Run with: cargo test --test integration_story_generator -- --ignored
async fn test_story_generator_generate_structure_happy_path() {
    let harness = TestHarness::new().await
        .expect("NATS server must be running on localhost:5222");

    // Valid structure generation request
    let params = json!({
        "requirements": {
            "age_group": "6-8",
            "theme": "animals",
            "educational_goals": ["reading", "comprehension"],
            "num_nodes": 5,
            "branching_factor": 2,
            "vocabulary_level": "basic"
        }
    });

    let response = harness.send_mcp_request(
        "mcp.story.generate",
        "generate_structure",
        params
    ).await;

    assert!(response.is_ok(), "Story generator should respond: {:?}", response.err());

    let tool_result = response.unwrap();

    // Verify no error in response
    assert_eq!(tool_result.is_error, Some(false), "Should not be an error response");

    // Verify content exists
    assert!(!tool_result.content.is_empty(), "Response should have content");
}

#[tokio::test]
#[ignore]
async fn test_story_generator_validate_paths_happy_path() {
    let harness = TestHarness::new().await.unwrap();

    // Valid graph with proper paths
    let params = json!({
        "graph": {
            "nodes": [
                {
                    "id": "node-1",
                    "content": {
                        "node_id": "node-1",
                        "type": "story",
                        "text": "Start of the story.",
                        "choices": [],
                        "next_nodes": ["node-2"],
                        "convergence_point": false,
                        "educational_content": null
                    },
                    "incoming_edges": 0,
                    "outgoing_edges": 1,
                    "generation_metadata": null
                },
                {
                    "id": "node-2",
                    "content": {
                        "node_id": "node-2",
                        "type": "story",
                        "text": "End of the story.",
                        "choices": [],
                        "next_nodes": [],
                        "convergence_point": false,
                        "educational_content": null
                    },
                    "incoming_edges": 1,
                    "outgoing_edges": 0,
                    "generation_metadata": null
                }
            ],
            "start_node_id": "node-1"
        }
    });

    let response = harness.send_mcp_request(
        "mcp.story.validate",
        "validate_paths",
        params
    ).await;

    assert!(response.is_ok(), "Should validate paths successfully: {:?}", response.err());

    let tool_result = response.unwrap();

    // Verify no error in response
    assert_eq!(tool_result.is_error, Some(false), "Should not be an error response");
}

#[tokio::test]
#[ignore]
async fn test_story_generator_invalid_parameters() {
    let harness = TestHarness::new().await.unwrap();

    // Invalid parameters - missing required fields
    let params = json!({
        "invalid_field": "value"
    });

    let response = harness.send_mcp_request(
        "mcp.story.generate",
        "generate_structure",
        params
    ).await;

    // Should fail due to parameter validation
    assert!(response.is_err() || response.unwrap().is_error == Some(true),
        "Should fail with invalid parameters");
}

#[tokio::test]
#[ignore]
async fn test_story_generator_unknown_tool() {
    let harness = TestHarness::new().await.unwrap();

    let params = json!({
        "requirements": {
            "age_group": "6-8",
            "theme": "animals",
            "educational_goals": [],
            "num_nodes": 5,
            "branching_factor": 2,
            "vocabulary_level": "basic"
        }
    });

    let response = harness.send_mcp_request(
        "mcp.story.generate",
        "unknown_tool",  // Tool doesn't exist
        params
    ).await;

    // Should fail with unknown tool error
    assert!(response.is_err() || response.unwrap().is_error == Some(true),
        "Should fail with unknown tool");
}

#[tokio::test]
#[ignore]
async fn test_story_generator_timeout() {
    let harness = TestHarness::new().await.unwrap();

    // Send request to non-existent subject to test timeout
    let params = json!({
        "requirements": {
            "age_group": "6-8",
            "theme": "animals",
            "educational_goals": [],
            "num_nodes": 5,
            "branching_factor": 2,
            "vocabulary_level": "basic"
        }
    });

    let response = harness.send_mcp_request(
        "mcp.story.nonexistent",  // Wrong subject
        "generate_structure",
        params
    ).await;

    // Should timeout
    assert!(response.is_err(), "Should timeout on non-existent subject");
}
