//! Integration test harness for NATS-based MCP service testing
//!
//! This harness connects to the EXISTING NATS server (must be running)
//! and provides utilities for sending MCP tool calls and receiving responses.

use async_nats::Client;
use std::time::Duration;
use std::path::Path;
use qollective::envelope::Envelope;
use qollective::types::mcp::McpData;
use rmcp::model::{CallToolRequest, CallToolRequestMethod, CallToolRequestParam, CallToolResult};
use serde_json::Value;
use uuid::Uuid;

pub struct TestHarness {
    nats_client: Client,
}

impl TestHarness {
    /// Connect to EXISTING NATS server at localhost:5222 with TLS
    ///
    /// # Errors
    /// Returns error if NATS server is not running or TLS configuration is invalid
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize rustls crypto provider (required for TLS)
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        // Load CA certificate for TLS
        let ca_cert_path = Path::new("../certs/ca.pem");
        if !ca_cert_path.exists() {
            return Err(format!("CA certificate not found at {:?}", ca_cert_path).into());
        }

        let ca_cert_pem = std::fs::read_to_string(ca_cert_path)?;
        let ca_cert = rustls_pemfile::certs(&mut ca_cert_pem.as_bytes())
            .collect::<Result<Vec<_>, _>>()?;

        let mut root_cert_store = rustls::RootCertStore::empty();
        for cert in ca_cert {
            root_cert_store.add(cert)?;
        }

        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        // Connect with TLS and test user credentials
        let nats_client = async_nats::ConnectOptions::new()
            .tls_client_config(tls_config)
            .user_and_password("test".to_string(), "test".to_string())
            .connect("tls://localhost:5222")
            .await?;

        Ok(Self { nats_client })
    }

    /// Send MCP tool call request via NATS and wait for response
    ///
    /// # Arguments
    /// * `subject` - NATS subject (e.g., "mcp.quality.validate")
    /// * `tool_name` - MCP tool name (e.g., "validate_content")
    /// * `params` - Tool parameters as JSON object
    ///
    /// # Returns
    /// Parsed response as JSON Value, or error
    pub async fn send_mcp_request(
        &self,
        subject: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<CallToolResult, Box<dyn std::error::Error>> {
        // 1. Create MCP CallToolRequest
        let arguments = if let Value::Object(map) = params {
            Some(map)
        } else {
            return Err("Parameters must be JSON object".into());
        };

        let tool_call = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: tool_name.to_string().into(),
                arguments,
            },
            extensions: Default::default(),
        };

        // 2. Wrap in McpData
        let mcp_data = McpData {
            tool_call: Some(tool_call),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        // 3. Create envelope with metadata
        let mut meta = qollective::envelope::Meta::default();
        meta.tenant = Some("test-tenant".to_string());
        meta.request_id = Some(Uuid::new_v4());
        meta.timestamp = Some(chrono::Utc::now());

        let envelope = Envelope::new(meta, mcp_data);

        // 4. Serialize and send via NATS request/reply
        let payload = serde_json::to_vec(&envelope)?;

        let response = tokio::time::timeout(
            Duration::from_secs(30),
            self.nats_client.request(subject.to_string(), payload.into())
        ).await??;

        // 5. Deserialize response envelope
        let response_envelope: Envelope<McpData> = serde_json::from_slice(&response.payload)?;

        // 6. Extract tool response
        let (_, mcp_data) = response_envelope.extract();
        let tool_response = mcp_data.tool_response
            .ok_or("No tool response in envelope")?;

        Ok(tool_response)
    }

    pub async fn cleanup(&self) {
        // Cleanup resources if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires NATS server running
    async fn test_harness_connects_to_nats() {
        let harness = TestHarness::new().await;
        assert!(harness.is_ok(), "Should connect to NATS server at localhost:5222: {:?}", harness.err());
    }
}
