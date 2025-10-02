// ABOUTME: Configurable LLM provider system for holodeck services with multi-provider support
// ABOUTME: Provides trait-based facade for OpenAI, Ollama, Anthropic, and Perplexity integration using rig-core

use std::fmt;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use rig::providers::{openai, ollama, anthropic};
#[cfg(test)]
use std::collections::HashMap;
use rig::agent::{Agent, AgentBuilder};
use rig::completion::Prompt;
use rig::client::CompletionClient;
// Import the specific schemars version that rig-core uses (0.8.22) using the alias
use schemars_v08::JsonSchema;

/// LLM provider configuration errors
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Generation error: {0}")]
    Generation(String),

    #[error("API key not found for provider: {0}")]
    ApiKeyMissing(String),

    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),

    #[error("Response parsing error: {0}")]
    ResponseParsingError(String),
}

/// Supported LLM provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmProviderType {
    OpenAI,
    Ollama,
    Anthropic,
    Perplexity,
}

impl fmt::Display for LlmProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmProviderType::OpenAI => write!(f, "openai"),
            LlmProviderType::Ollama => write!(f, "ollama"),
            LlmProviderType::Anthropic => write!(f, "anthropic"),
            LlmProviderType::Perplexity => write!(f, "perplexity"),
        }
    }
}

/// LLM configuration for provider setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Primary LLM provider to use
    pub provider: LlmProviderType,

    /// Model name to use (e.g., "gpt-4", "claude-3-sonnet", "llama3.2")
    pub model: String,

    /// API endpoint URL (for Ollama or custom endpoints)
    pub endpoint_url: Option<String>,

    /// Temperature setting for response creativity (0.0 - 1.0)
    pub temperature: Option<f32>,

    /// Maximum tokens for responses
    pub max_tokens: Option<u32>,

    /// Timeout for API requests in seconds
    pub timeout_seconds: Option<u64>,

    /// Fallback provider configuration
    pub fallback: Option<Box<LlmConfig>>,
}

/// Runtime LLM provider information - SERIALIZABLE CONFIG ONLY
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProviderInfo {
    pub provider_type: LlmProviderType,
    pub model_name: String,
    pub provider_name: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProviderType::OpenAI,
            model: "gpt-4".to_string(),
            endpoint_url: None,
            temperature: Some(0.7),
            max_tokens: Some(2048),
            timeout_seconds: Some(30),
            fallback: None,
        }
    }
}

/// LLM agent wrapper for character interactions
#[async_trait]
pub trait LlmAgent: Send + Sync {
    /// Generate a response based on the given prompt
    async fn generate_response(&self, prompt: &str) -> Result<String, LlmError>;

    /// Generate a structured JSON response based on the given prompt and deserialize it to type T
    /// This method enables schema-first LLM interactions with rig-core extractor system
    async fn generate_structured_response<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: JsonSchema + for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send + Sync + 'static,
        Self: Sized,
;


    /// Get the model name this agent is using
    fn model_name(&self) -> &str;

    /// Get the provider type this agent is using
    fn provider_type(&self) -> LlmProviderType;
}

/// Main LLM provider trait for creating agents and managing providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Create a reusable agent with optional provider context (system prompt)
    /// If provider_context is None, creates a generic agent. Returns LlmError if creation fails.
    async fn create_agent(&self, provider_context: Option<&str>) -> Result<Box<dyn LlmAgent>, LlmError>;

    /// Execute a prompt directly without creating a persistent agent
    /// Useful for one-off prompts that don't require agent state
    async fn prompt(&self, prompt: &str) -> Result<String, LlmError>;

    /// Get the provider name
    fn provider_name(&self) -> String;

    /// Get the provider type
    fn provider_type(&self) -> LlmProviderType;

    /// Test the provider connection
    async fn test_connection(&self) -> Result<(), LlmError>;

    /// Create provider from configuration
    fn create_from_config(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError>
    where
        Self: Sized;
    
    /// Get serializable provider information (config-safe)
    fn get_provider_info(&self) -> LlmProviderInfo {
        LlmProviderInfo {
            provider_type: self.provider_type(),
            model_name: "unknown".to_string(), // Will be overridden by implementations
            provider_name: self.provider_name(),
        }
    }
}

/// OpenAI provider implementation using rig-core
pub struct OpenAIProvider {
    client: openai::Client,
    config: LlmConfig,
}

