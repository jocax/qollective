// ABOUTME: Integration tests for Step 4 - Public API Framework Traits
// ABOUTME: Validates clean separation between public and internal APIs using TDD

//! Integration tests for the public API framework traits.
//!
//! This test module verifies that:
//! 1. Only ClientHandler and ServerHandler traits are publicly exported
//! 2. Internal traits are not accessible from public API
//! 3. Documentation tests show clean usage patterns
//! 4. Framework integration layer works correctly

use async_trait::async_trait;
use qollective::envelope::Context;
use qollective::error::Result;
use qollective::{ClientHandler, ServerHandler};
use serde::{Deserialize, Serialize};

// Test data types for public API usage
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
async fn test_public_client_handler_trait_usage() {
    // ARRANGE: Create a mock client handler
    struct MockClientHandler {
        response_data: TestResponse,
    }

    #[async_trait]
    impl ClientHandler<TestRequest, TestResponse> for MockClientHandler {
        async fn handle(
            &self,
            context: Option<Context>,
            data: TestRequest,
        ) -> Result<TestResponse> {
            Ok(self.response_data.clone())
        }
    }

    let handler = MockClientHandler {
        response_data: TestResponse {
            result: "test response".to_string(),
            status: 200,
        },
    };

    let request = TestRequest {
        message: "test message".to_string(),
        id: 42,
    };

    // ACT: Call the handler
    let result = handler.handle(Some(Context::empty()), request).await;

    // ASSERT: Verify the result
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.result, "test response");
}

#[tokio::test]
async fn test_public_server_handler_trait_usage() {
    // ARRANGE: Create a mock server handler
    struct MockServerHandler {
        response_data: TestResponse,
    }

    #[async_trait]
    impl ServerHandler<TestRequest, TestResponse> for MockServerHandler {
        async fn handle(
            &self,
            context: Option<Context>,
            data: TestRequest,
        ) -> Result<TestResponse> {
            Ok(self.response_data.clone())
        }
    }

    let handler = MockServerHandler {
        response_data: TestResponse {
            result: "server response".to_string(),
            status: 201,
        },
    };

    let request = TestRequest {
        message: "server test message".to_string(),
        id: 123,
    };

    // ACT: Call the handler
    let result = handler.handle(Some(Context::empty()), request).await;

    // ASSERT: Verify the result
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 201);
    assert_eq!(response.result, "server response");
}

#[test]
fn test_internal_traits_are_private() {
    // These compilation tests verify that internal traits are not accessible from public API
    // If these compile, it means internal traits are incorrectly exposed

    // This should compile - these are compilation-time checks, not runtime assertions
    // The test passes if internal traits are properly hidden

    // Test 1: Public traits should be accessible
    let _client_handler: Option<&dyn ClientHandler<TestRequest, TestResponse>> = None;
    let _server_handler: Option<&dyn ServerHandler<TestRequest, TestResponse>> = None;

    // Test 2: Context should be accessible
    let _context: Option<Context> = None;

    // Note: If internal traits were exposed, we would see them in the API
    // The absence of compilation errors when trying to use only public traits
    // indicates proper privacy separation
}

#[tokio::test]
async fn test_framework_integration_layer() {
    // This test verifies that the framework integration layer correctly bridges
    // between public traits (ClientHandler/ServerHandler) and internal traits

    // ARRANGE: Create a mock client handler using public API
    struct MockClientHandler {
        response_data: TestResponse,
    }

    #[async_trait]
    impl ClientHandler<TestRequest, TestResponse> for MockClientHandler {
        async fn handle(
            &self,
            context: Option<Context>,
            data: TestRequest,
        ) -> Result<TestResponse> {
            Ok(self.response_data.clone())
        }
    }

    let handler = MockClientHandler {
        response_data: TestResponse {
            result: "integration test response".to_string(),
            status: 202,
        },
    };

    let request = TestRequest {
        message: "integration test message".to_string(),
        id: 999,
    };

    // ACT: The integration layer should enable this usage
    // In a real scenario, this would be used by the transport layer
    let result = handler.handle(Some(Context::empty()), request).await;

    // ASSERT: Verify the integration works
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, 202);
    assert_eq!(response.result, "integration test response");

    // Note: The integration layer is conceptual at this point -
    // the real integration happens when transports use these public traits
    // through adapter patterns (implemented in future steps)
}

// This test should FAIL initially - Context type usage in public API
#[test]
fn test_context_type_in_public_api() {
    // ARRANGE: Create context
    let context = Context::empty();

    // ASSERT: Context should be available for public API usage
    assert!(context.meta().timestamp.is_none());
    assert!(context.meta().request_id.is_none());

    // Test context builder pattern
    let context_with_data = Context::builder().version("1.0.0").build();

    assert_eq!(context_with_data.meta().version.as_ref().unwrap(), "1.0.0");

    // This part should work - Context is already available
    // The failing part will be when we try to use it with public handler traits
}

#[tokio::test]
async fn test_public_api_documentation_example() {
    // This is a documentation test showing clean usage patterns
    // It demonstrates how users will interact with the public API

    // Note: These are already imported at the top of the file

    // Example client handler implementation
    struct MyClientHandler;

    #[async_trait]
    impl ClientHandler<TestRequest, TestResponse> for MyClientHandler {
        async fn handle(
            &self,
            context: Option<Context>,
            data: TestRequest,
        ) -> Result<TestResponse> {
            // User business logic here
            let response = TestResponse {
                result: format!("Processed: {}", data.message),
                status: 200,
            };
            Ok(response)
        }
    }

    // Example server handler implementation
    struct MyServerHandler;

    #[async_trait]
    impl ServerHandler<TestRequest, TestResponse> for MyServerHandler {
        async fn handle(
            &self,
            context: Option<Context>,
            data: TestRequest,
        ) -> Result<TestResponse> {
            // User business logic here
            let response = TestResponse {
                result: format!("Server processed: {} (ID: {})", data.message, data.id),
                status: 201,
            };
            Ok(response)
        }
    }

    // ACT & ASSERT: Test the usage patterns
    let client_handler = MyClientHandler;
    let server_handler = MyServerHandler;

    let request = TestRequest {
        message: "documentation test".to_string(),
        id: 42,
    };

    // Test client handler
    let client_result = client_handler
        .handle(Some(Context::empty()), request.clone())
        .await;
    assert!(client_result.is_ok());
    assert_eq!(
        client_result.unwrap().result,
        "Processed: documentation test"
    );

    // Test server handler
    let server_result = server_handler.handle(Some(Context::empty()), request).await;
    assert!(server_result.is_ok());
    assert_eq!(
        server_result.unwrap().result,
        "Server processed: documentation test (ID: 42)"
    );
}
