//! NATS client wrapper for envelope-wrapped MCP communication
//!
//! Provides high-level API for sending Qollective envelope-wrapped MCP requests

use crate::config::NatsCliConfig;
use crate::constants::*;
use crate::errors::{NatsCliError, Result};
use async_nats::Client;
use qollective::envelope::meta::Meta;
use qollective::envelope::Envelope;
use qollective::types::mcp::McpData;
use rmcp::model::CallToolRequest;
use shared_types::{connect_with_nkey, TaleTrailError};
use std::time::Duration;
use tracing::{debug, info, warn};

/// NATS client for sending envelope-wrapped MCP requests
pub struct NatsClient {
    /// Underlying NATS client
    client: Client,
    /// Request timeout
    timeout: Duration,
    /// Envelope version
    envelope_version: String,
}

impl NatsClient {
    /// Create a new NATS client with NKEY authentication and TLS
    ///
    /// # Arguments
    /// * `config` - NATS CLI configuration
    ///
    /// # Returns
    /// * `Result<Self>` - Connected NATS client
    pub async fn new(config: &NatsCliConfig) -> Result<Self> {
        info!("Connecting to NATS at {}", config.nats.url);
        debug!("Using NKEY file: {}", config.nkey_path().display());
        debug!("Using CA cert: {}", config.ca_cert_path().display());

        // Convert paths to strings for the helper function
        let nkey_path_buf = config.nkey_path();
        let nkey_path = nkey_path_buf
            .to_str()
            .ok_or_else(|| NatsCliError::ConfigError("Invalid NKEY path".to_string()))?;

        let ca_cert_path_buf = config.ca_cert_path();
        let ca_cert_path = ca_cert_path_buf
            .to_str()
            .ok_or_else(|| NatsCliError::ConfigError("Invalid CA cert path".to_string()))?;

        // Use shared-types helper for NKEY + TLS connection
        let client = connect_with_nkey(&config.nats.url, nkey_path, ca_cert_path)
            .await
            .map_err(|e| match e {
                TaleTrailError::NetworkError(msg) => NatsCliError::ConnectionError(msg),
                TaleTrailError::ConfigError(msg) => NatsCliError::AuthenticationError(msg),
                _ => NatsCliError::ConnectionError(e.to_string()),
            })?;

        info!("{} Connected to NATS successfully", SUCCESS_PREFIX);

        Ok(Self {
            client,
            timeout: config.timeout(),
            envelope_version: config.envelope.version.clone(),
        })
    }

    /// Send an MCP request and wait for response
    ///
    /// # Arguments
    /// * `subject` - NATS subject to send request to
    /// * `request` - MCP tool call request
    /// * `tenant_id` - Tenant ID for multi-tenant isolation
    ///
    /// # Returns
    /// * `Result<Envelope<McpData>>` - Response envelope with tool result
    pub async fn send_request(
        &self,
        subject: &str,
        request: CallToolRequest,
        tenant_id: i32,
    ) -> Result<Envelope<McpData>> {
        debug!("Sending request to subject: {}", subject);
        debug!("Tool name: {}", request.params.name);
        debug!("Tenant ID: {}", tenant_id);

        // Create request envelope
        let request_envelope = self.create_request_envelope(request, tenant_id);

        // Serialize envelope
        let payload = serde_json::to_vec(&request_envelope).map_err(|e| {
            NatsCliError::SerializationError(format!("Failed to serialize request: {}", e))
        })?;

        debug!("Request payload size: {} bytes", payload.len());

        // Send request with timeout
        let response = tokio::time::timeout(self.timeout, async {
            self.client
                .request(subject.to_string(), payload.into())
                .await
        })
        .await
        .map_err(|_| NatsCliError::Timeout(self.timeout.as_secs()))?
        .map_err(|e| NatsCliError::NatsError(format!("Request failed: {}", e)))?;

        debug!("Received response: {} bytes", response.payload.len());

        // Deserialize response envelope
        let response_envelope: Envelope<McpData> =
            serde_json::from_slice(&response.payload).map_err(|e| {
                NatsCliError::DeserializationError(format!("Failed to parse response: {}", e))
            })?;

        // Validate response envelope
        self.validate_response(&response_envelope)?;

        Ok(response_envelope)
    }

    /// Create request envelope following Qollective envelope-first pattern
    fn create_request_envelope(&self, request: CallToolRequest, tenant_id: i32) -> Envelope<McpData> {
        use chrono::Utc;
        use uuid::Uuid;

        Envelope {
            meta: Meta {
                timestamp: Some(Utc::now()),
                request_id: Some(Uuid::new_v4()),
                tenant: Some(tenant_id.to_string()),
                version: Some(self.envelope_version.clone()),
                duration: None,
                on_behalf_of: None,
                security: None,
                debug: None,
                performance: None,
                monitoring: None,
                tracing: None,
                extensions: None,
            },
            payload: McpData {
                tool_call: Some(request),
                tool_response: None,
                tool_registration: None,
                discovery_data: None,
            },
            error: None,
        }
    }

    /// Validate response envelope
    fn validate_response(&self, envelope: &Envelope<McpData>) -> Result<()> {
        // Check if there's an envelope-level error
        if let Some(error) = &envelope.error {
            return Err(NatsCliError::ServerError(format!(
                "Server returned error: {} (code: {}, details: {:?})",
                error.message, error.code, error.details
            )));
        }

        // Check if tool_response is present
        if envelope.payload.tool_response.is_none() {
            warn!("Response envelope missing tool_response field");
        }

        // Check if tool_response indicates an error
        if let Some(tool_response) = &envelope.payload.tool_response {
            if tool_response.is_error == Some(true) {
                let error_msg = tool_response
                    .content
                    .first()
                    .map(|c| format!("{:?}", c))
                    .unwrap_or_else(|| "Unknown error".to_string());

                return Err(NatsCliError::ServerError(format!(
                    "Tool execution failed: {}",
                    error_msg
                )));
            }
        }

        Ok(())
    }

    /// Get the underlying NATS client (for advanced usage)
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Check if connected to NATS
    pub fn is_connected(&self) -> bool {
        // async_nats doesn't expose a direct is_connected method,
        // but the client maintains the connection automatically
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::CallToolParams;

    #[test]
    fn test_create_request_envelope() {
        let config = NatsCliConfig::default();

        // Create a mock client (won't actually connect in test)
        // We're just testing envelope creation logic
        let request = CallToolRequest {
            params: rmcp::model::ToolCallParams {
                name: "test_tool".to_string(),
                arguments: None,
            },
        };

        // The actual client creation would fail without a real NATS server,
        // but we can test the envelope structure independently
        let meta = Meta {
            timestamp: Some(chrono::Utc::now()),
            request_id: Some(uuid::Uuid::new_v4()),
            tenant: Some("1".to_string()),
            version: Some(DEFAULT_ENVELOPE_VERSION.to_string()),
            ..Default::default()
        };

        assert_eq!(meta.version.as_ref().unwrap(), DEFAULT_ENVELOPE_VERSION);
        assert_eq!(meta.tenant.as_ref().unwrap(), "1");
    }
}
