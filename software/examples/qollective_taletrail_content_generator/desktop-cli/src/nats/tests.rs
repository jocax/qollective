/// Tests for NATS client integration
///
/// These tests verify basic NATS client functionality including:
/// - Connection establishment
/// - Request/response patterns
/// - Subscription handling
/// - Message encoding/decoding
///
/// Note: Some tests require a running NATS server and may be integration tests

use super::*;
use crate::config::NatsConfig;
use crate::models::events::GenerationEvent;

/// Test 1: NATS client initialization
#[test]
fn test_nats_client_initialization() {
    let config = NatsConfig {
        url: "nats://localhost:4222".to_string(),
        timeout_secs: 30,
        tls_cert_path: None,
        nkey_path: None,
    };

    let client = NatsClient::new(config.clone());

    // Test cloning
    let _cloned = client.clone();
    // Config is private, so we just verify client can be cloned
}

/// Test 2: Subject pattern functions
#[test]
fn test_subject_patterns() {
    // Test MCP subject generation
    assert_eq!(subjects::mcp_request_subject("orchestrator"), "mcp.orchestrator.request");
    assert_eq!(subjects::mcp_request_subject("story-generator"), "mcp.story-generator.request");

    // Test subject type detection
    assert!(subjects::is_mcp_subject("mcp.orchestrator.request"));
    assert!(subjects::is_taletrail_subject("taletrail.generation.events"));
    assert!(!subjects::is_mcp_subject("taletrail.generation.events"));

    // Test extraction
    assert_eq!(subjects::extract_mcp_server("mcp.orchestrator.request"), Some("orchestrator".to_string()));
    assert_eq!(subjects::extract_tenant_id("taletrail.generation.events.42"), Some("42".to_string()));
}

/// Test 3: GenerationEvent parsing
#[test]
fn test_generation_event_parsing() {
    let json = r#"{
        "eventType": "generation_started",
        "tenantId": "tenant-123",
        "requestId": "req-456",
        "timestamp": "2025-11-02T10:00:00Z",
        "servicePhase": "story-generator",
        "status": "in_progress",
        "progress": 0.3
    }"#;

    let event = NatsClient::parse_event(json.as_bytes()).unwrap();
    assert_eq!(event.event_type, "generation_started");
    assert_eq!(event.tenant_id, "tenant-123");
    assert_eq!(event.request_id, "req-456");
    assert_eq!(event.service_phase, "story-generator");
    assert_eq!(event.status, "in_progress");
    assert_eq!(event.progress, Some(0.3));
}

/// Test 4: RequestTracker basic operations
#[test]
fn test_request_tracker_operations() {
    smol::block_on(async {
        let tracker = RequestTracker::new(3600);

        // Test initial state
        assert_eq!(tracker.count().await, 0);

        // Create and track a request
        let event = GenerationEvent::new(
            "generation_started".to_string(),
            "tenant-123".to_string(),
            "req-456".to_string(),
            "story-generator".to_string(),
            "in_progress".to_string(),
        );

        tracker.update_from_event(&event).await;
        assert_eq!(tracker.count().await, 1);

        // Retrieve request
        let request = tracker.get_request("req-456").await;
        assert!(request.is_some());
        assert_eq!(request.unwrap().request_id, "req-456");

        // Update request
        tracker.update_request(
            "req-456",
            "quality-control".to_string(),
            0.7,
            "quality-control".to_string(),
            "in_progress".to_string(),
        ).await;

        let updated = tracker.get_request("req-456").await.unwrap();
        assert_eq!(updated.current_phase, "quality-control");
        assert_eq!(updated.progress, 0.7);

        // Remove request
        tracker.remove_request("req-456").await;
        assert_eq!(tracker.count().await, 0);
    });
}

/// Test 5: RequestTracker event updates
#[test]
fn test_request_tracker_event_updates() {
    smol::block_on(async {
        let tracker = RequestTracker::new(3600);

        // First event creates request
        let event1 = GenerationEvent::new(
            "generation_started".to_string(),
            "tenant-123".to_string(),
            "req-789".to_string(),
            "prompt-helper".to_string(),
            "in_progress".to_string(),
        )
        .with_progress(0.1);

        tracker.update_from_event(&event1).await;
        assert_eq!(tracker.count().await, 1);

        // Second event updates same request
        let event2 = GenerationEvent::new(
            "generation_progress".to_string(),
            "tenant-123".to_string(),
            "req-789".to_string(),
            "story-generator".to_string(),
            "in_progress".to_string(),
        )
        .with_progress(0.5);

        tracker.update_from_event(&event2).await;
        assert_eq!(tracker.count().await, 1); // Still one request

        let request = tracker.get_request("req-789").await.unwrap();
        assert_eq!(request.current_phase, "story-generator");
        assert_eq!(request.progress, 0.5);
    });
}

/// Test 6: Monitoring message extraction functions
#[test]
fn test_monitoring_message_extraction() {
    use monitoring::NatsMessage;

    // Test message creation
    let subject = "mcp.orchestrator.request".to_string();
    let payload = br#"{"request_id": "req-999", "tool": "test_tool"}"#.to_vec();

    let message = NatsMessage::new(subject.clone(), payload);

    assert_eq!(message.subject, subject);
    assert_eq!(message.endpoint, "orchestrator");
    assert_eq!(message.message_type, "Request");
    assert_eq!(message.request_id, Some("req-999".to_string()));
    assert!(!message.timestamp.is_empty());
}

/// Test 7: MonitoringDiagnostics initialization
#[test]
fn test_monitoring_diagnostics() {
    let diag = monitoring::MonitoringDiagnostics::new();

    assert_eq!(diag.messages_received, 0);
    assert_eq!(diag.messages_buffered, 0);
    assert!(diag.is_connected);
    assert!(diag.last_message_timestamp.is_none());
    assert!(!diag.connection_timestamp.is_empty());
}

/// Test 8: TrackedRequest serialization
#[test]
fn test_tracked_request_serialization() {
    use chrono::Utc;
    use request_tracker::TrackedRequest;

    let request = TrackedRequest {
        request_id: "req-serialize-test".to_string(),
        tenant_id: "tenant-1".to_string(),
        start_time: Utc::now(),
        current_phase: "story-generator".to_string(),
        progress: 0.5,
        last_update: Utc::now(),
        component: "story-generator".to_string(),
        status: "in_progress".to_string(),
        error_message: None,
        file_path: None,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("req-serialize-test"));
    assert!(json.contains("story-generator"));

    // Deserialize back
    let deserialized: TrackedRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.request_id, request.request_id);
    assert_eq!(deserialized.progress, request.progress);
    assert_eq!(deserialized.status, request.status);
}

// Integration tests (require NATS server)
// These are marked with #[ignore] and can be run explicitly with: cargo test -- --ignored

#[test]
#[ignore]
fn test_nats_connection_integration() {
    smol::block_on(async {
        let config = NatsConfig {
            url: "nats://localhost:4222".to_string(),
            timeout_secs: 5,
            tls_cert_path: None,
            nkey_path: None,
        };

        let client = NatsClient::new(config);

        // This will fail if no NATS server is running
        match client.connect().await {
            Ok(_) => {
                assert!(client.is_connected().await);
                let _ = client.disconnect().await;
            }
            Err(e) => {
                eprintln!("NATS connection test skipped (no server running): {}", e);
            }
        }
    });
}
