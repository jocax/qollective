// ABOUTME: Integration tests for HybridTransportClient UnifiedEnvelopeSender trait implementation
// ABOUTME: Tests the trait functionality in isolation to validate Step 2 completion

use qollective::config::transport::TransportConfig;
use qollective::prelude::{Envelope, Meta, UnifiedEnvelopeSender};
use qollective::transport::HybridTransportClient;
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
async fn test_hybrid_transport_implements_unified_envelope_sender() {
    // Create transport config with REST client configuration
    let mut transport_config = TransportConfig::default();
    transport_config.protocols.rest = Some(qollective::config::presets::RestConfig {
        client: Some(qollective::config::presets::RestClientConfig::default()),
        server: None,
    });

    let client = HybridTransportClient::from_config(transport_config)
        .await
        .expect("Should create client from config");

    let request_data = TestRequest {
        message: "test message".to_string(),
        id: 42,
    };

    let request_envelope = Envelope::new(Meta::default(), request_data);

    // Test that the trait is implemented and compiles
    let result: qollective::error::Result<Envelope<TestResponse>> = client
        .send_envelope("https://example.com/api/test", request_envelope)
        .await;

    // Verify that we get a real transport error (not a mock response)
    // Since this is testing with a non-existent endpoint, we expect an actual network error
    match result {
        Ok(_) => panic!("Expected transport error for non-existent endpoint, but got success"),
        Err(err) => {
            // Should get a real transport error, not a "client not available" error
            let error_msg = err.to_string();
            assert!(
                !error_msg.contains("client not available") && !error_msg.contains("not available"),
                "Should get real transport error, not missing client error. Got: {}",
                error_msg
            );
            println!("✅ Got expected transport error: {}", error_msg);
        }
    }
}

#[tokio::test]
async fn test_hybrid_transport_trait_with_different_types() {
    // Create transport config with REST client configuration
    let mut transport_config = TransportConfig::default();
    transport_config.protocols.rest = Some(qollective::config::presets::RestConfig {
        client: Some(qollective::config::presets::RestClientConfig::default()),
        server: None,
    });

    let client = HybridTransportClient::from_config(transport_config)
        .await
        .expect("Should create client from config");

    let request_data = TestRequest {
        message: "different message".to_string(),
        id: 100,
    };

    let request_envelope = Envelope::new(Meta::default(), request_data);

    // Test that the trait works with different endpoint
    let result: qollective::error::Result<Envelope<TestResponse>> = client
        .send_envelope("https://api.example.com/different", request_envelope)
        .await;

    // Verify that we get a real transport error (not a mock response)
    // Since this is testing with a non-existent endpoint, we expect an actual network error
    match result {
        Ok(_) => panic!("Expected transport error for non-existent endpoint, but got success"),
        Err(err) => {
            // Should get a real transport error, not a "client not available" error
            let error_msg = err.to_string();
            assert!(
                !error_msg.contains("client not available") && !error_msg.contains("not available"),
                "Should get real transport error, not missing client error. Got: {}",
                error_msg
            );
            println!("✅ Got expected transport error: {}", error_msg);
        }
    }
}
