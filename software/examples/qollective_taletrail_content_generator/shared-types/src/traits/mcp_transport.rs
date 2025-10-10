//! MCP transport trait for protocol-agnostic MCP communication
//!
//! Abstracts MCP communication layer to enable:
//! - Multiple transport implementations (NATS, HTTP, WebSocket)
//! - Mock-based integration testing
//! - Automatic envelope wrapping with tenant context
//!
//! # Example Usage
//!
//! ```rust,ignore
//! // Production NATS transport
//! let transport = NatsMcpTransport::new(nats_client).await?;
//! let response: PromptPackage = transport
//!     .call_mcp_tool(
//!         "mcp.prompt.generate",
//!         "generate_story_prompts",
//!         request,
//!     )
//!     .await?;
//!
//! // Testing with mock
//! let mut mock_transport = MockMcpTransport::new();
//! mock_transport
//!     .expect_call_mcp_tool::<PromptGenerationRequest, PromptPackage>()
//!     .returning(|_, _, _| Ok(mock_prompt_package()));
//! ```

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};

use crate::errors::TaleTrailError;

/// Abstracts MCP communication layer for protocol-agnostic tool calls
///
/// Implementations:
/// - `NatsMcpTransport`: Production NATS-based MCP transport with TLS
/// - `HttpMcpTransport`: Future HTTP-based MCP transport
/// - `MockMcpTransport`: Test mock for integration testing
///
/// # Envelope Wrapping
///
/// All implementations must wrap requests in `TaleTrailEnvelope` with:
/// - Tenant context propagation
/// - Request/response correlation IDs
/// - Tracing metadata
#[cfg_attr(any(test, feature = "mocking"), mockall::automock)]
#[async_trait]
pub trait McpTransport: Send + Sync + std::fmt::Debug {
    /// Call MCP tool on remote service
    ///
    /// Automatically wraps request in TaleTrailEnvelope and unwraps response.
    ///
    /// # Type Parameters
    /// * `Req` - Request payload type (must be Serialize + Send)
    /// * `Res` - Response payload type (must be DeserializeOwned)
    ///
    /// # Arguments
    /// * `subject` - NATS subject or HTTP endpoint (e.g., "mcp.story.generate")
    /// * `tool_name` - MCP tool to invoke (e.g., "generate_structure")
    /// * `request` - Request payload
    ///
    /// # Returns
    /// Deserialized response payload
    ///
    /// # Errors
    /// - `TaleTrailError::NatsError`: NATS communication failure
    /// - `TaleTrailError::TimeoutError`: Request timeout
    /// - `TaleTrailError::SerializationError`: Payload serialization failure
    async fn call_mcp_tool<Req, Res>(
        &self,
        subject: String,
        tool_name: String,
        request: Req,
    ) -> Result<Res, TaleTrailError>
    where
        Req: Serialize + Send + 'static,
        Res: DeserializeOwned + 'static;

    /// Check if MCP service is healthy and responsive
    ///
    /// # Arguments
    /// * `subject` - NATS subject or HTTP endpoint to check
    ///
    /// # Returns
    /// `true` if service responds to health check within timeout
    ///
    /// # Errors
    /// - `TaleTrailError::NetworkError`: Failed to reach service
    async fn health_check(&self, subject: String) -> Result<bool, TaleTrailError>;
}
