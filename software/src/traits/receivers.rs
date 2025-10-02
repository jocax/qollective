// ABOUTME: Receiver trait for unified server-side transport abstraction
// ABOUTME: Enables consistent envelope receiving across all transport protocols

//! Receiver trait for server-side transport operations.
//!
//! This trait provides a unified interface for receiving envelopes across
//! different transport protocols. All transport implementations should
//! implement this trait to enable consistent server-side behavior.

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Import handler traits from handlers module
use super::handlers::ContextDataHandler;

/// Unified envelope server trait for consistent server patterns.
///
/// This trait provides a standardized interface for all server implementations,
/// supporting both simple message handling and route-based message handling
/// to accommodate different transport protocol patterns.
#[async_trait]
pub trait UnifiedEnvelopeReceiver {
    /// Receive and process envelopes for simple protocols.
    ///
    /// This method is used by protocols that don't require routing
    /// (like NATS subjects or gRPC services). The handler processes
    /// all incoming messages of the specified type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The request data type
    /// * `R` - The response data type
    /// * `H` - The handler type
    ///
    /// # Arguments
    ///
    /// * `handler` - The business logic handler
    ///
    /// # Returns
    ///
    /// Returns a `Result<()>` when the server shuts down or encounters an error.
    async fn receive_envelope<T, R, H>(&mut self, handler: H) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
        H: ContextDataHandler<T, R> + Send + Sync + 'static;

    /// Receive and process envelopes at a specific route.
    ///
    /// This method is used by protocols that require routing
    /// (like REST paths or MCP method names). The handler processes
    /// messages that match the specified route.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The request data type
    /// * `R` - The response data type
    /// * `H` - The handler type
    ///
    /// # Arguments
    ///
    /// * `route` - The route pattern to match (e.g., "/api/users", "tools/list")
    /// * `handler` - The business logic handler
    ///
    /// # Returns
    ///
    /// Returns a `Result<()>` when the server shuts down or encounters an error.
    async fn receive_envelope_at<T, R, H>(&mut self, route: &str, handler: H) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
        H: ContextDataHandler<T, R> + Send + Sync + 'static;
}
