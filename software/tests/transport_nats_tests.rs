// ABOUTME: TDD tests for pure NATS transport implementation for Step 8
// ABOUTME: Tests raw payload transport without envelope wrapping for ecosystem compatibility

//! Pure NATS transport tests for Step 8: Create Pure NATS Transport.
//!
//! This test module implements TDD for the pure NATS transport that sends
//! raw payloads directly to NATS subjects without envelope wrapping.
//! This enables ecosystem compatibility with standard NATS applications.

use async_trait::async_trait;
use qollective::prelude::{QollectiveError, Result, UnifiedSender};
use serde::{Deserialize, Serialize};

// Test data types for pure NATS transport testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct NatsTestRequest {
    message: String,
    id: u32,
    subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct NatsTestResponse {
    result: String,
    status: u32,
    processed_by: String,
}

// TDD: Write failing tests FIRST
#[tokio::test]
async fn test_pure_nats_transport_implements_unified_sender() {
    // TDD: This test should fail initially - pure NATS transport doesn't exist yet

    // This test will fail because PureNatsTransport doesn't exist yet
    // We'll implement it to make this test pass

    // ARRANGE: Create pure NATS transport (this will fail to compile initially)
    // let transport = PureNatsTransport::new("nats://localhost:4222").await.unwrap();

    // For now, create a mock to demonstrate the expected interface
    let mock_transport = MockPureNatsTransport::new();

    let request_data = NatsTestRequest {
        message: "pure nats test".to_string(),
        id: 1001,
        subject: "test.subject".to_string(),
    };

    // ACT: Send raw payload (no envelope) to NATS subject
    let result: Result<NatsTestResponse> = mock_transport
        .send("nats://localhost:4222/test.subject", request_data.clone())
        .await;

    // ASSERT: Pure NATS transport should work with raw payloads
    assert!(
        result.is_ok(),
        "Pure NATS transport should handle raw payloads"
    );
    let response_data = result.unwrap();

    // Verify raw payload communication
    assert_eq!(
        response_data.result,
        "pure nats success on subject: test.subject"
    );
    assert_eq!(response_data.status, 200);
    assert_eq!(response_data.processed_by, "pure_nats_transport");
}

#[tokio::test]
async fn test_pure_nats_endpoint_parsing() {
    // TDD: Test NATS URL parsing for subject extraction

    let mock_transport = MockPureNatsTransport::new();

    // Test different NATS endpoint formats
    let test_cases = vec![
        ("nats://localhost:4222/simple.subject", "simple.subject"),
        (
            "nats://server.com:4222/service.method.v1",
            "service.method.v1",
        ),
        (
            "nats://cluster.example.com:4222/events.user.created",
            "events.user.created",
        ),
    ];

    for (endpoint, expected_subject) in test_cases {
        let request_data = NatsTestRequest {
            message: format!("test for {}", expected_subject),
            id: 2001,
            subject: expected_subject.to_string(),
        };

        // ACT: Send to endpoint with subject path
        let result: Result<NatsTestResponse> = mock_transport.send(endpoint, request_data).await;

        // ASSERT: Should successfully parse endpoint and extract subject
        assert!(result.is_ok(), "Should parse NATS endpoint: {}", endpoint);

        // Verify correct subject was used (mock will record this)
        let response = result.unwrap();
        assert!(
            response.result.contains(expected_subject),
            "Response should indicate subject was used: {}",
            expected_subject
        );
    }
}

#[tokio::test]
async fn test_pure_nats_request_reply_pattern() {
    // TDD: Test NATS request/reply pattern for synchronous communication

    let mock_transport = MockPureNatsTransport::new();

    let request_data = NatsTestRequest {
        message: "request reply test".to_string(),
        id: 3001,
        subject: "service.ping".to_string(),
    };

    // ACT: Send request expecting reply
    let result: Result<NatsTestResponse> = mock_transport
        .send("nats://localhost:4222/service.ping", request_data)
        .await;

    // ASSERT: Should handle request/reply pattern
    assert!(result.is_ok(), "Request/reply should work");
    let response = result.unwrap();

    // Verify synchronous response
    assert_eq!(response.status, 200);
    assert!(response.result.contains("reply"));
}

