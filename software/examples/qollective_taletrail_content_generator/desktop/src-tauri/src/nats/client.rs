use async_nats::{Client, ConnectOptions, Subscriber};
use crate::error::{AppError, AppResult};
use crate::models::{GenerationEvent, GenerationRequest};
use std::sync::Arc;
use tokio::sync::RwLock;

/// NATS subject patterns for generation events
pub mod subjects {
    pub const GENERATION_EVENTS: &str = "taletrail.generation.events";
    pub const GENERATION_EVENTS_TENANT: &str = "taletrail.generation.events.{tenant_id}";
}

/// Configuration for NATS client
#[derive(Debug, Clone)]
pub struct NatsConfig {
    /// NATS server URL
    pub url: String,
    /// Optional client name
    pub name: Option<String>,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Path to CA certificate for TLS
    pub ca_cert_path: String,
    /// Path to NKey seed file for authentication
    pub nkey_file_path: String,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:5222".to_string(),
            name: Some("taletrail-desktop".to_string()),
            timeout_secs: crate::constants::timeouts::DEFAULT_REQUEST_TIMEOUT_SECS,
            ca_cert_path: "/Users/ms/development/qollective/software/examples/qollective_taletrail_content_generator/certs/ca.pem".to_string(),
            nkey_file_path: "/Users/ms/development/qollective/software/examples/qollective_taletrail_content_generator/nkeys/desktop.nk".to_string(),
        }
    }
}

/// NATS client wrapper with connection management
pub struct NatsClient {
    config: NatsConfig,
    client: Arc<RwLock<Option<Client>>>,
    subscriber: Arc<RwLock<Option<Subscriber>>>,
}

impl NatsClient {
    /// Create a new NATS client with the given configuration
    pub fn new(config: NatsConfig) -> Self {
        Self {
            config,
            client: Arc::new(RwLock::new(None)),
            subscriber: Arc::new(RwLock::new(None)),
        }
    }

    /// Connect to the NATS server with TLS and NKey authentication
    pub async fn connect(&self) -> AppResult<()> {
        let mut client_guard = self.client.write().await;

        // If already connected, return early
        if client_guard.is_some() {
            return Ok(());
        }

        // Build connect options
        let mut opts = ConnectOptions::new();
        if let Some(ref name) = self.config.name {
            opts = opts.name(name.clone());
        }

        // Load NKey seed from file for authentication
        let nkey_seed = std::fs::read_to_string(&self.config.nkey_file_path)
            .map_err(|e| AppError::ConnectionError(format!("Failed to read NKey file from {}: {}", self.config.nkey_file_path, e)))?;

        opts = opts.nkey(nkey_seed.trim().to_string());

        // Configure TLS with CA certificate
        let ca_cert = std::fs::read(&self.config.ca_cert_path)
            .map_err(|e| AppError::ConnectionError(format!("Failed to read CA cert from {}: {}", self.config.ca_cert_path, e)))?;

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

        // Set request timeout for long-running operations like content generation
        opts = opts.request_timeout(Some(std::time::Duration::from_secs(self.config.timeout_secs)));

        // Connect to NATS
        let client = opts
            .connect(&self.config.url)
            .await
            .map_err(|e| AppError::ConnectionError(format!("Failed to connect to NATS: {}", e)))?;

        *client_guard = Some(client);
        Ok(())
    }

    /// Disconnect from the NATS server
    pub async fn disconnect(&self) -> AppResult<()> {
        let mut client_guard = self.client.write().await;
        let mut subscriber_guard = self.subscriber.write().await;

        // Unsubscribe if active
        *subscriber_guard = None;

        // Close connection
        if let Some(client) = client_guard.take() {
            client
                .flush()
                .await
                .map_err(|e| AppError::NatsError(format!("Failed to flush NATS client: {}", e)))?;
        }

        Ok(())
    }

    /// Check if the client is connected
    pub async fn is_connected(&self) -> bool {
        self.client.read().await.is_some()
    }

    /// Subscribe to generation events
    /// If tenant_id is provided, subscribes to tenant-specific events
    /// Otherwise, subscribes to all generation events
    pub async fn subscribe(&self, tenant_id: Option<String>) -> AppResult<Subscriber> {
        let client_guard = self.client.read().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AppError::ConnectionError("Not connected to NATS".to_string()))?;

        let subject = if let Some(tenant) = tenant_id {
            subjects::GENERATION_EVENTS_TENANT.replace("{tenant_id}", &tenant)
        } else {
            format!("{}.*", subjects::GENERATION_EVENTS)
        };

        let subscriber = client
            .subscribe(subject.clone())
            .await
            .map_err(|e| AppError::NatsError(format!("Failed to subscribe to {}: {}", subject, e)))?;

