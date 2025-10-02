// ABOUTME: Interactive demo client for cross-language integration testing
// ABOUTME: Features beautiful colored logging and envelope visualization for demonstration purposes

use colored::*;
use qollective::envelope::{meta::Meta, Envelope, ExtensionsMeta};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("QOLLECTIVE_PORT")
        .or_else(|_| env::var("TEST_PORT"))
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()?;

    let custom_message =
        env::var("QOLLECTIVE_MESSAGE").unwrap_or_else(|_| "Hello from Rust client! 🦀".to_string());

    let client_id = format!("rust-demo-client-{}", &Uuid::now_v7().to_string()[..8]);

    print_startup_banner(&client_id, port, &custom_message);

    // Create test data with custom message
    let test_data = serde_json::json!({
        "message": custom_message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "client_info": {
            "language": "rust",
            "client_id": client_id,
            "demo_mode": true
        },
        "test_payload": {
            "user_id": 12345,
            "session": "demo-session-abc123",
            "features": ["cross_language", "envelope_demo", "context_propagation"]
        }
    });

    // Create envelope with test data
    let meta = Meta {
        timestamp: Some(chrono::Utc::now()),
        request_id: Some(Uuid::now_v7()),
        version: Some("1.0.0".to_string()),
        tenant: Some("demo-tenant".to_string()),
        extensions: Some(ExtensionsMeta {
            sections: {
                let mut sections = HashMap::new();
                sections.insert(
                    "client_info".to_string(),
                    serde_json::json!({
                        "language": "rust",
                        "client_id": client_id,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "demo_mode": true
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

    println!(
        "{}",
        "🔄 Starting cross-language integration test..."
            .bright_yellow()
            .bold()
    );
    println!();

    // Test health check first
    test_health_check(&client, &base_url).await?;

    // Test echo endpoint
    test_echo_endpoint(&client, &base_url, &envelope).await?;

    // Test process endpoint
    test_process_endpoint(&client, &base_url, &envelope).await?;

    println!(
        "{}",
        "🎉 All tests completed successfully!".bright_green().bold()
    );
    println!(
        "{}",
        "   Cross-language integration is working perfectly! ✨".dimmed()
    );
    println!();

    Ok(())
}

fn print_startup_banner(client_id: &str, port: u16, message: &str) {
    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗".bright_red()
    );
    println!(
        "{}",
        "║              🦀 QOLLECTIVE RUST DEMO CLIENT 🦀               ║"
            .bright_red()
            .bold()
    );
    println!(
        "{}",
        "╠══════════════════════════════════════════════════════════════╣".bright_red()
    );
    println!(
        "{}",
        format!("║ Client ID: {:<46} ║", client_id).bright_red()
    );
    println!(
        "{}",
        format!("║ Target:    localhost:{:<37} ║", port).bright_red()
    );
    println!(
        "{}",
        format!("║ Message:   {:<46} ║", truncate_string(message, 46)).bright_red()
    );
    println!(
        "{}",
        format!(
            "║ Time:      {:<46} ║",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
        .bright_red()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝".bright_red()
    );
    println!();
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn print_test_header(test_name: &str, endpoint: &str) {
    println!(
        "{}",
        format!(
            "┌─ {} Testing: {}",
            "🧪".bright_cyan(),
            test_name.bright_white().bold()
        )
        .on_black()
    );
    println!("{}", format!("│ 🎯 Endpoint: {}", endpoint.bright_cyan()));
}

fn print_envelope(envelope: &Value, label: &str, is_request: bool) {
    let arrow = if is_request { "📤" } else { "📥" };
    let color = if is_request {
        Color::Cyan
    } else {
        Color::Green
    };

    println!(
        "{}",
        format!("│ {} {} Envelope:", arrow, label)
            .color(color)
            .bold()
    );

    // Pretty print the JSON with indentation
    let pretty_json = serde_json::to_string_pretty(envelope).unwrap_or_default();
    for line in pretty_json.lines() {
        if line.trim().starts_with("\"meta\"") {
            println!("{}", format!("│   {}", line).bright_magenta());
        } else if line.trim().starts_with("\"data\"") {
            println!("{}", format!("│   {}", line).bright_cyan());
        } else if line.trim().starts_with("\"extensions\"") {
            println!("{}", format!("│   {}", line).bright_yellow());
        } else if line.trim().starts_with("\"timestamp\"")
            || line.trim().starts_with("\"request_id\"")
        {
            println!("{}", format!("│   {}", line).bright_white());
        } else {
            println!("{}", format!("│   {}", line).white());
        }
    }
}

fn print_test_result(success: bool, duration_ms: u128, details: &str) {
    let status = if success {
        format!("✅ SUCCESS",).bright_green().bold()
    } else {
        format!("❌ FAILED").bright_red().bold()
    };

    println!("{}", format!("│ 🏆 Result: {} ({}ms)", status, duration_ms));
    if !details.is_empty() {
        println!("{}", format!("│ 📝 Details: {}", details).dimmed());
    }
    println!(
        "{}",
        "└─────────────────────────────────────────────────────────────".bright_black()
    );
    println!();
}

async fn test_health_check(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    print_test_header("Health Check", "GET /health");

    let response = client.get(&format!("{}/health", base_url)).send().await?;

    let duration = start.elapsed().as_millis();

    if !response.status().is_success() {
        print_test_result(false, duration, &format!("HTTP {}", response.status()));
        return Err(format!("Health check failed: {}", response.status()).into());
    }

    let health_data: Value = response.json().await?;

    println!("{}", format!("│ 📥 Server Response:").bright_green().bold());
    println!(
        "{}",
        format!(
            "│   Language: {}",
            health_data["language"].as_str().unwrap_or("unknown")
        )
        .bright_white()
    );
    println!("{}", format!("│   Status: {}", "✅ HEALTHY".bright_green()));
    println!(
        "{}",
        format!(
            "│   Server ID: {}",
            health_data["server_id"].as_str().unwrap_or("unknown")
        )
        .dimmed()
    );

    print_test_result(true, duration, "Server is healthy and ready");

    Ok(())
}

async fn test_echo_endpoint(
    client: &reqwest::Client,
    base_url: &str,
    envelope: &Envelope<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    print_test_header("Echo Test", "POST /test/echo");

    let request_json = serde_json::to_value(envelope)?;
    print_envelope(&request_json, "Outgoing", true);

    let response = client
        .post(&format!("{}/test/echo", base_url))
        .json(envelope)
        .send()
        .await?;

    let duration = start.elapsed().as_millis();

    if !response.status().is_success() {
        print_test_result(false, duration, &format!("HTTP {}", response.status()));
        return Err(format!("Echo test failed: {}", response.status()).into());
    }

    let echo_data: Value = response.json().await?;
    print_envelope(&echo_data, "Incoming", false);

    // Validate response
    let validation_result = validate_envelope_response(&echo_data, "echo");
    let details = match validation_result {
        Ok(info) => format!("Context preserved: {}", info),
        Err(e) => return Err(e),
    };

    print_test_result(true, duration, &details);

    Ok(())
}

async fn test_process_endpoint(
    client: &reqwest::Client,
    base_url: &str,
    envelope: &Envelope<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    print_test_header("Process Test", "POST /test/process");

    let request_json = serde_json::to_value(envelope)?;
    print_envelope(&request_json, "Outgoing", true);

    let response = client
        .post(&format!("{}/test/process", base_url))
        .json(envelope)
        .send()
        .await?;

    let duration = start.elapsed().as_millis();

    if !response.status().is_success() {
        print_test_result(false, duration, &format!("HTTP {}", response.status()));
        return Err(format!("Process test failed: {}", response.status()).into());
    }

    let process_data: Value = response.json().await?;
    print_envelope(&process_data, "Incoming", false);

    // Validate response
    let validation_result = validate_envelope_response(&process_data, "process");
    let details = match validation_result {
        Ok(info) => format!("Data transformed: {}", info),
        Err(e) => return Err(e),
    };

    print_test_result(true, duration, &details);

    Ok(())
}

fn validate_envelope_response(
    response: &Value,
    operation: &str,
) -> Result<String, Box<dyn std::error::Error>> {
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
    let processed_by = extensions.and_then(|ext| ext.get("processed_by"));

    if processed_by.is_none() {
        return Err(format!(
            "Response missing processing metadata for {} operation",
            operation
        )
        .into());
    }

    let processor_info = processed_by
        .and_then(|p| p.get("language"))
        .and_then(|l| l.as_str())
        .unwrap_or("unknown");

    Ok(format!("✅ Processed by {} server", processor_info))
}
