// ABOUTME: Tests validating that documentation examples work correctly
// ABOUTME: Ensures all rustdoc examples compile and run as expected

//! Documentation Examples Tests
//! 
//! This module contains tests that validate all documentation examples
//! work correctly and demonstrate proper usage patterns.

#[cfg(feature = "openapi")]
use qollective::openapi::OpenApiUtils;
use qollective::envelope::{Envelope, Meta};
use qollective::envelope::meta::{SecurityMeta, AuthMethod, PerformanceMeta, TracingMeta, SpanKind, SpanStatus, SpanStatusCode, TraceValue, ExternalCall, CallStatus, CacheOperations};
use std::collections::HashMap;

#[test]
fn test_basic_envelope_creation_example() {
    // Example from basic usage documentation
    let meta = Meta::for_new_request();
    let envelope = Envelope::new(meta, "Hello, World!".to_string());
    
    assert!(!envelope.payload.is_empty());
    assert!(envelope.error.is_none());
    assert!(envelope.is_success());
}

#[test]
fn test_envelope_builder_example() {
    // Example from EnvelopeBuilder documentation
    use qollective::envelope::builder::EnvelopeBuilder;
    let envelope = EnvelopeBuilder::new()
        .with_payload("test data".to_string())
        .with_tenant("enterprise_starfleet".to_string())
        .with_timestamp()
        .with_version("1.0".to_string())
        .build()
        .expect("Failed to build envelope");
    
    assert_eq!(envelope.payload, "test data");
    assert_eq!(envelope.meta.tenant.as_deref(), Some("enterprise_starfleet"));
    assert_eq!(envelope.meta.version.as_deref(), Some("1.0"));
    assert!(envelope.meta.timestamp.is_some());
}

#[test]
fn test_comprehensive_metadata_example() {
    // Example from comprehensive metadata documentation
    let mut meta = Meta::for_new_request();
    
    // Add security metadata
    meta.security = Some(SecurityMeta {
        user_id: Some("picard_001".to_string()),
        session_id: Some("bridge_session_123".to_string()),
        auth_method: Some(AuthMethod::Jwt),
        permissions: vec!["READ".to_string(), "WRITE".to_string(), "ADMIN".to_string()],
        roles: vec!["CAPTAIN".to_string(), "STARFLEET_OFFICER".to_string()],
        tenant_id: Some("enterprise_starfleet".to_string()),
        ip_address: Some("192.168.1.100".to_string()),
        user_agent: Some("Starfleet Command Interface v2.0".to_string()),
        token_expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(8)),
    });
    
    // Add performance metadata
    meta.performance = Some(PerformanceMeta {
        db_query_time: Some(25.5),
        db_query_count: Some(3),
        cache_hit_ratio: Some(0.85),
        cache_operations: Some(CacheOperations {
            hits: Some(85),
            misses: Some(15),
            sets: Some(10),
        }),
        memory_allocated: Some(512),
        memory_peak: Some(768),
        cpu_usage: Some(0.15),
        network_latency: Some(12.3),
        external_calls: vec![
            ExternalCall {
                service: "warp_core_status".to_string(),
                duration: 15.2,
                status: CallStatus::Success,
                endpoint: Some("/api/v1/warp/status".to_string()),
            }
        ],
        gc_collections: Some(2),
        gc_time: Some(5.4),
        thread_count: Some(8),
        processing_time_ms: Some(150),
    });
    
    // Add tracing metadata
    let mut tags = HashMap::new();
    tags.insert("environment".to_string(), TraceValue::String("production".to_string()));
    tags.insert("service".to_string(), TraceValue::String("bridge_command".to_string()));
    tags.insert("version".to_string(), TraceValue::String("2.0.1".to_string()));
    
    meta.tracing = Some(TracingMeta {
        trace_id: Some("trace_enterprise_001".to_string()),
        span_id: Some("span_bridge_command".to_string()),
        parent_span_id: Some("span_starfleet_main".to_string()),
        operation_name: Some("execute_bridge_command".to_string()),
        baggage: HashMap::new(),
        sampling_rate: Some(1.0),
        sampled: Some(true),
        trace_state: Some("starfleet=active,priority=high".to_string()),
        span_kind: Some(SpanKind::Internal),
        span_status: Some(SpanStatus {
            code: SpanStatusCode::Ok,
            message: None,
        }),
        tags,
    });
    
    let envelope = Envelope::new(meta, "Bridge command executed successfully".to_string());
    
    // Validate security metadata
    let security = envelope.meta.security.as_ref().unwrap();
    assert_eq!(security.user_id.as_deref(), Some("picard_001"));
    assert!(security.permissions.contains(&"ADMIN".to_string()));
    assert!(security.roles.contains(&"CAPTAIN".to_string()));
    
    // Validate performance metadata
    let performance = envelope.meta.performance.as_ref().unwrap();
    assert_eq!(performance.db_query_count, Some(3));
    assert_eq!(performance.external_calls.len(), 1);
    assert_eq!(performance.external_calls[0].service, "warp_core_status");
    
    // Validate tracing metadata
    let tracing = envelope.meta.tracing.as_ref().unwrap();
    assert_eq!(tracing.trace_id.as_deref(), Some("trace_enterprise_001"));
    assert_eq!(tracing.operation_name.as_deref(), Some("execute_bridge_command"));
    assert!(tracing.sampled.unwrap_or(false));
}

