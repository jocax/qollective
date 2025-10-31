use async_nats::{Client, ConnectOptions, Subscriber};
use async_trait::async_trait;
use chrono::Utc;
use crate::error::{AppError, AppResult};
use crate::models::{GenerationEvent, GenerationRequest};
use crate::services::traits::NatsService;
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
    pub ca_cert_path: std::path::PathBuf,
    /// Path to NKey seed file for authentication
    pub nkey_file_path: std::path::PathBuf,
}

impl NatsConfig {
    /// Create NatsConfig from AppConfig
    pub fn from_app_config(app_config: &crate::config::AppConfig) -> Self {
        Self {
            url: app_config.nats.url.clone(),
            name: Some(crate::constants::defaults::NATS_CLIENT_NAME.to_string()),
            timeout_secs: app_config.nats.request_timeout_ms / 1000,
            ca_cert_path: app_config.ca_cert_path(),
            nkey_file_path: app_config.nkey_path(),
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
            .map_err(|e| AppError::ConnectionError(format!("Failed to read NKey file from {:?}: {}", self.config.nkey_file_path, e)))?;

        opts = opts.nkey(nkey_seed.trim().to_string());

        // Configure TLS with CA certificate
        let ca_cert = std::fs::read(&self.config.ca_cert_path)
            .map_err(|e| AppError::ConnectionError(format!("Failed to read CA cert from {:?}: {}", self.config.ca_cert_path, e)))?;

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
        metadata.request_id = Some(Uuid::new_v4());
        metadata.timestamp = Some(Utc::now());
        metadata.version = Some(crate::constants::defaults::ENVELOPE_VERSION.to_string());
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

    /// Send MCP tool call request via NATS
    ///
    /// Wraps the CallToolRequest in a Qollective envelope and sends it to the specified
    /// NATS subject using request-reply pattern. Returns the complete response envelope
    /// with all metadata preserved.
    ///
    /// # Arguments
    /// * `subject` - NATS subject to send the request to
    /// * `tool_call` - rmcp CallToolRequest to send
    /// * `tenant_id` - Tenant ID for multi-tenancy support
    /// * `timeout` - Optional timeout duration (uses config default if None)
    ///
    /// # Returns
    /// * `Ok(Envelope<McpData>)` - The complete response envelope with metadata
    /// * `Err(AppError)` - Error if request fails
    ///
    /// # Errors
    /// Returns `AppError` if:
    /// - Not connected to NATS
    /// - Envelope encoding fails
    /// - NATS request operation fails
    /// - Response decoding fails
    /// - MCP server returns an error result
    pub async fn send_mcp_tool_call(
        &self,
        subject: &str,
        tool_call: rmcp::model::CallToolRequest,
        tenant_id: i32,
        timeout: Option<std::time::Duration>,
    ) -> AppResult<qollective::envelope::Envelope<qollective::types::mcp::McpData>> {
        use qollective::envelope::{Envelope, Meta, TracingMeta};
        use qollective::types::mcp::McpData;
        use qollective::envelope::nats_codec::NatsEnvelopeCodec;
        use uuid::Uuid;

        let client_guard = self.client.read().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AppError::ConnectionError("Not connected to NATS".to_string()))?;

        // Wrap in McpData
        let mcp_data = McpData::with_tool_call(tool_call);

        // Create envelope metadata with tenant and tracing info
        let trace_id = Uuid::new_v4().to_string();
        let tool_name = mcp_data
            .tool_call
            .as_ref()
            .map(|tc| tc.params.name.as_ref().to_string())
            .unwrap_or_else(|| "unknown_tool".to_string());

        let mut metadata = Meta::default();
        metadata.request_id = Some(Uuid::new_v4());
        metadata.timestamp = Some(Utc::now());
        metadata.version = Some(crate::constants::defaults::ENVELOPE_VERSION.to_string());
        metadata.tenant = Some(tenant_id.to_string());
        metadata.tracing = Some(TracingMeta {
            trace_id: Some(trace_id.clone()),
            parent_span_id: None,
            span_id: None,
            baggage: Default::default(),
            sampling_rate: None,
            sampled: None,
            trace_state: None,
            operation_name: Some(tool_name.clone()),
            span_kind: None,
            span_status: None,
            tags: Default::default(),
        });

        // Create envelope
        let envelope = Envelope::new(metadata, mcp_data);

        // DIAGNOSTIC: Log envelope creation
        eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Preparing MCP request:");
        eprintln!("  - Tool name: {}", tool_name);
        eprintln!("  - Tenant ID: {}", tenant_id);
        eprintln!("  - Trace ID: {}", trace_id);
        eprintln!("  - Request ID: {:?}", envelope.meta.request_id);

        // Encode envelope
        let payload = NatsEnvelopeCodec::encode(&envelope)
            .map_err(|e| AppError::NatsError(format!("Failed to encode envelope: {}", e)))?;

        // DIAGNOSTIC: Log encoding success
        eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Envelope encoded: {} bytes", payload.len());

        // Use custom timeout if provided, otherwise use default from config
        let request_timeout = timeout.unwrap_or_else(|| {
            std::time::Duration::from_secs(self.config.timeout_secs)
        });

        eprintln!("[TaleTrail NATS] Sending MCP tool request:");
        eprintln!("  - subject: {}", subject);
        eprintln!("  - timeout: {:?}", request_timeout);
        eprintln!("  - payload size: {} bytes", payload.len());

        // Send request and wait for response with timeout using tokio timeout
        let response = tokio::time::timeout(
            request_timeout,
            client.request(subject.to_string(), payload.clone().into())
        )
            .await
            .map_err(|_| {
                eprintln!("[TaleTrail NATS] Request timed out after {:?}", request_timeout);
                AppError::NatsError(format!("Request to {} timed out after {:?}", subject, request_timeout))
            })?
            .map_err(|e| {
                eprintln!("[TaleTrail NATS] Request failed: {}", e);
                AppError::NatsError(format!("Failed to send request to {}: {}", subject, e))
            })?;

        eprintln!("[TaleTrail NATS] Response received: {} bytes", response.payload.len());

        // DIAGNOSTIC: Publish copy for monitoring (fire-and-forget)
        // This ensures monitoring wildcard subscription 'mcp.>' receives the message
        // since request-reply pattern uses private _INBOX that doesn't hit wildcards
        eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Publishing monitoring copy to {}", subject);
        match client.publish(subject.to_string(), payload.into()).await {
            Ok(_) => {
                eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Successfully published monitoring copy");
            }
            Err(e) => {
                eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Warning: Failed to publish monitoring copy: {}", e);
                // Non-fatal - don't return error, monitoring is nice-to-have
            }
        }
        eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Response subject: {}", response.subject);

        // Decode response envelope
        let response_envelope: Envelope<McpData> = NatsEnvelopeCodec::decode(&response.payload)
            .map_err(|e| AppError::NatsError(format!("Failed to decode response: {}", e)))?;

        eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Response envelope decoded successfully");
        eprintln!("  - Response request_id: {:?}", response_envelope.meta.request_id);

        // Check if response indicates an error in the tool_response
        if let Some(ref tool_response) = response_envelope.payload.tool_response {
            if tool_response.is_error == Some(true) {
                // Extract error message from content
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
                eprintln!("[TaleTrail NATS] MCP tool returned error: {}", error_msg);
                // Note: We still return the complete envelope so the frontend can display the error
            }
        }

        // Return the complete envelope with all metadata
        Ok(response_envelope)
    }

    /// Send a pre-built envelope via NATS
    ///
    /// This method sends a complete Envelope<McpData> that was loaded from a template,
    /// allowing full control over metadata (request_id, tenant, tracing) and payload.
    ///
    /// # Arguments
    /// * `subject` - NATS subject to send to
    /// * `envelope` - Pre-built envelope from template
    ///
    /// # Returns
    /// * `Ok(Envelope<McpData>)` - The complete response envelope with metadata
    /// * `Err(AppError)` - If request fails
    pub async fn send_envelope(
        &self,
        subject: &str,
        envelope: qollective::envelope::Envelope<qollective::types::mcp::McpData>,
    ) -> AppResult<qollective::envelope::Envelope<qollective::types::mcp::McpData>> {
        use qollective::envelope::nats_codec::NatsEnvelopeCodec;
        use qollective::envelope::Envelope;
        use qollective::types::mcp::McpData;

        let client_guard = self.client.read().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AppError::ConnectionError("Not connected to NATS".to_string()))?;

        // Encode envelope
        let payload = NatsEnvelopeCodec::encode(&envelope)
            .map_err(|e| AppError::NatsError(format!("Failed to encode envelope: {}", e)))?;

        // Send request and wait for response
        let response = client
            .request(subject.to_string(), payload.clone().into())
            .await
            .map_err(|e| AppError::NatsError(format!("Failed to send request to {}: {}", subject, e)))?;

        // DIAGNOSTIC: Publish copy for monitoring (fire-and-forget)
        // This ensures monitoring wildcard subscription 'mcp.>' receives the message
        // since request-reply pattern uses private _INBOX that doesn't hit wildcards
        eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Publishing monitoring copy to {}", subject);
        match client.publish(subject.to_string(), payload.into()).await {
            Ok(_) => {
                eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Successfully published monitoring copy");
            }
            Err(e) => {
                eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Warning: Failed to publish monitoring copy: {}", e);
                // Non-fatal - don't return error, monitoring is nice-to-have
            }
        }

        // Flush to ensure message is sent immediately to monitoring
        if let Err(e) = client.flush().await {
            eprintln!("[TaleTrail NATS] [DIAGNOSTIC] Warning: Failed to flush after publish: {}", e);
        }

        // Decode response envelope
        let response_envelope: Envelope<McpData> = NatsEnvelopeCodec::decode(&response.payload)
            .map_err(|e| AppError::NatsError(format!("Failed to decode response: {}", e)))?;

        // Check if response indicates an error in the tool_response
        if let Some(ref tool_response) = response_envelope.payload.tool_response {
            if tool_response.is_error == Some(true) {
                let error_msg = tool_response.content
                    .iter()
                    .filter_map(|c| {
                        if let Ok(json) = serde_json::to_value(c) {
                            json.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                eprintln!("[TaleTrail NATS] MCP tool returned error: {}", error_msg);
                // Note: We still return the complete envelope so the frontend can display the error
            }
        }

        // Return the complete envelope with all metadata
        Ok(response_envelope)
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

// Implement NatsService trait for NatsClient
#[async_trait]
impl NatsService for NatsClient {
    async fn connect(&self) -> AppResult<()> {
        self.connect().await
    }

    async fn subscribe(&self, tenant_id: Option<String>) -> AppResult<()> {
        let subscriber = self.subscribe(tenant_id).await?;
        self.set_subscriber(subscriber).await;
        Ok(())
    }

    async fn publish_request(&self, request: &GenerationRequest) -> AppResult<()> {
        self.publish_request(request).await
    }

    async fn is_connected(&self) -> bool {
        self.is_connected().await
    }

    async fn disconnect(&self) -> AppResult<()> {
        self.disconnect().await
    }

    async fn unsubscribe(&self) -> AppResult<()> {
        self.unsubscribe().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_config_from_app_config() {
        use crate::config::AppConfig;

        let app_config = AppConfig::create_test_app_config();

        let config = NatsConfig::from_app_config(&app_config);
        assert_eq!(config.url, crate::constants::network::DEFAULT_NATS_URL);
        assert_eq!(config.name, Some(crate::constants::defaults::NATS_CLIENT_NAME.to_string()));
        assert_eq!(config.timeout_secs, crate::constants::network::DEFAULT_REQUEST_TIMEOUT_MS / 1000);
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
        use crate::config::AppConfig;

        let app_config = AppConfig::create_test_app_config();

        let config = NatsConfig::from_app_config(&app_config);
        let client = NatsClient::new(config);

        assert!(!client.is_connected().await);
    }

    #[test]
    fn test_mcp_tool_call_request_structure() {
        use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam};
        use serde_json::json;

        let arguments = json!({
            "generation_request": {
                "theme": "Space Adventure",
                "age_group": "9-11",
                "language": "en"
            }
        });

        let tool_call = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "orchestrate_generation".into(),
                arguments: arguments.as_object().cloned(),
            },
            extensions: Default::default(),
        };

        assert_eq!(tool_call.params.name.as_ref(), "orchestrate_generation");
        assert!(tool_call.params.arguments.is_some());
    }

    #[test]
    fn test_mcp_envelope_structure() {
        use qollective::envelope::{Envelope, Meta};
        use qollective::types::mcp::McpData;
        use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam};

        let tool_call = CallToolRequest {
            method: CallToolRequestMethod::default(),
            params: CallToolRequestParam {
                name: "test_tool".into(),
                arguments: None,
            },
            extensions: Default::default(),
        };

        let mcp_data = McpData::with_tool_call(tool_call);
        let mut meta = Meta::default();
        meta.tenant = Some("42".to_string());

        let envelope = Envelope::new(meta, mcp_data);

        assert_eq!(envelope.meta.tenant, Some("42".to_string()));
        assert!(envelope.payload.tool_call.is_some());
    }
}