        Ok(subscriber)
    }

    /// Unsubscribe from current subscription
    pub async fn unsubscribe(&self) -> AppResult<()> {
        let mut subscriber_guard = self.subscriber.write().await;

        if let Some(mut subscriber) = subscriber_guard.take() {
            subscriber
                .unsubscribe()
                .await
                .map_err(|e| AppError::NatsError(format!("Failed to unsubscribe: {}", e)))?;
        }

        Ok(())
    }

    /// Store the active subscriber
    pub async fn set_subscriber(&self, subscriber: Subscriber) {
        let mut subscriber_guard = self.subscriber.write().await;
        *subscriber_guard = Some(subscriber);
    }

    /// Parse a NATS message payload into a GenerationEvent
    pub fn parse_event(payload: &[u8]) -> AppResult<GenerationEvent> {
        serde_json::from_slice(payload)
            .map_err(|e| AppError::JsonError(e))
    }

    /// Publish a generation event
    pub async fn publish(&self, subject: &str, event: &GenerationEvent) -> AppResult<()> {
        let client_guard = self.client.read().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AppError::ConnectionError("Not connected to NATS".to_string()))?;

        let payload = serde_json::to_vec(event)
            .map_err(|e| AppError::JsonError(e))?;

        client
            .publish(subject.to_string(), payload.into())
            .await
            .map_err(|e| AppError::NatsError(format!("Failed to publish: {}", e)))?;

        Ok(())
    }

    /// Publish a generation request to the orchestrator via MCP envelope
    ///
    /// This wraps the request in an MCP tool call envelope and sends it to the
    /// orchestrator using the request-response pattern on `mcp.orchestrator.request`.
    ///
    /// # Arguments
    /// * `request` - The generation request to publish
    ///
    /// # Errors
    /// Returns `AppError` if:
    /// - Not connected to NATS
    /// - Request validation fails
    /// - Envelope creation or encoding fails
    /// - NATS request operation fails
    /// - Response decoding fails
    pub async fn publish_request(&self, request: &GenerationRequest) -> AppResult<()> {
        use qollective::envelope::{Envelope, Meta, TracingMeta};
        use qollective::types::mcp::McpData;
        use qollective::envelope::nats_codec::NatsEnvelopeCodec;
        use rmcp::model::{CallToolRequest, CallToolRequestParam, CallToolRequestMethod};
        use serde_json::json;
        use uuid::Uuid;

        // Validate request before publishing
        request.validate()?;

        let client_guard = self.client.read().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AppError::ConnectionError("Not connected to NATS".to_string()))?;

        // Convert desktop request to shared type format expected by orchestrator
        let shared_request = request.to_shared_type()?;

        // Create MCP tool call request with converted request
        let tool_params = json!({
            "generation_request": shared_request
        });

        let tool_call_request = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "orchestrate_generation".to_string().into(),
                arguments: tool_params.as_object().cloned(),
            },
            extensions: Default::default(),
        };

        // Wrap in McpData
        let mcp_data = McpData::with_tool_call(tool_call_request);

        // Create envelope metadata with tenant and tracing info
        let trace_id = Uuid::new_v4().to_string();

        let mut metadata = Meta::default();
        metadata.tenant = Some(request.tenant_id.clone());
        metadata.tracing = Some(TracingMeta {
            trace_id: Some(trace_id.clone()),
            parent_span_id: None,
            span_id: None,
            baggage: Default::default(),
            sampling_rate: None,
            sampled: None,
            trace_state: None,
            operation_name: Some("orchestrate_generation".to_string()),
            span_kind: None,
            span_status: None,
            tags: Default::default(),
        });

        // Create envelope
        let envelope = Envelope::new(metadata, mcp_data);

        // Encode envelope
        let payload = NatsEnvelopeCodec::encode(&envelope)
            .map_err(|e| AppError::NatsError(format!("Failed to encode envelope: {}", e)))?;

        // Send request and wait for response
        let response = client
            .request(crate::constants::nats::ORCHESTRATOR_REQUEST_SUBJECT.to_string(), payload.into())
            .await
            .map_err(|e| AppError::NatsError(format!("Failed to send request: {}", e)))?;

        // Decode response envelope
        let response_envelope: Envelope<McpData> = NatsEnvelopeCodec::decode(&response.payload)
            .map_err(|e| AppError::NatsError(format!("Failed to decode response: {}", e)))?;

        // Check if response indicates an error
        if let Some(tool_response) = response_envelope.payload.tool_response {
            if tool_response.is_error == Some(true) {
                let error_msg = tool_response.content
                    .iter()
                    .filter_map(|c| {
                        // Extract text content from MCP Content
                        if let Ok(json) = serde_json::to_value(c) {
                            json.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(AppError::NatsError(format!("Orchestrator error: {}", error_msg)));
            }
        }

        Ok(())
    }
}

impl Clone for NatsClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: Arc::clone(&self.client),
            subscriber: Arc::clone(&self.subscriber),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_config_default() {
        let config = NatsConfig::default();
        assert_eq!(config.url, "nats://localhost:5222");
        assert_eq!(config.name, Some("taletrail-desktop".to_string()));
        assert_eq!(config.timeout_secs, 180);
    }

    #[test]
    fn test_subject_patterns() {
        assert_eq!(subjects::GENERATION_EVENTS, "taletrail.generation.events");

        let tenant_subject = subjects::GENERATION_EVENTS_TENANT.replace("{tenant_id}", "tenant-123");
        assert_eq!(tenant_subject, "taletrail.generation.events.tenant-123");
    }

    #[test]
    fn test_parse_event() {
        let json = r#"{
            "eventType": "generation_started",
            "tenantId": "tenant-123",
            "requestId": "req-456",
            "timestamp": "2025-10-22T10:00:00Z",
            "servicePhase": "story-generator",
            "status": "in_progress"
        }"#;

        let event = NatsClient::parse_event(json.as_bytes()).unwrap();
        assert_eq!(event.event_type, "generation_started");
        assert_eq!(event.tenant_id, "tenant-123");
    }

    #[tokio::test]
    async fn test_nats_client_initialization() {
        let config = NatsConfig::default();
        let client = NatsClient::new(config);

        assert!(!client.is_connected().await);
    }
}
