//! DefaultDynamicLlmClientProvider implementation
//!
//! This module provides the default implementation of DynamicLlmClientProvider
//! with three-tier configuration priority and runtime credential support.

use crate::client::RigDynamicLlmClient;
use crate::config::LlmConfig;
use crate::constants::*;
use crate::error::LlmError;
use crate::model_selector::{merge_model_mappings, select_model_for_language};
use crate::parameters::{LlmParameters, ProviderType, SystemPromptStyle};
use crate::rig_wrapper::RigClientWrapper;
use crate::traits::{DynamicLlmClient, DynamicLlmClientProvider};
use async_trait::async_trait;
use rig::providers::{anthropic, gemini, openai};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Default implementation of DynamicLlmClientProvider
///
/// This provider implements three-tier configuration priority:
/// 1. Runtime TenantLlmConfig (from request) - Premium feature
/// 2. Static TOML tenant config (server-managed)
/// 3. Default TOML config (fallback)
///
/// # Example
///
/// ```no_run
/// use shared_types_llm::{LlmConfig, DefaultDynamicLlmClientProvider, DynamicLlmClientProvider, LlmParameters};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = LlmConfig::load("config.toml")?;
/// let provider = DefaultDynamicLlmClientProvider::new(config);
///
/// let params = LlmParameters {
///     language_code: "en".to_string(),
///     ..Default::default()
/// };
///
/// let client = provider.get_dynamic_llm_client(&params).await?;
/// # Ok(())
/// # }
/// ```
pub struct DefaultDynamicLlmClientProvider {
    /// Loaded configuration from TOML
    config: LlmConfig,
}

impl DefaultDynamicLlmClientProvider {
    /// Create a new provider with loaded configuration
    pub fn new(config: LlmConfig) -> Self {
        debug!("Creating DefaultDynamicLlmClientProvider");
        Self { config }
    }

