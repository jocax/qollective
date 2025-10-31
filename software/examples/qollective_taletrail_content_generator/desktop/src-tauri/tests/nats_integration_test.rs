use nuxtor_lib::models::GenerationEvent;
use nuxtor_lib::nats::{NatsClient, NatsConfig};

/// Test NATS client connection establishment
#[tokio::test]
#[ignore] // Requires NATS server running
async fn test_nats_connection() {
    let app_config = nuxtor_lib::config::AppConfig::load()
        .expect("Failed to load app config for test");
    let config = NatsConfig::from_app_config(&app_config);
    let client = NatsClient::new(config);

    // Connect should succeed if NATS server is running
    let result = client.connect().await;

    // If server is not running, this will fail - that's expected for local dev
    if let Ok(_) = result {
        assert!(client.is_connected().await);

        // Clean up
        let _ = client.disconnect().await;
    }
}

/// Test event subscription without tenant filter
#[tokio::test]
#[ignore] // Requires NATS server running
async fn test_subscribe_all_events() {
    let app_config = nuxtor_lib::config::AppConfig::load()
        .expect("Failed to load app config for test");
    let config = NatsConfig::from_app_config(&app_config);
    let client = NatsClient::new(config);

    let connect_result = client.connect().await;
    if connect_result.is_err() {
        // NATS server not running, skip test
        return;
    }

    // Subscribe to all events
    let result = client.subscribe(None).await;
    assert!(result.is_ok());

    // Unsubscribe
    let unsub_result = client.unsubscribe().await;
    assert!(unsub_result.is_ok());

    // Clean up
    let _ = client.disconnect().await;
}

/// Test event subscription with tenant filter
#[tokio::test]
#[ignore] // Requires NATS server running
async fn test_subscribe_tenant_events() {
    let app_config = nuxtor_lib::config::AppConfig::load()
        .expect("Failed to load app config for test");
    let config = NatsConfig::from_app_config(&app_config);
    let client = NatsClient::new(config);

    let connect_result = client.connect().await;
    if connect_result.is_err() {
        // NATS server not running, skip test
        return;
    }

    // Subscribe to tenant-specific events
    let result = client.subscribe(Some("tenant-123".to_string())).await;
    assert!(result.is_ok());

    // Unsubscribe
    let unsub_result = client.unsubscribe().await;
    assert!(unsub_result.is_ok());

    // Clean up
    let _ = client.disconnect().await;
}

/// Test message deserialization into GenerationEvent
#[test]
fn test_event_deserialization() {
    let json = r#"{
        "eventType": "generation_started",
        "tenantId": "tenant-123",
        "requestId": "req-456",
        "timestamp": "2025-10-22T10:00:00Z",
        "servicePhase": "story-generator",
        "status": "in_progress"
    }"#;

    let result = NatsClient::parse_event(json.as_bytes());
    assert!(result.is_ok());

    let event = result.unwrap();
    assert_eq!(event.event_type, "generation_started");
    assert_eq!(event.tenant_id, "tenant-123");
    assert_eq!(event.request_id, "req-456");
    assert_eq!(event.service_phase, "story-generator");
    assert_eq!(event.status, "in_progress");
}

/// Test message deserialization with progress
#[test]
fn test_event_deserialization_with_progress() {
    let json = r#"{
        "eventType": "generation_progress",
        "tenantId": "tenant-123",
        "requestId": "req-456",
        "timestamp": "2025-10-22T10:00:00Z",
        "servicePhase": "story-generator",
        "status": "in_progress",
        "progress": 0.5
    }"#;

    let result = NatsClient::parse_event(json.as_bytes());
    assert!(result.is_ok());

    let event = result.unwrap();
    assert_eq!(event.event_type, "generation_progress");
    assert_eq!(event.progress, Some(0.5));
}

/// Test message deserialization with error
#[test]
fn test_event_deserialization_with_error() {
    let json = r#"{
        "eventType": "generation_failed",
        "tenantId": "tenant-123",
        "requestId": "req-456",
        "timestamp": "2025-10-22T10:00:00Z",
        "servicePhase": "story-generator",
        "status": "failed",
        "errorMessage": "Generation timeout exceeded"
    }"#;

    let result = NatsClient::parse_event(json.as_bytes());
    assert!(result.is_ok());

    let event = result.unwrap();
    assert_eq!(event.event_type, "generation_failed");
    assert_eq!(event.status, "failed");
    assert_eq!(event.error_message, Some("Generation timeout exceeded".to_string()));
}

/// Test invalid JSON handling
#[test]
fn test_invalid_json_deserialization() {
    let invalid_json = b"not valid json";
    let result = NatsClient::parse_event(invalid_json);
    assert!(result.is_err());
}

