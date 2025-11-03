use async_nats::{Client, ConnectOptions, Subscriber};
use chrono::Utc;
use crate::error::{AppError, Result};
use crate::models::events::GenerationEvent;
use crate::config::NatsConfig;
use std::sync::Arc;
use std::time::Duration;
use futures::lock::Mutex;
use futures::FutureExt;

/// NATS client wrapper with connection management
/// Adapted for use with smol async executor (required by Iocraft)
pub struct NatsClient {
    config: NatsConfig,
    client: Arc<Mutex<Option<Client>>>,
}

impl NatsClient {
    /// Create a new NATS client with the given configuration
    pub fn new(config: NatsConfig) -> Self {
        Self {
            config,
            client: Arc::new(Mutex::new(None)),
        }
    }

    /// Connect to the NATS server with TLS and NKey authentication
    ///
    /// This method is compatible with smol async executor
    pub async fn connect(&self) -> Result<()> {
        let mut client_guard = self.client.lock().await;

        // If already connected, return early
        if client_guard.is_some() {
            return Ok(());
        }

        // Build connect options
        let mut opts = ConnectOptions::new().name("taletrail-cli");

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

        // Set request timeout for long-running operations
        opts = opts.request_timeout(Some(Duration::from_secs(self.config.timeout_secs)));

        // Connect to NATS
        let client = opts
            .connect(&self.config.url)
            .await
            .map_err(|e| AppError::NatsConnection(format!("Failed to connect to NATS at {}: {}", self.config.url, e)))?;

        *client_guard = Some(client);
        Ok(())
    }

    /// Disconnect from the NATS server
    pub async fn disconnect(&self) -> Result<()> {
        let mut client_guard = self.client.lock().await;

        // Close connection
        if let Some(client) = client_guard.take() {
            client
                .flush()
                .await
                .map_err(|e| AppError::NatsConnection(format!("Failed to flush NATS client: {}", e)))?;
        }

        Ok(())
    }

    /// Check if the client is connected
    pub async fn is_connected(&self) -> bool {
        self.client.lock().await.is_some()
    }