    /// Resolve effective configuration using three-tier priority
    fn resolve_config<'a>(
        &'a self,
        params: &'a LlmParameters,
    ) -> ResolvedConfig<'a> {
        // Priority 1: Runtime tenant config (Premium feature)
        if let Some(ref tenant_config) = params.tenant_config {
            info!(
                tenant_id = %tenant_config.tenant_id,
                provider = %tenant_config.provider_type,
                "{}",
                AUDIT_RUNTIME_TENANT_CONFIG
            );

            // For Google provider, prioritize google_credentials.api_key over api_key
            let effective_api_key = if tenant_config.provider_type == ProviderType::Google {
                tenant_config
                    .google_credentials
                    .as_ref()
                    .map(|c| c.api_key.as_str())
                    .or_else(|| tenant_config.api_key.as_deref())
            } else {
                tenant_config.api_key.as_deref()
            };

            return ResolvedConfig {
                provider_type: tenant_config.provider_type.clone(),
                base_url: tenant_config
                    .base_url
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| tenant_config.provider_type.default_url()),
                api_key: effective_api_key,
                default_model: self.config.provider.default_model.as_str(),
                use_fallback: self.config.provider.use_default_model_fallback,
                language_models: tenant_config.model_overrides.clone(),
                max_tokens: tenant_config
                    .max_tokens
                    .unwrap_or(self.config.provider.max_tokens),
                temperature: tenant_config
                    .temperature
                    .unwrap_or(self.config.provider.temperature),
                system_prompt_style: tenant_config
                    .system_prompt_style
                    .unwrap_or(self.config.provider.system_prompt_style),
                debug_config: self.config.provider.debug.clone(),
                source: ConfigSource::RuntimeTenant,
            };
        }

        // Priority 2: Static TOML tenant config
        if let Some(tenant_id) = &params.tenant_id {
            if let Some(tenant_config) = self.config.get_tenant_config(tenant_id) {
                info!(
                    tenant_id = %tenant_id,
                    "{}",
                    AUDIT_STATIC_TENANT_CONFIG
                );

                let merged_models = merge_model_mappings(
                    &self.config.provider.models,
                    &tenant_config.models,
                );

                return ResolvedConfig {
                    provider_type: tenant_config
                        .provider_type
                        .clone()
                        .unwrap_or_else(|| self.config.provider.provider_type.clone()),
                    base_url: tenant_config
                        .url
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or(&self.config.provider.url),
                    api_key: tenant_config
                        .api_key
                        .as_deref()
                        .or(self.config.provider.api_key.as_deref()),
                    default_model: tenant_config
                        .default_model
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or(&self.config.provider.default_model),
                    use_fallback: tenant_config
                        .use_default_model_fallback
                        .unwrap_or(self.config.provider.use_default_model_fallback),
                    language_models: merged_models,
                    max_tokens: tenant_config
                        .max_tokens
                        .unwrap_or(self.config.provider.max_tokens),
                    temperature: tenant_config
                        .temperature
                        .unwrap_or(self.config.provider.temperature),
                    system_prompt_style: tenant_config
                        .system_prompt_style
                        .unwrap_or(self.config.provider.system_prompt_style),
                    debug_config: self.config.provider.debug.clone(),
                    source: ConfigSource::StaticTenant,
                };
            } else {
                warn!(
                    tenant_id = %tenant_id,
                    "Tenant config not found, falling back to default config"
                );
            }
        }

        // Priority 3: Default TOML config
        debug!("{}", AUDIT_DEFAULT_CONFIG);
        ResolvedConfig {
            provider_type: self.config.provider.provider_type.clone(),
            base_url: &self.config.provider.url,
            api_key: self.config.provider.api_key.as_deref(),
            default_model: &self.config.provider.default_model,
            use_fallback: self.config.provider.use_default_model_fallback,
            language_models: self.config.provider.models.clone(),
            max_tokens: self.config.provider.max_tokens,
            temperature: self.config.provider.temperature,
            system_prompt_style: self.config.provider.system_prompt_style,
            debug_config: self.config.provider.debug.clone(),
            source: ConfigSource::Default,
        }
    }

    /// Build a rig-core client for the resolved configuration
    ///
    /// This function creates the appropriate native client (OpenAI, Anthropic, or Google)
    /// based on the provider type, with proper API key handling.
    fn build_rig_client(
        &self,
        resolved: &ResolvedConfig,
    ) -> Result<RigClientWrapper, LlmError> {
        match resolved.provider_type {
            ProviderType::Shimmy | ProviderType::LmStudio => {
                // OpenAI-compatible local servers (no API key needed)
                debug!(
                    provider = %resolved.provider_type,
                    base_url = %resolved.base_url,
                    "Building OpenAI-compatible client for local provider"
                );

                let client = openai::Client::builder("not-needed")
                    .base_url(resolved.base_url)
                    .build();

                Ok(RigClientWrapper::OpenAI(Arc::new(client)))
            }

            ProviderType::OpenAI => {
                // Native OpenAI API
                let api_key = resolved.api_key.ok_or_else(|| {
                    LlmError::missing_credentials(format!(
                        "OpenAI: {}",
                        ERROR_MISSING_API_KEY
                    ))
                })?;

                debug!(
                    provider = %resolved.provider_type,
                    base_url = %resolved.base_url,
                    "Building native OpenAI client"
                );

                let client = openai::Client::builder(api_key)
                    .base_url(resolved.base_url)
                    .build();

                Ok(RigClientWrapper::OpenAI(Arc::new(client)))
            }

            ProviderType::Anthropic => {
                // Native Anthropic/Claude API
                let api_key = resolved.api_key.ok_or_else(|| {
                    LlmError::missing_credentials(format!(
                        "Anthropic: {}",
                        ERROR_MISSING_API_KEY
                    ))
                })?;

                debug!(
                    provider = %resolved.provider_type,
                    base_url = %resolved.base_url,
                    "Building native Anthropic client"
                );

                // Note: Anthropic client doesn't support custom base_url in rig-core
                // It uses the default Anthropic API endpoint
                let client = anthropic::ClientBuilder::new(api_key)
                    .build()
                    .map_err(|e| {
                        LlmError::config_error(format!("Failed to build Anthropic client: {}", e))
                    })?;

                Ok(RigClientWrapper::Anthropic(Arc::new(client)))
            }

            ProviderType::Google => {
                // Native Google Gemini API
                let api_key = resolved.api_key.ok_or_else(|| {
                    LlmError::missing_credentials(format!(
                        "Google: {}",
                        ERROR_MISSING_API_KEY
                    ))
                })?;

                info!(
                    provider = %resolved.provider_type,
                    base_url = %resolved.base_url,
                    "Building native Google Gemini client with custom base URL"
                );

                // Use builder pattern to configure base_url
                // This ensures the configured LLM_URL is respected instead of using hardcoded v1beta
                let client = gemini::Client::builder(api_key)
                    .base_url(resolved.base_url)
                    .build()
                    .map_err(|e| {
                        LlmError::config_error(format!("Failed to build Google client: {}", e))
                    })?;

                Ok(RigClientWrapper::Google(Arc::new(client)))
            }
        }
    }
}

