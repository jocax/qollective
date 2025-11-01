/// NATS monitoring module for real-time message tracking
///
/// Subscribes to wildcard subjects and emits messages to frontend via Tauri events.
/// Auto-connects on app startup and maintains connection state.

use async_nats::{Client, ConnectOptions, Subscriber};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use chrono::Utc;
use tauri::Emitter;
use futures::StreamExt;

/// Global monitoring state
static MONITOR_STATE: once_cell::sync::Lazy<Arc<RwLock<Option<MonitoringState>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// Monitoring diagnostics for tracking connection health and message flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringDiagnostics {
    /// When monitoring connection was established (ISO 8601)
    pub connection_timestamp: String,
    /// Total number of messages received from NATS
    pub messages_received: u64,
    /// Total number of messages successfully emitted to frontend
    pub messages_emitted: u64,
    /// Total number of emission failures
    pub emission_failures: u64,
    /// Timestamp of last received message (ISO 8601)
    pub last_message_timestamp: Option<String>,
    /// Current connection status
    pub is_connected: bool,
}

impl MonitoringDiagnostics {
    /// Create new diagnostics with default values
    pub fn new() -> Self {
        Self {
            connection_timestamp: Utc::now().to_rfc3339(),
            messages_received: 0,
            messages_emitted: 0,
            emission_failures: 0,
            last_message_timestamp: None,
            is_connected: true,
        }
    }
}

