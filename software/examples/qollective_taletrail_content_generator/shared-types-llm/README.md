# shared-types-llm

Dynamic LLM client library for the Qollective TaleTrail Content Generator project.

## Overview

`shared-types-llm` provides a flexible, multi-provider LLM client abstraction with:

- **Native Multi-Provider Support**: Native clients for Shimmy, LM Studio, OpenAI, Anthropic, Google (via rig-core 0.21)
- **API Key Priority System**: Runtime > Static TOML > Default - flexible credential management
- **Language-Based Model Selection**: Automatic model selection based on language code
- **Four-Tier Configuration**: .env file → TOML config → Environment variables → Runtime credentials
- **.env File Support**: Automatic loading of .env files for API keys and configuration
- **System Prompt Flexibility**: Support for different prompt styles (native, prepend, chatml, none)
- **Integration with rig-core 0.21**: Native provider clients for optimal performance

## Features

### Core Features

✅ **Native Provider Support** - Native OpenAI, Anthropic, and Google Gemini clients (not OpenAI-compatible adapters)
✅ **Dynamic Provider Selection** - Switch between local and remote LLM providers at runtime
✅ **.env File Support** - Automatic loading of .env files for API keys and configuration (dotenvy)
✅ **API Key Priority System** - Runtime → Static TOML → Default for flexible credential management
✅ **Provider-Specific API Keys** - Dedicated environment variables for each provider (LLM_OPENAI_API_KEY, etc.)
✅ **Language → Model Mapping** - Explicit mapping with fallback support
✅ **Configuration Inheritance** - Four-tier priority: .env → TOML → env vars → runtime
✅ **Premium Tenant Support** - Runtime credentials for tenants with their own API keys
✅ **Audit Logging** - Automatic logging when runtime credentials are used
✅ **Comprehensive Error Handling** - Rich error types with thiserror
✅ **Full Test Coverage** - 60 passing unit tests + 12 doctests with mockall support

### Architecture Principles

- **CONSTANTS FIRST**: All hardcoded values defined in `constants.rs`
- **Configuration Priority**: .env file → TOML config → System env vars → Runtime tenant config
- **API Key Priority**: Runtime `tenant_config.api_key` → System env vars → Static TOML → .env file
- **Native Provider Clients**: Direct use of rig-core's OpenAI, Anthropic, and Gemini clients (not adapters)
- **Security**: Runtime credentials can be provided per-request for multi-tenant scenarios
- **Fail-Fast**: Validate credentials on first LLM call, not during client creation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
shared-types-llm = { path = "../shared-types-llm" }

# Enable mocking for tests
[dev-dependencies]
shared-types-llm = { path = "../shared-types-llm", features = ["mocking"] }
```

## Quick Start

### Basic Usage (Default Configuration)

```rust
use shared_types_llm::*;

#[tokio::main]
async fn main() -> Result<(), LlmError> {
    // Load configuration from TOML
    let config = LlmConfig::load("config.toml")?;
    let provider = DefaultDynamicLlmClientProvider::new(config);

    // Create parameters for English language
    let params = LlmParameters {
        language_code: "en".to_string(),
        model_name: None, // Use language mapping from config
        system_prompt_style: SystemPromptStyle::ChatML,
        tenant_id: None,
        tenant_config: None,
    };

    // Get dynamic client
    let client = provider.get_dynamic_llm_client(&params).await?;

    // Generate content
    let system_prompt = "You are a helpful assistant.";
    let user_prompt = "Tell me a short story about space.";
    let response = client.prompt(
        &client.format_prompt(system_prompt, user_prompt)
    ).await?;

    println!("Response: {}", response);
    Ok(())
}
```

### Static Tenant Configuration

```rust
// Tenant "acme-corp" configured in config.toml
let params = LlmParameters {
    language_code: "de".to_string(),
    tenant_id: Some("acme-corp".to_string()),
    ..Default::default()
};

// Uses tenant-specific model from [llm.tenants.acme-corp.models]
let client = provider.get_dynamic_llm_client(&params).await?;
```

### Premium Tenant with Runtime Credentials

```rust
use std::collections::HashMap;

// Premium tenant provides their own OpenAI credentials
let tenant_config = TenantLlmConfig {
    tenant_id: "premium-corp".to_string(),
    provider_type: ProviderType::OpenAI,
    api_key: Some("sk-tenant-owned-key-xyz".to_string()),
    base_url: Some("https://api.openai.com/v1".to_string()),
    model_overrides: HashMap::from([
        ("en".to_string(), "gpt-4-turbo".to_string()),
        ("de".to_string(), "gpt-4-turbo".to_string()),
    ]),
    max_tokens: Some(8192),
    temperature: Some(0.8),
    timeout_secs: Some(120),
    system_prompt_style: Some(SystemPromptStyle::Native),
    use_default_model_fallback: false,
};

