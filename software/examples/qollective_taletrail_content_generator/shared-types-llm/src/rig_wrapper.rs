//! Wrapper for different rig provider clients
//!
//! This module provides a unified wrapper around different rig-core provider clients,
//! allowing dynamic dispatch to the appropriate provider at runtime.

use rig::providers::{anthropic, gemini, openai};
use std::sync::Arc;

/// Wrapper for different rig provider clients
///
/// This enum provides a unified interface over rig-core's different provider clients,
/// enabling runtime selection of the appropriate provider while maintaining type safety.
///
/// # Supported Providers
///
/// - **OpenAI**: Native OpenAI API and OpenAI-compatible providers (Shimmy, LM Studio)
/// - **Anthropic**: Native Anthropic/Claude API
/// - **Google**: Native Google Gemini/Vertex AI API
///
/// # Example
///
/// ```no_run
/// use shared_types_llm::rig_wrapper::RigClientWrapper;
/// use rig::providers::openai;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let openai_client = openai::Client::builder("sk-test")
///     .base_url("https://api.openai.com/v1")
///     .build()?;
///
/// let wrapper = RigClientWrapper::OpenAI(Arc::new(openai_client));
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub enum RigClientWrapper {
    /// OpenAI and OpenAI-compatible providers (Shimmy, LM Studio, OpenAI)
    OpenAI(Arc<openai::Client>),
    /// Native Anthropic/Claude API
    Anthropic(Arc<anthropic::Client>),
    /// Native Google Gemini/Vertex AI
    Google(Arc<gemini::Client>),
}

impl RigClientWrapper {
    /// Get the underlying client reference based on provider type
    ///
    /// This returns a reference to the Arc-wrapped client
    pub fn as_openai(&self) -> Option<&Arc<openai::Client>> {
        match self {
            Self::OpenAI(client) => Some(client),
            _ => None,
        }
    }

    pub fn as_anthropic(&self) -> Option<&Arc<anthropic::Client>> {
        match self {
            Self::Anthropic(client) => Some(client),
            _ => None,
        }
    }

    pub fn as_gemini(&self) -> Option<&Arc<gemini::Client>> {
        match self {
            Self::Google(client) => Some(client),
            _ => None,
        }
    }

    /// Get the provider type as a string
    ///
    /// Returns a human-readable string identifying the provider type.
    pub fn provider_type(&self) -> &'static str {
        match self {
            Self::OpenAI(_) => "openai-compatible",
            Self::Anthropic(_) => "anthropic",
            Self::Google(_) => "google",
        }
    }
}

impl Clone for RigClientWrapper {
    fn clone(&self) -> Self {
        match self {
            Self::OpenAI(client) => Self::OpenAI(Arc::clone(client)),
            Self::Anthropic(client) => Self::Anthropic(Arc::clone(client)),
            Self::Google(client) => Self::Google(Arc::clone(client)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_openai() {
        let client = openai::Client::builder("test-key")
            .base_url("http://localhost:11435/v1")
            .build()
            .unwrap();

        let wrapper = RigClientWrapper::OpenAI(Arc::new(client));
        assert_eq!(wrapper.provider_type(), "openai-compatible");
    }

    #[test]
    fn test_provider_type_anthropic() {
        let client = anthropic::ClientBuilder::new("test-key").build().unwrap();
        let wrapper = RigClientWrapper::Anthropic(Arc::new(client));
        assert_eq!(wrapper.provider_type(), "anthropic");
    }

    #[test]
    fn test_provider_type_google() {
        let client = gemini::Client::new("test-key");
        let wrapper = RigClientWrapper::Google(Arc::new(client));
        assert_eq!(wrapper.provider_type(), "google");
    }

    #[test]
    fn test_clone() {
        let client = openai::Client::builder("test-key")
            .base_url("http://localhost:11435/v1")
            .build()
            .unwrap();

        let wrapper = RigClientWrapper::OpenAI(Arc::new(client));
        let cloned = wrapper.clone();

        assert_eq!(wrapper.provider_type(), cloned.provider_type());
    }
}
