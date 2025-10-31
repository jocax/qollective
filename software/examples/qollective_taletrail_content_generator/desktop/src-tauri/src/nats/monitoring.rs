/// NATS monitoring module for real-time message tracking
///
/// Subscribes to wildcard subjects and emits messages to frontend via Tauri events.
/// Auto-connects on app startup and maintains connection state.

use async_nats::{Client, ConnectOptions, Subscriber};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use tauri::Emitter;
use futures::StreamExt;

/// Global monitoring state
static MONITOR_STATE: once_cell::sync::Lazy<Arc<RwLock<Option<MonitoringState>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// Internal monitoring state
struct MonitoringState {
    client: Client,
    subscribers: Vec<Subscriber>,
    is_connected: bool,
}

/// NATS message structure for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsMessage {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// NATS subject
    pub subject: String,
    /// Endpoint extracted from subject
    pub endpoint: String,
    /// Message type (Request/Response/Event)
    pub message_type: String,
    /// JSON payload
    pub payload: String,
    /// Optional request ID from payload
    pub request_id: Option<String>,
}

impl NatsMessage {
    /// Create a new NATS message from subject and payload
    pub fn new(subject: String, payload: Vec<u8>) -> Self {
        let endpoint = extract_endpoint(&subject);
        let message_type = determine_message_type(&subject);
        let payload_str = String::from_utf8_lossy(&payload).to_string();
        let request_id = extract_request_id(&payload_str);

        Self {
            timestamp: Utc::now().to_rfc3339(),
            subject,
            endpoint,
            message_type,
            payload: payload_str,
            request_id,
        }
    }
}

/// Extract endpoint name from subject
/// Examples:
/// - "mcp.orchestrator.request" -> "orchestrator"
/// - "taletrail.generation.events.42" -> "generation"
/// - "mcp.story-generator.request" -> "story-generator"
fn extract_endpoint(subject: &str) -> String {
    let parts: Vec<&str> = subject.split('.').collect();

    if parts.len() >= 2 {
        // For "mcp.orchestrator.request" -> return "orchestrator"
        // For "taletrail.generation.events" -> return "generation"
        parts[1].to_string()
    } else {
        // Fallback to full subject if parsing fails
        subject.to_string()
    }
}

/// Determine message type from subject
fn determine_message_type(subject: &str) -> String {
    if subject.contains(".request") {
        "Request".to_string()
    } else if subject.contains(".response") {
        "Response".to_string()
    } else if subject.contains(".events") {
        "Event".to_string()
    } else {
        "Unknown".to_string()
    }
}