#[async_trait]
impl DynamicLlmClientProvider for DefaultDynamicLlmClientProvider {
    async fn get_dynamic_llm_client(
        &self,
        params: &LlmParameters,
    ) -> Result<Box<dyn DynamicLlmClient>, LlmError> {
        debug!(
            language_code = %params.language_code,
            has_tenant_config = params.tenant_config.is_some(),
            tenant_id = ?params.tenant_id,
            "Resolving LLM client configuration"
        );

        // Resolve configuration using three-tier priority
        let resolved = self.resolve_config(params);

        // Select the appropriate model
        let model_name = select_model_for_language(
            &params.language_code,
            params.model_name.as_deref(),
            &resolved.language_models,
            resolved.default_model,
            resolved.use_fallback,
        )?;

        debug!(
            model = %model_name,
            provider = %resolved.provider_type,
            config_source = ?resolved.source,
            "Selected model for language code"
        );

        // Build rig-core client
        let rig_client = self.build_rig_client(&resolved)?;

        // Create our wrapper client
        let client = RigDynamicLlmClient::new(
            rig_client,
            model_name,
            resolved.provider_type.to_string(),
            resolved.base_url.to_string(),
            resolved.max_tokens,
            resolved.temperature,
            resolved.system_prompt_style,
            resolved.debug_config.clone(),
        );

        Ok(Box::new(client))
    }
}

/// Resolved configuration from three-tier priority system
#[derive(Debug)]
struct ResolvedConfig<'a> {
    provider_type: ProviderType,
    base_url: &'a str,
    api_key: Option<&'a str>,
    default_model: &'a str,
    use_fallback: bool,
    language_models: std::collections::HashMap<String, String>,
    max_tokens: u32,
    temperature: f32,
    system_prompt_style: SystemPromptStyle,
    debug_config: crate::config::DebugConfig,
    source: ConfigSource,
}

/// Source of resolved configuration
#[derive(Debug, Clone, Copy)]
enum ConfigSource {
    RuntimeTenant,
    StaticTenant,
    Default,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::TenantLlmConfig;
    use std::collections::HashMap;

    fn create_test_config() -> LlmConfig {
        let toml = r#"
[llm]
type = "shimmy"
url = "http://127.0.0.1:11435/v1"
default_model = "default-model"
use_default_model_fallback = true
max_tokens = 4096
temperature = 0.7
timeout_secs = 60

[llm.models]
en = "en-model"
es = "es-model"

[llm.tenants.test-tenant]
type = "lmstudio"
url = "http://localhost:1234/v1"
default_model = "tenant-model"

[llm.tenants.test-tenant.models]
en = "tenant-en-model"
        "#;

        LlmConfig::from_toml_str(toml).unwrap()
    }

    #[tokio::test]
    async fn test_resolve_config_default() {
        let config = create_test_config();
        let provider = DefaultDynamicLlmClientProvider::new(config);

        let params = LlmParameters {
            language_code: "en".to_string(),
            ..Default::default()
        };

        let resolved = provider.resolve_config(&params);
        assert_eq!(resolved.provider_type, ProviderType::Shimmy);
        assert_eq!(resolved.base_url, "http://127.0.0.1:11435/v1");
        assert!(matches!(resolved.source, ConfigSource::Default));
    }

