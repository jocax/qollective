// ABOUTME: WASM REST client with fetch API integration
// ABOUTME: Provides HTTPS REST communication for envelope-based messaging in browsers

//! WASM REST client implementation using the fetch API.
//!
//! This module provides REST communication capabilities for WASM applications
//! using the browser's fetch API while maintaining envelope patterns.

use crate::config::rest::RestClientConfig;
use crate::constants::{limits, timeouts};
use crate::error::{QollectiveError, Result};
use crate::wasm::crypto::{WasmCertificateManager, CertificateBundle};
use crate::wasm::js_types::{WasmEnvelope, WasmMeta};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, RequestMode, Response};

/// WASM REST client using fetch API
#[derive(Debug, Clone)]
pub struct WasmRestClient {
    config: RestClientConfig,
    cert_manager: WasmCertificateManager,
}

/// HTTP method enumeration
#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
}

impl ToString for HttpMethod {
    fn to_string(&self) -> String {
        match self {
            HttpMethod::GET => "GET".to_string(),
            HttpMethod::POST => "POST".to_string(),
            HttpMethod::PUT => "PUT".to_string(),
            HttpMethod::DELETE => "DELETE".to_string(),
            HttpMethod::PATCH => "PATCH".to_string(),
            HttpMethod::OPTIONS => "OPTIONS".to_string(),
        }
    }
}

/// Request retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: limits::DEFAULT_RETRY_ATTEMPTS,
            base_delay_ms: timeouts::DEFAULT_REST_RETRY_DELAY_MS,
            max_delay_ms: timeouts::DEFAULT_REST_MAX_RETRY_DELAY_MS,
            exponential_backoff: true,
        }
    }
}

impl WasmRestClient {
    /// Create new WASM REST client
    pub fn new(config: RestClientConfig, cert_manager: WasmCertificateManager) -> Result<Self> {
        // CONFIG FIRST PRINCIPLE - validate config
        if config.base.base_url.is_empty() {
            return Err(QollectiveError::validation("Base URL cannot be empty"));
        }

        Ok(Self {
            config,
            cert_manager,
        })
    }