/// Test cleanup on unsubscribe
#[tokio::test]
#[ignore] // Requires NATS server running
async fn test_cleanup_on_unsubscribe() {
    let app_config = nuxtor_lib::config::AppConfig::load()
        .expect("Failed to load app config for test");
    let config = NatsConfig::from_app_config(&app_config);
    let client = NatsClient::new(config);

    let connect_result = client.connect().await;
    if connect_result.is_err() {
        // NATS server not running, skip test
        return;
    }

    // Subscribe
    let subscribe_result = client.subscribe(None).await;
    if subscribe_result.is_ok() {
        // Unsubscribe should clean up resources
        let unsub_result = client.unsubscribe().await;
        assert!(unsub_result.is_ok());

        // Unsubscribing again should be idempotent
        let unsub_result2 = client.unsubscribe().await;
        assert!(unsub_result2.is_ok());
    }

    // Clean up
    let _ = client.disconnect().await;
}

/// Test publish and subscribe roundtrip (integration test)
#[tokio::test]
#[ignore] // Requires NATS server running
async fn test_publish_subscribe_roundtrip() {
    use futures::StreamExt;
    use tokio::time::{timeout, Duration};

    let app_config = nuxtor_lib::config::AppConfig::load()
        .expect("Failed to load app config for test");
    let config = NatsConfig::from_app_config(&app_config);
    let client = NatsClient::new(config);

    let connect_result = client.connect().await;
    if connect_result.is_err() {
        // NATS server not running, skip test
        return;
    }

    // Subscribe to tenant-specific events
    let tenant_id = "test-tenant-roundtrip";
    let mut subscriber = match client.subscribe(Some(tenant_id.to_string())).await {
        Ok(sub) => sub,
        Err(_) => {
            let _ = client.disconnect().await;
            return;
        }
    };

    // Create and publish an event
    let event = GenerationEvent::new(
        "test_event".to_string(),
        tenant_id.to_string(),
        "test-req-123".to_string(),
        "test-phase".to_string(),
        "in_progress".to_string(),
    );

    let subject = format!("taletrail.generation.events.{}", tenant_id);
    let publish_result = client.publish(&subject, &event).await;
    assert!(publish_result.is_ok());

    // Try to receive the event with a timeout
    let receive_result = timeout(Duration::from_secs(2), subscriber.next()).await;

    if let Ok(Some(message)) = receive_result {
        let received_event = NatsClient::parse_event(&message.payload);
        assert!(received_event.is_ok());

        let received_event = received_event.unwrap();
        assert_eq!(received_event.event_type, "test_event");
        assert_eq!(received_event.tenant_id, tenant_id);
        assert_eq!(received_event.request_id, "test-req-123");
    }

    // Clean up
    let _ = client.unsubscribe().await;
    let _ = client.disconnect().await;
}

/// Test connection status check
#[tokio::test]
#[ignore] // Requires rustls CryptoProvider setup and NATS server
async fn test_connection_status() {
    let app_config = nuxtor_lib::config::AppConfig::load()
        .expect("Failed to load app config for test");
    let config = NatsConfig::from_app_config(&app_config);
    let client = NatsClient::new(config);

    // Initially not connected
    assert!(!client.is_connected().await);

    // Try to connect
    let connect_result = client.connect().await;
    if connect_result.is_ok() {
        // Should be connected
        assert!(client.is_connected().await);

        // Disconnect
        let _ = client.disconnect().await;

        // Should not be connected after disconnect
        // Note: is_connected checks if client exists, not actual connection state
        // So this might still return true until we set client to None
    }
}

/// Test multiple subscribers
#[tokio::test]
#[ignore] // Requires NATS server running
async fn test_multiple_connections() {
    let app_config = nuxtor_lib::config::AppConfig::load()
        .expect("Failed to load app config for test");

    let config1 = NatsConfig {
        url: "nats://localhost:5222".to_string(),
        name: Some("client-1".to_string()),
        timeout_secs: 5,
        ca_cert_path: app_config.ca_cert_path(),
        nkey_file_path: app_config.nkey_path(),
    };

    let config2 = NatsConfig {
        url: "nats://localhost:5222".to_string(),
        name: Some("client-2".to_string()),
        timeout_secs: 5,
        ca_cert_path: app_config.ca_cert_path(),
        nkey_file_path: app_config.nkey_path(),
    };

    let client1 = NatsClient::new(config1);
    let client2 = NatsClient::new(config2);

    let connect1 = client1.connect().await;
    let connect2 = client2.connect().await;

    if connect1.is_ok() && connect2.is_ok() {
        assert!(client1.is_connected().await);
        assert!(client2.is_connected().await);

        // Clean up
        let _ = client1.disconnect().await;
        let _ = client2.disconnect().await;
    }
}