    #[tokio::test]
    async fn test_resolve_config_static_tenant() {
        let config = create_test_config();
        let provider = DefaultDynamicLlmClientProvider::new(config);

        let params = LlmParameters {
            language_code: "en".to_string(),
            tenant_id: Some("test-tenant".to_string()),
            ..Default::default()
        };

        let resolved = provider.resolve_config(&params);
        assert_eq!(resolved.provider_type, ProviderType::LmStudio);
        assert_eq!(resolved.base_url, "http://localhost:1234/v1");
        assert!(matches!(resolved.source, ConfigSource::StaticTenant));
    }

    #[tokio::test]
    async fn test_resolve_config_runtime_tenant() {
        let config = create_test_config();
        let provider = DefaultDynamicLlmClientProvider::new(config);

        let tenant_config = TenantLlmConfig {
            tenant_id: "runtime-tenant".to_string(),
            provider_type: ProviderType::OpenAI,
            api_key: Some("sk-test".to_string()),
            google_credentials: None,
            base_url: Some("https://api.openai.com/v1".to_string()),
            model_overrides: HashMap::from([("en".to_string(), "gpt-4".to_string())]),
            max_tokens: Some(8192),
            temperature: Some(0.8),
            timeout_secs: Some(120),
            system_prompt_style: Some(SystemPromptStyle::Native),
        };

        let params = LlmParameters {
            language_code: "en".to_string(),
            tenant_config: Some(tenant_config),
            ..Default::default()
        };

        let resolved = provider.resolve_config(&params);
        assert_eq!(resolved.provider_type, ProviderType::OpenAI);
        assert_eq!(resolved.base_url, "https://api.openai.com/v1");
        assert_eq!(resolved.api_key, Some("sk-test"));
        assert_eq!(resolved.max_tokens, 8192);
        assert_eq!(resolved.temperature, 0.8);
        assert!(matches!(resolved.source, ConfigSource::RuntimeTenant));
    }

    #[tokio::test]
    async fn test_get_client_with_language_mapping() {
        let config = create_test_config();
        let provider = DefaultDynamicLlmClientProvider::new(config);

        let params = LlmParameters {
            language_code: "en".to_string(),
            ..Default::default()
        };

        let result = provider.get_dynamic_llm_client(&params).await;
        assert!(result.is_ok());

        let client = result.unwrap();
        assert_eq!(client.model_name(), "en-model");
        assert_eq!(client.provider_type(), "shimmy");
    }

    #[tokio::test]
    async fn test_get_client_with_explicit_model() {
        let config = create_test_config();
        let provider = DefaultDynamicLlmClientProvider::new(config);

        let params = LlmParameters {
            language_code: "en".to_string(),
            model_name: Some("explicit-model".to_string()),
            ..Default::default()
        };

        let result = provider.get_dynamic_llm_client(&params).await;
        assert!(result.is_ok());

        let client = result.unwrap();
        assert_eq!(client.model_name(), "explicit-model");
    }