    /// Send envelope via REST with specified HTTP method
    pub async fn send_envelope_with_method(
        &self,
        method: HttpMethod,
        url: &str,
        envelope: WasmEnvelope,
    ) -> Result<WasmEnvelope> {
        // Validate URL
        if url.is_empty() {
            return Err(QollectiveError::validation("URL cannot be empty"));
        }

        // Build full URL
        let full_url = if url.starts_with("http") {
            url.to_string()
        } else {
            format!("{}{}", self.config.base.base_url.trim_end_matches('/'), url)
        };

        // Setup retry configuration
        let retry_config = RetryConfig {
            max_attempts: self.config.base.retry_attempts,
            ..RetryConfig::default()
        };

        let mut last_error = QollectiveError::transport("No attempts made".to_string());

        // Retry loop
        for attempt in 0..retry_config.max_attempts {
            match self.perform_request(&method, &full_url, &envelope).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = e;
                    
                    // If not the last attempt, wait before retrying
                    if attempt < retry_config.max_attempts - 1 {
                        let delay = self.calculate_retry_delay(&retry_config, attempt);
                        self.sleep_ms(delay).await;
                        
                        web_sys::console::warn_1(&format!(
                            "REST request attempt {} failed, retrying in {}ms", 
                            attempt + 1, delay
                        ).into());
                    }
                }
            }
        }

        Err(last_error)
    }

    /// Send envelope via REST (defaults to POST)
    pub async fn send_envelope(&self, url: &str, envelope: WasmEnvelope) -> Result<WasmEnvelope> {
        self.send_envelope_with_method(HttpMethod::POST, url, envelope).await
    }

    /// Send GET request with envelope
    pub async fn get(&self, url: &str, envelope: WasmEnvelope) -> Result<WasmEnvelope> {
        self.send_envelope_with_method(HttpMethod::GET, url, envelope).await
    }

    /// Send POST request with envelope
    pub async fn post(&self, url: &str, envelope: WasmEnvelope) -> Result<WasmEnvelope> {
        self.send_envelope_with_method(HttpMethod::POST, url, envelope).await
    }

    /// Send PUT request with envelope
    pub async fn put(&self, url: &str, envelope: WasmEnvelope) -> Result<WasmEnvelope> {
        self.send_envelope_with_method(HttpMethod::PUT, url, envelope).await
    }

    /// Send DELETE request with envelope
    pub async fn delete(&self, url: &str, envelope: WasmEnvelope) -> Result<WasmEnvelope> {
        self.send_envelope_with_method(HttpMethod::DELETE, url, envelope).await
    }

    /// Send PATCH request with envelope
    pub async fn patch(&self, url: &str, envelope: WasmEnvelope) -> Result<WasmEnvelope> {
        self.send_envelope_with_method(HttpMethod::PATCH, url, envelope).await
    }

    /// Send OPTIONS request with envelope
    pub async fn options(&self, url: &str, envelope: WasmEnvelope) -> Result<WasmEnvelope> {
        self.send_envelope_with_method(HttpMethod::OPTIONS, url, envelope).await
    }

    /// Perform the actual HTTP request
    async fn perform_request(
        &self,
        method: &HttpMethod,
        url: &str,
        envelope: &WasmEnvelope,
    ) -> Result<WasmEnvelope> {
        let window = web_sys::window()
            .ok_or_else(|| QollectiveError::environment("No window object available"))?;

        // Build headers from envelope and config
        let headers = self.build_headers(envelope).await?;

        // Serialize envelope to JSON for request body (except for GET/OPTIONS)
        let body = match method {
            HttpMethod::GET | HttpMethod::OPTIONS => None,
            _ => {
                let json_str = serde_json::to_string(envelope)
                    .map_err(|e| QollectiveError::serialization(format!("Failed to serialize envelope: {}", e)))?;
                Some(json_str)
            }
        };

        // Create request init
        let mut request_init = RequestInit::new();
        request_init.method(&method.to_string());
        request_init.headers(&headers);
        request_init.mode(RequestMode::Cors);

        // Set body if present
        if let Some(body_str) = &body {
            request_init.body(Some(&JsValue::from_str(body_str)));
        }

        // Create and send request
        let request = Request::new_with_str_and_init(url, &request_init)
            .map_err(|_| QollectiveError::transport("Failed to create request".to_string()))?;

        let response_promise = window.fetch_with_request(&request);
        let response = JsFuture::from(response_promise).await
            .map_err(|_| QollectiveError::transport("Request failed".to_string()))?;

        let response: Response = response.dyn_into()
            .map_err(|_| QollectiveError::transport("Invalid response type".to_string()))?;

        // Check response status
        if !response.ok() {
            return Err(QollectiveError::transport(format!(
                "HTTP error: {} {}",
                response.status(),
                response.status_text()
            )));
        }

        // Parse response body
        let json_promise = response.json()
            .map_err(|_| QollectiveError::transport("Failed to read response".to_string()))?;
        
        let json_value = JsFuture::from(json_promise).await
            .map_err(|_| QollectiveError::transport("Failed to parse JSON".to_string()))?;

        // Convert JS value back to envelope
        let response_envelope: WasmEnvelope = serde_wasm_bindgen::from_value(json_value)
            .map_err(|e| QollectiveError::deserialization(format!("Failed to deserialize response: {}", e)))?;

        Ok(response_envelope)
    }

    /// Build HTTP headers from envelope metadata and config
    async fn build_headers(&self, envelope: &WasmEnvelope) -> Result<Headers> {
        let headers = Headers::new()
            .map_err(|_| QollectiveError::transport("Failed to create headers".to_string()))?;

        // Content-Type for JSON
        headers.set("Content-Type", "application/json")
            .map_err(|_| QollectiveError::transport("Failed to set content type".to_string()))?;

        // User-Agent from config
        headers.set("User-Agent", &self.config.user_agent)
            .map_err(|_| QollectiveError::transport("Failed to set user agent".to_string()))?;

        // Default headers from config
        for (key, value) in &self.config.default_headers {
            headers.set(key, value)
                .map_err(|_| QollectiveError::transport(format!("Failed to set header {}", key)))?;
        }

        // Envelope metadata headers
        let meta = envelope.meta();
        
        // Request ID header
        if let Some(request_id) = meta.request_id() {
            headers.set("X-Request-ID", &request_id.to_string())
                .map_err(|_| QollectiveError::transport("Failed to set request ID header".to_string()))?;
        }

        // Tenant header
        if let Some(tenant) = meta.tenant() {
            headers.set(&self.config.jwt_config.tenant_header_name, tenant)
                .map_err(|_| QollectiveError::transport("Failed to set tenant header".to_string()))?;
        }

        // JWT Authorization header (if available in context)
        if let Some(context) = meta.context() {
            if let Some(jwt_token) = context.get("jwt_token").and_then(|v| v.as_str()) {
                let auth_value = format!("{}{}", self.config.jwt_config.header_prefix, jwt_token);
                headers.set(&self.config.jwt_config.header_name, &auth_value)
                    .map_err(|_| QollectiveError::transport("Failed to set JWT header".to_string()))?;
            }
        }

        // mTLS certificate header (if configured)
        if let Some(cert_bundle) = self.get_certificate_for_url(envelope.meta().request_id().map(|id| id.to_string()).as_deref().unwrap_or("default")).await? {
            let cert_header = cert_bundle.to_client_cert_header()?;
            headers.set("X-Client-Cert", &cert_header)
                .map_err(|_| QollectiveError::transport("Failed to set certificate header".to_string()))?;
        }

        Ok(headers)
    }

    /// Get certificate bundle for URL domain
    async fn get_certificate_for_url(&self, _request_context: &str) -> Result<Option<CertificateBundle>> {
        // For now, return the first available certificate
        // In a real implementation, this would parse the URL and match domains
        let cert_names = self.cert_manager.list_certificates();
        if let Some(first_cert) = cert_names.first() {
            if let Some(cert_info) = self.cert_manager.get_certificate_info(first_cert)? {
                if cert_info.valid() {
                    return self.cert_manager.get_certificate_for_domain(&cert_info.domains()[0]);
                }
            }
        }
        Ok(None)
    }

    /// Calculate retry delay based on configuration
    fn calculate_retry_delay(&self, config: &RetryConfig, attempt: u32) -> u64 {
        if config.exponential_backoff {
            let delay = config.base_delay_ms * (2_u64.pow(attempt));
            delay.min(config.max_delay_ms)
        } else {
            config.base_delay_ms
        }
    }

    /// Sleep for specified milliseconds using setTimeout
    async fn sleep_ms(&self, ms: u64) {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let closure = Closure::once_into_js(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            });
            
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    ms as i32,
                )
                .unwrap();
        });

        JsFuture::from(promise).await.unwrap();
    }

    /// Test connectivity to endpoint
    pub async fn test_connectivity(&self, url: &str) -> Result<ConnectivityResult> {
        let start_time = js_sys::Date::now() as u64;

        // Create a simple health check envelope
        let meta = WasmMeta::with_auto_fields();
        let envelope = WasmEnvelope::new(meta, serde_wasm_bindgen::to_value(&serde_json::json!({
            "action": "health_check",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))?)?;

        match self.send_envelope_with_method(HttpMethod::OPTIONS, url, envelope).await {
            Ok(_) => {
                let end_time = js_sys::Date::now() as u64;
                Ok(ConnectivityResult {
                    response_time_ms: end_time - start_time,
                    status_code: 200,
                    success: true,
                })
            }
            Err(_) => {
                let end_time = js_sys::Date::now() as u64;
                
                // For connectivity test, we don't fail completely - we return info about the failure
                Ok(ConnectivityResult {
                    response_time_ms: end_time - start_time,
                    status_code: 0, // Unknown status
                    success: false,
                })
            }
        }
    }

    /// Get client configuration
    pub fn config(&self) -> &RestClientConfig {
        &self.config
    }

    /// Get certificate manager
    pub fn cert_manager(&self) -> &WasmCertificateManager {
        &self.cert_manager
    }
}

