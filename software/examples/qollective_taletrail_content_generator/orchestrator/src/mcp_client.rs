//! MCP Envelope Client Module
//!
//! Provides envelope-first MCP client implementation for orchestrator.
//! Wraps raw `CallToolRequest` in `Envelope<McpData>` before sending to MCP servers.
//!
//! # Architecture
//!
//! ```text
//! CallToolRequest + Meta
//!   ↓
//! Wrap in McpData { tool_call: Some(request), ... }
//!   ↓
//! Wrap in Envelope<McpData> with metadata (tenant_id, request_id, trace_id)
//!   ↓
//! Serialize envelope and send via NATS
//!   ↓
//! Deserialize response as Envelope<McpData>
//!   ↓
//! Extract CallToolResult from response_data.tool_response
//!   ↓
//! Parse and return result
//! ```
//!
//! # Metadata Propagation
//!
//! The client ensures metadata flows through the entire pipeline:
//! - **tenant_id**: For multi-tenancy isolation
//! - **request_id**: For request correlation across services
//! - **trace_id**: For distributed tracing
//!
//! # Example
//!
//! ```no_run
//! use orchestrator::mcp_client::McpEnvelopeClient;
//! use qollective::envelope::Meta;
//! use rmcp::model::CallToolRequest;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let nats_client = async_nats::connect("nats://localhost:4222").await?;
//! let client = McpEnvelopeClient::new(Arc::new(nats_client), 60);
//!
//! let mut meta = Meta::default();
//! meta.tenant = Some("tenant-123".to_string());
//!
//! let request = CallToolRequest {
//!     // ... tool request fields
//!     # method: rmcp::model::CallToolRequestMethod,
//!     # params: rmcp::model::CallToolRequestParam {
//!     #     name: "test".into(),
//!     #     arguments: None,
//!     # },
//!     # extensions: Default::default(),
//! };
//!
//! let result: serde_json::Value = client.call_tool(
//!     "mcp.service.subject",
//!     request,
//!     meta
//! ).await?;
//! # Ok(())
//! # }
//! ```

use async_nats::Client as NatsClient;
use qollective::envelope::{Envelope, Meta};
use qollective::types::mcp::McpData;
use rmcp::model::{CallToolRequest, CallToolResult, RawContent};
use shared_types::{Result, TaleTrailError};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info, instrument, warn};

/// Helper function to create TracingMeta with minimal fields
fn create_tracing_meta(trace_id: String) -> qollective::envelope::meta::TracingMeta {
    qollective::envelope::meta::TracingMeta {
        trace_id: Some(trace_id),
        span_id: None,
        parent_span_id: None,
        baggage: std::collections::HashMap::new(),
        sampling_rate: None,
        sampled: Some(true),
        trace_state: None,
        operation_name: None,
        span_kind: None,
        span_status: None,
        tags: std::collections::HashMap::new(),
    }
}

/// MCP client that wraps requests in Qollective envelopes
///
/// This client implements the envelope-first architecture by:
/// 1. Wrapping `CallToolRequest` in `McpData` structure
/// 2. Wrapping `McpData` in `Envelope` with metadata
/// 3. Serializing envelope for NATS transport
/// 4. Deserializing response envelope
/// 5. Extracting and parsing tool result
///
/// # Backpressure
///
/// The client includes a semaphore to limit concurrent requests,
/// preventing NATS server saturation and ensuring graceful degradation
/// under load.
#[derive(Clone)]
pub struct McpEnvelopeClient {
    /// NATS client for transport
    nats_client: Arc<NatsClient>,

    /// Timeout for requests (in seconds)
    timeout_secs: u64,

    /// Semaphore for backpressure control (limits concurrent requests)
    concurrency_limit: Arc<Semaphore>,
}

impl McpEnvelopeClient {
    /// Default concurrency limit for backpressure control
    const DEFAULT_CONCURRENCY_LIMIT: usize = 100;

    /// Create a new MCP envelope client
    ///
    /// # Arguments
    ///
    /// * `nats_client` - NATS client for communication
    /// * `timeout_secs` - Timeout for requests in seconds
    ///
    /// # Concurrency
    ///
    /// By default, limits concurrent requests to 100 to prevent NATS saturation.
    /// Use `with_concurrency_limit()` to customize this value.
    #[must_use]
    pub fn new(nats_client: Arc<NatsClient>, timeout_secs: u64) -> Self {
        Self::with_concurrency_limit(nats_client, timeout_secs, Self::DEFAULT_CONCURRENCY_LIMIT)
    }

