// ABOUTME: Test-only mock transport implementation for unit testing
// ABOUTME: Provides simple mock functionality for dependency injection testing without external systems

//! Mock transport implementation for unit testing.
//!
//! This module provides a simple mock transport that implements the
//! `UnifiedEnvelopeSender` trait for testing purposes. It's only available
//! when compiled with test configuration to avoid bloating production builds.

#[cfg(test)]
use crate::envelope::{Envelope, Meta};
#[cfg(test)]
use crate::error::{QollectiveError, Result};
#[cfg(test)]
use crate::traits::senders::UnifiedEnvelopeSender;
#[cfg(test)]
use async_trait::async_trait;
#[cfg(test)]
use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json;
#[cfg(test)]
use std::collections::HashMap;

/// Simple mock transport for unit testing.
///
/// This mock transport allows configuring responses for specific endpoints
/// and records all requests for verification in tests. It's designed to be
/// lightweight and focused on enabling dependency injection testing.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockTransport {
    /// Pre-configured responses keyed by endpoint
    responses: HashMap<String, serde_json::Value>,
    /// Recorded requests for verification (endpoint, envelope JSON)
    recorded_requests: std::sync::Arc<std::sync::Mutex<Vec<(String, serde_json::Value)>>>,
    /// Whether to simulate network failure
    should_fail: bool,
}

#[cfg(test)]
impl MockTransport {
    /// Create a new mock transport with empty responses.
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            recorded_requests: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            should_fail: false,
        }
    }

    /// Add a response envelope for a specific endpoint.
    pub fn with_response<R: Serialize>(mut self, endpoint: &str, response_data: R) -> Self {
        // Create a response envelope with the provided data
        let response_envelope = Envelope::new(Meta::default(), response_data);
        let json_response = serde_json::to_value(response_envelope)
            .expect("Failed to serialize mock response envelope");
        self.responses.insert(endpoint.to_string(), json_response);
        self
    }

    /// Configure the mock to fail all requests.
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    /// Get all recorded requests for verification.
    /// Returns (endpoint, envelope_json) pairs.
    pub fn recorded_requests(&self) -> Vec<(String, serde_json::Value)> {
        self.recorded_requests.lock().unwrap().clone()
    }

    /// Clear all recorded requests.
    pub fn clear_requests(&self) {
        self.recorded_requests.lock().unwrap().clear();
    }
}

