use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qollective::envelope::{Envelope, Meta};
use std::time::Duration;

#[cfg(feature = "openapi")]
use qollective::openapi::OpenApiUtils;

fn bench_envelope_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("envelope_creation");

    // Test minimal envelope creation
    group.bench_function("minimal_envelope", |b| {
        b.iter(|| {
            let meta = Meta::for_new_request();
            let envelope = Envelope::new(black_box(meta), black_box("test data".to_string()));
            black_box(envelope)
        })
    });

    // Test envelope builder pattern
    group.bench_function("envelope_builder", |b| {
        b.iter(|| {
            let envelope = Envelope::builder()
                .with_payload(black_box("test data".to_string()))
                .with_tenant(black_box("test_tenant".to_string()))
                .with_timestamp()
                .with_version(black_box("1.0".to_string()))
                .build()
                .expect("Failed to build envelope");
            black_box(envelope)
        })
    });

    group.finish();
}

fn bench_envelope_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("envelope_serialization");

    // Create test envelope with comprehensive metadata
    let mut meta = Meta::for_new_request();
    meta.tenant = Some("benchmark_tenant".to_string());
    meta.version = Some("1.0".to_string());

    // Add security metadata
    meta.security = Some(qollective::envelope::meta::SecurityMeta {
        user_id: Some("benchmark_user".to_string()),
        session_id: Some("bench_session_001".to_string()),
        auth_method: Some(qollective::envelope::meta::AuthMethod::Jwt),
        permissions: vec!["READ".to_string(), "WRITE".to_string()],
        roles: vec!["USER".to_string(), "ADMIN".to_string()],
        ip_address: Some("192.168.1.100".to_string()),
        user_agent: Some("Benchmark Client v1.0".to_string()),
        token_expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
    });

    // Add performance metadata
    meta.performance = Some(qollective::envelope::meta::PerformanceMeta {
        db_query_time: Some(50.0),
        db_query_count: Some(3),
        cache_hit_ratio: Some(0.8),
        cache_operations: None,
        memory_allocated: Some(128),
        memory_peak: Some(256),
        cpu_usage: Some(0.5),
        network_latency: Some(25.0),
        external_calls: vec![qollective::envelope::meta::ExternalCall {
            service: "benchmark_service".to_string(),
            duration: 25.0,
            status: qollective::envelope::meta::CallStatus::Success,
            endpoint: Some("/test".to_string()),
        }],
        gc_collections: Some(2),
        gc_time: Some(10.0),
        thread_count: Some(4),
        processing_time_ms: Some(100),
    });

    let test_data = "This is test data for serialization benchmarking".to_string();
    let envelope = Envelope::new(meta, test_data);

    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&envelope))
                .expect("Failed to serialize envelope");
            black_box(json)
        })
    });

    group.bench_function("json_serialize_pretty", |b| {
        b.iter(|| {
            let json = serde_json::to_string_pretty(black_box(&envelope))
                .expect("Failed to serialize envelope");
            black_box(json)
        })
    });

    // Test deserialization
    let json_data = serde_json::to_string(&envelope).expect("Failed to serialize");

    // Debug: let's see what the JSON looks like
    // This will be compiled out in release builds
    #[cfg(debug_assertions)]
    println!("JSON data: {}", json_data);

    group.bench_function("json_deserialize", |b| {
        b.iter(|| {
            let envelope: Envelope<String> = serde_json::from_str(black_box(&json_data))
                .expect("Failed to deserialize envelope");
            black_box(envelope)
        })
    });

    group.finish();
}

#[cfg(feature = "openapi")]
fn bench_openapi_schema_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("openapi_schema_generation");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    group.bench_function("generate_openapi_spec", |b| {
        b.iter(|| {
            let spec = OpenApiUtils::generate_spec();
            black_box(spec)
        })
    });

    group.bench_function("generate_openapi_spec_string", |b| {
        b.iter(|| {
            let spec_string = OpenApiUtils::generate_spec_string();
            black_box(spec_string)
        })
    });

    group.bench_function("generate_example_envelope", |b| {
        b.iter(|| {
            let envelope = OpenApiUtils::generate_example_envelope();
            black_box(envelope)
        })
    });

    group.bench_function("generate_example_error_envelope", |b| {
        b.iter(|| {
            let envelope = OpenApiUtils::generate_example_error_envelope();
            black_box(envelope)
        })
    });

    group.bench_function("generate_api_examples", |b| {
        b.iter(|| {
            let examples = OpenApiUtils::generate_api_examples();
            black_box(examples)
        })
    });

    group.finish();
}