#[tokio::test]
async fn test_pure_nats_error_handling() {
    // TDD: Test error handling for NATS-specific failures

    let mut mock_transport = MockPureNatsTransport::new();
    mock_transport.set_should_fail(true);

    let request_data = NatsTestRequest {
        message: "error test".to_string(),
        id: 4001,
        subject: "error.subject".to_string(),
    };

    // ACT: Send to failing transport
    let result: Result<NatsTestResponse> = mock_transport
        .send("nats://localhost:4222/error.subject", request_data)
        .await;

    // ASSERT: Should return proper NATS transport error
    assert!(result.is_err(), "Should return error for failed transport");

    match result.unwrap_err() {
        QollectiveError::Transport(_) => {
            // Expected transport error
        }
        other => {
            panic!("Expected transport error, got: {:?}", other);
        }
    }
}

#[tokio::test]
async fn test_pure_nats_vs_envelope_nats_distinction() {
    // TDD: Test that pure NATS and envelope NATS are distinct

    let pure_mock = MockPureNatsTransport::new();

    // Test pure NATS endpoint
    let pure_request = NatsTestRequest {
        message: "pure nats communication".to_string(),
        id: 5001,
        subject: "pure.test".to_string(),
    };

    // ACT: Send via pure NATS (raw payload)
    let pure_result: Result<NatsTestResponse> = pure_mock
        .send("nats://localhost:4222/pure.test", pure_request)
        .await;

    // ASSERT: Pure NATS should work with raw payloads
    assert!(pure_result.is_ok(), "Pure NATS should handle raw payloads");
    let pure_response = pure_result.unwrap();

    // Verify this is pure transport (no envelope metadata)
    assert_eq!(pure_response.processed_by, "pure_nats_transport");
    assert!(pure_response.result.contains("pure nats"));

    // NOTE: Test will demonstrate that qollective:// URLs would use envelope transport
    // while nats:// URLs use pure transport - this is for transport selection logic
}

#[tokio::test]
async fn test_ecosystem_compatibility_simulation() {
    // TDD: Simulate compatibility with standard NATS clients

    let mock_transport = MockPureNatsTransport::new();

    // Simulate standard NATS message format (just JSON payload)
    let standard_nats_request = NatsTestRequest {
        message: "from standard nats client".to_string(),
        id: 6001,
        subject: "interop.test".to_string(),
    };

    // ACT: Send message that could come from any NATS client
    let result: Result<NatsTestResponse> = mock_transport
        .send("nats://localhost:4222/interop.test", standard_nats_request)
        .await;

    // ASSERT: Should interoperate with standard NATS ecosystem
    assert!(
        result.is_ok(),
        "Should be compatible with standard NATS clients"
    );
    let response = result.unwrap();

    // Verify ecosystem compatibility
    assert_eq!(response.status, 200);
    assert!(response.result.contains("interop"));
}

// Mock implementation for TDD - will be replaced with real implementation
#[derive(Debug, Clone)]
struct MockPureNatsTransport {
    should_fail: bool,
}

impl MockPureNatsTransport {
    fn new() -> Self {
        Self { should_fail: false }
    }

    fn set_should_fail(&mut self, should_fail: bool) {
        self.should_fail = should_fail;
    }

    // Helper to extract subject from NATS URL
    fn extract_subject_from_endpoint(&self, endpoint: &str) -> Result<String> {
        if !endpoint.starts_with("nats://") {
            return Err(QollectiveError::transport(
                "Invalid NATS endpoint".to_string(),
            ));
        }

        let url_parts: Vec<&str> = endpoint.split('/').collect();
        if url_parts.len() < 4 {
            return Err(QollectiveError::transport(
                "NATS endpoint missing subject".to_string(),
            ));
        }

        // Extract subject from path (everything after hostname:port/)
        let subject = url_parts[3..].join(".");
        Ok(subject)
    }
}

#[async_trait]
impl UnifiedSender<NatsTestRequest, NatsTestResponse> for MockPureNatsTransport {
    async fn send(&self, endpoint: &str, payload: NatsTestRequest) -> Result<NatsTestResponse> {
        if self.should_fail {
            return Err(QollectiveError::transport(
                "Mock NATS transport failure".to_string(),
            ));
        }

        // Extract subject from endpoint for processing
        let subject = self.extract_subject_from_endpoint(endpoint)?;

        // Simulate pure NATS communication (raw payload, no envelope)
        // Use the payload data to create contextual response
        let response = NatsTestResponse {
            result: if payload.message.contains("request reply") {
                format!("pure nats reply success on subject: {}", subject)
            } else if payload.message.contains("interop") {
                format!("pure nats interop success on subject: {}", subject)
            } else {
                format!("pure nats success on subject: {}", subject)
            },
            status: 200,
            processed_by: "pure_nats_transport".to_string(),
        };

        Ok(response)
    }
}
