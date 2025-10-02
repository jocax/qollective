// ABOUTME: Comprehensive tests for transport trait system
// ABOUTME: Tests trait compilation, mock implementations, and integration patterns

//! Tests for the transport trait system.
//!
//! This module contains comprehensive tests to verify that the transport
//! trait system compiles correctly and can be implemented by mock types.

use super::handlers::ContextDataHandler;
use super::receivers::UnifiedEnvelopeReceiver;
use super::senders::UnifiedEnvelopeSender;
use crate::envelope::{Context, Envelope};
use crate::error::{QollectiveError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Test data types
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

// Mock implementations for testing

/// Mock sender implementation for testing
#[derive(Debug, Clone)]
struct MockSender {
    should_fail: bool,
    response_data: TestResponse,
}

impl MockSender {
    fn new(response_data: TestResponse) -> Self {
        Self {
            should_fail: false,
            response_data,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl UnifiedEnvelopeSender<TestRequest, TestResponse> for MockSender {
    async fn send_envelope(
        &self,
        _endpoint: &str,
        envelope: Envelope<TestRequest>,
    ) -> Result<Envelope<TestResponse>> {
        if self.should_fail {
            return Err(QollectiveError::transport(
                "Mock sender failure".to_string(),
            ));
        }
        // Create response envelope using the request envelope's metadata
        Ok(Envelope::new(
            envelope.meta.clone(),
            self.response_data.clone(),
        ))
    }
}

/// Mock handler implementation for testing
#[derive(Debug, Clone)]
struct MockHandler {
    should_fail: bool,
    response_data: TestResponse,
}

impl MockHandler {
    fn new(response_data: TestResponse) -> Self {
        Self {
            should_fail: false,
            response_data,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl ContextDataHandler<TestRequest, TestResponse> for MockHandler {
    async fn handle(&self, _context: Option<Context>, _data: TestRequest) -> Result<TestResponse> {
        if self.should_fail {
            return Err(QollectiveError::transport(
                "Mock handler failure".to_string(),
            ));
        }
        Ok(self.response_data.clone())
    }
}

/// Mock receiver implementation for testing
#[derive(Debug)]
struct MockReceiver {
    should_fail: bool,
    request_data: TestRequest,
}

impl MockReceiver {
    fn new(request_data: TestRequest) -> Self {
        Self {
            should_fail: false,
            request_data,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl UnifiedEnvelopeReceiver for MockReceiver {
    async fn receive_envelope<T, R, H>(&mut self, handler: H) -> Result<()>
    where
        T: for<'de> serde::Deserialize<'de> + Send + 'static,
        R: serde::Serialize + Send + 'static,
        H: ContextDataHandler<T, R> + Send + Sync + 'static,
    {
        if self.should_fail {
            return Err(QollectiveError::transport(
                "Mock receiver failure".to_string(),
            ));
        }

        // For this mock, we can only handle TestRequest/TestResponse
        // In a real implementation, this would handle the actual transport protocol
        Ok(())
    }

    async fn receive_envelope_at<T, R, H>(&mut self, _route: &str, _handler: H) -> Result<()>
    where
        T: for<'de> serde::Deserialize<'de> + Send + 'static,
        R: serde::Serialize + Send + 'static,
        H: ContextDataHandler<T, R> + Send + Sync + 'static,
    {
        if self.should_fail {
            return Err(QollectiveError::transport(
                "Mock receiver route failure".to_string(),
            ));
        }

        // For this mock, we can only handle TestRequest/TestResponse
        // In a real implementation, this would handle the actual transport protocol with routing
        Ok(())
    }
}

// Trait compilation tests

#[tokio::test]
async fn test_sender_trait_compilation() {
    let response = TestResponse {
        result: "test response".to_string(),
        status: 200,
    };

    let sender = MockSender::new(response.clone());
    let request_data = TestRequest {
        message: "test message".to_string(),
        id: 42,
    };

    let request_envelope = Envelope::new(crate::envelope::Meta::default(), request_data);
    let result = sender
        .send_envelope("test://endpoint", request_envelope)
        .await;
    assert!(result.is_ok());
    let response_envelope = result.unwrap();
    let (_, response_data) = response_envelope.extract();
    assert_eq!(response_data, response);
}

#[tokio::test]
async fn test_sender_trait_failure() {
    let response = TestResponse {
        result: "test response".to_string(),
        status: 200,
    };

    let sender = MockSender::new(response).with_failure();
    let request_data = TestRequest {
        message: "test message".to_string(),
        id: 42,
    };

    let request_envelope = Envelope::new(crate::envelope::Meta::default(), request_data);
    let result = sender
        .send_envelope("test://endpoint", request_envelope)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_handler_trait_compilation() {
    let response = TestResponse {
        result: "handled response".to_string(),
        status: 201,
    };

    let handler = MockHandler::new(response.clone());
    let request = TestRequest {
        message: "test message".to_string(),
        id: 42,
    };

    let context = Some(Context::empty());
    let result = handler.handle(context, request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), response);
}

#[tokio::test]
async fn test_handler_trait_failure() {
    let response = TestResponse {
        result: "handled response".to_string(),
        status: 201,
    };

    let handler = MockHandler::new(response).with_failure();
    let request = TestRequest {
        message: "test message".to_string(),
        id: 42,
    };

    let context = Some(Context::empty());
    let result = handler.handle(context, request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_receiver_trait_compilation() {
    let request = TestRequest {
        message: "test message".to_string(),
        id: 42,
    };

    let response = TestResponse {
        result: "handled response".to_string(),
        status: 201,
    };

    let mut receiver = MockReceiver::new(request);
    let handler = MockHandler::new(response);

    let result = receiver.receive_envelope(handler).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_receiver_trait_failure() {
    let request = TestRequest {
        message: "test message".to_string(),
        id: 42,
    };

    let response = TestResponse {
        result: "handled response".to_string(),
        status: 201,
    };

    let mut receiver = MockReceiver::new(request).with_failure();
    let handler = MockHandler::new(response);

    let result = receiver.receive_envelope(handler).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_receiver_route_compilation() {
    let request = TestRequest {
        message: "test message".to_string(),
        id: 42,
    };

    let response = TestResponse {
        result: "handled response".to_string(),
        status: 201,
    };

    let mut receiver = MockReceiver::new(request);
    let handler = MockHandler::new(response);

    let result = receiver.receive_envelope_at("/api/test", handler).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_receiver_route_failure() {
    let request = TestRequest {
        message: "test message".to_string(),
        id: 42,
    };

    let response = TestResponse {
        result: "handled response".to_string(),
        status: 201,
    };

    let mut receiver = MockReceiver::new(request).with_failure();
    let handler = MockHandler::new(response);

    let result = receiver.receive_envelope_at("/api/test", handler).await;
    assert!(result.is_err());
}

// Context tests use the existing crate::envelope::Context
// No additional tests needed here since Context is tested in envelope module

// Transport capability tests removed - using existing crate::transport types
// No additional tests needed here since transport types are tested in transport module