let params = LlmParameters {
    language_code: "en".to_string(),
    tenant_config: Some(tenant_config), // Runtime credentials
    ..Default::default()
};

// Client uses tenant's OpenAI account
// Automatically logged via tracing::info!
let client = provider.get_dynamic_llm_client(&params).await?;
```

### Google Tenant with Runtime Credentials

```rust
use std::collections::HashMap;

// Google Vertex AI / Gemini with simple API key
let tenant_config = TenantLlmConfig {
    tenant_id: "google-premium".to_string(),
    provider_type: ProviderType::Google,
    api_key: Some("AIza-tenant-owned-key-xyz".to_string()),
    base_url: Some("https://generativelanguage.googleapis.com/v1".to_string()),
    model_overrides: HashMap::from([
        ("en".to_string(), "gemini-pro".to_string()),
        ("de".to_string(), "gemini-pro".to_string()),
    ]),
    max_tokens: Some(8192),
    temperature: Some(0.8),
    timeout_secs: Some(120),
    system_prompt_style: Some(SystemPromptStyle::Native),
    use_default_model_fallback: false,
};

// Google with full credentials (project-based)
let tenant_config = TenantLlmConfig {
    tenant_id: "google-enterprise".to_string(),
    provider_type: ProviderType::Google,
    google_credentials: Some(GoogleCredentials {
        project_id: "my-gcp-project-123".to_string(),
        api_key: "AIza-tenant-owned-key-xyz".to_string(),
    }),
    model_overrides: HashMap::from([
        ("en".to_string(), "gemini-pro".to_string()),
    ]),
    ..Default::default()
};

let params = LlmParameters {
    language_code: "en".to_string(),
    tenant_config: Some(tenant_config),
    ..Default::default()
};

// Client uses tenant's Google account
// Automatically logged via tracing::info!
let client = provider.get_dynamic_llm_client(&params).await?;
```

## Configuration

### Example `config.toml`

```toml
# Example 1: Shimmy (local, no API key)
[llm]
type = "shimmy"
url = "http://127.0.0.1:11435/v1"
default_model = "qwen2.5-32b-instruct-q4_k_m"
use_default_model_fallback = true
timeout_secs = 60
max_tokens = 4096
temperature = 0.7
system_prompt_style = "chatml"

# Language → model mappings
[llm.models]
en = "qwen2.5-32b-instruct-q4_k_m"
de = "qwen2.5-32b-instruct-q4_k_m"
fr = "llama-3.3-70b-instruct-q4_k_m"

# Example 2: Native OpenAI (with server's default API key)
# [llm]
# type = "openai"
# url = "https://api.openai.com/v1"
# api_key = "sk-server-default-key"  # Server's OpenAI account
# default_model = "gpt-4-turbo"
# max_tokens = 4096
# temperature = 0.7

# Example 3: Native Anthropic
# [llm]
# type = "anthropic"
# url = "https://api.anthropic.com/v1"  # Note: URL not used by Anthropic client
# api_key = "sk-ant-server-key"  # Server's Anthropic account
# default_model = "claude-3-5-sonnet-20241022"
# max_tokens = 4096
# temperature = 0.7

# Example 4: Native Google Gemini
# [llm]
# type = "google"
# api_key = "AIza-server-key"  # Server's Google API key
# default_model = "gemini-pro"
# max_tokens = 8192
# temperature = 0.8

# Static tenant configurations (can have their own API keys)
[llm.tenants.acme-corp]
type = "openai"
url = "https://api.openai.com/v1"
api_key = "sk-acme-corp-server-key"  # Server-managed API key for this tenant
default_model = "gpt-4"

[llm.tenants.acme-corp.models]
en = "gpt-4"
de = "gpt-4"
```

### .env File Support

Create a `.env` file in your project root for local development:

```bash
# .env - Local Development Configuration
# This file is automatically loaded on startup

# Provider-Specific API Keys (Recommended)
LLM_OPENAI_API_KEY=sk-your-openai-key-here
LLM_ANTHROPIC_API_KEY=sk-ant-your-anthropic-key
LLM_GOOGLE_API_KEY=AIza-your-google-key

# Generic API Key (Fallback)
LLM_API_KEY=your-generic-api-key

# Provider Configuration
LLM_TYPE=openai
LLM_URL=https://api.openai.com/v1
LLM_DEFAULT_MODEL=gpt-4-turbo
LLM_MAX_TOKENS=8192
LLM_TEMPERATURE=0.7
LLM_TIMEOUT_SECS=120
```

**Configuration Priority:**
1. `.env` file (lowest priority) - Loaded automatically from current directory
2. `config.toml` file - Application configuration
3. System environment variables (highest priority) - Runtime overrides

**See:** `.env.example` in project root for complete example

### Environment Variable Reference

All TOML configuration values can be overridden using environment variables with `LLM_` prefix:

#### Provider-Specific API Keys

```bash
# OpenAI API Key (for GPT models)
export LLM_OPENAI_API_KEY=sk-...

