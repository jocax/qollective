// ABOUTME: NATS envelope codec for serialization/deserialization of Qollective envelopes
// ABOUTME: Preserves all envelope metadata during roundtrip conversion using binary serialization

//! NATS envelope codec for serialization/deserialization.
//!
//! This module provides efficient binary serialization for Qollective envelopes
//! when transmitted over NATS messaging. It preserves all metadata fields during
//! roundtrip conversion and provides comprehensive error handling.

use super::Envelope;
use crate::error::{QollectiveError, Result};
use serde::{Deserialize, Serialize};

use serde_json;

/// Error types specific to NATS envelope codec operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodecError {
    /// Serialization failed
    SerializationFailed(String),
    /// Deserialization failed
    DeserializationFailed(String),
    /// Envelope validation failed
    ValidationFailed(String),
    /// Unsupported envelope version
    UnsupportedVersion(String),
    /// Corrupt or malformed data
    CorruptData(String),
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::SerializationFailed(msg) => write!(f, "Serialization failed: {}", msg),
            CodecError::DeserializationFailed(msg) => write!(f, "Deserialization failed: {}", msg),
            CodecError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            CodecError::UnsupportedVersion(version) => {
                write!(f, "Unsupported version: {}", version)
            }
            CodecError::CorruptData(msg) => write!(f, "Corrupt data: {}", msg),
        }
    }
}

impl std::error::Error for CodecError {}

/// NATS envelope codec for binary serialization/deserialization
#[cfg(any(feature = "nats-client", feature = "nats-server"))]
#[derive(Debug)]
pub struct NatsEnvelopeCodec;

#[cfg(any(feature = "nats-client", feature = "nats-server"))]
impl NatsEnvelopeCodec {
    /// Encode an envelope to binary format for NATS transmission
    pub fn encode<T>(envelope: &Envelope<T>) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        // Validate envelope before encoding
        Self::validate_envelope(envelope)?;

        // Use JSON for reliable serialization (can be optimized to bincode later)
        serde_json::to_vec(envelope)
            .map_err(|e| QollectiveError::envelope(format!("Failed to encode envelope: {}", e)))
    }

    /// Decode binary data back to envelope
    pub fn decode<T>(data: &[u8]) -> Result<Envelope<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if data.is_empty() {
            return Err(QollectiveError::envelope("Cannot decode empty data"));
        }

        let envelope: Envelope<T> = serde_json::from_slice(data)
            .map_err(|e| QollectiveError::envelope(format!("Failed to decode envelope: {}", e)))?;

        // Validate the decoded envelope
        Self::validate_envelope(&envelope)?;

        Ok(envelope)
    }

    /// Validate envelope structure and metadata
    pub fn validate_envelope<T>(envelope: &Envelope<T>) -> Result<()> {
        // Validate metadata fields
        let meta = &envelope.meta;

        // Check for required fields based on envelope type
        if let Some(ref request_id) = meta.request_id {
            if request_id.to_string().is_empty() {
                return Err(QollectiveError::envelope("Request ID cannot be empty"));
            }
        }

        if let Some(ref version) = meta.version {
            if version.is_empty() {
                return Err(QollectiveError::envelope("Version cannot be empty"));
            }
        }

        // Validate tenant information if present
        if let Some(ref tenant) = meta.tenant {
            if tenant.trim().is_empty() {
                return Err(QollectiveError::envelope(
                    "Tenant ID cannot be empty or whitespace-only",
                ));
            }
        }

        // Validate duration if present
        if let Some(duration) = meta.duration {
            if duration < 0.0 {
                return Err(QollectiveError::envelope("Duration cannot be negative"));
            }
        }

        // Validate error information if present
        if let Some(ref error) = envelope.error {
            if error.code.is_empty() {
                return Err(QollectiveError::envelope("Error code cannot be empty"));
            }
            if error.message.is_empty() {
                return Err(QollectiveError::envelope("Error message cannot be empty"));
            }
        }

        Ok(())
    }

    /// Estimate the size of an encoded envelope without actually encoding it
    pub fn estimate_size<T>(envelope: &Envelope<T>) -> usize
    where
        T: Serialize,
    {
        // This is a rough estimate - actual bincode size may vary
        // Base size for envelope structure
        let mut size = 64; // Base metadata overhead

        // Add size for string fields
        if let Some(ref version) = envelope.meta.version {
            size += version.len();
        }
        if let Some(ref tenant) = envelope.meta.tenant {
            size += tenant.len();
        }

        // Add estimated size for optional sections
        if envelope.meta.security.is_some() {
            size += 128; // Estimated security metadata size
        }
        if envelope.meta.performance.is_some() {
            size += 64; // Estimated performance metadata size
        }
        if envelope.meta.monitoring.is_some() {
            size += 64; // Estimated monitoring metadata size
        }
        if envelope.meta.tracing.is_some() {
            size += 128; // Estimated tracing metadata size
        }
        if envelope.meta.extensions.is_some() {
            size += 256; // Estimated extensions metadata size
        }

        // Add error size if present
        if let Some(ref error) = envelope.error {
            size += error.code.len() + error.message.len() + 32;
        }

        size
    }
}