#[test]
fn test_error_envelope_example() {
    // Example from error handling documentation
    use qollective::envelope::builder::EnvelopeError;
    
    let error = EnvelopeError {
        code: "VALIDATION_FAILED".to_string(),
        message: "Required tenant_id field is missing".to_string(),
        details: Some(serde_json::json!({
            "field": "tenant_id",
            "expected": "string",
            "got": "null"
        })),
        trace: Some("at validate_envelope (envelope.rs:123)".to_string()),
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        http_status_code: Some(400),
    };
    
    let envelope = Envelope::builder()
        .with_payload("invalid request".to_string())
        .with_error(error)
        .build_error()
        .expect("Failed to build error envelope");
    
    assert!(envelope.has_error());
    assert!(!envelope.is_success());
    
    let envelope_error = envelope.error.as_ref().unwrap();
    assert_eq!(envelope_error.code, "VALIDATION_FAILED");
    assert!(envelope_error.details.is_some());
}

#[test]
fn test_serialization_example() {
    // Example from serialization documentation
    use qollective::envelope::builder::EnvelopeBuilder;
    let envelope = EnvelopeBuilder::new()
        .with_payload(serde_json::json!({
            "message": "Welcome to the Enterprise Bridge",
            "status": "ready",
            "crew_count": 1012
        }))
        .with_tenant("enterprise_starfleet".to_string())
        .with_timestamp()
        .build()
        .expect("Failed to build envelope");
    
    // Test JSON serialization
    let json = serde_json::to_string(&envelope)
        .expect("Failed to serialize envelope");
    assert!(!json.is_empty());
    assert!(json.contains("enterprise_starfleet"));
    assert!(json.contains("Welcome to the Enterprise Bridge"));
    
    // Test JSON deserialization
    let deserialized: Envelope<serde_json::Value> = serde_json::from_str(&json)
        .expect("Failed to deserialize envelope");
    assert_eq!(deserialized.meta.tenant, envelope.meta.tenant);
    assert_eq!(deserialized.payload["message"], "Welcome to the Enterprise Bridge");
}

#[cfg(feature = "openapi")]
#[test]
fn test_openapi_schema_generation_example() {
    // Example from OpenAPI documentation
    let spec = OpenApiUtils::generate_spec();
    
    // Verify core schema components are present
    let spec_json = serde_json::to_value(&spec)
        .expect("Failed to serialize OpenAPI spec");
    
    // Check for envelope-related schemas (they may have generic naming patterns)
    assert!(spec_json["components"]["schemas"].as_object().unwrap().keys().any(|k| k.contains("Envelope")));
    assert!(spec_json["components"]["schemas"]["Meta"].is_object());
    assert!(spec_json["components"]["schemas"]["SecurityMeta"].is_object());
    assert!(spec_json["components"]["schemas"]["PerformanceMeta"].is_object());
    assert!(spec_json["components"]["schemas"]["TracingMeta"].is_object());
    
    // Verify spec string generation
    let spec_string = OpenApiUtils::generate_spec_string();
    assert!(!spec_string.is_empty());
    assert!(spec_string.contains("openapi"));
    assert!(spec_string.contains("Envelope"));
}