impl OpenAIProvider {
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| LlmError::ApiKeyMissing("OPENAI_API_KEY".to_string()))?;

        let client = openai::Client::new(&api_key);

        Ok(Self { client, config })
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn create_agent(&self, provider_context: Option<&str>) -> Result<Box<dyn LlmAgent>, LlmError> {
        let completion_model = self.client.completion_model(&self.config.model);
        let mut agent_builder = AgentBuilder::new(completion_model);
        
        // Apply provider context as preamble if provided
        if let Some(context) = provider_context {
            agent_builder = agent_builder.preamble(context);
        }
        
        let agent = agent_builder.build();

        Ok(Box::new(OpenAIAgent {
            agent,
            client: self.client.clone(),
            model_name: self.config.model.clone(),
            provider_type_field: LlmProviderType::OpenAI,
        }))
    }

    async fn prompt(&self, prompt: &str) -> Result<String, LlmError> {
        let completion_model = self.client.completion_model(&self.config.model);
        let agent = AgentBuilder::new(completion_model).build();
        
        let response = agent.prompt(prompt).await
            .map_err(|e| LlmError::Generation(format!("OpenAI direct prompt failed: {}", e)))?;
        
        Ok(response)
    }

    fn provider_name(&self) -> String {
        format!("OpenAI ({})", self.config.model)
    }

    fn provider_type(&self) -> LlmProviderType {
        LlmProviderType::OpenAI
    }

    async fn test_connection(&self) -> Result<(), LlmError> {
        // Use the new prompt method for connection testing
        self.prompt("Hello")
            .await
            .map(|_| ()) // Discard response, we just want to test connectivity
            .map_err(|e| LlmError::Provider(format!("OpenAI connection test failed: {}", e)))
    }

    fn create_from_config(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
        Ok(Box::new(OpenAIProvider::new(config.clone())?))
    }
    
    fn get_provider_info(&self) -> LlmProviderInfo {
        LlmProviderInfo {
            provider_type: self.provider_type(),
            model_name: self.config.model.clone(),
            provider_name: self.provider_name(),
        }
    }
}

/// OpenAI agent implementation
pub struct OpenAIAgent {
    agent: Agent<openai::responses_api::ResponsesCompletionModel>,
    client: openai::Client,
    model_name: String,
    provider_type_field: LlmProviderType,
}

#[async_trait]
impl LlmAgent for OpenAIAgent {
    async fn generate_response(&self, prompt: &str) -> Result<String, LlmError> {
        let response = self.agent.prompt(prompt).await
            .map_err(|e| LlmError::Generation(format!("OpenAI generation failed: {}", e)))?;
        Ok(response)
    }

    async fn generate_structured_response<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: JsonSchema + for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send + Sync + 'static,
        Self: Sized,
    {
        // Use rig-core's native extractor system directly - no fallback needed
        self.try_enhanced_extraction(prompt).await
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn provider_type(&self) -> LlmProviderType {
        self.provider_type_field
    }
}

impl OpenAIAgent {
    /// Use rig-core's native extractor system with built-in JSON healing
    async fn try_enhanced_extraction<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: JsonSchema + for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send + Sync + 'static,
    {
        // Build rig-core extractor with proper instructions
        let extractor = self.client
            .extractor::<T>(&self.model_name)
            .preamble("Extract structured data from the following content. Respond with valid JSON only.")
            .build();

        // Use rig-core's built-in extraction with JSON healing
        extractor.extract(prompt).await
            .map_err(|e| LlmError::ResponseParsingError(format!("rig-core OpenAI extraction failed: {}", e)))
    }
}

/// Ollama provider implementation using rig-core
pub struct OllamaProvider {
    client: ollama::Client,
    config: LlmConfig,
}

impl OllamaProvider {
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        let client = if let Some(endpoint) = &config.endpoint_url {
            ollama::Client::from_url(endpoint)
        } else {
            ollama::Client::new()
        };

        Ok(Self { client, config })
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn create_agent(&self, provider_context: Option<&str>) -> Result<Box<dyn LlmAgent>, LlmError> {
        let completion_model = self.client.completion_model(&self.config.model);
        let mut agent_builder = AgentBuilder::new(completion_model);
        
        // Apply provider context as preamble if provided
        if let Some(context) = provider_context {
            agent_builder = agent_builder.preamble(context);
        }
        
        let agent = agent_builder.build();

        Ok(Box::new(OllamaAgent {
            agent,
            client: self.client.clone(),
            model_name: self.config.model.clone(),
            provider_type_field: LlmProviderType::Ollama,
        }))
    }

