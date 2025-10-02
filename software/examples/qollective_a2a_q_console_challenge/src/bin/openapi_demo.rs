// ABOUTME: OpenAPI demonstration using the Qollective envelope-first architecture
// ABOUTME: Shows how to integrate utoipa OpenAPI 3.1 schema generation with Star Trek themed examples

//! OpenAPI Integration Demo for Qollective Framework
//!
//! This example demonstrates how to use the new utoipa OpenAPI 3.1 integration
//! features in the Qollective framework. It shows:
//! 
//! - Envelope structure schema generation
//! - Comprehensive metadata schema documentation  
//! - Enterprise-themed example data generation
//! - OpenAPI specification output and Swagger UI integration
//!
//! The demo uses Star Trek Enterprise themes to showcase realistic multi-tenant
//! scenarios with security, performance, and tracing metadata.

#[cfg(feature = "openapi")]
use qollective::openapi::{OpenApiUtils, QollectiveApiDoc};

#[cfg(feature = "openapi")]
use qollective::envelope::builder::{EnvelopeBuilder, EnvelopeError};

#[cfg(feature = "openapi")]
use qollective::envelope::meta::{
    Meta, SecurityMeta, AuthMethod, PerformanceMeta, TracingMeta, 
    SpanKind, SpanStatus, SpanStatusCode, TraceValue, ExternalCall, 
    CallStatus, CacheOperations
};

#[cfg(feature = "openapi")]
use std::collections::HashMap;

#[cfg(feature = "openapi")]
use serde_json;

/// Enterprise command data for OpenAPI demonstration
#[cfg(feature = "openapi")]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Enterprise Command",
    description = "Starfleet command structure for bridge operations",
    example = json!({
        "command": "engage_warp_drive",
        "priority": 1,
        "officer": "picard",
        "destination": "Wolf 359"
    })
))]
struct EnterpriseCommand {
    /// Command to execute (e.g., engage_warp_drive, raise_shields)
    #[cfg_attr(feature = "openapi", schema(example = "engage_warp_drive"))]
    command: String,
    
    /// Command priority level (1 = highest, 5 = lowest)
    #[cfg_attr(feature = "openapi", schema(minimum = 1, maximum = 5, example = 1))]
    priority: u8,
    
    /// Commanding officer identifier
    #[cfg_attr(feature = "openapi", schema(example = "picard"))]
    officer: String,
    
    /// Optional destination for navigation commands
    #[cfg_attr(feature = "openapi", schema(example = "Wolf 359"))]
    destination: Option<String>,
}

