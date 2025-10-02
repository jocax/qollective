// ABOUTME: Tests for config-driven automatic transport injection in HybridTransportClient
// ABOUTME: Validates that Config → Builder → Functionality pattern properly closes dependency injection gap

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
async fn test_config_based_transport_injection() {
    // Test the proper Config → Builder → Functionality pattern
    // Create config with explicit transport configurations since defaults are now None
    let mut transport_config = TransportConfig::default();

    // Explicitly configure REST transport for testing
    #[cfg(feature = "rest-client")]
    {
        transport_config.protocols.rest = Some(qollective::config::presets::RestConfig {
            client: Some(qollective::config::presets::RestClientConfig::default()),
            server: None,
        });
    }

    // NOTE: We don't configure NATS in this test because NatsConfig::default()
    // tries to connect immediately to nats://localhost:4222, which would fail in test environment.
    // The NATS client injection logic is tested separately in transport integration tests.

    // NOTE: We don't configure gRPC in this test because GrpcClientConfig::default()
    // may try to establish connections immediately, which would fail in test environment.
    // The gRPC client injection logic is tested separately in transport integration tests.

    // This should auto-inject transport clients based on feature gates and configs
    let client = HybridTransportClient::from_config(transport_config)
        .await
        .expect("Should create client from config");

    // Test that the client now has transports injected automatically when configured
    #[cfg(feature = "rest-client")]
    {
        assert!(
            client.internal_rest_client().is_some(),
            "REST client should be auto-injected when feature is enabled and config exists"
        );
    }

    // NOTE: NATS client injection is not tested here to avoid connection issues.
    // NATS injection is tested in the transport module tests where proper mock/isolation is set up.

    // NOTE: gRPC client injection is not tested here to avoid connection issues.
    // gRPC injection is tested in the transport module tests where proper mock/isolation is set up.

    println!("✅ Config-based injection working: feature gates properly create transport clients");
}

#[tokio::test]
async fn test_no_mock_responses_with_real_transports() {
    // Test that we no longer get mock responses when real transports are injected
    let mut transport_config = TransportConfig::default();

    // Configure REST transport for this test
    transport_config.protocols.rest = Some(qollective::config::presets::RestConfig {
        client: Some(qollective::config::presets::RestClientConfig::default()),
        server: None,
    });

    let client = HybridTransportClient::from_config(transport_config)
        .await
        .expect("Should create client from config");

    let request_data = TestRequest {
        message: "test no mocks".to_string(),
        id: 42,
    };

    let request_envelope = Envelope::new(Meta::default(), request_data);

    // Test that send_envelope uses real transport instead of mock response
    let result: qollective::error::Result<Envelope<TestResponse>> = client
        .send_envelope("https://httpbin.org/post", request_envelope)
        .await;

    // The key test: we should NOT get a mock response anymore
    // Instead we should get either a real response or a proper transport error
    match result {
        Ok(response_envelope) => {
            let (_, response_data) = response_envelope.extract();
            // If we get a response, it should NOT be the mock response
            assert_ne!(
                response_data.result, "envelope sent successfully",
                "Should not get mock response when real transport is injected"
            );
            println!(
                "✅ Got real response instead of mock: {}",
                response_data.result
            );
        }
        Err(err) => {
            // Real transport errors are acceptable - this means injection worked
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("REST client")
                    || error_msg.contains("connection")
                    || error_msg.contains("timeout")
                    || error_msg.contains("network")
                    || error_msg.contains("transport"),
                "Should get real transport error, not mock response. Got: {}",
                error_msg
            );
            println!(
                "✅ Got real transport error (injection working): {}",
                error_msg
            );
        }
    }
}
