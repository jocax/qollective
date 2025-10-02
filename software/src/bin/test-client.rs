// ABOUTME: Test client for cross-language integration testing
// ABOUTME: Sends requests with Qollective envelopes to test servers and validates responses

use qollective::envelope::{meta::Meta, Envelope, ExtensionsMeta};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("TEST_PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()?;

    let test_data_json = env::var("TEST_DATA").unwrap_or_else(|_| {
        serde_json::json!({
            "request_id": "test-rust-client-123",
            "test_data": {
                "message": "cross-language integration test from Rust client",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "user_id": 12345
            }
        })
        .to_string()
    });

    let test_data: Value = serde_json::from_str(&test_data_json)?;

    // Create envelope with test data
    let meta = Meta {
        timestamp: Some(chrono::Utc::now()),
        request_id: Some(Uuid::now_v7()),
        version: Some("1.0.0".to_string()),
        tenant: Some("test-tenant".to_string()),
        extensions: Some(ExtensionsMeta {
            sections: {
                let mut sections = HashMap::new();
                sections.insert(
                    "client_info".to_string(),
                    serde_json::json!({
                        "language": "rust",
                        "client_id": format!("rust-test-client-{}", Uuid::now_v7()),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }),
                );
                sections
            },
        }),
        ..Default::default()
    };

    let envelope = Envelope::new(meta, test_data);

    // Create HTTP client
    let client = reqwest::Client::new();
    let base_url = format!("http://localhost:{}", port);

    // Test health check first
    // println!("Testing health check...");
    let health_response = client.get(&format!("{}/health", base_url)).send().await?;

    if !health_response.status().is_success() {
        eprintln!("Health check failed: {}", health_response.status());
        std::process::exit(1);
    }

    let health_data: Value = health_response.json().await?;
    // println!("Server health: {}", serde_json::to_string_pretty(&health_data)?);

    // Test echo endpoint
    // println!("Testing echo endpoint...");
    let echo_response = client
        .post(&format!("{}/test/echo", base_url))
        .json(&envelope)
        .send()
        .await?;

    if !echo_response.status().is_success() {
        eprintln!("Echo test failed: {}", echo_response.status());
        let error_text = echo_response.text().await?;
        eprintln!("Error response: {}", error_text);
        std::process::exit(1);
    }

    let echo_data: Value = echo_response.json().await?;

    // Test process endpoint
    // println!("Testing process endpoint...");
    let process_response = client
        .post(&format!("{}/test/process", base_url))
        .json(&envelope)
        .send()
        .await?;

    if !process_response.status().is_success() {
        eprintln!("Process test failed: {}", process_response.status());
        let error_text = process_response.text().await?;
        eprintln!("Error response: {}", error_text);
        std::process::exit(1);
    }

    let process_data: Value = process_response.json().await?;

    // Validate responses have envelope structure
    let echo_envelope = validate_envelope_response(&echo_data, "echo")?;
    let process_envelope = validate_envelope_response(&process_data, "process")?;

    // Create final response for the integration test
    let final_response = serde_json::json!({
        "meta": {
            "client_language": "rust",
            "test_successful": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "tests_performed": ["health", "echo", "process"],
            "context_propagated": true
        },
        "results": {
            "health": health_data,
            "echo": echo_envelope,
            "process": process_envelope
        }
    });

    // Output the final response as JSON for the integration test framework
    println!("{}", serde_json::to_string_pretty(&final_response)?);

    Ok(())
}

fn validate_envelope_response(
    response: &Value,
    operation: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    // Check if response has envelope structure
    let meta = response
        .get("meta")
        .ok_or_else(|| format!("Response missing 'meta' field for {} operation", operation))?;

    let _data = response
        .get("data")
        .ok_or_else(|| format!("Response missing 'data' field for {} operation", operation))?;

    // Validate meta contains required fields
    if meta.get("timestamp").is_none() {
        return Err(format!(
            "Response meta missing 'timestamp' for {} operation",
            operation
        )
        .into());
    }

    // Check if our context was preserved/propagated
    let extensions = meta.get("extensions");
    let _processed_by = extensions
        .and_then(|ext| ext.get("processed_by"))
        .ok_or_else(|| {
            format!(
                "Response missing processing metadata for {} operation",
                operation
            )
        })?;

    // println!("✅ {} operation successful, processed by: {}", operation, _processed_by);

    Ok(response.clone())
}