#[cfg(feature = "openapi")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Qollective OpenAPI 3.1 Integration Demo");
    println!("===========================================\n");
    
    // 1. Generate and display the OpenAPI specification
    println!("📋 Generating OpenAPI 3.1 Specification...");
    let spec = OpenApiUtils::generate_spec();
    let spec_json = serde_json::to_string_pretty(&spec)?;
    
    println!("✅ Generated OpenAPI spec ({} bytes)", spec_json.len());
    println!("   Specification includes schemas for:");
    println!("   - Envelope<T> with generic type support");
    println!("   - Meta with comprehensive metadata fields");
    println!("   - SecurityMeta for authentication and authorization");
    println!("   - PerformanceMeta for monitoring and optimization");
    println!("   - TracingMeta for OpenTelemetry integration");
    println!("   - EnvelopeError for detailed error handling\n");
    
    // 2. Create a comprehensive envelope example
    println!("🎭 Creating Enterprise Command Envelope...");
    
    let command = EnterpriseCommand {
        command: "engage_warp_drive".to_string(),
        priority: 1,
        officer: "picard".to_string(),
        destination: Some("Wolf 359".to_string()),
    };
    
    // Build comprehensive metadata
    let mut meta = Meta::for_new_request();
    meta.tenant = Some("starfleet_enterprise".to_string());
    meta.version = Some("2.0.1".to_string());
    
    // Add security context
    meta.security = Some(SecurityMeta {
        user_id: Some("picard_001".to_string()),
        session_id: Some("bridge_session_789".to_string()),
        auth_method: Some(AuthMethod::Jwt),
        permissions: vec!["COMMAND".to_string(), "NAVIGATE".to_string(), "TACTICAL".to_string()],
        roles: vec!["CAPTAIN".to_string(), "COMMANDING_OFFICER".to_string()],
        tenant_id: Some("starfleet_enterprise".to_string()),
        ip_address: Some("10.0.1.100".to_string()),
        user_agent: Some("Enterprise Bridge Interface v2.1".to_string()),
        token_expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(8)),
    });
    
    // Add performance metadata
    meta.performance = Some(PerformanceMeta {
        db_query_time: Some(12.5),
        db_query_count: Some(2),
        cache_hit_ratio: Some(0.92),
        cache_operations: Some(CacheOperations {
            hits: Some(92),
            misses: Some(8),
            sets: Some(5),
        }),
        memory_allocated: Some(384),
        memory_peak: Some(512),
        cpu_usage: Some(0.08),
        network_latency: Some(5.2),
        external_calls: vec![
            ExternalCall {
                service: "warp_core_diagnostics".to_string(),
                duration: 8.7,
                status: CallStatus::Success,
                endpoint: Some("/api/v2/warp/status".to_string()),
            }
        ],
        gc_collections: Some(1),
        gc_time: Some(2.1),
        thread_count: Some(6),
        processing_time_ms: Some(45),
    });
    
    // Add tracing metadata
    let mut tags = HashMap::new();
    tags.insert("environment".to_string(), TraceValue::String("production".to_string()));
    tags.insert("ship_class".to_string(), TraceValue::String("Galaxy".to_string()));
    tags.insert("command_level".to_string(), TraceValue::Number(1.0));
    
    meta.tracing = Some(TracingMeta {
        trace_id: Some("trace_enterprise_warp_001".to_string()),
        span_id: Some("span_bridge_command".to_string()),
        parent_span_id: Some("span_bridge_main".to_string()),
        operation_name: Some("execute_warp_command".to_string()),
        baggage: HashMap::new(),
        sampling_rate: Some(1.0),
        sampled: Some(true),
        trace_state: Some("enterprise=active,priority=command".to_string()),
        span_kind: Some(SpanKind::Server),
        span_status: Some(SpanStatus {
            code: SpanStatusCode::Ok,
            message: None,
        }),
        tags,
    });
    
    // Create the envelope using the builder pattern
    let envelope = EnvelopeBuilder::new()
        .with_payload(command)
        .with_meta(meta)
        .build()?;
    
    println!("✅ Created envelope with comprehensive metadata");
    println!("   🔐 Security: JWT auth as Captain Picard");
    println!("   📊 Performance: 92% cache hit rate, 45ms processing");
    println!("   🔍 Tracing: Full OpenTelemetry span context");
    println!("   🏢 Tenant: starfleet_enterprise\n");
    
    // 3. Demonstrate JSON serialization with OpenAPI compatibility
    println!("📤 Serializing envelope to JSON...");
    let envelope_json = serde_json::to_string_pretty(&envelope)?;
    println!("✅ Serialized envelope ({} bytes)", envelope_json.len());
    println!("   JSON structure matches OpenAPI 3.1 schemas\n");
    
    // 4. Show example error envelope
    println!("⚠️  Creating Example Error Envelope...");
    let error = EnvelopeError {
        code: "WARP_DRIVE_OFFLINE".to_string(),
        message: "Warp drive is currently offline due to antimatter containment failure".to_string(),
        details: Some(serde_json::json!({
            "system": "warp_core",
            "subsystem": "antimatter_containment",
            "error_level": "critical",
            "estimated_repair_time": "2.5 hours"
        })),
        trace: Some("at warp_core_manager.rs:245 -> antimatter_containment.rs:89".to_string()),
        #[cfg(any(
            feature = "rest-server", 
            feature = "rest-client",
            feature = "websocket-server", 
            feature = "websocket-client",
            feature = "a2a"
        ))]
        http_status_code: Some(503),
    };
    
    let error_data = serde_json::json!({
        "command_status": "failed",
        "attempted_command": "engage_warp_drive",
        "failure_timestamp": chrono::Utc::now()
    });
    
    let error_envelope = EnvelopeBuilder::new()
        .with_payload(error_data)
        .with_meta(Meta::for_new_request())
        .with_error(error)
        .build_error()?;
    
    println!("✅ Created error envelope with detailed failure information");
    println!("   💥 Error: WARP_DRIVE_OFFLINE");
    println!("   🔧 Details: Antimatter containment system failure");
    println!("   📍 Trace: Full stack trace for debugging\n");
    
    // 5. Generate example envelope using OpenApiUtils
    println!("🎲 Generating Standard Example Envelope...");
    let example_envelope = OpenApiUtils::generate_example_envelope();
    println!("✅ Generated example: {}", example_envelope.payload.message);
    println!("   🏢 Tenant: {}", example_envelope.meta.tenant.as_deref().unwrap_or("none"));
    println!("   📊 Status: {}", example_envelope.payload.status);
    
    let example_error = OpenApiUtils::generate_example_error_envelope();
    println!("   ⚠️  Error example: {}", example_error.error.as_ref().unwrap().code);
    
    // 6. Display integration information
    println!("\n🔧 Integration Information:");
    println!("   📚 OpenAPI Features Available:");
    println!("      - utoipa 5.4 with OpenAPI 3.1 support");
    println!("      - Swagger UI 9.0 for interactive documentation");
    println!("      - Comprehensive schema generation for all envelope structures");
    println!("      - Enterprise-themed examples for realistic scenarios");
    println!("   🌐 Web Integration:");
    println!("      - Axum server integration with Swagger UI endpoints");
    println!("      - Actix Web server integration examples");
    println!("      - JSON schema validation for API endpoints");
    println!("   🚀 Production Ready:");
    println!("      - Comprehensive metadata schema documentation");
    println!("      - Multi-tenant aware example generation");
    println!("      - Error handling with detailed schema definitions");
    
    println!("\n✨ OpenAPI 3.1 integration demo completed successfully!");
    
    Ok(())
}

#[cfg(not(feature = "openapi"))]
fn main() {
    println!("⚠️  OpenAPI demo requires the 'openapi' feature to be enabled.");
    println!("   Run with: cargo run --bin openapi_demo --features openapi");
    println!("   Or add to default features in Cargo.toml");
}