#[cfg(feature = "openapi")]
#[test]
fn test_openapi_example_envelope_generation() {
    // Example from OpenAPI example generation documentation
    let example_envelope = OpenApiUtils::generate_example_envelope();
    
    assert_eq!(example_envelope.payload.message, "Bridge to Engineering: Warp core online");
    assert_eq!(example_envelope.meta.tenant.as_deref(), Some("enterprise_starfleet"));
    assert_eq!(example_envelope.payload.status, "operational");
    assert_eq!(example_envelope.payload.priority, Some(1));
    assert!(example_envelope.payload.created_at.is_some());
    
    // Verify the example can be serialized
    let json = serde_json::to_string_pretty(&example_envelope)
        .expect("Failed to serialize example envelope");
    assert!(!json.is_empty());
}

#[cfg(feature = "openapi")]
#[test]
fn test_openapi_error_envelope_example() {
    // Example from OpenAPI error handling documentation
    let error_envelope = OpenApiUtils::generate_example_error_envelope();
    
    assert!(error_envelope.has_error());
    assert!(error_envelope.payload.status.contains("error"));
    
    let error = error_envelope.error.as_ref().unwrap();
    assert_eq!(error.code, "WARP_CORE_FAILURE");
    assert!(error.message.contains("dilithium crystals"));
    
    // Verify error envelope serialization
    let json = serde_json::to_string_pretty(&error_envelope)
        .expect("Failed to serialize error envelope");
    assert!(json.contains("WARP_CORE_FAILURE"));
}

#[test]
fn test_tenant_extraction_example() {
    // Example from tenant extraction documentation
    let mut meta = Meta::for_new_request();
    meta.tenant = Some("starfleet_command".to_string());
    
    // Simulate tenant context propagation
    meta.security = Some(SecurityMeta {
        user_id: Some("kirk_001".to_string()),
        session_id: Some("command_session_456".to_string()),
        auth_method: Some(AuthMethod::Jwt),
        permissions: vec!["READ".to_string(), "COMMAND".to_string()],
        roles: vec!["CAPTAIN".to_string()],
        tenant_id: Some("starfleet_command".to_string()),
        ip_address: Some("10.0.0.1".to_string()),
        user_agent: Some("Starfleet Command Console v1.5".to_string()),
        token_expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(4)),
    });
    
    let envelope = Envelope::new(meta, "Command authorized".to_string());
    
    // Verify tenant context is preserved
    assert_eq!(envelope.meta.tenant.as_deref(), Some("starfleet_command"));
    
    let security = envelope.meta.security.as_ref().unwrap();
    assert_eq!(security.tenant_id.as_deref(), Some("starfleet_command"));
    assert_eq!(security.user_id.as_deref(), Some("kirk_001"));
}

#[test]
fn test_envelope_chaining_example() {
    // Example from envelope chaining documentation
    use qollective::envelope::builder::EnvelopeBuilder;
    let parent_envelope = EnvelopeBuilder::new()
        .with_payload("Initialize warp drive".to_string())
        .with_tenant("enterprise_starfleet".to_string())
        .with_timestamp()
        .build()
        .expect("Failed to build parent envelope");
    
    // Create child envelope that inherits context
    let mut child_meta = Meta::for_new_request();
    child_meta.tenant = parent_envelope.meta.tenant.clone();
    child_meta.request_id = parent_envelope.meta.request_id;
    
    let child_envelope = Envelope::new(child_meta, "Warp drive initialized".to_string());
    
    // Verify context inheritance
    assert_eq!(child_envelope.meta.tenant, parent_envelope.meta.tenant);
    assert_eq!(child_envelope.meta.request_id, parent_envelope.meta.request_id);
}

#[test]
fn test_performance_monitoring_example() {
    // Example from performance monitoring documentation
    let start_time = std::time::Instant::now();
    
    // Simulate some work
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let duration = start_time.elapsed();
    
    let mut meta = Meta::for_new_request();
    meta.performance = Some(PerformanceMeta {
        db_query_time: Some(5.2),
        db_query_count: Some(1),
        cache_hit_ratio: Some(0.95),
        cache_operations: Some(CacheOperations {
            hits: Some(95),
            misses: Some(5),
            sets: Some(2),
        }),
        memory_allocated: Some(256),
        memory_peak: Some(384),
        cpu_usage: Some(0.08),
        network_latency: Some(8.5),
        external_calls: vec![],
        gc_collections: Some(0),
        gc_time: Some(0.0),
        thread_count: Some(4),
        processing_time_ms: Some(duration.as_millis() as u64),
    });
    
    let envelope = Envelope::new(meta, "Operation completed".to_string());
    
    let performance = envelope.meta.performance.as_ref().unwrap();
    assert!(performance.processing_time_ms.unwrap() >= 10);
    assert_eq!(performance.cache_hit_ratio, Some(0.95));
}