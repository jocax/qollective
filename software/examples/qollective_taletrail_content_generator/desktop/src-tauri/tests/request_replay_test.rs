use nuxtor_lib::error::AppError;
use nuxtor_lib::models::{GenerationRequest, RequestConstraints, RequestMetadata};
use nuxtor_lib::nats::{NatsClient, NatsConfig};
use nuxtor_lib::constants::validation;

/// Test GenerationRequest type deserialization from JSON
#[test]
fn test_generation_request_deserialization() {
    let json = r#"{
        "requestId": "req-123",
        "tenantId": "tenant-456",
        "theme": "Space Adventure",
        "ageGroup": "6-8",
        "language": "en",
        "vocabularyLevel": "simple",
        "nodeCount": 10
    }"#;

    let request: GenerationRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.request_id, "req-123");
    assert_eq!(request.tenant_id, "tenant-456");
    assert_eq!(request.theme, "Space Adventure");
    assert_eq!(request.age_group, "6-8");
    assert_eq!(request.language, "en");
    assert_eq!(request.vocabulary_level, "simple");
    assert_eq!(request.node_count, 10);
}

/// Test GenerationRequest deserialization with all optional fields
#[test]
fn test_generation_request_full_deserialization() {
    let json = r#"{
        "requestId": "req-123",
        "tenantId": "tenant-456",
        "theme": "Space Adventure",
        "ageGroup": "6-8",
        "language": "en",
        "vocabularyLevel": "simple",
        "nodeCount": 10,
        "educationalFocus": ["science", "teamwork"],
        "constraints": {
            "maxChoicesPerNode": 4,
            "minStoryLength": 500,
            "forbiddenTopics": ["violence"],
            "requiredTopics": ["friendship"]
        },
        "metadata": {
            "submittedAt": "2025-10-22T10:00:00Z",
            "submittedBy": "user-123",
            "originalRequestId": "req-original"
        }
    }"#;

    let request: GenerationRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.request_id, "req-123");
    assert!(request.educational_focus.is_some());
    assert_eq!(request.educational_focus.unwrap().len(), 2);

    assert!(request.constraints.is_some());
    let constraints = request.constraints.unwrap();
    assert_eq!(constraints.max_choices_per_node, Some(4));
    assert_eq!(constraints.min_story_length, Some(500));

    assert!(request.metadata.is_some());
    let metadata = request.metadata.unwrap();
    assert_eq!(metadata.submitted_by, Some("user-123".to_string()));
}

/// Test GenerationRequest validation with valid data
#[test]
fn test_generation_request_validation_success() {
    let request = GenerationRequest {
        request_id: "req-123".to_string(),
        tenant_id: "tenant-456".to_string(),
        theme: "Space Adventure".to_string(),
        age_group: "6-8".to_string(),
        language: "en".to_string(),
        vocabulary_level: "simple".to_string(),
        node_count: 10,
        educational_focus: None,
        constraints: None,
        metadata: None,
    };

    assert!(request.validate().is_ok());
}

