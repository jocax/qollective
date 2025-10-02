// ABOUTME: Configuration management for holodeck-validator service with LLM provider settings
// ABOUTME: Handles service configuration loading, validation, and LLM config conversion with .env fallback

use serde::{Deserialize, Serialize};
use shared_types::llm::{LlmConfig, LlmError, LlmProviderType};
use std::path::Path;
use std::env;
use tracing::{info, warn};

/// Service configuration for holodeck-validator
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    pub service: ServiceInfo,
    pub llm: LlmServiceConfig,
    pub validator: ValidatorConfig,
}

/// Basic service information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
}

/// LLM configuration for validator service
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmServiceConfig {
    /// Primary LLM provider: "openai", "ollama", "anthropic", "perplexity"
    pub provider: String,
    
    /// Model name to use
    pub model: String,
    
    /// Optional: Custom API key (overrides .env file)
    pub api_key: Option<String>,
    
    /// Optional: Custom endpoint URL (for Ollama or custom providers)
    pub endpoint_url: Option<String>,
    
    /// Response parameters
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

/// Validator-specific configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidatorConfig {
    /// Validation performance targets
    pub response_time_target_ms: u32,
    
    /// Quality scoring thresholds
    pub min_quality_score: u8,
    pub min_canon_compliance: u8,
    pub min_character_authenticity: u8,
    
    /// Validation strictness settings
    pub structure_analysis_depth: String, // "basic", "standard", "comprehensive"
    pub canon_strictness_default: String, // "lenient", "standard", "strict"
}

impl ServiceConfig {
    /// Load configuration from TOML file with .env fallback for API keys
    pub fn load_from_file(config_path: &str, env_path: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Loading validator service configuration from: {}", config_path);
        
        // Load .env file if provided
        if let Some(env_file) = env_path {
            if Path::new(env_file).exists() {
                match dotenv::from_path(env_file) {
                    Ok(_) => info!("Loaded environment variables from: {}", env_file),
                    Err(e) => warn!("Failed to load .env file {}: {}", env_file, e),
                }
            }
        }
        
        // Read and parse TOML configuration
        let config_content = std::fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
            
        let mut config: ServiceConfig = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file {}: {}", config_path, e))?;
        
        // Resolve API key from environment if not in config
        config.resolve_api_key()?;
        
        info!("Configuration loaded successfully for service: {}", config.service.name);
        info!("LLM Provider: {} with model: {}", config.llm.provider, config.llm.model);
        
        Ok(config)
    }
    
    /// Resolve API key from environment variables if not provided in config
    fn resolve_api_key(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.llm.api_key.is_none() {
            let env_var = match self.llm.provider.to_lowercase().as_str() {
                "openai" => "OPENAI_API_KEY",
                "anthropic" => "ANTHROPIC_API_KEY", 
                "perplexity" => "PERPLEXITY_API_KEY",
                "ollama" => return Ok(()), // Ollama doesn't require API key
                _ => return Err(format!("Unknown LLM provider: {}", self.llm.provider).into()),
            };
            
            if let Ok(api_key) = env::var(env_var) {
                info!("Using API key from environment variable: {}", env_var);
                self.llm.api_key = Some(api_key);
            } else if self.llm.provider != "ollama" {
                return Err(format!("API key not found in config or environment variable: {}", env_var).into());
            }
        }
        
        Ok(())
    }
    
    /// Convert service config to LLM config for provider creation
    pub fn to_llm_config(&self) -> Result<LlmConfig, LlmError> {
        let provider_type = match self.llm.provider.to_lowercase().as_str() {
            "openai" => LlmProviderType::OpenAI,
            "ollama" => LlmProviderType::Ollama,
            "anthropic" => LlmProviderType::Anthropic,
            "perplexity" => LlmProviderType::Perplexity,
            _ => return Err(LlmError::UnsupportedProvider(self.llm.provider.clone())),
        };
        
        Ok(LlmConfig {
            provider: provider_type,
            model: self.llm.model.clone(),
            endpoint_url: self.llm.endpoint_url.clone(),
            temperature: Some(self.llm.temperature),
            max_tokens: Some(self.llm.max_tokens),
            timeout_seconds: Some(self.llm.timeout_seconds),
            fallback: None, // Could be extended for fallback support
        })
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            service: ServiceInfo {
                name: "holodeck-validator".to_string(),
                version: "0.1.0".to_string(),
            },
            llm: LlmServiceConfig {
                provider: "ollama".to_string(),
                model: "gemma2:2b".to_string(),
                api_key: None,
                endpoint_url: None,
                temperature: 0.4, // Lower temperature for validation accuracy
                max_tokens: 3000, // Larger for detailed validation analysis
                timeout_seconds: 45, // Longer timeout for comprehensive analysis
            },
            validator: ValidatorConfig {
                response_time_target_ms: 400, // Target < 400ms as per PRP
                min_quality_score: 75,
                min_canon_compliance: 80,
                min_character_authenticity: 80,
                structure_analysis_depth: "standard".to_string(),
                canon_strictness_default: "standard".to_string(),
            },
        }
    }
}