/// Internal monitoring state
struct MonitoringState {
    client: Client,
    subscribers: Vec<Subscriber>,
    is_connected: bool,
    diagnostics: Arc<RwLock<MonitoringDiagnostics>>,
    shutdown_tx: broadcast::Sender<()>,
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

/// Emit event with retry logic
async fn emit_with_retry<T: Serialize + Clone>(
    app_handle: &tauri::AppHandle,
    event: &str,
    payload: &T,
) -> Result<(), String> {
    use crate::constants::monitoring::{EMISSION_RETRY_ATTEMPTS, EMISSION_RETRY_DELAY_MS};

    for attempt in 1..=EMISSION_RETRY_ATTEMPTS {
        match app_handle.emit(event, payload) {
            Ok(_) => {
                if attempt > 1 {
                    eprintln!(
                        "[NATS Monitor] [{}] [INFO] Successfully emitted '{}' on attempt {}",
                        Utc::now().to_rfc3339(),
                        event,
                        attempt
                    );
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!(
                    "[NATS Monitor] [{}] [WARN] Emission attempt {}/{} failed for '{}': {}",
                    Utc::now().to_rfc3339(),
                    attempt,
                    EMISSION_RETRY_ATTEMPTS,
                    event,
                    e
                );

                if attempt < EMISSION_RETRY_ATTEMPTS {
                    tokio::time::sleep(tokio::time::Duration::from_millis(EMISSION_RETRY_DELAY_MS)).await;
                } else {
                    return Err(format!("Failed after {} attempts: {}", EMISSION_RETRY_ATTEMPTS, e));
                }
            }
        }
    }

    Err("All retry attempts exhausted".to_string())
}

/// Emit diagnostics to frontend
async fn emit_diagnostics(
    app_handle: &tauri::AppHandle,
    diagnostics: &Arc<RwLock<MonitoringDiagnostics>>,
) {
    use crate::constants::monitoring::events;

    let diag = diagnostics.read().await.clone();

    if let Err(e) = emit_with_retry(app_handle, events::DIAGNOSTICS, &diag).await {
        eprintln!(
            "[NATS Monitor] [{}] [ERROR] Failed to emit diagnostics: {}",
            Utc::now().to_rfc3339(),
            e
        );
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
    use crate::constants::monitoring::events;

    // Check if already monitoring
    {
        let state = MONITOR_STATE.read().await;
        if state.is_some() {
            eprintln!(
                "[NATS Monitor] [{}] [INFO] Already monitoring, skipping start",
                Utc::now().to_rfc3339()
            );
            return Ok(()); // Already monitoring
        }
    }

    let start_time = Utc::now();
    eprintln!(
        "[NATS Monitor] [{}] [INFO] Starting NATS monitoring connection to {}",
        start_time.to_rfc3339(),
        nats_url
    );

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

    eprintln!(
        "[NATS Monitor] [{}] [INFO] Connected to {} for monitoring",
        Utc::now().to_rfc3339(),
        nats_url
    );

    // Subscribe to wildcard subjects
    let mcp_subscriber = client
        .subscribe("mcp.>".to_string())
        .await
        .map_err(|e| AppError::NatsError(format!("Failed to subscribe to mcp.>: {}", e)))?;

    let taletrail_subscriber = client
        .subscribe("taletrail.>".to_string())
        .await
        .map_err(|e| AppError::NatsError(format!("Failed to subscribe to taletrail.>: {}", e)))?;

    eprintln!(
        "[NATS Monitor] [{}] [INFO] Subscribed to mcp.> and taletrail.>",
        Utc::now().to_rfc3339()
    );

    // DIAGNOSTIC: Log subscription details
    eprintln!(
        "[NATS Monitor] [{}] [DIAGNOSTIC] Monitoring subscriptions established:",
        Utc::now().to_rfc3339()
    );
    eprintln!("  - MCP subscription: mcp.> (catches all mcp.* subjects)");
    eprintln!("  - TaleTrail subscription: taletrail.> (catches all taletrail.* subjects)");
    eprintln!("  - Connection: {}", nats_url);

    // Create diagnostics tracker
    let diagnostics = Arc::new(RwLock::new(MonitoringDiagnostics::new()));

    // Create shutdown channel
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    // Spawn background task for MCP messages
    let mcp_handle = app_handle.clone();
    let mcp_diagnostics = diagnostics.clone();
    let mut mcp_shutdown_rx = shutdown_tx.subscribe();
    tauri::async_runtime::spawn(async move {
        let mut mcp_sub = mcp_subscriber;
        eprintln!(
            "[NATS Monitor] [{}] [DIAGNOSTIC] MCP subscription loop started, waiting for messages on 'mcp.>'",
            Utc::now().to_rfc3339()
        );
        loop {
            tokio::select! {
                Some(msg) = mcp_sub.next() => {
                    let timestamp = Utc::now();
                    eprintln!(
                        "[NATS Monitor] [{}] [INFO] Received MCP message on subject: {}",
                        timestamp.to_rfc3339(),
                        msg.subject
                    );

                    // DIAGNOSTIC: Log message details
                    eprintln!(
                        "[NATS Monitor] [{}] [DIAGNOSTIC] MCP message details:",
                        timestamp.to_rfc3339()
                    );
                    eprintln!("  - Subject: {}", msg.subject);
                    eprintln!("  - Payload size: {} bytes", msg.payload.len());
                    eprintln!("  - Reply subject: {:?}", msg.reply);
                    let payload_preview = String::from_utf8_lossy(&msg.payload[..msg.payload.len().min(100)]);
                    eprintln!("  - Payload preview: {}...", payload_preview);

                    // Update diagnostics
                    {
                        let mut diag = mcp_diagnostics.write().await;
                        diag.messages_received += 1;
                        diag.last_message_timestamp = Some(timestamp.to_rfc3339());
                    }

                    let nats_message = NatsMessage::new(msg.subject.to_string(), msg.payload.to_vec());

                    // DIAGNOSTIC: Log before emission attempt
                    eprintln!(
                        "[NATS Monitor] [{}] [DIAGNOSTIC] Attempting to emit to frontend event '{}'",
                        Utc::now().to_rfc3339(),
                        events::NATS_MESSAGE
                    );

                    // Emit to frontend with retry
                    match emit_with_retry(&mcp_handle, events::NATS_MESSAGE, &nats_message).await {
                        Ok(_) => {
                            let mut diag = mcp_diagnostics.write().await;
                            diag.messages_emitted += 1;
                            eprintln!(
                                "[NATS Monitor] [{}] [INFO] Successfully emitted MCP message to frontend",
                                Utc::now().to_rfc3339()
                            );
                            eprintln!(
                                "[NATS Monitor] [{}] [DIAGNOSTIC] Total emitted: {}, Total received: {}",
                                Utc::now().to_rfc3339(),
                                diag.messages_emitted,
                                diag.messages_received
                            );
                        }
                        Err(e) => {
                            let mut diag = mcp_diagnostics.write().await;
                            diag.emission_failures += 1;
                            eprintln!(
                                "[NATS Monitor] [{}] [ERROR] Failed to emit MCP message after retries: {}",
                                Utc::now().to_rfc3339(),
                                e
                            );

                            // Emit error event to frontend
                            let error_payload = serde_json::json!({
                                "subject": msg.subject.to_string(),
                                "error": e,
                                "timestamp": Utc::now().to_rfc3339()
                            });
                            let _ = mcp_handle.emit(events::ERROR, &error_payload);
                        }
                    }
                }
                _ = mcp_shutdown_rx.recv() => {
                    eprintln!(
                        "[NATS Monitor] [{}] [INFO] MCP subscription received shutdown signal",
                        Utc::now().to_rfc3339()
                    );
                    break;
                }
            }
        }
        eprintln!(
            "[NATS Monitor] [{}] [INFO] MCP subscription ended",
            Utc::now().to_rfc3339()
        );
    });

    // Spawn background task for TaleTrail messages
    let taletrail_handle = app_handle.clone();
    let taletrail_diagnostics = diagnostics.clone();
    let mut taletrail_shutdown_rx = shutdown_tx.subscribe();
    tauri::async_runtime::spawn(async move {
        let mut taletrail_sub = taletrail_subscriber;
        eprintln!(
            "[NATS Monitor] [{}] [DIAGNOSTIC] TaleTrail subscription loop started, waiting for messages on 'taletrail.>'",
            Utc::now().to_rfc3339()
        );
        loop {
            tokio::select! {
                Some(msg) = taletrail_sub.next() => {
                    let timestamp = Utc::now();
                    eprintln!(
                        "[NATS Monitor] [{}] [INFO] Received TaleTrail message on subject: {}",
                        timestamp.to_rfc3339(),
                        msg.subject
                    );

                    // DIAGNOSTIC: Log message details
                    eprintln!(
                        "[NATS Monitor] [{}] [DIAGNOSTIC] TaleTrail message details:",
                        timestamp.to_rfc3339()
                    );
                    eprintln!("  - Subject: {}", msg.subject);
                    eprintln!("  - Payload size: {} bytes", msg.payload.len());

                    // Update diagnostics
                    {
                        let mut diag = taletrail_diagnostics.write().await;
                        diag.messages_received += 1;
                        diag.last_message_timestamp = Some(timestamp.to_rfc3339());
                    }

                    let nats_message = NatsMessage::new(msg.subject.to_string(), msg.payload.to_vec());

                    // Emit to frontend with retry
                    match emit_with_retry(&taletrail_handle, events::NATS_MESSAGE, &nats_message).await {
                        Ok(_) => {
                            let mut diag = taletrail_diagnostics.write().await;
                            diag.messages_emitted += 1;
                            eprintln!(
                                "[NATS Monitor] [{}] [INFO] Successfully emitted TaleTrail message to frontend",
                                Utc::now().to_rfc3339()
                            );
                            eprintln!(
                                "[NATS Monitor] [{}] [DIAGNOSTIC] Total emitted: {}, Total received: {}",
                                Utc::now().to_rfc3339(),
                                diag.messages_emitted,
                                diag.messages_received
                            );
                        }
                        Err(e) => {
                            let mut diag = taletrail_diagnostics.write().await;
                            diag.emission_failures += 1;
                            eprintln!(
                                "[NATS Monitor] [{}] [ERROR] Failed to emit TaleTrail message after retries: {}",
                                Utc::now().to_rfc3339(),
                                e
                            );

                            // Emit error event to frontend
                            let error_payload = serde_json::json!({
                                "subject": msg.subject.to_string(),
                                "error": e,
                                "timestamp": Utc::now().to_rfc3339()
                            });
                            let _ = taletrail_handle.emit(events::ERROR, &error_payload);
                        }
                    }
                }
                _ = taletrail_shutdown_rx.recv() => {
                    eprintln!(
                        "[NATS Monitor] [{}] [INFO] TaleTrail subscription received shutdown signal",
                        Utc::now().to_rfc3339()
                    );
                    break;
                }
            }
        }
        eprintln!(
            "[NATS Monitor] [{}] [INFO] TaleTrail subscription ended",
            Utc::now().to_rfc3339()
        );
    });

    // DIAGNOSTIC: Spawn test task to verify subscription after loops are ready
    let test_client = client.clone();
    tauri::async_runtime::spawn(async move {
        // Small delay to ensure subscription loops are fully active
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        eprintln!("[NATS Monitor] [DIAGNOSTIC] Testing subscription by publishing test message to mcp.test");
        if let Err(e) = test_client.publish("mcp.test".to_string(), "test-payload".into()).await {
            eprintln!("[NATS Monitor] [DIAGNOSTIC] Failed to publish test message: {}", e);
        } else {
            eprintln!("[NATS Monitor] [DIAGNOSTIC] Test message published successfully");
            if let Err(e) = test_client.flush().await {
                eprintln!("[NATS Monitor] [DIAGNOSTIC] Failed to flush test message: {}", e);
            } else {
                eprintln!("[NATS Monitor] [DIAGNOSTIC] Test message flushed successfully");
            }
        }
    });

    // Spawn diagnostics reporting task
    let diagnostics_handle = app_handle.clone();
    let diagnostics_reporter = diagnostics.clone();
    let mut diagnostics_shutdown_rx = shutdown_tx.subscribe();
    tauri::async_runtime::spawn(async move {
        use crate::constants::monitoring::DIAGNOSTICS_INTERVAL_SECS;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(DIAGNOSTICS_INTERVAL_SECS));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    emit_diagnostics(&diagnostics_handle, &diagnostics_reporter).await;
                }
                _ = diagnostics_shutdown_rx.recv() => {
                    eprintln!(
                        "[NATS Monitor] [{}] [INFO] Diagnostics reporter received shutdown signal",
                        Utc::now().to_rfc3339()
                    );
                    break;
                }
            }
        }
        eprintln!(
            "[NATS Monitor] [{}] [INFO] Diagnostics reporter ended",
            Utc::now().to_rfc3339()
        );
    });

    // Store state (without subscribers since they're moved to background tasks)
    {
        let mut state = MONITOR_STATE.write().await;
        *state = Some(MonitoringState {
            client,
            subscribers: vec![], // Subscribers are owned by background tasks
            is_connected: true,
            diagnostics: diagnostics.clone(),
            shutdown_tx: shutdown_tx.clone(),
        });
    }

    // Emit connection status event with retry
    let status_payload = serde_json::json!({
        "connected": true,
        "message": "NATS monitoring started successfully",
        "timestamp": Utc::now().to_rfc3339()
    });

    if let Err(e) = emit_with_retry(&app_handle, events::STATUS, &status_payload).await {
        eprintln!(
            "[NATS Monitor] [{}] [ERROR] Failed to emit connection status: {}",
            Utc::now().to_rfc3339(),
            e
        );
    }

    // Emit initial diagnostics
    emit_diagnostics(&app_handle, &diagnostics).await;

    eprintln!(
        "[NATS Monitor] [{}] [INFO] Monitoring started successfully",
        Utc::now().to_rfc3339()
    );

    Ok(())
}