    async fn prompt(&self, prompt: &str) -> Result<String, LlmError> {
        let completion_model = self.client.completion_model(&self.config.model);
        let agent = AgentBuilder::new(completion_model).build();
        
        let response = agent.prompt(prompt).await
            .map_err(|e| LlmError::Generation(format!("Ollama direct prompt failed: {}", e)))?;
        
        Ok(response)
    }

    fn provider_name(&self) -> String {
        format!("Ollama ({})", self.config.model)
    }

    fn provider_type(&self) -> LlmProviderType {
        LlmProviderType::Ollama
    }

    async fn test_connection(&self) -> Result<(), LlmError> {
        // Use the new prompt method for connection testing
        self.prompt("Hello")
            .await
            .map(|_| ()) // Discard response, we just want to test connectivity
            .map_err(|e| LlmError::Provider(format!("Ollama connection test failed: {}", e)))
    }

    fn create_from_config(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
        Ok(Box::new(OllamaProvider::new(config.clone())?))
    }
    
    fn get_provider_info(&self) -> LlmProviderInfo {
        LlmProviderInfo {
            provider_type: self.provider_type(),
            model_name: self.config.model.clone(),
            provider_name: self.provider_name(),
        }
    }
}

/// Ollama agent implementation
pub struct OllamaAgent {
    agent: Agent<ollama::CompletionModel>,
    client: ollama::Client,
    model_name: String,
    provider_type_field: LlmProviderType,
}

#[async_trait]
impl LlmAgent for OllamaAgent {
    async fn generate_response(&self, prompt: &str) -> Result<String, LlmError> {
        let response = self.agent.prompt(prompt).await
            .map_err(|e| LlmError::Generation(format!("Ollama generation failed: {}", e)))?;
        Ok(response)
    }

    async fn generate_structured_response<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: JsonSchema + for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send + Sync + 'static,
        Self: Sized,
    {
        // Use rig-core's native extractor system directly - no fallback needed
        self.try_extract_with_rig(prompt).await
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn provider_type(&self) -> LlmProviderType {
        self.provider_type_field
    }
}

impl OllamaAgent {
    /// Use rig-core's native extractor system with built-in JSON healing
    async fn try_extract_with_rig<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: JsonSchema + for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send + Sync + 'static,
    {
        // Build rig-core extractor with proper instructions
        let extractor = self.client
            .extractor::<T>(&self.model_name)
            .preamble("Extract structured data from the following content. Respond with valid JSON only.")
            .build();

        // Use rig-core's built-in extraction with JSON healing
        extractor.extract(prompt).await
            .map_err(|e| LlmError::ResponseParsingError(format!("rig-core Ollama extraction failed: {}", e)))
    }
}

/// Anthropic provider implementation using rig-core
pub struct AnthropicProvider {
    client: anthropic::Client,
    config: LlmConfig,
}

impl AnthropicProvider {
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| LlmError::ApiKeyMissing("ANTHROPIC_API_KEY".to_string()))?;

        let client = anthropic::Client::new(&api_key, "https://api.anthropic.com", None, "2023-06-01");

        Ok(Self { client, config })
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn create_agent(&self, provider_context: Option<&str>) -> Result<Box<dyn LlmAgent>, LlmError> {
        let completion_model = self.client.completion_model(&self.config.model);
        let mut agent_builder = AgentBuilder::new(completion_model);
        
        // Apply provider context as preamble if provided
        if let Some(context) = provider_context {
            agent_builder = agent_builder.preamble(context);
        }
        
        let agent = agent_builder.build();

        Ok(Box::new(AnthropicAgent {
            agent,
            client: self.client.clone(),
            model_name: self.config.model.clone(),
            provider_type_field: LlmProviderType::Anthropic,
        }))
    }

    async fn prompt(&self, prompt: &str) -> Result<String, LlmError> {
        let completion_model = self.client.completion_model(&self.config.model);
        let agent = AgentBuilder::new(completion_model).build();
        
        let response = agent.prompt(prompt).await
            .map_err(|e| LlmError::Generation(format!("Anthropic direct prompt failed: {}", e)))?;
        
        Ok(response)
    }