# Anthropic API Key (for Claude models)
export LLM_ANTHROPIC_API_KEY=sk-ant-...

# Google API Key (for Gemini models)
export LLM_GOOGLE_API_KEY=AIza...

# Generic API Key (fallback for all providers)
export LLM_API_KEY=your-api-key-here
```

#### General Configuration

```bash
# Provider type: shimmy | lmstudio | openai | anthropic | google
export LLM_TYPE=openai

# Provider base URL
export LLM_URL=https://api.openai.com/v1

# Default model name
export LLM_DEFAULT_MODEL=gpt-4-turbo

# Request timeout in seconds
export LLM_TIMEOUT_SECS=120

# Maximum tokens for completions
export LLM_MAX_TOKENS=8192

# Temperature (0.0 - 1.0)
export LLM_TEMPERATURE=0.7

# System prompt style: native | prepend | chatml | none
export LLM_SYSTEM_PROMPT_STYLE=native

# Use default model as fallback (true | false)
export LLM_USE_DEFAULT_MODEL_FALLBACK=true
```

## Provider Types

### Local Providers (No Authentication)

- **Shimmy**: `http://127.0.0.1:11435/v1` - Recommended for local development
- **LM Studio**: `http://127.0.0.1:1234/v1` - Alternative local provider

### Remote Providers (Require API Keys)

- **OpenAI**: `https://api.openai.com/v1` - Native OpenAI client for GPT-4, GPT-3.5, etc.
- **Anthropic**: `https://api.anthropic.com/v1` - Native Anthropic client for Claude models
- **Google**: `https://generativelanguage.googleapis.com/v1` - Native Gemini client for Google models

### API Key Priority

When multiple API keys are available, they are prioritized as follows:

1. **Runtime** - `TenantLlmConfig.api_key` (passed in request) - Highest priority
2. **Static Tenant** - `[llm.tenants.X].api_key` (in config.toml) - Medium priority
3. **Default** - `[llm].api_key` (in config.toml) - Lowest priority

This allows flexible deployment scenarios:
- **Development**: Use default server API key
- **Production**: Use tenant-specific server API keys
- **Premium**: Allow tenants to provide their own API keys at request time

## System Prompt Styles

### Native
Model handles system prompts natively (separate system role).

### Prepend
System prompt prepended to user message with separator.

### ChatML
ChatML format with `<|im_start|>` and `<|im_end|>` tokens.

### None
No system prompt support (user prompt only).

## Configuration Priority

Configurations are applied in this order (lowest to highest priority):

1. **`.env` file** (lowest priority) - Local development configuration
2. **TOML `[llm]`** - Default application configuration
3. **TOML `[llm.tenants.{tenant_id}]`** - Static tenant overrides
4. **System environment variables** - Runtime environment overrides (LLM_* prefix)
5. **Runtime `TenantLlmConfig`** (in request) - Premium feature (highest priority)

This layered approach allows flexible deployment scenarios:
- **Local Development**: Use .env file for API keys
- **Server Deployment**: Use TOML config + system environment variables
- **Premium Tenants**: Provide runtime credentials per request

Example priority resolution:

```toml
# Default: Shimmy with Qwen
[llm]
type = "shimmy"
default_model = "qwen2.5-32b-instruct-q4_k_m"

# Tenant override: Shimmy with Magistral
[llm.tenants.acme-corp]
default_model = "magistral-small-2509-q8_0"
```

```rust
// Runtime override: OpenAI with GPT-4
let tenant_config = TenantLlmConfig {
    provider_type: ProviderType::OpenAI,
    default_model: Some("gpt-4-turbo".to_string()),
    ..Default::default()
};

// Result: OpenAI + GPT-4 (runtime wins)
```

## Error Handling

### Error Types

- `ConfigError` - Invalid configuration
- `MissingCredentials` - Required API key not provided
- `ModelNotAvailable` - Requested model not found (no fallback)
- `ProviderUnreachable` - Cannot connect to LLM provider
- `InvalidTenantConfig` - Malformed tenant configuration
- `RequestFailed` - LLM request failed
- `RigError` - rig-core internal error

### Example Error Handling

```rust
match provider.get_dynamic_llm_client(&params).await {
    Ok(client) => {
        // Use client
    }
    Err(LlmError::MissingCredentials(msg)) => {
        eprintln!("Credentials required: {}", msg);
    }
    Err(LlmError::ModelNotAvailable(model)) => {
        eprintln!("Model {} not available", model);
    }
    Err(LlmError::ProviderUnreachable(msg)) => {
        eprintln!("Provider unreachable: {}", msg);
    }
    Err(e) => {
        eprintln!("LLM error: {}", e);
    }
}
```