/// Test validation fails for empty request_id
#[test]
fn test_validation_empty_request_id() {
    let request = GenerationRequest {
        request_id: "".to_string(),
        tenant_id: "tenant-456".to_string(),
        theme: "Space Adventure".to_string(),
        age_group: "6-8".to_string(),
        language: "en".to_string(),
        vocabulary_level: "simple".to_string(),
        node_count: 10,
        educational_focus: None,
        constraints: None,
        metadata: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    match result {
        Err(AppError::ValidationError(msg)) => {
            assert!(msg.contains("request_id"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation fails for invalid age group
#[test]
fn test_validation_invalid_age_group() {
    let request = GenerationRequest {
        request_id: "req-123".to_string(),
        tenant_id: "tenant-456".to_string(),
        theme: "Space Adventure".to_string(),
        age_group: "invalid-age".to_string(),
        language: "en".to_string(),
        vocabulary_level: "simple".to_string(),
        node_count: 10,
        educational_focus: None,
        constraints: None,
        metadata: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    match result {
        Err(AppError::ValidationError(msg)) => {
            assert!(msg.contains("age_group"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation fails for invalid language code
#[test]
fn test_validation_invalid_language() {
    let request = GenerationRequest {
        request_id: "req-123".to_string(),
        tenant_id: "tenant-456".to_string(),
        theme: "Space Adventure".to_string(),
        age_group: "6-8".to_string(),
        language: "invalid-lang".to_string(),
        vocabulary_level: "simple".to_string(),
        node_count: 10,
        educational_focus: None,
        constraints: None,
        metadata: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    match result {
        Err(AppError::ValidationError(msg)) => {
            assert!(msg.contains("language"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation fails for invalid vocabulary level
#[test]
fn test_validation_invalid_vocabulary_level() {
    let request = GenerationRequest {
        request_id: "req-123".to_string(),
        tenant_id: "tenant-456".to_string(),
        theme: "Space Adventure".to_string(),
        age_group: "6-8".to_string(),
        language: "en".to_string(),
        vocabulary_level: "expert".to_string(),
        node_count: 10,
        educational_focus: None,
        constraints: None,
        metadata: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    match result {
        Err(AppError::ValidationError(msg)) => {
            assert!(msg.contains("vocabulary_level"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation fails for node count below minimum
#[test]
fn test_validation_node_count_too_low() {
    let request = GenerationRequest {
        request_id: "req-123".to_string(),
        tenant_id: "tenant-456".to_string(),
        theme: "Space Adventure".to_string(),
        age_group: "6-8".to_string(),
        language: "en".to_string(),
        vocabulary_level: "simple".to_string(),
        node_count: validation::MIN_NODE_COUNT - 1,
        educational_focus: None,
        constraints: None,
        metadata: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    match result {
        Err(AppError::ValidationError(msg)) => {
            assert!(msg.contains("node_count"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation fails for node count above maximum
#[test]
fn test_validation_node_count_too_high() {
    let request = GenerationRequest {
        request_id: "req-123".to_string(),
        tenant_id: "tenant-456".to_string(),
        theme: "Space Adventure".to_string(),
        age_group: "6-8".to_string(),
        language: "en".to_string(),
        vocabulary_level: "simple".to_string(),
        node_count: validation::MAX_NODE_COUNT + 1,
        educational_focus: None,
        constraints: None,
        metadata: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    match result {
        Err(AppError::ValidationError(msg)) => {
            assert!(msg.contains("node_count"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation of constraints
#[test]
fn test_validation_constraints() {
    let mut request = GenerationRequest {
        request_id: "req-123".to_string(),
        tenant_id: "tenant-456".to_string(),
        theme: "Space Adventure".to_string(),
        age_group: "6-8".to_string(),
        language: "en".to_string(),
        vocabulary_level: "simple".to_string(),
        node_count: 10,
        educational_focus: None,
        constraints: None,
        metadata: None,
    };

    // Valid constraints
    request.constraints = Some(RequestConstraints {
        max_choices_per_node: Some(4),
        min_story_length: Some(500),
        forbidden_topics: None,
        required_topics: None,
    });
    assert!(request.validate().is_ok());

    // Invalid max_choices_per_node (too low)
    request.constraints = Some(RequestConstraints {
        max_choices_per_node: Some(validation::MIN_CHOICES_PER_NODE - 1),
        min_story_length: None,
        forbidden_topics: None,
        required_topics: None,
    });
    assert!(request.validate().is_err());

    // Invalid max_choices_per_node (too high)
    request.constraints = Some(RequestConstraints {
        max_choices_per_node: Some(validation::MAX_CHOICES_PER_NODE + 1),
        min_story_length: None,
        forbidden_topics: None,
        required_topics: None,
    });
    assert!(request.validate().is_err());
}

/// Test JSON serialization round-trip
#[test]
fn test_json_serialization_roundtrip() {
    let original = GenerationRequest {
        request_id: "req-123".to_string(),
        tenant_id: "tenant-456".to_string(),
        theme: "Space Adventure".to_string(),
        age_group: "6-8".to_string(),
        language: "en".to_string(),
        vocabulary_level: "simple".to_string(),
        node_count: 10,
        educational_focus: Some(vec!["science".to_string()]),
        constraints: Some(RequestConstraints {
            max_choices_per_node: Some(4),
            min_story_length: Some(500),
            forbidden_topics: Some(vec!["violence".to_string()]),
            required_topics: Some(vec!["friendship".to_string()]),
        }),
        metadata: Some(RequestMetadata::new()),
    };

    // Serialize to JSON
    let json = original.to_json().unwrap();

    // Deserialize back
    let deserialized: GenerationRequest = serde_json::from_str(&json).unwrap();

    // Verify fields match
    assert_eq!(original.request_id, deserialized.request_id);
    assert_eq!(original.tenant_id, deserialized.tenant_id);
    assert_eq!(original.theme, deserialized.theme);
    assert_eq!(original.node_count, deserialized.node_count);
}

/// Test builder pattern creates valid request
#[test]
fn test_builder_pattern() {
    let result = GenerationRequest::builder()
        .request_id("req-789".to_string())
        .tenant_id("tenant-abc".to_string())
        .theme("Medieval Quest".to_string())
        .age_group("9-12".to_string())
        .language("es".to_string())
        .vocabulary_level("moderate".to_string())
        .node_count(20)
        .build();

    assert!(result.is_ok());
    let request = result.unwrap();
    assert_eq!(request.request_id, "req-789");
    assert_eq!(request.theme, "Medieval Quest");
}

/// Test builder fails with missing required fields
#[test]
fn test_builder_missing_fields() {
    let result = GenerationRequest::builder()
        .request_id("req-789".to_string())
        // Missing tenant_id and other required fields
        .build();

    assert!(result.is_err());
}

/// Test NATS client initialization (unit test - no actual connection)
#[tokio::test]
async fn test_nats_client_creation() {
    let config = NatsConfig::default();
    let _client = NatsClient::new(config);

    // Client should be created but not connected
    assert!(!_client.is_connected().await);
}

/// Integration test for NATS request submission
/// This test will only pass if NATS server is running on localhost:5222
/// It's designed to be skipped if NATS is not available
#[tokio::test]
#[ignore] // Ignore by default - run with `cargo test -- --ignored` when NATS is running
async fn test_nats_request_submission() {
    let config = NatsConfig::default();
    let client = NatsClient::new(config);

    // Try to connect - this will fail if NATS is not running
    if client.connect().await.is_err() {
        println!("NATS server not running on localhost:5222 - skipping integration test");
        return;
    }

    // Create a valid request
    let request = GenerationRequest {
        request_id: "test-req-123".to_string(),
        tenant_id: "test-tenant-456".to_string(),
        theme: "Test Adventure".to_string(),
        age_group: "6-8".to_string(),
        language: "en".to_string(),
        vocabulary_level: "simple".to_string(),
        node_count: 10,
        educational_focus: None,
        constraints: None,
        metadata: Some(RequestMetadata::new()),
    };

    // Publish the request
    let result = client.publish_request(&request).await;

    // Should succeed if NATS is running
    if let Err(e) = result {
        println!("Failed to publish request: {}", e);
        panic!("NATS publish failed");
    }

    // Clean up
    let _ = client.disconnect().await;
}

/// Test error handling for invalid requests
#[tokio::test]
async fn test_invalid_request_handling() {
    let config = NatsConfig::default();
    let client = NatsClient::new(config);

    // Create an invalid request (invalid age group)
    let request = GenerationRequest {
        request_id: "test-req-123".to_string(),
        tenant_id: "test-tenant-456".to_string(),
        theme: "Test Adventure".to_string(),
        age_group: "invalid".to_string(), // Invalid
        language: "en".to_string(),
        vocabulary_level: "simple".to_string(),
        node_count: 10,
        educational_focus: None,
        constraints: None,
        metadata: None,
    };

    // Validation should fail before even trying to connect to NATS
    let result = request.validate();
    assert!(result.is_err());
}

/// Test metadata creation with helpers
#[test]
fn test_metadata_helpers() {
    let metadata = RequestMetadata::new()
        .with_user("user-123".to_string())
        .with_original("original-req-456".to_string());

    assert_eq!(metadata.submitted_by, Some("user-123".to_string()));
    assert_eq!(metadata.original_request_id, Some("original-req-456".to_string()));
    assert!(!metadata.submitted_at.is_empty());
}

/// Test all valid age groups
#[test]
fn test_all_valid_age_groups() {
    for age_group in validation::VALID_AGE_GROUPS {
        let request = GenerationRequest {
            request_id: "req-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            theme: "Adventure".to_string(),
            age_group: age_group.to_string(),
            language: "en".to_string(),
            vocabulary_level: "simple".to_string(),
            node_count: 10,
            educational_focus: None,
            constraints: None,
            metadata: None,
        };

        assert!(
            request.validate().is_ok(),
            "Failed for age_group: {}",
            age_group
        );
    }
}

/// Test all valid vocabulary levels
#[test]
fn test_all_valid_vocabulary_levels() {
    for vocab_level in validation::VALID_VOCABULARY_LEVELS {
        let request = GenerationRequest {
            request_id: "req-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            theme: "Adventure".to_string(),
            age_group: "6-8".to_string(),
            language: "en".to_string(),
            vocabulary_level: vocab_level.to_string(),
            node_count: 10,
            educational_focus: None,
            constraints: None,
            metadata: None,
        };

        assert!(
            request.validate().is_ok(),
            "Failed for vocabulary_level: {}",
            vocab_level
        );
    }
}