/// Extract request ID from JSON payload if present
fn extract_request_id(payload: &str) -> Option<String> {
    // Try to parse as JSON and extract request_id field
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(payload) {
        // Try various common field names for request ID
        json.get("request_id")
            .or_else(|| json.get("requestId"))
            .or_else(|| json.get("id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            // Also try nested meta.request_id
            .or_else(|| {
                json.get("meta")
                    .and_then(|m| m.get("request_id"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
    } else {
        None
    }
}

/// Start NATS monitoring
///
/// Connects to NATS and subscribes to wildcard subjects:
/// - `mcp.>`
/// - `taletrail.>`
///
/// Runs in background task and emits messages to frontend.
pub async fn start_monitoring(
    nats_url: String,
    app_handle: tauri::AppHandle,
    ca_cert_path: std::path::PathBuf,
    nkey_file_path: std::path::PathBuf,
) -> AppResult<()> {
    // Check if already monitoring
    {
        let state = MONITOR_STATE.read().await;
        if state.is_some() {
            return Ok(()); // Already monitoring
        }
    }

    // Build connect options
    let mut opts = ConnectOptions::new()
        .name(crate::constants::defaults::NATS_CLIENT_NAME);

    // Load NKey seed from file for authentication
    let nkey_seed = std::fs::read_to_string(&nkey_file_path)
        .map_err(|e| AppError::ConnectionError(format!("Failed to read NKey file from {:?}: {}", nkey_file_path, e)))?;

    opts = opts.nkey(nkey_seed.trim().to_string());

    // Configure TLS with CA certificate
    let ca_cert = std::fs::read(&ca_cert_path)
        .map_err(|e| AppError::ConnectionError(format!("Failed to read CA cert from {:?}: {}", ca_cert_path, e)))?;

    let root_cert_store = {
        let mut store = rustls::RootCertStore::empty();
        let certs: Vec<_> = rustls_pemfile::certs(&mut ca_cert.as_slice())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AppError::ConnectionError(format!("Failed to parse CA cert: {}", e)))?;
        for cert in certs {
            store.add(cert)
                .map_err(|e| AppError::ConnectionError(format!("Failed to add CA cert to store: {}", e)))?;
        }
        store
    };

    let tls_client = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    opts = opts.tls_client_config(tls_client);

    // Connect to NATS
    let client = opts
        .connect(&nats_url)
        .await
        .map_err(|e| AppError::ConnectionError(format!("Failed to connect to NATS at {}: {}", nats_url, e)))?;

    eprintln!("[NATS Monitor] Connected to {} for monitoring", nats_url);

    // Subscribe to wildcard subjects
    let mcp_subscriber = client
        .subscribe("mcp.>".to_string())
        .await
        .map_err(|e| AppError::NatsError(format!("Failed to subscribe to mcp.>: {}", e)))?;

    let taletrail_subscriber = client
        .subscribe("taletrail.>".to_string())
        .await
        .map_err(|e| AppError::NatsError(format!("Failed to subscribe to taletrail.>: {}", e)))?;

    eprintln!("[NATS Monitor] Subscribed to mcp.> and taletrail.>");

    // Spawn background task for MCP messages
    let mcp_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let mut mcp_sub = mcp_subscriber;
        while let Some(msg) = mcp_sub.next().await {
            let nats_message = NatsMessage::new(msg.subject.to_string(), msg.payload.to_vec());

            // Emit to frontend
            if let Err(e) = mcp_handle.emit("nats-message", &nats_message) {
                eprintln!("[NATS Monitor] Failed to emit message: {}", e);
            }
        }
        eprintln!("[NATS Monitor] MCP subscription ended");
    });

    // Spawn background task for TaleTrail messages
    let taletrail_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let mut taletrail_sub = taletrail_subscriber;
        while let Some(msg) = taletrail_sub.next().await {
            let nats_message = NatsMessage::new(msg.subject.to_string(), msg.payload.to_vec());

            // Emit to frontend
            if let Err(e) = taletrail_handle.emit("nats-message", &nats_message) {
                eprintln!("[NATS Monitor] Failed to emit message: {}", e);
            }
        }
        eprintln!("[NATS Monitor] TaleTrail subscription ended");
    });

    // Store state (without subscribers since they're moved to background tasks)
    {
        let mut state = MONITOR_STATE.write().await;
        *state = Some(MonitoringState {
            client,
            subscribers: vec![], // Subscribers are owned by background tasks
            is_connected: true,
        });
    }

    // Emit connection status event
    if let Err(e) = app_handle.emit("nats-monitor-status", &serde_json::json!({
        "connected": true,
        "message": "NATS monitoring started"
    })) {
        eprintln!("[NATS Monitor] Failed to emit status: {}", e);
    }

    Ok(())
}

/// Stop NATS monitoring
pub async fn stop_monitoring() -> AppResult<()> {
    let mut state = MONITOR_STATE.write().await;

    if let Some(monitoring_state) = state.take() {
        // Unsubscribe all
        for mut subscriber in monitoring_state.subscribers {
            let _ = subscriber.unsubscribe().await;
        }

        // Flush and disconnect
        monitoring_state.client.flush().await
            .map_err(|e| AppError::NatsError(format!("Failed to flush NATS client: {}", e)))?;

        eprintln!("[NATS Monitor] Stopped monitoring");
    }

    Ok(())
}

/// Get current connection status
pub async fn get_connection_status() -> bool {
    let state = MONITOR_STATE.read().await;
    state.as_ref().map(|s| s.is_connected).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_endpoint() {
        assert_eq!(extract_endpoint("mcp.orchestrator.request"), "orchestrator");
        assert_eq!(extract_endpoint("mcp.story-generator.request"), "story-generator");
        assert_eq!(extract_endpoint("taletrail.generation.events"), "generation");
        assert_eq!(extract_endpoint("taletrail.generation.events.42"), "generation");
    }

    #[test]
    fn test_determine_message_type() {
        assert_eq!(determine_message_type("mcp.orchestrator.request"), "Request");
        assert_eq!(determine_message_type("mcp.orchestrator.response"), "Response");
        assert_eq!(determine_message_type("taletrail.generation.events"), "Event");
        assert_eq!(determine_message_type("custom.subject"), "Unknown");
    }

    #[test]
    fn test_extract_request_id() {
        let payload = r#"{"request_id": "req-123", "data": "test"}"#;
        assert_eq!(extract_request_id(payload), Some("req-123".to_string()));

        let payload_camel = r#"{"requestId": "req-456", "data": "test"}"#;
        assert_eq!(extract_request_id(payload_camel), Some("req-456".to_string()));

        let payload_nested = r#"{"meta": {"request_id": "req-789"}, "data": "test"}"#;
        assert_eq!(extract_request_id(payload_nested), Some("req-789".to_string()));

        let payload_no_id = r#"{"data": "test"}"#;
        assert_eq!(extract_request_id(payload_no_id), None);
    }

    #[test]
    fn test_nats_message_creation() {
        let subject = "mcp.orchestrator.request".to_string();
        let payload = br#"{"request_id": "req-123", "tool": "test"}"#.to_vec();

        let msg = NatsMessage::new(subject.clone(), payload);

        assert_eq!(msg.subject, subject);
        assert_eq!(msg.endpoint, "orchestrator");
        assert_eq!(msg.message_type, "Request");
        assert_eq!(msg.request_id, Some("req-123".to_string()));
        assert!(!msg.timestamp.is_empty());
    }
}