## Testing

### Unit Tests

```bash
cd shared-types-llm
cargo test
```

All 60 unit tests + 12 doc tests pass:

```
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured
```

### Mock Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let mut mock_provider = MockDynamicLlmClientProvider::new();

        mock_provider
            .expect_get_dynamic_llm_client()
            .returning(|_| {
                let mut mock_client = MockDynamicLlmClient::new();
                mock_client
                    .expect_prompt()
                    .returning(|_| Ok("Mock response".to_string()));
                Ok(Box::new(mock_client))
            });

        let params = LlmParameters::default();
        let client = mock_provider.get_dynamic_llm_client(&params).await.unwrap();
        let response = client.prompt("test").await.unwrap();

        assert_eq!(response, "Mock response");
    }
}
```

## Integration with MCP Servers

### prompt-helper Integration

```rust
use shared_types_llm::*;

pub struct PromptHelperServer {
    llm_provider: Arc<DefaultDynamicLlmClientProvider>,
}

impl PromptHelperServer {
    pub fn new(config_path: &str) -> Result<Self, LlmError> {
        let config = LlmConfig::load(config_path)?;
        let llm_provider = Arc::new(DefaultDynamicLlmClientProvider::new(config));
        Ok(Self { llm_provider })
    }

    pub async fn generate_prompt(
        &self,
        request: &PromptGenerationRequest,
    ) -> Result<(String, String), LlmError> {
        let params = LlmParameters {
            language_code: request.generation_request.language.to_string(),
            tenant_id: request.tenant_id.clone(),
            tenant_config: request.tenant_llm_config.clone(),
            ..Default::default()
        };

        let client = self.llm_provider.get_dynamic_llm_client(&params).await?;
        let formatted = client.format_prompt(system_prompt, user_prompt);
        let response = client.prompt(&formatted).await?;

        // Parse response into (system_prompt, user_prompt)
        Ok(parse_llm_response(&response)?)
    }
}
```

### story-generator Integration

```rust
use shared_types_llm::*;

pub struct StoryGeneratorServer {
    llm_provider: Arc<DefaultDynamicLlmClientProvider>,
}

impl StoryGeneratorServer {
    pub async fn generate_content(
        &self,
        prompt_package: &PromptPackage,
        node_context: &NodeContext,
        tenant_config: Option<TenantLlmConfig>,
    ) -> Result<String, LlmError> {
        let params = LlmParameters {
            language_code: prompt_package.language.to_string(),
            tenant_config,
            ..Default::default()
        };

        let client = self.llm_provider.get_dynamic_llm_client(&params).await?;

        let formatted = client.format_prompt(
            &prompt_package.system_prompt,
            &self.build_content_prompt(prompt_package, node_context),
        );

        client.prompt(&formatted).await
    }
}
```

## Security Considerations

### Runtime Credentials

- ✅ Runtime credentials (`TenantLlmConfig`) are ONLY passed in requests
- ✅ NEVER stored in TOML configuration files
- ✅ Automatically logged via `tracing::info!` for audit trail
- ✅ No validation until first LLM call (fail-fast approach)

### API Key Management

```rust
// ❌ BAD: Never hardcode API keys
let tenant_config = TenantLlmConfig {
    api_key: Some("sk-hardcoded-key".to_string()),
    ..Default::default()
};

// ✅ GOOD: Load from secure storage
let tenant_config = TenantLlmConfig {
    api_key: Some(get_tenant_api_key_from_vault(tenant_id)?),
    ..Default::default()
};

// ✅ GOOD: Extract from request headers
let tenant_config = TenantLlmConfig {
    api_key: extract_bearer_token_from_request(&req)?,
    ..Default::default()
};
```

## Performance Considerations

- **Connection Reuse**: Clients can be cached per tenant/language combination
- **Model Selection**: Language mapping uses HashMap lookup (O(1))
- **Configuration Loading**: Load once at startup, reuse across requests
- **Async Operations**: Full async/await support with tokio

## License

Part of the Qollective TaleTrail Content Generator project.

## Example Files

- **`config.toml.example`** - Complete TOML configuration example with all providers and features
- **`.env.example`** (in project root) - Example .env file with all environment variables
- **PlantUML Diagram** - See `concept/capstone/capstone_llm_config_flow.puml` for configuration flow visualization

## See Also

- [rig-core Documentation](https://docs.rs/rig-core/0.21)
- [Shimmy GitHub Repository](https://github.com/Michael-A-Kuykendall/shimmy)
- [LM Studio](https://lmstudio.ai/)
- [dotenvy Documentation](https://docs.rs/dotenvy) - .env file loader