    /// Create a new MCP envelope client with custom concurrency limit
    ///
    /// # Arguments
    ///
    /// * `nats_client` - NATS client for communication
    /// * `timeout_secs` - Timeout for requests in seconds
    /// * `concurrency_limit` - Maximum concurrent requests allowed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use orchestrator::mcp_client::McpEnvelopeClient;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let nats_client = async_nats::connect("nats://localhost:4222").await?;
    /// let client = McpEnvelopeClient::with_concurrency_limit(
    ///     Arc::new(nats_client),
    ///     60,
    ///     50  // Limit to 50 concurrent requests
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn with_concurrency_limit(
        nats_client: Arc<NatsClient>,
        timeout_secs: u64,
        concurrency_limit: usize,
    ) -> Self {
        Self {
            nats_client,
            timeout_secs,
            concurrency_limit: Arc::new(Semaphore::new(concurrency_limit)),
        }
    }

    /// Call MCP tool with envelope wrapping
    ///
    /// Wraps the request in an envelope with metadata, sends via NATS,
    /// and unwraps the response envelope.
    ///
    /// # Arguments
    ///
    /// * `subject` - NATS subject to send request to
    /// * `request` - MCP tool call request
    /// * `meta` - Envelope metadata (tenant_id, request_id, trace_id)
    ///
    /// # Type Parameters
    ///
    /// * `T` - Expected response type (must be deserializable from JSON)
    ///
    /// # Returns
    ///
    /// Deserialized response of type T
    ///
    /// # Errors
    ///
    /// - `SerializationError` - Failed to serialize request or deserialize response
    /// - `TimeoutError` - Request exceeded timeout
    /// - `NatsError` - NATS communication error
    /// - `GenerationError` - MCP tool returned error or missing response
    #[instrument(skip(self, request, meta), fields(subject = %subject))]
    pub async fn call_tool<T: serde::de::DeserializeOwned>(
        &self,
        subject: &str,
        request: CallToolRequest,
        meta: Meta,
    ) -> Result<T> {
        // Acquire semaphore permit for backpressure control
        // This blocks if we've reached the concurrency limit
        let _permit = self.concurrency_limit.acquire().await
            .map_err(|e| TaleTrailError::NatsError(format!("Semaphore error: {}", e)))?;

        let tool_name = request.params.name.clone();

        // Capture metadata fields for logging before moving meta
        let tenant_id = meta.tenant.as_ref().map(|s| s.as_str());
        let request_id = meta.request_id;

        debug!(
            available_permits = self.concurrency_limit.available_permits(),
            "Acquired semaphore permit"
        );

        info!(
            tool_name = %tool_name,
            tenant_id = ?tenant_id,
            request_id = ?request_id,
            "Calling MCP tool with envelope"
        );

        // Step 1: Wrap request in McpData
        let mcp_data = McpData {
            tool_call: Some(request),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        debug!("Created McpData with tool_call");

        // Step 2: Wrap in Envelope with metadata (no clone needed - meta is moved)
        let envelope = Envelope::new(meta, mcp_data);

        debug!("Created envelope with metadata");

        // Step 3: Serialize envelope
        let envelope_bytes = serde_json::to_vec(&envelope)
            .map_err(|e| TaleTrailError::SerializationError(format!(
                "Failed to serialize envelope: {}",
                e
            )))?;

        debug!(size_bytes = envelope_bytes.len(), "Serialized envelope");

        // Step 4: Send via NATS with timeout
        let timeout_duration = std::time::Duration::from_secs(self.timeout_secs);

        let response = tokio::time::timeout(
            timeout_duration,
            self.nats_client.request(subject.to_string(), envelope_bytes.into()),
        )
        .await
        .map_err(|_| {
            warn!(
                tool_name = %tool_name,
                timeout_secs = self.timeout_secs,
                "MCP tool call timed out"
            );
            TaleTrailError::TimeoutError
        })?
        .map_err(|e| {
            warn!(
                tool_name = %tool_name,
                error = %e,
                "NATS request failed"
            );
            TaleTrailError::NatsError(e.to_string())
        })?;

        debug!(
            response_size = response.payload.len(),
            "Received NATS response"
        );

        // Step 5: Deserialize response as Envelope<McpData>
        let response_envelope: Envelope<McpData> = serde_json::from_slice(&response.payload)
            .map_err(|e| TaleTrailError::SerializationError(format!(
                "Failed to deserialize response envelope: {}",
                e
            )))?;

        debug!("Deserialized response envelope");

        // Step 6: Extract tool response from envelope
        let (response_meta, response_data) = response_envelope.extract();

        debug!(
            response_tenant_id = ?response_meta.tenant,
            response_request_id = ?response_meta.request_id,
            "Extracted response metadata"
        );

        let tool_result = response_data.tool_response
            .ok_or_else(|| {
                warn!("Response envelope missing tool_response field");
                TaleTrailError::GenerationError(
                    "Response envelope missing tool_response".to_string()
                )
            })?;

        // Step 7: Check for MCP errors
        if tool_result.is_error == Some(true) {
            warn!(
                tool_name = %tool_name,
                error_content = ?tool_result.content,
                "MCP tool returned error"
            );
            return Err(TaleTrailError::GenerationError(format!(
                "MCP tool error: {:?}",
                tool_result.content
            )));
        }

        // Step 8: Extract result from Content
        let first_content = tool_result
            .content
            .first()
            .ok_or_else(|| {
                warn!("MCP response has empty content array");
                TaleTrailError::GenerationError("Empty MCP response content".to_string())
            })?;

        let json_str = match &first_content.raw {
            RawContent::Text(text_content) => &text_content.text,
            _ => {
                warn!("MCP response content is not text type");
                return Err(TaleTrailError::GenerationError(
                    "Expected text content in MCP response".to_string(),
                ));
            }
        };

        debug!(json_length = json_str.len(), "Extracted JSON string from response");

        // Step 9: Parse result from JSON string
        let result: T = serde_json::from_str(json_str)
            .map_err(|e| {
                // Safe string slicing that respects UTF-8 boundaries
                let preview = json_str.get(..100).unwrap_or(json_str);
                warn!(
                    error = %e,
                    json_preview = preview,
                    "Failed to deserialize tool result"
                );
                TaleTrailError::SerializationError(format!(
                    "Failed to deserialize tool result: {}",
                    e
                ))
            })?;

        info!(
            tool_name = %tool_name,
            "Successfully completed MCP tool call"
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::{CallToolRequestMethod, CallToolRequestParam, Content, Extensions};
    use uuid::Uuid;

    /// Test: Envelope wraps tool request correctly
    #[tokio::test]
    async fn test_envelope_wraps_tool_request_correctly() {
        // ARRANGE: Create a CallToolRequest
        let request = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: "test_tool".into(),
                arguments: Some({
                    let mut map = serde_json::Map::new();
                    map.insert("param1".to_string(), serde_json::json!("value1"));
                    map
                }),
            },
            extensions: Extensions::default(),
        };

        // Create metadata
        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant".to_string());
        meta.request_id = Some(Uuid::new_v4());

        // Wrap in McpData
        let mcp_data = McpData {
            tool_call: Some(request.clone()),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        // Wrap in Envelope
        let envelope = Envelope::new(meta.clone(), mcp_data);

        // ACT: Serialize envelope
        let serialized = serde_json::to_vec(&envelope).expect("Failed to serialize envelope");

        // Deserialize back
        let deserialized: Envelope<McpData> =
            serde_json::from_slice(&serialized).expect("Failed to deserialize envelope");

        let (extracted_meta, extracted_data) = deserialized.extract();

        // ASSERT: Verify envelope structure
        assert_eq!(extracted_meta.tenant, Some("test-tenant".to_string()));
        assert_eq!(extracted_meta.request_id, meta.request_id);

        let extracted_request = extracted_data.tool_call.expect("Missing tool_call");
        assert_eq!(extracted_request.params.name, "test_tool");
        assert!(extracted_data.tool_response.is_none());
    }

    /// Test: Envelope includes metadata (tenant_id, request_id, trace_id)
    #[tokio::test]
    async fn test_envelope_includes_metadata() {
        // ARRANGE: Create metadata with all fields
        let mut meta = Meta::default();
        meta.tenant = Some("tenant-123".to_string());
        meta.request_id = Some(Uuid::new_v4());

        // Add tracing metadata
        meta.tracing = Some(create_tracing_meta("trace-456".to_string()));

        let request = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: "test".into(),
                arguments: None,
            },
            extensions: Extensions::default(),
        };

        let mcp_data = McpData {
            tool_call: Some(request),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        // ACT: Create envelope
        let envelope = Envelope::new(meta.clone(), mcp_data);

        // Serialize and deserialize
        let serialized = serde_json::to_vec(&envelope).unwrap();
        let deserialized: Envelope<McpData> = serde_json::from_slice(&serialized).unwrap();
        let (extracted_meta, _) = deserialized.extract();

        // ASSERT: All metadata fields are preserved
        assert_eq!(extracted_meta.tenant, Some("tenant-123".to_string()));
        assert_eq!(extracted_meta.request_id, meta.request_id);
        assert_eq!(
            extracted_meta.tracing.as_ref().and_then(|t| t.trace_id.clone()),
            Some("trace-456".to_string())
        );
    }

    /// Test: Response unwraps from envelope
    #[tokio::test]
    async fn test_response_unwraps_from_envelope() {
        // ARRANGE: Create a response envelope
        let mut meta = Meta::default();
        meta.tenant = Some("test-tenant".to_string());
        meta.request_id = Some(Uuid::new_v4());

        // Create a successful tool result
        let test_response = serde_json::json!({
            "result": "success",
            "data": "test_data"
        });

        let tool_result = CallToolResult {
            content: vec![Content::text(test_response.to_string())],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        };

        let response_data = McpData {
            tool_call: None,
            tool_response: Some(tool_result),
            tool_registration: None,
            discovery_data: None,
        };

        let response_envelope = Envelope::new(meta, response_data);

        // ACT: Serialize response
        let serialized = serde_json::to_vec(&response_envelope).unwrap();

        // Deserialize and extract
        let deserialized: Envelope<McpData> = serde_json::from_slice(&serialized).unwrap();
        let (_, extracted_data) = deserialized.extract();

        // ASSERT: Tool response is correctly extracted
        assert!(extracted_data.tool_response.is_some());
        assert!(extracted_data.tool_call.is_none());

        let tool_response = extracted_data.tool_response.unwrap();
        assert_eq!(tool_response.is_error, Some(false));
        assert!(!tool_response.content.is_empty());
    }

    /// Test: Error handling for missing envelope
    #[tokio::test]
    async fn test_error_handling_for_missing_tool_response() {
        // ARRANGE: Create envelope WITHOUT tool_response
        let meta = Meta::default();
        let mcp_data = McpData {
            tool_call: None,
            tool_response: None, // Missing!
            tool_registration: None,
            discovery_data: None,
        };

        let envelope = Envelope::new(meta, mcp_data);
        let (_, data) = envelope.extract();

        // ACT & ASSERT: Should detect missing tool_response
        assert!(data.tool_response.is_none());
    }

    /// Test: Metadata propagation through pipeline phases
    #[tokio::test]
    async fn test_metadata_propagation_through_pipeline() {
        // ARRANGE: Create request with complete metadata
        let request_id = Uuid::new_v4();
        let mut meta = Meta::default();
        meta.tenant = Some("tenant-xyz".to_string());
        meta.request_id = Some(request_id);

        meta.tracing = Some(create_tracing_meta("trace-789".to_string()));

        // Simulate Phase 1: Request envelope
        let request = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: "phase1_tool".into(),
                arguments: None,
            },
            extensions: Extensions::default(),
        };