    fn provider_name(&self) -> String {
        format!("Anthropic ({})", self.config.model)
    }

    fn provider_type(&self) -> LlmProviderType {
        LlmProviderType::Anthropic
    }

    async fn test_connection(&self) -> Result<(), LlmError> {
        // Use the new prompt method for connection testing
        self.prompt("Hello")
            .await
            .map(|_| ()) // Discard response, we just want to test connectivity
            .map_err(|e| LlmError::Provider(format!("Anthropic connection test failed: {}", e)))
    }

    fn create_from_config(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
        Ok(Box::new(AnthropicProvider::new(config.clone())?))
    }
    
    fn get_provider_info(&self) -> LlmProviderInfo {
        LlmProviderInfo {
            provider_type: self.provider_type(),
            model_name: self.config.model.clone(),
            provider_name: self.provider_name(),
        }
    }
}

/// Anthropic agent implementation
pub struct AnthropicAgent {
    agent: Agent<anthropic::completion::CompletionModel>,
    client: anthropic::Client,
    model_name: String,
    provider_type_field: LlmProviderType,
}

#[async_trait]
impl LlmAgent for AnthropicAgent {
    async fn generate_response(&self, prompt: &str) -> Result<String, LlmError> {
        let response = self.agent.prompt(prompt).await
            .map_err(|e| LlmError::Generation(format!("Anthropic generation failed: {}", e)))?;
        Ok(response)
    }

    async fn generate_structured_response<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: JsonSchema + for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send + Sync + 'static,
        Self: Sized,
    {
        // Use rig-core's native extractor system directly - no fallback needed
        self.try_enhanced_extraction(prompt).await
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn provider_type(&self) -> LlmProviderType {
        self.provider_type_field
    }
}

impl AnthropicAgent {
    /// Use rig-core's native extractor system with built-in JSON healing
    async fn try_enhanced_extraction<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: JsonSchema + for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send + Sync + 'static,
    {
        // Build rig-core extractor with proper instructions
        let extractor = self.client
            .extractor::<T>(&self.model_name)
            .preamble("Extract structured data from the following content. Respond with valid JSON only.")
            .build();

        // Use rig-core's built-in extraction with JSON healing
        extractor.extract(prompt).await
            .map_err(|e| LlmError::ResponseParsingError(format!("rig-core Anthropic extraction failed: {}", e)))
    }
}

/// Manual Debug implementation for LlmProvider trait - only shows non-sensitive info
impl std::fmt::Debug for dyn LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LlmProvider {{ provider_name: \"{}\", provider_type: {:?} }}", 
               self.provider_name(), self.provider_type())
    }
}

/// Factory function to create LLM provider from configuration
pub fn create_llm_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
    match config.provider {
        LlmProviderType::OpenAI => OpenAIProvider::create_from_config(config),
        LlmProviderType::Ollama => OllamaProvider::create_from_config(config),
        LlmProviderType::Anthropic => AnthropicProvider::create_from_config(config),
        LlmProviderType::Perplexity => {
            // Perplexity uses OpenAI-compatible API
            let mut perplexity_config = config.clone();
            perplexity_config.endpoint_url = Some("https://api.perplexity.ai".to_string());

            // Would need PERPLEXITY_API_KEY environment variable
            std::env::var("PERPLEXITY_API_KEY")
                .map_err(|_| LlmError::ApiKeyMissing("PERPLEXITY_API_KEY".to_string()))?;

            Err(LlmError::UnsupportedProvider("Perplexity integration pending rig-core support".to_string()))
        }
    }
}

/// Mock LLM provider for testing
#[cfg(test)]
pub struct MockLlmProvider {
    responses: HashMap<String, String>,
}

#[cfg(test)]
impl MockLlmProvider {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    pub fn add_response(&mut self, prompt: &str, response: &str) {
        self.responses.insert(prompt.to_string(), response.to_string());
    }
}