#[cfg(test)]
impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[async_trait]
impl<T, R> UnifiedEnvelopeSender<T, R> for MockTransport
where
    T: Serialize + Send + 'static,
    R: for<'de> Deserialize<'de> + Send + 'static,
{
    async fn send_envelope(&self, endpoint: &str, envelope: Envelope<T>) -> Result<Envelope<R>> {
        // Record the request envelope
        let json_envelope = serde_json::to_value(&envelope).map_err(|e| {
            QollectiveError::serialization(format!("Failed to serialize request envelope: {}", e))
        })?;

        self.recorded_requests
            .lock()
            .unwrap()
            .push((endpoint.to_string(), json_envelope));

        // Check if we should simulate failure
        if self.should_fail {
            return Err(QollectiveError::transport(
                "Mock transport configured to fail",
            ));
        }

        // Look up configured response envelope
        match self.responses.get(endpoint) {
            Some(response_json) => serde_json::from_value::<Envelope<R>>(response_json.clone())
                .map_err(|e| {
                    QollectiveError::serialization(format!(
                        "Failed to deserialize mock response envelope: {}",
                        e
                    ))
                }),
            None => Err(QollectiveError::transport(format!(
                "No mock response envelope configured for endpoint: {}",
                endpoint
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

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

    #[tokio::test]
    async fn test_mock_transport_implements_unified_envelope_sender() {
        let response_data = TestResponse {
            result: "success".to_string(),
            status: 200,
        };

        let mock = MockTransport::new().with_response("test-endpoint", response_data.clone());

        let request_data = TestRequest {
            message: "test".to_string(),
            id: 42,
        };

        // Create request envelope
        let request_envelope = Envelope::new(Meta::default(), request_data);

        let result: Result<Envelope<TestResponse>> =
            mock.send_envelope("test-endpoint", request_envelope).await;

        assert!(result.is_ok());
        let response_envelope = result.unwrap();
        let (_, actual_response_data) = response_envelope.extract();
        assert_eq!(actual_response_data, response_data);
    }

    #[tokio::test]
    async fn test_mock_transport_records_requests() {
        let response_data = TestResponse {
            result: "recorded".to_string(),
            status: 200,
        };

        let mock = MockTransport::new().with_response("record-endpoint", response_data);

        let request_data = TestRequest {
            message: "record me".to_string(),
            id: 123,
        };

        let request_envelope = Envelope::new(Meta::default(), request_data.clone());

        let _result: Result<Envelope<TestResponse>> = mock
            .send_envelope("record-endpoint", request_envelope)
            .await;

        let recorded = mock.recorded_requests();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].0, "record-endpoint");

        // Extract request from recorded envelope
        let recorded_envelope: Envelope<TestRequest> =
            serde_json::from_value(recorded[0].1.clone()).unwrap();
        let (_, recorded_request_data) = recorded_envelope.extract();
        assert_eq!(recorded_request_data, request_data);
    }

    #[tokio::test]
    async fn test_mock_transport_failure_simulation() {
        let mock = MockTransport::new().with_failure();

        let request_data = TestRequest {
            message: "will fail".to_string(),
            id: 999,
        };

        let request_envelope = Envelope::new(Meta::default(), request_data);

        let result: Result<Envelope<TestResponse>> =
            mock.send_envelope("any-endpoint", request_envelope).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Mock transport configured to fail"));
    }

    #[tokio::test]
    async fn test_mock_transport_missing_response() {
        let mock = MockTransport::new();

        let request_data = TestRequest {
            message: "no response configured".to_string(),
            id: 404,
        };

        let request_envelope = Envelope::new(Meta::default(), request_data);

        let result: Result<Envelope<TestResponse>> = mock
            .send_envelope("missing-endpoint", request_envelope)
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No mock response envelope configured for endpoint"));
    }

    #[tokio::test]
    async fn test_mock_transport_clear_requests() {
        let response_data = TestResponse {
            result: "clear test".to_string(),
            status: 200,
        };

        let mock = MockTransport::new().with_response("clear-endpoint", response_data);

        let request_data = TestRequest {
            message: "clear me".to_string(),
            id: 1,
        };

        let request_envelope = Envelope::new(Meta::default(), request_data);

        let _result: Result<Envelope<TestResponse>> =
            mock.send_envelope("clear-endpoint", request_envelope).await;

        assert_eq!(mock.recorded_requests().len(), 1);

        mock.clear_requests();
        assert_eq!(mock.recorded_requests().len(), 0);
    }

    // Comprehensive dependency injection example
    #[tokio::test]
    async fn test_dependency_injection_example() {
        /// Example service that uses dependency injection for transport
        struct ExampleService<T> {
            transport: T,
            service_name: String,
        }

        impl<T> ExampleService<T>
        where
            T: UnifiedEnvelopeSender<TestRequest, TestResponse>,
        {
            fn new(transport: T, service_name: String) -> Self {
                Self {
                    transport,
                    service_name,
                }
            }

            async fn process_request(&self, message: &str, id: u32) -> Result<String> {
                let request_data = TestRequest {
                    message: message.to_string(),
                    id,
                };

                // Create envelope with request data
                let request_envelope = Envelope::new(Meta::default(), request_data);

                let endpoint = format!("service://{}/process", self.service_name);
                let response_envelope = self
                    .transport
                    .send_envelope(&endpoint, request_envelope)
                    .await?;

                // Extract response data from envelope
                let (_, response_data) = response_envelope.extract();
                Ok(response_data.result)
            }
        }

        // Set up mock transport with expected response
        let expected_response_data = TestResponse {
            result: "Service processed successfully".to_string(),
            status: 200,
        };

        let mock_transport = MockTransport::new()
            .with_response("service://test-service/process", expected_response_data);

        // Inject mock transport into service
        let service = ExampleService::new(mock_transport.clone(), "test-service".to_string());

        // Test the service
        let result = service.process_request("test message", 42).await;

        // Verify results
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Service processed successfully");

        // Verify the mock recorded the correct request envelope
        let recorded = mock_transport.recorded_requests();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].0, "service://test-service/process");

        // Extract request data from recorded envelope
        let recorded_envelope: Envelope<TestRequest> =
            serde_json::from_value(recorded[0].1.clone()).unwrap();
        let (_, recorded_request_data) = recorded_envelope.extract();
        assert_eq!(recorded_request_data.message, "test message");
        assert_eq!(recorded_request_data.id, 42);
    }
}
