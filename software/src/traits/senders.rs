// ABOUTME: Sender trait for unified client-side transport abstraction
// ABOUTME: Enables consistent envelope sending across all transport protocols

//! Sender trait for client-side transport operations.
//!
//! This trait provides a unified interface for sending envelopes across
//! different transport protocols. All transport implementations should
//! implement this trait to enable consistent client-side behavior.

use crate::envelope::Envelope;
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Universal sender trait for transport implementations.
///
/// This trait defines the core interface for sending envelopes through
/// various transport protocols. All transport clients implement this
/// trait to provide a consistent API for envelope transmission.
///
/// # Type Parameters
///
/// * `T` - The request data type contained within the envelope
/// * `R` - The response data type contained within the response envelope
///
/// # Examples
///
/// ```rust
/// use qollective::prelude::{UnifiedEnvelopeSender, Envelope, Meta};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct RequestData {
///     message: String,
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct ResponseData {
///     result: String,
/// }
///
/// async fn send_message<S: UnifiedEnvelopeSender<RequestData, ResponseData>>(
///     sender: &S,
///     endpoint: &str,
///     data: RequestData
/// ) -> qollective::error::Result<ResponseData> {
///     // Create envelope with metadata and data (envelope-first approach)
///     let request_envelope = Envelope::new(Meta::default(), data);
///     
///     // Send envelope and receive envelope response
///     let response_envelope = sender.send_envelope(endpoint, request_envelope).await?;
///     
///     // Extract response data from envelope
///     let (_, response_data) = response_envelope.extract();
///     Ok(response_data)
/// }
/// ```
#[async_trait]
pub trait UnifiedEnvelopeSender<T, R>: Send + Sync
where
    T: Serialize + Send + 'static,
    R: for<'de> Deserialize<'de> + Send + 'static,
{
    /// Send an envelope to the specified endpoint.
    ///
    /// This method handles the complete request/response cycle including:
    /// - Transport-specific protocol handling
    /// - Envelope serialization and deserialization
    /// - Metadata preservation and context propagation
    /// - Error handling and retries
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The target endpoint (format depends on transport)
    /// * `envelope` - The request envelope containing metadata and data
    ///
    /// # Returns
    ///
    /// Returns a `Result<Envelope<R>>` containing either the response envelope
    /// with metadata and data or an error if the operation failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The endpoint is unreachable or invalid
    /// - Envelope serialization/deserialization fails
    /// - Transport-specific errors occur
    /// - Network timeouts or connectivity issues
    async fn send_envelope(&self, endpoint: &str, envelope: Envelope<T>) -> Result<Envelope<R>>;
}

/// Universal sender trait for raw payload transport implementations.
///
/// This trait defines the core interface for sending raw payloads through
/// various transport protocols without envelope wrapping. This is used for
/// native protocol compatibility with external systems that don't understand
/// Qollective envelopes.
///
/// # Type Parameters
///
/// * `T` - The request data type (raw payload)
/// * `R` - The response data type (raw payload)
///
/// # Examples
///
/// ```rust
/// use qollective::prelude::UnifiedSender;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct RequestData {
///     message: String,
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct ResponseData {
///     result: String,
/// }
///
/// async fn send_raw_message<S: UnifiedSender<RequestData, ResponseData>>(
///     sender: &S,
///     endpoint: &str,
///     data: RequestData
/// ) -> qollective::error::Result<ResponseData> {
///     // Send raw payload directly (no envelope wrapping)
///     let response_data = sender.send(endpoint, data).await?;
///     Ok(response_data)
/// }
/// ```
#[async_trait]
pub trait UnifiedSender<T, R>: Send + Sync
where
    T: Serialize + Send + 'static,
    R: for<'de> Deserialize<'de> + Send + 'static,
{
    /// Send a raw payload to the specified endpoint.
    ///
    /// This method handles direct payload communication without envelope wrapping:
    /// - Transport-specific protocol handling
    /// - Raw payload serialization and deserialization
    /// - Native protocol compatibility
    /// - Error handling and retries
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The target endpoint (format depends on transport)
    /// * `payload` - The request payload (raw data, no envelope)
    ///
    /// # Returns
    ///
    /// Returns a `Result<R>` containing either the response payload
    /// or an error if the operation failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The endpoint is unreachable or invalid
    /// - Payload serialization/deserialization fails
    /// - Transport-specific errors occur
    /// - Network timeouts or connectivity issues
    async fn send(&self, endpoint: &str, payload: T) -> Result<R>;
}