#[cfg(test)]
#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn create_agent(&self, _provider_context: Option<&str>) -> Result<Box<dyn LlmAgent>, LlmError> {
        Ok(Box::new(MockLlmAgent {
            responses: self.responses.clone(),
        }))
    }

    async fn prompt(&self, prompt: &str) -> Result<String, LlmError> {
        Ok(self.responses.get(prompt)
            .cloned()
            .unwrap_or_else(|| format!("Mock response for: {}", prompt)))
    }

    fn provider_name(&self) -> String {
        "Mock Provider".to_string()
    }

    fn provider_type(&self) -> LlmProviderType {
        LlmProviderType::OpenAI // Use OpenAI as default for mock
    }

    async fn test_connection(&self) -> Result<(), LlmError> {
        Ok(())
    }

    fn create_from_config(_config: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
        Ok(Box::new(MockLlmProvider::new()))
    }
    
    fn get_provider_info(&self) -> LlmProviderInfo {
        LlmProviderInfo {
            provider_type: self.provider_type(),
            model_name: "mock-model".to_string(),
            provider_name: self.provider_name(),
        }
    }
}

#[cfg(test)]
pub struct MockLlmAgent {
    responses: HashMap<String, String>,
}

#[cfg(test)]
#[async_trait]
impl LlmAgent for MockLlmAgent {
    async fn generate_response(&self, prompt: &str) -> Result<String, LlmError> {
        Ok(self.responses.get(prompt)
            .cloned()
            .unwrap_or_else(|| format!("Mock response for: {}", prompt)))
    }

    async fn generate_structured_response<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: JsonSchema + for<'a> serde::de::Deserialize<'a> + serde::Serialize + Send + Sync + 'static,
        Self: Sized,
    {
        // For mock testing, return a simple JSON object that can be deserialized
        let response_json = self.responses.get(prompt)
            .cloned()
            .unwrap_or_else(|| r#"{"mock": "structured_response"}"#.to_string());
        
        serde_json::from_str(&response_json)
            .map_err(|e| LlmError::ResponseParsingError(format!("Mock structured response parsing failed: {}", e)))
    }

    fn model_name(&self) -> &str {
        "mock-model"
    }

    fn provider_type(&self) -> LlmProviderType {
        LlmProviderType::OpenAI
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_mock_llm_provider() {
        let mut provider = MockLlmProvider::new();
        provider.add_response("test prompt", "test response");

        let agent = provider.create_agent(Some("character prompt")).await.unwrap();
        let response = agent.generate_response("test prompt").await.unwrap();

        assert_eq!(response, "test response");
    }

    #[tokio::test]
    async fn test_mock_llm_provider_direct_prompt() {
        let mut provider = MockLlmProvider::new();
        provider.add_response("direct prompt", "direct response");

        let response = provider.prompt("direct prompt").await.unwrap();
        assert_eq!(response, "direct response");

        // Test default response for unknown prompt
        let default_response = provider.prompt("unknown prompt").await.unwrap();
        assert_eq!(default_response, "Mock response for: unknown prompt");
    }

    #[tokio::test]
    async fn test_agent_creation_with_context() {
        let provider = MockLlmProvider::new();
        
        // Test agent creation with context
        let agent_with_context = provider.create_agent(Some("You are a helpful assistant")).await.unwrap();
        assert_eq!(agent_with_context.provider_type(), LlmProviderType::OpenAI);
        
        // Test agent creation without context
        let agent_without_context = provider.create_agent(None).await.unwrap();
        assert_eq!(agent_without_context.provider_type(), LlmProviderType::OpenAI);
    }

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.provider, LlmProviderType::OpenAI);
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(LlmProviderType::OpenAI.to_string(), "openai");
        assert_eq!(LlmProviderType::Ollama.to_string(), "ollama");
        assert_eq!(LlmProviderType::Anthropic.to_string(), "anthropic");
        assert_eq!(LlmProviderType::Perplexity.to_string(), "perplexity");
    }

    #[test]
    fn test_llm_error_display() {
        let error = LlmError::ApiKeyMissing("TEST_API_KEY".to_string());
        assert!(error.to_string().contains("API key not found for provider: TEST_API_KEY"));
    }

    #[test]
    fn test_llm_config_serialization() {
        let config = LlmConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: LlmConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.provider, deserialized.provider);
        assert_eq!(config.model, deserialized.model);
    }

    #[tokio::test]
    async fn test_mock_provider_factory() {
        let config = LlmConfig::default();
        let provider = MockLlmProvider::create_from_config(&config).unwrap();
        assert_eq!(provider.provider_name(), "Mock Provider");
        assert_eq!(provider.provider_type(), LlmProviderType::OpenAI);
    }
}