    /// Subscribe to generation events
    /// If tenant_id is provided, subscribes to tenant-specific events
    /// Otherwise, subscribes to all generation events
    pub async fn subscribe(&self, tenant_id: Option<String>) -> Result<Subscriber> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AppError::NatsConnection("Not connected to NATS".to_string()))?;

        let subject = if let Some(tenant) = tenant_id {
            super::subjects::TALETRAIL_GENERATION_EVENTS_TENANT.replace("{tenant_id}", &tenant)
        } else {
            format!("{}.*", super::subjects::TALETRAIL_GENERATION_EVENTS)
        };

        let subscriber = client
            .subscribe(subject.clone())
            .await
            .map_err(|e| AppError::NatsRequest(format!("Failed to subscribe to {}: {}", subject, e)))?;

        Ok(subscriber)
    }

    /// Subscribe to a specific subject pattern
    pub async fn subscribe_to(&self, subject: &str) -> Result<Subscriber> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AppError::NatsConnection("Not connected to NATS".to_string()))?;

        let subscriber = client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| AppError::NatsRequest(format!("Failed to subscribe to {}: {}", subject, e)))?;

        Ok(subscriber)
    }

    /// Parse a NATS message payload into a GenerationEvent
    pub fn parse_event(payload: &[u8]) -> Result<GenerationEvent> {
        serde_json::from_slice(payload)
            .map_err(|e| AppError::JsonParse(format!("Failed to parse GenerationEvent: {}", e)))
    }

    /// Publish a generation event
    pub async fn publish(&self, subject: &str, event: &GenerationEvent) -> Result<()> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AppError::NatsConnection("Not connected to NATS".to_string()))?;

        let payload = serde_json::to_vec(event)
            .map_err(|e| AppError::Serialization(e))?;

        client
            .publish(subject.to_string(), payload.into())
            .await
            .map_err(|e| AppError::NatsRequest(format!("Failed to publish to {}: {}", subject, e)))?;

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
    pub async fn send_mcp_tool_call(
        &self,
        subject: &str,
        tool_call: rmcp::model::CallToolRequest,
        tenant_id: String,
        timeout: Option<Duration>,
    ) -> Result<qollective::envelope::Envelope<qollective::types::mcp::McpData>> {
        use qollective::envelope::{Envelope, Meta, TracingMeta};
        use qollective::types::mcp::McpData;
        use qollective::envelope::nats_codec::NatsEnvelopeCodec;
        use uuid::Uuid;

        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AppError::NatsConnection("Not connected to NATS".to_string()))?;

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
        metadata.version = Some("1.0".to_string());
        metadata.tenant = Some(tenant_id.clone());
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

        // Encode envelope
        let payload = NatsEnvelopeCodec::encode(&envelope)
            .map_err(|e| AppError::NatsRequest(format!("Failed to encode envelope: {}", e)))?;

        // Use custom timeout if provided, otherwise use default from config
        let request_timeout = timeout.unwrap_or_else(|| Duration::from_secs(self.config.timeout_secs));

        eprintln!("[NATS Client] Sending MCP tool request:");
        eprintln!("  - subject: {}", subject);
        eprintln!("  - timeout: {:?}", request_timeout);
        eprintln!("  - payload size: {} bytes", payload.len());

        // Send request and wait for response with timeout
        // Use smol::future::or for timeout (smol doesn't have timeout combinator)
        let request_future = client.request(subject.to_string(), payload.clone().into());
        let timeout_future = smol::Timer::after(request_timeout);

        let response = futures::select! {
            result = request_future.fuse() => {
                result.map_err(|e| {
                    eprintln!("[NATS Client] Request failed: {}", e);
                    AppError::NatsRequest(format!("Failed to send request to {}: {}", subject, e))
                })?
            }
            _ = timeout_future.fuse() => {
                eprintln!("[NATS Client] Request timed out after {:?}", request_timeout);
                return Err(AppError::NatsRequest(format!("Request to {} timed out after {:?}", subject, request_timeout)));
            }
        };

        eprintln!("[NATS Client] Response received: {} bytes", response.payload.len());

        // Decode response envelope
        let response_envelope: Envelope<McpData> = NatsEnvelopeCodec::decode(&response.payload)
            .map_err(|e| AppError::NatsRequest(format!("Failed to decode response: {}", e)))?;

        eprintln!("[NATS Client] Response envelope decoded successfully");

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
                eprintln!("[NATS Client] MCP tool returned error: {}", error_msg);
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
    ) -> Result<qollective::envelope::Envelope<qollective::types::mcp::McpData>> {
        use qollective::envelope::nats_codec::NatsEnvelopeCodec;
        use qollective::envelope::Envelope;
        use qollective::types::mcp::McpData;

        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| AppError::NatsConnection("Not connected to NATS".to_string()))?;

        // Encode envelope
        let payload = NatsEnvelopeCodec::encode(&envelope)
            .map_err(|e| AppError::NatsRequest(format!("Failed to encode envelope: {}", e)))?;

        // Send request and wait for response
        let response = client
            .request(subject.to_string(), payload.clone().into())
            .await
            .map_err(|e| AppError::NatsRequest(format!("Failed to send request to {}: {}", subject, e)))?;

        // Decode response envelope
        let response_envelope: Envelope<McpData> = NatsEnvelopeCodec::decode(&response.payload)
            .map_err(|e| AppError::NatsRequest(format!("Failed to decode response: {}", e)))?;

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
                eprintln!("[NATS Client] MCP tool returned error: {}", error_msg);
            }
        }

        // Return the complete envelope with all metadata
        Ok(response_envelope)
    }

    /// Get a cloned client handle for sharing across tasks
    pub fn clone_handle(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: Arc::clone(&self.client),
        }
    }
}

impl Clone for NatsClient {
    fn clone(&self) -> Self {
        self.clone_handle()
    }
}