#[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
pub struct NatsEnvelopeCodec;

#[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
impl NatsEnvelopeCodec {
    pub fn encode<T>(_envelope: &Envelope<T>) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        Err(QollectiveError::feature_not_enabled(
            "NATS codec requires nats-client or nats-server feature",
        ))
    }

    pub fn decode<T>(_data: &[u8]) -> Result<Envelope<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        Err(QollectiveError::feature_not_enabled(
            "NATS codec requires nats-client or nats-server feature",
        ))
    }

    pub fn validate_envelope<T>(_envelope: &Envelope<T>) -> Result<()> {
        Err(QollectiveError::feature_not_enabled(
            "NATS codec requires nats-client or nats-server feature",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::{EnvelopeError, Meta};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        message: String,
        count: u32,
    }

    fn create_test_envelope() -> Envelope<TestData> {
        let data = TestData {
            message: "test message".to_string(),
            count: 42,
        };

        let meta = Meta {
            timestamp: Some(chrono::Utc::now()),
            request_id: Some(Uuid::now_v7()),
            version: Some("1.0.0".to_string()),
            duration: Some(100.0), // Duration in milliseconds as f64
            tenant: Some("test-tenant".to_string()),
            on_behalf_of: None,
            security: None,
            debug: None,
            performance: None,
            monitoring: None,
            tracing: None,
            extensions: None,
        };

        Envelope {
            meta,
            payload: data,
            error: None,
        }
    }

    fn create_envelope_with_error() -> Envelope<TestData> {
        let mut envelope = create_test_envelope();
        envelope.error = Some(EnvelopeError {
            code: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
            details: Some(serde_json::json!({"field": "value"})),
            trace: Some("Stack trace here".to_string()),
            #[cfg(any(
                feature = "rest-server", 
                feature = "rest-client",
                feature = "websocket-server", 
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: None,
        });
        envelope
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_roundtrip_preservation() {
        // ARRANGE: Create test envelope
        let original = create_test_envelope();

        // ACT: Encode and decode
        let encoded = NatsEnvelopeCodec::encode(&original).unwrap();
        let decoded: Envelope<TestData> = NatsEnvelopeCodec::decode(&encoded).unwrap();

        // ASSERT: All fields preserved
        assert_eq!(decoded.payload.message, original.payload.message);
        assert_eq!(decoded.payload.count, original.payload.count);
        assert_eq!(decoded.meta.version, original.meta.version);
        assert_eq!(decoded.meta.tenant, original.meta.tenant);
        assert_eq!(decoded.meta.request_id, original.meta.request_id);
        assert_eq!(decoded.meta.duration, original.meta.duration);
        // Note: timestamp comparison may have precision differences
        assert!(decoded.meta.timestamp.is_some());
        assert!(decoded.error.is_none());
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_envelope_with_error_roundtrip() {
        // ARRANGE: Create envelope with error
        let original = create_envelope_with_error();

        // ACT: Encode and decode
        let encoded = NatsEnvelopeCodec::encode(&original).unwrap();
        let decoded: Envelope<TestData> = NatsEnvelopeCodec::decode(&encoded).unwrap();

        // ASSERT: Error information preserved
        assert!(decoded.error.is_some());
        let error = decoded.error.unwrap();
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test error message");
        assert!(error.details.is_some());
        assert!(error.trace.is_some());
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_empty_data_decode_fails() {
        // ARRANGE: Empty data
        let empty_data = vec![];

        // ACT: Attempt to decode
        let result: Result<Envelope<TestData>> = NatsEnvelopeCodec::decode(&empty_data);

        // ASSERT: Should fail
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("empty data"));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_corrupt_data_decode_fails() {
        // ARRANGE: Corrupt binary data
        let corrupt_data = vec![0xFF, 0xFE, 0xFD, 0xFC];

        // ACT: Attempt to decode
        let result: Result<Envelope<TestData>> = NatsEnvelopeCodec::decode(&corrupt_data);

        // ASSERT: Should fail
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to decode"));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_validation_empty_error_code_fails() {
        // ARRANGE: Envelope with empty error code
        let mut envelope = create_test_envelope();
        envelope.error = Some(EnvelopeError {
            code: "".to_string(), // Empty code
            message: "Valid message".to_string(),
            details: None,
            trace: None,
            #[cfg(any(
                feature = "rest-server", 
                feature = "rest-client",
                feature = "websocket-server", 
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: None,
        });

        // ACT: Attempt validation
        let result = NatsEnvelopeCodec::validate_envelope(&envelope);

        // ASSERT: Should fail
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Error code cannot be empty"));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_validation_empty_error_message_fails() {
        // ARRANGE: Envelope with empty error message
        let mut envelope = create_test_envelope();
        envelope.error = Some(EnvelopeError {
            code: "VALID_CODE".to_string(),
            message: "".to_string(), // Empty message
            details: None,
            trace: None,
            #[cfg(any(
                feature = "rest-server", 
                feature = "rest-client",
                feature = "websocket-server", 
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: None,
        });

        // ACT: Attempt validation
        let result = NatsEnvelopeCodec::validate_envelope(&envelope);

        // ASSERT: Should fail
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Error message cannot be empty"));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_validation_empty_tenant_fails() {
        // ARRANGE: Envelope with empty tenant
        let mut envelope = create_test_envelope();
        envelope.meta.tenant = Some("   ".to_string()); // Whitespace-only tenant

        // ACT: Attempt validation
        let result = NatsEnvelopeCodec::validate_envelope(&envelope);

        // ASSERT: Should fail
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Tenant ID cannot be empty or whitespace-only"));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_validation_empty_version_fails() {
        // ARRANGE: Envelope with empty version
        let mut envelope = create_test_envelope();
        envelope.meta.version = Some("".to_string()); // Empty version

        // ACT: Attempt validation
        let result = NatsEnvelopeCodec::validate_envelope(&envelope);

        // ASSERT: Should fail
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Version cannot be empty"));
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_validation_success_for_valid_envelope() {
        // ARRANGE: Valid envelope
        let envelope = create_test_envelope();

        // ACT: Validate
        let result = NatsEnvelopeCodec::validate_envelope(&envelope);

        // ASSERT: Should succeed
        assert!(result.is_ok());
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_estimate_size_returns_reasonable_value() {
        // ARRANGE: Test envelope
        let envelope = create_test_envelope();

        // ACT: Estimate size
        let estimated_size = NatsEnvelopeCodec::estimate_size(&envelope);

        // ASSERT: Should return reasonable size
        assert!(estimated_size > 64); // Should be larger than base size
        assert!(estimated_size < 10000); // Should not be unreasonably large
    }

    #[cfg(any(feature = "nats-client", feature = "nats-server"))]
    #[test]
    fn test_estimate_size_with_error_is_larger() {
        // ARRANGE: Envelopes with and without error
        let envelope_no_error = create_test_envelope();
        let envelope_with_error = create_envelope_with_error();

        // ACT: Estimate sizes
        let size_no_error = NatsEnvelopeCodec::estimate_size(&envelope_no_error);
        let size_with_error = NatsEnvelopeCodec::estimate_size(&envelope_with_error);

        // ASSERT: Envelope with error should be larger
        assert!(size_with_error > size_no_error);
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    #[test]
    fn test_feature_not_enabled_encode() {
        // ARRANGE: Test envelope
        let envelope = create_test_envelope();

        // ACT: Attempt encode without feature
        let result = NatsEnvelopeCodec::encode(&envelope);

        // ASSERT: Should fail with feature error
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("feature"));
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    #[test]
    fn test_feature_not_enabled_decode() {
        // ARRANGE: Test data
        let data = vec![1, 2, 3, 4];

        // ACT: Attempt decode without feature
        let result: Result<Envelope<TestData>> = NatsEnvelopeCodec::decode(&data);

        // ASSERT: Should fail with feature error
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("feature"));
    }

    #[cfg(not(any(feature = "nats-client", feature = "nats-server")))]
    #[test]
    fn test_feature_not_enabled_validate() {
        // ARRANGE: Test envelope
        let envelope = create_test_envelope();

        // ACT: Attempt validate without feature
        let result = NatsEnvelopeCodec::validate_envelope(&envelope);

        // ASSERT: Should fail with feature error
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("feature"));
    }
}