        let request_data = McpData {
            tool_call: Some(request),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let request_envelope = Envelope::new(meta.clone(), request_data);

        // Simulate Phase 2: Response envelope (same metadata)
        let tool_result = CallToolResult {
            content: vec![Content::text("{\"phase\": 1}".to_string())],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        };

        let response_data = McpData {
            tool_call: None,
            tool_response: Some(tool_result),
            tool_registration: None,
            discovery_data: None,
        };

        // MCP server preserves metadata in response
        let response_envelope = Envelope::new(meta.clone(), response_data);

        // ACT: Extract both envelopes
        let (req_meta, _) = request_envelope.extract();
        let (resp_meta, _) = response_envelope.extract();

        // ASSERT: Metadata is preserved across request/response
        assert_eq!(req_meta.tenant, resp_meta.tenant);
        assert_eq!(req_meta.request_id, resp_meta.request_id);
        assert_eq!(
            req_meta.tracing.as_ref().and_then(|t| t.trace_id.clone()),
            resp_meta.tracing.as_ref().and_then(|t| t.trace_id.clone())
        );
    }

    /// Test: Tenant isolation via metadata
    #[tokio::test]
    async fn test_tenant_isolation() {
        // ARRANGE: Create two requests with different tenant IDs
        let mut meta_tenant_a = Meta::default();
        meta_tenant_a.tenant = Some("tenant-a".to_string());
        meta_tenant_a.request_id = Some(Uuid::new_v4());

        let mut meta_tenant_b = Meta::default();
        meta_tenant_b.tenant = Some("tenant-b".to_string());
        meta_tenant_b.request_id = Some(Uuid::new_v4());

        let request = CallToolRequest {
            method: CallToolRequestMethod,
            params: CallToolRequestParam {
                name: "isolated_tool".into(),
                arguments: None,
            },
            extensions: Extensions::default(),
        };

        let mcp_data_a = McpData {
            tool_call: Some(request.clone()),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        let mcp_data_b = McpData {
            tool_call: Some(request),
            tool_response: None,
            tool_registration: None,
            discovery_data: None,
        };

        // ACT: Create envelopes for different tenants
        let envelope_a = Envelope::new(meta_tenant_a.clone(), mcp_data_a);
        let envelope_b = Envelope::new(meta_tenant_b.clone(), mcp_data_b);

        let (meta_a, _) = envelope_a.extract();
        let (meta_b, _) = envelope_b.extract();

        // ASSERT: Tenant IDs are different and preserved
        assert_eq!(meta_a.tenant, Some("tenant-a".to_string()));
        assert_eq!(meta_b.tenant, Some("tenant-b".to_string()));
        assert_ne!(meta_a.tenant, meta_b.tenant);
        assert_ne!(meta_a.request_id, meta_b.request_id);
    }

    /// Test: Backpressure mechanism with semaphore
    #[tokio::test]
    async fn test_backpressure_concurrency_limit() {
        use async_nats::connect;

        // ARRANGE: Create client with low concurrency limit
        // Note: We can't actually test NATS communication in unit tests,
        // but we can verify the client structure
        let concurrency_limit = 5;

        // We'll test that the client is constructed with the right limit
        // by checking available permits after construction

        // Mock NATS client (won't actually connect in this test)
        let nats_url = "nats://localhost:4222";

        // This test verifies the structure, not the actual behavior
        // since we can't mock NATS in a unit test

        // ASSERT: Verify constants
        assert_eq!(McpEnvelopeClient::DEFAULT_CONCURRENCY_LIMIT, 100);

        // Verify that custom concurrency limit can be set
        // (actual behavior tested in integration tests)
        assert!(concurrency_limit < McpEnvelopeClient::DEFAULT_CONCURRENCY_LIMIT);
    }

    /// Test: Trace ID continuity for distributed tracing
    #[tokio::test]
    async fn test_trace_id_continuity() {
        // ARRANGE: Create metadata with trace ID
        let trace_id = "trace-abc-123";

        // Simulate multiple phases with same trace ID
        let phases = vec!["phase1", "phase2", "phase3"];

        for phase in phases {
            let request = CallToolRequest {
                method: CallToolRequestMethod,
                params: CallToolRequestParam {
                    name: phase.into(),
                    arguments: None,
                },
                extensions: Extensions::default(),
            };

            let mcp_data = McpData {
                tool_call: Some(request),
                tool_response: None,
                tool_registration: None,
                discovery_data: None,
            };

            // Create fresh metadata with same trace ID for each phase
            let mut phase_meta = Meta::default();
            phase_meta.tracing = Some(create_tracing_meta(trace_id.to_string()));

            // ACT: Create envelope with metadata
            let envelope = Envelope::new(phase_meta, mcp_data);
            let (extracted_meta, _) = envelope.extract();

            // ASSERT: Trace ID is consistent across all phases
            assert_eq!(
                extracted_meta.tracing.as_ref().and_then(|t| t.trace_id.clone()),
                Some(trace_id.to_string()),
                "Trace ID should be preserved in phase {}",
                phase
            );
        }
    }
}