    #[tokio::test]
    async fn test_build_rig_client_requires_api_key() {
        use crate::config::DebugConfig;

        let config = create_test_config();
        let provider = DefaultDynamicLlmClientProvider::new(config);

        let resolved = ResolvedConfig {
            provider_type: ProviderType::OpenAI,
            base_url: "https://api.openai.com/v1",
            api_key: None,
            default_model: "gpt-4",
            use_fallback: true,
            language_models: HashMap::new(),
            max_tokens: 4096,
            temperature: 0.7,
            system_prompt_style: SystemPromptStyle::Native,
            debug_config: DebugConfig::default(),
            source: ConfigSource::Default,
        };

        let result = provider.build_rig_client(&resolved);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::MissingCredentials(_)));
    }

    #[tokio::test]
    async fn test_google_with_simple_api_key() {
        let config = create_test_config();
        let provider = DefaultDynamicLlmClientProvider::new(config);

        let tenant_config = TenantLlmConfig {
            tenant_id: "google-simple".to_string(),
            provider_type: ProviderType::Google,
            api_key: Some("AIza-test-key".to_string()),
            google_credentials: None,
            base_url: Some("https://generativelanguage.googleapis.com/v1".to_string()),
            model_overrides: HashMap::from([("en".to_string(), "gemini-pro".to_string())]),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            timeout_secs: Some(60),
            system_prompt_style: Some(SystemPromptStyle::Native),
        };

        let params = LlmParameters {
            language_code: "en".to_string(),
            tenant_config: Some(tenant_config),
            ..Default::default()
        };

        let resolved = provider.resolve_config(&params);
        assert_eq!(resolved.provider_type, ProviderType::Google);
        assert_eq!(resolved.api_key, Some("AIza-test-key"));
    }

    #[tokio::test]
    async fn test_google_with_full_credentials() {
        use crate::parameters::GoogleCredentials;

        let config = create_test_config();
        let provider = DefaultDynamicLlmClientProvider::new(config);

        let tenant_config = TenantLlmConfig {
            tenant_id: "google-full".to_string(),
            provider_type: ProviderType::Google,
            api_key: None,
            google_credentials: Some(GoogleCredentials {
                project_id: "my-project-123".to_string(),
                api_key: "AIza-from-creds".to_string(),
            }),
            base_url: Some("https://generativelanguage.googleapis.com/v1".to_string()),
            model_overrides: HashMap::from([("en".to_string(), "gemini-pro".to_string())]),
            max_tokens: Some(8192),
            temperature: Some(0.8),
            timeout_secs: Some(120),
            system_prompt_style: Some(SystemPromptStyle::Native),
        };

        let params = LlmParameters {
            language_code: "en".to_string(),
            tenant_config: Some(tenant_config),
            ..Default::default()
        };

        let resolved = provider.resolve_config(&params);
        assert_eq!(resolved.provider_type, ProviderType::Google);
        assert_eq!(resolved.api_key, Some("AIza-from-creds"));
    }

    #[tokio::test]
    async fn test_google_credentials_priority() {
        use crate::parameters::GoogleCredentials;

        let config = create_test_config();
        let provider = DefaultDynamicLlmClientProvider::new(config);

        // google_credentials.api_key should take priority over api_key
        let tenant_config = TenantLlmConfig {
            tenant_id: "google-priority".to_string(),
            provider_type: ProviderType::Google,
            api_key: Some("AIza-fallback".to_string()),
            google_credentials: Some(GoogleCredentials {
                project_id: "my-project-123".to_string(),
                api_key: "AIza-priority".to_string(),
            }),
            base_url: Some("https://generativelanguage.googleapis.com/v1".to_string()),
            model_overrides: HashMap::new(),
            max_tokens: None,
            temperature: None,
            timeout_secs: None,
            system_prompt_style: None,
        };

        let params = LlmParameters {
            language_code: "en".to_string(),
            tenant_config: Some(tenant_config),
            ..Default::default()
        };

        let resolved = provider.resolve_config(&params);
        assert_eq!(resolved.provider_type, ProviderType::Google);
        assert_eq!(resolved.api_key, Some("AIza-priority"));
    }

    #[tokio::test]
    async fn test_google_missing_credentials() {
        use crate::config::DebugConfig;

        let config = create_test_config();
        let provider = DefaultDynamicLlmClientProvider::new(config);

        let resolved = ResolvedConfig {
            provider_type: ProviderType::Google,
            base_url: "https://generativelanguage.googleapis.com/v1",
            api_key: None,
            default_model: "gemini-pro",
            use_fallback: true,
            language_models: HashMap::new(),
            max_tokens: 4096,
            temperature: 0.7,
            system_prompt_style: SystemPromptStyle::Native,
            debug_config: DebugConfig::default(),
            source: ConfigSource::Default,
        };

        let result = provider.build_rig_client(&resolved);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::MissingCredentials(_)));
    }
}