fn bench_metadata_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_creation");

    group.bench_function("minimal_meta", |b| {
        b.iter(|| {
            let meta = Meta::for_new_request();
            black_box(meta)
        })
    });

    group.bench_function("comprehensive_meta", |b| {
        b.iter(|| {
            let mut meta = Meta::for_new_request();
            meta.tenant = Some(black_box("test_tenant".to_string()));
            meta.version = Some(black_box("1.0".to_string()));

            // Add security metadata
            meta.security = Some(qollective::envelope::meta::SecurityMeta {
                user_id: Some(black_box("user123".to_string())),
                session_id: Some(black_box("session456".to_string())),
                auth_method: Some(qollective::envelope::meta::AuthMethod::Jwt),
                permissions: black_box(vec!["READ".to_string(), "WRITE".to_string()]),
                roles: black_box(vec!["USER".to_string()]),
                tenant_id: Some(black_box("tenant789".to_string())),
                ip_address: Some(black_box("192.168.1.1".to_string())),
                user_agent: Some(black_box("Test Client v1.0".to_string())),
                token_expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            });

            // Add tracing metadata
            meta.tracing = Some(qollective::envelope::meta::TracingMeta {
                trace_id: Some(black_box("trace123".to_string())),
                span_id: Some(black_box("span456".to_string())),
                parent_span_id: Some(black_box("parent789".to_string())),
                operation_name: Some(black_box("test_operation".to_string())),
                baggage: std::collections::HashMap::new(),
                sampling_rate: Some(1.0),
                sampled: Some(true),
                trace_state: None,
                span_kind: Some(qollective::envelope::meta::SpanKind::Internal),
                span_status: Some(qollective::envelope::meta::SpanStatus {
                    code: qollective::envelope::meta::SpanStatusCode::Ok,
                    message: None,
                }),
                tags: {
                    let mut tags = std::collections::HashMap::new();
                    tags.insert("environment".to_string(), qollective::envelope::meta::TraceValue::String("benchmark".to_string()));
                    tags
                },
            });

            black_box(meta)
        })
    });

    group.finish();
}

fn bench_envelope_varying_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("envelope_varying_sizes");

    // Test different payload sizes
    let sizes = [1, 10, 100, 1000, 10000];

    for size in sizes.iter() {
        let payload = "x".repeat(*size);
        let meta = Meta::for_new_request();

        group.bench_with_input(
            BenchmarkId::new("envelope_creation", size),
            size,
            |b, _| {
                b.iter(|| {
                    let envelope = Envelope::new(black_box(meta.clone()), black_box(payload.clone()));
                    black_box(envelope)
                })
            },
        );

        let envelope = Envelope::new(meta, payload);
        group.bench_with_input(
            BenchmarkId::new("envelope_serialization", size),
            size,
            |b, _| {
                b.iter(|| {
                    let json = serde_json::to_string(black_box(&envelope))
                        .expect("Failed to serialize");
                    black_box(json)
                })
            },
        );
    }

    group.finish();
}

#[cfg(feature = "openapi")]
fn bench_schema_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("schema_validation");

    let envelope = OpenApiUtils::generate_example_envelope();

    group.bench_function("validate_envelope_schema", |b| {
        b.iter(|| {
            let result = OpenApiUtils::validate_envelope_schema(black_box(&envelope));
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_envelope_creation,
    bench_envelope_serialization,
    bench_metadata_creation,
    bench_envelope_varying_sizes,
);

#[cfg(feature = "openapi")]
criterion_group!(
    openapi_benches,
    bench_openapi_schema_generation,
    bench_schema_validation,
);

#[cfg(feature = "openapi")]
criterion_main!(benches, openapi_benches);

#[cfg(not(feature = "openapi"))]
criterion_main!(benches);
