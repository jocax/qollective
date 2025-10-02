// ABOUTME: Handler trait definitions for framework-internal processing
// ABOUTME: Provides envelope and context-data handler traits for transport abstraction

//! Handler trait definitions for framework-internal processing.
//!
//! This module provides the core handler traits used internally by the
//! framework for processing envelopes and context-data conversions.
//! These traits bridge between transport layers and user business logic.

use crate::envelope::{Context, Envelope, Meta};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{future::Future, sync::Arc};

/// Handler trait for processing envelope messages across all transport protocols.
///
/// This trait is used internally by the framework for envelope-to-envelope
/// processing. It receives a complete envelope (with metadata and context)
/// and returns a response envelope, preserving all envelope semantics.
///
/// # Type Parameters
///
/// * `T` - The request data type within the envelope
/// * `R` - The response data type within the envelope
pub trait EnvelopeHandler<T, R>: Send + Sync
where
    T: for<'de> serde::Deserialize<'de> + Send,
    R: serde::Serialize + Send,
{
    /// Handle an envelope and return a response envelope.
    ///
    /// This method processes a complete envelope, including its metadata
    /// and context, and returns a complete response envelope. The framework
    /// uses this for internal envelope processing before extracting data
    /// for user business logic handlers.
    ///
    /// # Arguments
    ///
    /// * `envelope` - The complete request envelope with metadata
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to a `Result<Envelope<R>>` containing
    /// either the response envelope or an error if processing failed.
    fn handle(&self, envelope: Envelope<T>) -> impl Future<Output = Result<Envelope<R>>> + Send;
}

/// Context and data handler trait for framework-internal processing.
///
/// This trait is used internally by the framework to process context and data
/// extracted from envelopes. It bridges between envelope handling and user
/// business logic handlers.
///
/// # Type Parameters
///
/// * `T` - The request data type
/// * `R` - The response data type
#[async_trait]
pub trait ContextDataHandler<T, R>: Send + Sync
where
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + Send + 'static,
{
    /// Handle context and data extracted from an envelope.
    ///
    /// This method processes the context and data extracted from an
    /// envelope by the framework and returns response data that will
    /// be wrapped back into an envelope.
    ///
    /// # Arguments
    ///
    /// * `context` - Optional context information from the envelope
    /// * `data` - The request data extracted from the envelope
    ///
    /// # Returns
    ///
    /// Returns a `Result<R>` containing either the response data
    /// or an error if processing failed.
    async fn handle(&self, context: Option<Context>, data: T) -> Result<R>;
}

/// Default transport-agnostic implementation of `EnvelopeHandler`.
///
/// This handler bridges between envelope processing and context-data processing
/// by extracting context and data from the request envelope, delegating to a
/// `ContextDataHandler`, and wrapping the response data back into an envelope.
///
/// # Type Parameters
///
/// * `T` - The request data type within the envelope
/// * `R` - The response data type within the envelope
/// * `H` - The inner `ContextDataHandler` implementation
#[derive(Clone)]
pub struct DefaultEnvelopeHandler<H> {
    /// Inner handler that processes context and data
    inner_handler: H,
}

impl<H> DefaultEnvelopeHandler<H> {
    /// Create a new `DefaultEnvelopeHandler` with the given inner handler.
    ///
    /// # Arguments
    ///
    /// * `handler` - The inner `ContextDataHandler` to delegate to
    ///
    /// # Returns
    ///
    /// Returns a new `DefaultEnvelopeHandler` instance.
    pub fn new(handler: H) -> Self {
        Self {
            inner_handler: handler,
        }
    }

    /// Get a reference to the inner handler.
    pub fn inner(&self) -> &H {
        &self.inner_handler
    }
}

impl<T, R, H> EnvelopeHandler<T, R> for DefaultEnvelopeHandler<H>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + Send + 'static,
    H: ContextDataHandler<T, R> + Send + Sync,
{
    async fn handle(&self, envelope: Envelope<T>) -> Result<Envelope<R>> {
        // Extract context and data from the request envelope
        let (request_meta, request_data) = envelope.extract();

        // Convert envelope metadata to context
        let context = Some(Context::from(request_meta.clone()));

        // Delegate to the inner ContextDataHandler
        let response_data = self.inner_handler.handle(context, request_data).await?;

        // Create response metadata using the proper preservation utility
        // This ensures consistent metadata handling across all transports
        let response_meta = Meta::preserve_for_response(Some(&request_meta));

        // Create and return the response envelope
        Ok(Envelope::new(response_meta, response_data))
    }
}

/// Default transport-agnostic implementation of `ContextDataHandler`.
///
/// This handler serves as a bridge between the framework's context-data processing
/// and user-defined business logic handlers. It can wrap any handler that implements
/// a custom business logic interface.
///
/// # Type Parameters
///
/// * `T` - The request data type
/// * `R` - The response data type
/// * `H` - The inner business logic handler implementation
pub struct DefaultContextDataHandler<H> {
    /// Inner business logic handler
    inner_handler: H,
}