/// Connectivity test result
#[derive(Debug, Clone)]
pub struct ConnectivityResult {
    pub response_time_ms: u64,
    pub status_code: u16,
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::wasm::CertificateConfig;
    use crate::wasm::js_types::WasmMeta;

    fn create_test_config() -> RestClientConfig {
        use crate::client::common::ClientConfig;
        
        RestClientConfig {
            base: ClientConfig {
                base_url: "https://api.example.com".to_string(),
                timeout_seconds: timeouts::DEFAULT_REST_REQUEST_TIMEOUT_MS / 1000,
                retry_attempts: limits::DEFAULT_RETRY_ATTEMPTS,
                ..ClientConfig::default()
            },
            user_agent: "qollective-wasm-test/1.0".to_string(),
            ..RestClientConfig::default()
        }
    }

    fn create_test_cert_manager() -> WasmCertificateManager {
        let cert_config = CertificateConfig::default();
        WasmCertificateManager::new(&cert_config).unwrap()
    }

    #[test]
    fn test_wasm_rest_client_creation() {
        let config = create_test_config();
        let cert_manager = create_test_cert_manager();
        
        let client = WasmRestClient::new(config, cert_manager);
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.config().base.base_url, "https://api.example.com");
        assert_eq!(client.config().base.retry_attempts, limits::DEFAULT_RETRY_ATTEMPTS);
    }

    #[test]
    fn test_http_method_to_string() {
        assert_eq!(HttpMethod::GET.to_string(), "GET");
        assert_eq!(HttpMethod::POST.to_string(), "POST");
        assert_eq!(HttpMethod::PUT.to_string(), "PUT");
        assert_eq!(HttpMethod::DELETE.to_string(), "DELETE");
        assert_eq!(HttpMethod::PATCH.to_string(), "PATCH");
        assert_eq!(HttpMethod::OPTIONS.to_string(), "OPTIONS");
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, limits::DEFAULT_RETRY_ATTEMPTS);
        assert_eq!(config.base_delay_ms, timeouts::DEFAULT_REST_RETRY_DELAY_MS);
        assert_eq!(config.max_delay_ms, timeouts::DEFAULT_REST_MAX_RETRY_DELAY_MS);
        assert!(config.exponential_backoff);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = create_test_config();
        let cert_manager = create_test_cert_manager();
        let client = WasmRestClient::new(config, cert_manager).unwrap();
        
        let retry_config = RetryConfig {
            base_delay_ms: 1000,
            max_delay_ms: 10000,
            exponential_backoff: true,
            max_attempts: 3,
        };

        // Test exponential backoff
        assert_eq!(client.calculate_retry_delay(&retry_config, 0), 1000);  // 1000 * 2^0
        assert_eq!(client.calculate_retry_delay(&retry_config, 1), 2000);  // 1000 * 2^1
        assert_eq!(client.calculate_retry_delay(&retry_config, 2), 4000);  // 1000 * 2^2
        assert_eq!(client.calculate_retry_delay(&retry_config, 4), 10000); // Capped at max

        // Test linear backoff
        let linear_config = RetryConfig {
            base_delay_ms: 1000,
            exponential_backoff: false,
            ..retry_config
        };
        
        assert_eq!(client.calculate_retry_delay(&linear_config, 0), 1000);
        assert_eq!(client.calculate_retry_delay(&linear_config, 1), 1000);
        assert_eq!(client.calculate_retry_delay(&linear_config, 2), 1000);
    }

    #[test]
    fn test_config_validation() {
        let mut config = create_test_config();
        config.base.base_url = "".to_string(); // Invalid empty URL
        
        let cert_manager = create_test_cert_manager();
        let result = WasmRestClient::new(config, cert_manager);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Base URL cannot be empty"));
    }

    #[test]
    fn test_connectivity_result() {
        let result = ConnectivityResult {
            response_time_ms: 150,
            status_code: 200,
            success: true,
        };

        assert_eq!(result.response_time_ms, 150);
        assert_eq!(result.status_code, 200);
        assert!(result.success);
    }

    // Note: Browser-based tests would require wasm-bindgen-test framework
    // These tests verify the structure and configuration handling
}