/// Stop NATS monitoring
pub async fn stop_monitoring() -> AppResult<()> {
    use crate::constants::monitoring::SHUTDOWN_TIMEOUT_SECS;

    let mut state = MONITOR_STATE.write().await;

    if let Some(monitoring_state) = state.take() {
        eprintln!(
            "[NATS Monitor] [{}] [INFO] Initiating graceful shutdown",
            Utc::now().to_rfc3339()
        );

        // Send shutdown signal to all background tasks
        let _ = monitoring_state.shutdown_tx.send(());

        // Update diagnostics to reflect disconnection
        {
            let mut diag = monitoring_state.diagnostics.write().await;
            diag.is_connected = false;
        }

        // Wait for background tasks to complete (with timeout)
        eprintln!(
            "[NATS Monitor] [{}] [INFO] Waiting for background tasks to complete (timeout: {}s)",
            Utc::now().to_rfc3339(),
            SHUTDOWN_TIMEOUT_SECS
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(SHUTDOWN_TIMEOUT_SECS)).await;

        // Unsubscribe all (subscribers vec is empty as they're owned by tasks, but keeping for future use)
        for mut subscriber in monitoring_state.subscribers {
            let _ = subscriber.unsubscribe().await;
        }

        // Flush and disconnect
        monitoring_state.client.flush().await
            .map_err(|e| AppError::NatsError(format!("Failed to flush NATS client: {}", e)))?;

        eprintln!(
            "[NATS Monitor] [{}] [INFO] NATS monitoring stopped successfully",
            Utc::now().to_rfc3339()
        );
    } else {
        eprintln!(
            "[NATS Monitor] [{}] [INFO] No active monitoring to stop",
            Utc::now().to_rfc3339()
        );
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