impl<H> DefaultContextDataHandler<H> {
    /// Create a new `DefaultContextDataHandler` with the given business logic handler.
    ///
    /// # Arguments
    ///
    /// * `handler` - The inner business logic handler
    ///
    /// # Returns
    ///
    /// Returns a new `DefaultContextDataHandler` instance.
    pub fn new(handler: H) -> Self {
        Self {
            inner_handler: handler,
        }
    }

    /// Get a reference to the inner handler.
    pub fn inner(&self) -> &H {
        &self.inner_handler
    }
}

#[async_trait]
impl<T, R, H> ContextDataHandler<T, R> for DefaultContextDataHandler<H>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + Send + 'static,
    H: ContextDataHandler<T, R> + Send + Sync,
{
    async fn handle(&self, context: Option<Context>, data: T) -> Result<R> {
        // Delegate directly to the inner handler
        // This implementation is mainly useful for composition and middleware patterns
        self.inner_handler.handle(context, data).await
    }
}

/// Implementation of `ContextDataHandler` for `Arc<T>` where `T: ContextDataHandler`.
///
/// This allows handlers that are wrapped in `Arc` for sharing across threads
/// to be used directly as `ContextDataHandler` instances.
#[async_trait]
impl<T, R, H> ContextDataHandler<T, R> for Arc<H>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
    R: Serialize + Send + 'static,
    H: ContextDataHandler<T, R> + Send + Sync,
{
    async fn handle(&self, context: Option<Context>, data: T) -> Result<R> {
        // Delegate to the Arc'd handler
        self.as_ref().handle(context, data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::QollectiveError;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRequest {
        message: String,
        id: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestResponse {
        result: String,
        status: u32,
    }

    /// Mock handler for testing
    struct MockContextDataHandler {
        should_fail: bool,
    }

    impl MockContextDataHandler {
        fn new() -> Self {
            Self { should_fail: false }
        }

        fn new_failing() -> Self {
            Self { should_fail: true }
        }
    }

    #[async_trait]
    impl ContextDataHandler<TestRequest, TestResponse> for MockContextDataHandler {
        async fn handle(
            &self,
            context: Option<Context>,
            data: TestRequest,
        ) -> Result<TestResponse> {
            if self.should_fail {
                return Err(QollectiveError::internal("Mock handler failure"));
            }

            let has_context = context.is_some();
            Ok(TestResponse {
                result: format!("Processed: {} (context: {})", data.message, has_context),
                status: 200,
            })
        }
    }

    #[tokio::test]
    async fn test_default_envelope_handler_success() {
        // ARRANGE: Create test components
        let inner_handler = MockContextDataHandler::new();
        let envelope_handler = DefaultEnvelopeHandler::new(inner_handler);

        let mut request_meta = Meta::default();
        request_meta.request_id = Some(Uuid::now_v7());
        request_meta.tenant = Some("test-tenant".to_string());

        let request_data = TestRequest {
            message: "Hello DefaultEnvelopeHandler".to_string(),
            id: 123,
        };
        let request_envelope = Envelope::new(request_meta.clone(), request_data);

        // ACT: Process envelope through handler
        let result = envelope_handler.handle(request_envelope).await;

        // ASSERT: Verify successful processing
        assert!(result.is_ok(), "DefaultEnvelopeHandler should succeed");

        let response_envelope = result.unwrap();
        let (response_meta, response_data) = response_envelope.extract();

        // Verify response data
        assert_eq!(response_data.status, 200);
        assert!(response_data
            .result
            .contains("Processed: Hello DefaultEnvelopeHandler"));
        assert!(response_data.result.contains("context: true")); // Should have context

        // Verify metadata preservation
        assert_eq!(response_meta.request_id, request_meta.request_id);
        assert_eq!(response_meta.tenant, request_meta.tenant);
        assert!(response_meta.timestamp.is_some()); // Should set response timestamp
    }

    #[tokio::test]
    async fn test_default_envelope_handler_with_empty_metadata() {
        // ARRANGE: Create envelope with minimal metadata
        let inner_handler = MockContextDataHandler::new();
        let envelope_handler = DefaultEnvelopeHandler::new(inner_handler);

        let request_meta = Meta::default(); // Empty metadata
        let request_data = TestRequest {
            message: "Minimal metadata test".to_string(),
            id: 456,
        };
        let request_envelope = Envelope::new(request_meta, request_data);

        // ACT: Process envelope
        let result = envelope_handler.handle(request_envelope).await;

        // ASSERT: Should still work with empty metadata
        assert!(result.is_ok(), "Should handle empty metadata gracefully");

        let response_envelope = result.unwrap();
        let (response_meta, response_data) = response_envelope.extract();

        // Verify response
        assert_eq!(response_data.status, 200);
        assert!(response_data.result.contains("context: true")); // Should still have context

        // Verify empty metadata is preserved correctly
        assert_eq!(response_meta.request_id, None);
        assert_eq!(response_meta.tenant, None);
        assert!(response_meta.timestamp.is_some()); // Should still set timestamp
    }

    #[tokio::test]
    async fn test_default_envelope_handler_inner_failure() {
        // ARRANGE: Create failing inner handler
        let inner_handler = MockContextDataHandler::new_failing();
        let envelope_handler = DefaultEnvelopeHandler::new(inner_handler);

        let request_meta = Meta::default();
        let request_data = TestRequest {
            message: "This will fail".to_string(),
            id: 789,
        };
        let request_envelope = Envelope::new(request_meta, request_data);

        // ACT: Process envelope with failing handler
        let result = envelope_handler.handle(request_envelope).await;

        // ASSERT: Should propagate inner handler error
        assert!(result.is_err(), "Should propagate inner handler failure");

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Mock handler failure"));
    }

    #[tokio::test]
    async fn test_default_envelope_handler_context_conversion() {
        // Test that Meta is properly converted to Context

        // ARRANGE: Create envelope with rich metadata
        let inner_handler = MockContextDataHandler::new();
        let envelope_handler = DefaultEnvelopeHandler::new(inner_handler);

        let mut request_meta = Meta::default();
        request_meta.request_id = Some(Uuid::now_v7());
        request_meta.tenant = Some("context-test-tenant".to_string());
        request_meta.timestamp = Some(chrono::Utc::now());

        let request_data = TestRequest {
            message: "Context conversion test".to_string(),
            id: 999,
        };
        let request_envelope = Envelope::new(request_meta, request_data);

        // ACT: Process envelope
        let result = envelope_handler.handle(request_envelope).await;

        // ASSERT: Context should be available to inner handler
        assert!(result.is_ok());

        let response_envelope = result.unwrap();
        let (_, response_data) = response_envelope.extract();

        // The mock handler reports whether it received context
        assert!(response_data.result.contains("context: true"));
    }

    #[tokio::test]
    async fn test_default_context_data_handler_delegation() {
        // Test that DefaultContextDataHandler properly delegates

        // ARRANGE: Create nested handler
        let base_handler = MockContextDataHandler::new();
        let context_handler = DefaultContextDataHandler::new(base_handler);

        let context = Some(Context::empty());
        let request_data = TestRequest {
            message: "Delegation test".to_string(),
            id: 555,
        };

        // ACT: Process through delegation
        let result = context_handler.handle(context, request_data).await;

        // ASSERT: Should delegate successfully
        assert!(result.is_ok());

        let response_data = result.unwrap();
        assert_eq!(response_data.status, 200);
        assert!(response_data.result.contains("Processed: Delegation test"));
        assert!(response_data.result.contains("context: true"));
    }

    #[tokio::test]
    async fn test_default_context_data_handler_error_propagation() {
        // Test error propagation through delegation

        // ARRANGE: Create failing nested handler
        let base_handler = MockContextDataHandler::new_failing();
        let context_handler = DefaultContextDataHandler::new(base_handler);

        let context = None;
        let request_data = TestRequest {
            message: "Error propagation test".to_string(),
            id: 666,
        };

        // ACT: Process through delegation
        let result = context_handler.handle(context, request_data).await;

        // ASSERT: Should propagate error
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Mock handler failure"));
    }

    #[tokio::test]
    async fn test_default_handlers_composition() {
        // Test composing DefaultEnvelopeHandler with DefaultContextDataHandler

        // ARRANGE: Create composed handlers
        let base_handler = MockContextDataHandler::new();
        let context_handler = DefaultContextDataHandler::new(base_handler);
        let envelope_handler = DefaultEnvelopeHandler::new(context_handler);

        let mut request_meta = Meta::default();
        request_meta.request_id = Some(Uuid::now_v7());
        request_meta.tenant = Some("composition-test".to_string());

        let request_data = TestRequest {
            message: "Composition test".to_string(),
            id: 777,
        };
        let request_envelope = Envelope::new(request_meta.clone(), request_data);

        // ACT: Process through composed handlers
        let result = envelope_handler.handle(request_envelope).await;

        // ASSERT: Should work through entire composition
        assert!(result.is_ok());

        let response_envelope = result.unwrap();
        let (response_meta, response_data) = response_envelope.extract();

        // Verify data processing
        assert_eq!(response_data.status, 200);
        assert!(response_data.result.contains("Processed: Composition test"));

        // Verify metadata preservation through composition
        assert_eq!(response_meta.request_id, request_meta.request_id);
        assert_eq!(response_meta.tenant, request_meta.tenant);
    }

    #[tokio::test]
    async fn test_default_envelope_handler_accessor_methods() {
        // Test public accessor methods

        // ARRANGE: Create handler
        let inner_handler = MockContextDataHandler::new();
        let envelope_handler = DefaultEnvelopeHandler::new(inner_handler);

        // ACT & ASSERT: Test accessor methods
        let inner_ref = envelope_handler.inner();
        assert!(!inner_ref.should_fail); // Access inner handler properties
    }

    #[tokio::test]
    async fn test_default_context_data_handler_accessor_methods() {
        // Test public accessor methods

        // ARRANGE: Create handler
        let base_handler = MockContextDataHandler::new();
        let context_handler = DefaultContextDataHandler::new(base_handler);

        // ACT & ASSERT: Test accessor methods
        let inner_ref = context_handler.inner();
        assert!(!inner_ref.should_fail); // Access inner handler properties
    }
}
