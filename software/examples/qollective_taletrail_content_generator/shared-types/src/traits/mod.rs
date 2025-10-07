//! Service trait boundaries for dependency injection and testability
//!
//! All service implementations must implement these traits to enable:
//! - Unit testing with mockall-generated mocks
//! - Dependency injection for different implementations
//! - Clear interface boundaries between components
//!
//! # Testing Pattern
//!
//! ```rust,ignore
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use mockall::predicate::*;
//!
//!     #[tokio::test]
//!     async fn test_with_mock_llm() {
//!         let mut mock_llm = MockLlmService::new();
//!         mock_llm
//!             .expect_generate_prompt()
//!             .returning(|_, _| Ok(("system".into(), "user".into())));
//!
//!         // Use mock_llm in tests...
//!     }
//! }
//! ```

pub mod llm_service;
pub mod prompt_helper_service;
pub mod mcp_transport;
pub mod request_mapper;
pub mod story_generator_service;
pub mod validation_service;

// Re-export traits for convenience
pub use llm_service::LlmService;
pub use prompt_helper_service::PromptHelperService;
pub use mcp_transport::McpTransport;
pub use request_mapper::RequestMapper;
pub use story_generator_service::StoryGeneratorService;
pub use validation_service::ValidationService;

// Re-export mock types when mocking feature is enabled
#[cfg(any(test, feature = "mocking"))]
pub use llm_service::MockLlmService;
#[cfg(any(test, feature = "mocking"))]
pub use prompt_helper_service::MockPromptHelperService;
#[cfg(any(test, feature = "mocking"))]
pub use mcp_transport::MockMcpTransport;
#[cfg(any(test, feature = "mocking"))]
pub use request_mapper::MockRequestMapper;
#[cfg(any(test, feature = "mocking"))]
pub use story_generator_service::MockStoryGeneratorService;
#[cfg(any(test, feature = "mocking"))]
pub use validation_service::MockValidationService;
