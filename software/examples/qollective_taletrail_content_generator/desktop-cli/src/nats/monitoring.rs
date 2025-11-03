/// NATS monitoring module for real-time message tracking
///
/// Subscribes to wildcard subjects and buffers messages for TUI display.
/// Adapted for desktop-cli (no Tauri, uses smol executor, channels for event streaming)

use async_nats::{Client, ConnectOptions, Subscriber};
use crate::error::{AppError, Result};
use crate::config::NatsConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures::lock::Mutex;
use chrono::Utc;
use futures::StreamExt;
use async_channel::{Sender, Receiver, unbounded};

/// Monitoring diagnostics for tracking connection health and message flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringDiagnostics {
    /// When monitoring connection was established (ISO 8601)
    pub connection_timestamp: String,
    /// Total number of messages received from NATS
    pub messages_received: u64,
    /// Total number of messages buffered
    pub messages_buffered: u64,
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
            messages_buffered: 0,
            last_message_timestamp: None,
            is_connected: true,
        }
    }
}

impl Default for MonitoringDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

/// NATS message structure for TUI
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
        parts[1].to_string()
    } else {
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
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(payload) {
        json.get("request_id")
            .or_else(|| json.get("requestId"))
            .or_else(|| json.get("id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
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

/// Monitoring state and message buffer
pub struct NatsMonitor {
    config: NatsConfig,
    client: Arc<Mutex<Option<Client>>>,
    diagnostics: Arc<Mutex<MonitoringDiagnostics>>,
    message_tx: Sender<NatsMessage>,
    message_rx: Receiver<NatsMessage>,
    buffer_size: usize,
}

impl NatsMonitor {
    /// Create a new NATS monitor with the given configuration
    ///
    /// # Arguments
    /// * `config` - NATS configuration
    /// * `buffer_size` - Maximum number of messages to buffer (default: 1000)
    pub fn new(config: NatsConfig, buffer_size: Option<usize>) -> Self {
        let (message_tx, message_rx) = unbounded();

        Self {
            config,
            client: Arc::new(Mutex::new(None)),
            diagnostics: Arc::new(Mutex::new(MonitoringDiagnostics::new())),
            message_tx,
            message_rx,
            buffer_size: buffer_size.unwrap_or(1000),
        }
    }

    /// Connect to NATS and start monitoring
    pub async fn start(&self) -> Result<()> {
        let mut client_guard = self.client.lock().await;

        // If already connected, return early
        if client_guard.is_some() {
            return Ok(());
        }

        eprintln!("[NATS Monitor] Starting monitoring connection to {}", self.config.url);

        // Build connect options
        let mut opts = ConnectOptions::new().name("taletrail-cli-monitor");

        // Load NKey seed from file for authentication if provided
        if let Some(ref nkey_path) = self.config.nkey_path {
            let nkey_seed = smol::fs::read_to_string(nkey_path)
                .await
                .map_err(|e| AppError::NatsConnection(format!("Failed to read NKey file from {:?}: {}", nkey_path, e)))?;

            opts = opts.nkey(nkey_seed.trim().to_string());
        }

        // Configure TLS with CA certificate if provided
        if let Some(ref ca_cert_path) = self.config.tls_cert_path {
            let ca_cert = smol::fs::read(ca_cert_path)
                .await
                .map_err(|e| AppError::NatsConnection(format!("Failed to read CA cert from {:?}: {}", ca_cert_path, e)))?;

            let root_cert_store = {
                let mut store = rustls::RootCertStore::empty();
                let certs: Vec<_> = rustls_pemfile::certs(&mut ca_cert.as_slice())
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| AppError::NatsConnection(format!("Failed to parse CA cert: {}", e)))?;
                for cert in certs {
                    store.add(cert)
                        .map_err(|e| AppError::NatsConnection(format!("Failed to add CA cert to store: {}", e)))?;
                }
                store
            };

            let tls_client = rustls::ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_no_client_auth();

            opts = opts.tls_client_config(tls_client);
        }

        // Connect to NATS
        let client = opts
            .connect(&self.config.url)
            .await
            .map_err(|e| AppError::NatsConnection(format!("Failed to connect to NATS at {}: {}", self.config.url, e)))?;

        eprintln!("[NATS Monitor] Connected to {}", self.config.url);

        // Subscribe to wildcard subjects
        let mcp_subscriber = client
            .subscribe(super::subjects::MCP_WILDCARD.to_string())
            .await
            .map_err(|e| AppError::NatsRequest(format!("Failed to subscribe to mcp.>: {}", e)))?;

        let taletrail_subscriber = client
            .subscribe(super::subjects::TALETRAIL_WILDCARD.to_string())
            .await
            .map_err(|e| AppError::NatsRequest(format!("Failed to subscribe to taletrail.>: {}", e)))?;

        eprintln!("[NATS Monitor] Subscribed to mcp.> and taletrail.>");

        *client_guard = Some(client);

        // Spawn background tasks for message processing
        self.spawn_subscription_task(mcp_subscriber, "MCP");
        self.spawn_subscription_task(taletrail_subscriber, "TaleTrail");

        Ok(())
    }

    /// Spawn a background task to process subscription messages
    fn spawn_subscription_task(&self, subscriber: Subscriber, label: &str) {
        let message_tx = self.message_tx.clone();
        let diagnostics = Arc::clone(&self.diagnostics);
        let label = label.to_string();

        smol::spawn(async move {
            let mut sub = subscriber;
            eprintln!("[NATS Monitor] {} subscription loop started", label);

            while let Some(msg) = sub.next().await {
                let timestamp = Utc::now();
                eprintln!("[NATS Monitor] Received {} message on subject: {}", label, msg.subject);

                // Update diagnostics
                {
                    let mut diag = diagnostics.lock().await;
                    diag.messages_received += 1;
                    diag.last_message_timestamp = Some(timestamp.to_rfc3339());
                }

                let nats_message = NatsMessage::new(msg.subject.to_string(), msg.payload.to_vec());

                // Send to channel (non-blocking)
                if let Err(e) = message_tx.try_send(nats_message) {
                    eprintln!("[NATS Monitor] Failed to buffer {} message: {}", label, e);
                } else {
                    let mut diag = diagnostics.lock().await;
                    diag.messages_buffered += 1;
                }
            }

            eprintln!("[NATS Monitor] {} subscription ended", label);
        })
        .detach();
    }

    /// Stop monitoring and disconnect
    pub async fn stop(&self) -> Result<()> {
        let mut client_guard = self.client.lock().await;

        if let Some(client) = client_guard.take() {
            eprintln!("[NATS Monitor] Stopping monitoring");

            // Update diagnostics
            {
                let mut diag = self.diagnostics.lock().await;
                diag.is_connected = false;
            }

            // Flush and disconnect
            client.flush().await
                .map_err(|e| AppError::NatsConnection(format!("Failed to flush NATS client: {}", e)))?;

            eprintln!("[NATS Monitor] Monitoring stopped");
        }

        Ok(())
    }

    /// Check if monitoring is active
    pub async fn is_connected(&self) -> bool {
        self.client.lock().await.is_some()
    }

    /// Get a clone of the message receiver for consuming messages
    pub fn message_receiver(&self) -> Receiver<NatsMessage> {
        self.message_rx.clone()
    }

    /// Get current diagnostics
    pub async fn get_diagnostics(&self) -> MonitoringDiagnostics {
        self.diagnostics.lock().await.clone()
    }

    /// Get a cloned handle for sharing across tasks
    pub fn clone_handle(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: Arc::clone(&self.client),
            diagnostics: Arc::clone(&self.diagnostics),
            message_tx: self.message_tx.clone(),
            message_rx: self.message_rx.clone(),
            buffer_size: self.buffer_size,
        }
    }
}

impl Clone for NatsMonitor {
    fn clone(&self) -> Self {
        self.clone_handle()
    }
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
