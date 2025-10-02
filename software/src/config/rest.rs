// ABOUTME: REST-specific configuration utilities and helpers
// ABOUTME: URL management, header manipulation, logging, and performance utilities

//! REST-specific configuration utilities and helpers.

use super::presets::{LoggingConfig, PerformanceConfig, RestClientConfig, RestServerConfig};
use crate::error::{QollectiveError, Result};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

/// URL and endpoint management utilities
pub struct UrlManager {
    base_url: Option<String>,
    path_parameters: HashMap<String, String>,
    query_parameters: HashMap<String, String>,
}

impl UrlManager {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            base_url,
            path_parameters: HashMap::new(),
            query_parameters: HashMap::new(),
        }
    }

    pub fn with_base_url(base_url: &str) -> Self {
        Self::new(Some(base_url.to_string()))
    }

    pub fn add_path_parameter<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.path_parameters.insert(key.into(), value.into());
        self
    }

    pub fn add_query_parameter<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.query_parameters.insert(key.into(), value.into());
        self
    }

    pub fn build_url(&self, endpoint: &str) -> Result<String> {
        let base = self.base_url.as_deref().unwrap_or("");
        let mut url = if base.is_empty() {
            endpoint.to_string()
        } else {
            format!(
                "{}/{}",
                base.trim_end_matches('/'),
                endpoint.trim_start_matches('/')
            )
        };

        // Replace path parameters
        for (key, value) in &self.path_parameters {
            let placeholder = format!("{{{}}}", key);
            url = url.replace(&placeholder, value);
        }

        // Add query parameters
        if !self.query_parameters.is_empty() {
            let query_string: Vec<String> = self
                .query_parameters
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect();
            url = format!("{}?{}", url, query_string.join("&"));
        }

        Ok(url)
    }

    pub fn validate_url(&self, url: &str) -> Result<()> {
        if url.is_empty() {
            return Err(QollectiveError::Internal("URL cannot be empty".to_string()));
        }

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with('/') {
            return Err(QollectiveError::Internal(
                "URL must be absolute or start with /".to_string(),
            ));
        }

        Ok(())
    }
}

/// Header manipulation utilities
pub struct HeaderManager {
    headers: HashMap<String, String>,
}

impl HeaderManager {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    pub fn from_config(config: &RestClientConfig) -> Self {
        Self {
            headers: config.default_headers.clone(),
        }
    }

    pub fn add_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn add_authorization_bearer(self, token: &str) -> Self {
        self.add_header("Authorization", format!("Bearer {}", token))
    }

    pub fn add_content_type(self, content_type: &str) -> Self {
        self.add_header("Content-Type", content_type)
    }

    pub fn add_accept(self, accept: &str) -> Self {
        self.add_header("Accept", accept)
    }

    pub fn add_user_agent(self, user_agent: &str) -> Self {
        self.add_header("User-Agent", user_agent)
    }

    pub fn add_custom_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }

    pub fn remove_header(mut self, key: &str) -> Self {
        self.headers.remove(key);
        self
    }

    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    pub fn get_headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn into_headers(self) -> HashMap<String, String> {
        self.headers
    }

    pub fn merge_with(&mut self, other: &HeaderManager) {
        self.headers.extend(other.headers.clone());
    }

    pub fn validate_headers(&self) -> Result<()> {
        for (key, value) in &self.headers {
            if key.is_empty() {
                return Err(QollectiveError::Internal(
                    "Header key cannot be empty".to_string(),
                ));
            }
            if value.contains('\n') || value.contains('\r') {
                return Err(QollectiveError::Internal(format!(
                    "Header value for '{}' contains invalid characters",
                    key
                )));
            }
        }
        Ok(())
    }
}

impl Default for HeaderManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Request/response logging utilities
pub struct RequestLogger {
    config: LoggingConfig,
}

impl RequestLogger {
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }

    pub fn should_log_requests(&self) -> bool {
        self.config.enabled && self.config.log_requests
    }

    pub fn should_log_responses(&self) -> bool {
        self.config.enabled && self.config.log_responses
    }

    pub fn should_log_headers(&self) -> bool {
        self.config.enabled && self.config.log_headers
    }

    pub fn should_log_body(&self) -> bool {
        self.config.enabled && self.config.log_body
    }

    pub fn log_request(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: Option<&str>,
    ) {
        if !self.should_log_requests() {
            return;
        }

        // For now, we'll just suppress the parameters since we don't have tracing enabled
        // In a real implementation, this would log to the configured logging system
        let _ = (method, url, headers, body); // Suppress unused warnings
    }

    pub fn log_response(
        &self,
        status: u16,
        headers: &HashMap<String, String>,
        body: Option<&str>,
        duration: Duration,
    ) {
        if !self.should_log_responses() {
            return;
        }

        // For now, we'll just suppress the parameters since we don't have tracing enabled
        // In a real implementation, this would log to the configured logging system
        let _ = (status, headers, body, duration); // Suppress unused warnings
    }

    pub fn log_error(&self, error: &QollectiveError, url: &str) {
        if self.config.enabled {
            // For now, we'll just suppress the parameters since we don't have tracing enabled
            // In a real implementation, this would log to the configured logging system
            let _ = (error, url); // Suppress unused warnings
        }
    }
}

/// Performance benchmarking helpers
pub struct PerformanceBenchmark {
    config: PerformanceConfig,
    start_time: Option<Instant>,
    metrics: HashMap<String, f64>,
}

impl PerformanceBenchmark {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            start_time: None,
            metrics: HashMap::new(),
        }
    }

    pub fn start(&mut self) {
        if self.config.enabled && self.config.benchmarking_enabled {
            self.start_time = Some(Instant::now());
        }
    }

    pub fn end(&mut self) -> Option<Duration> {
        if let Some(start) = self.start_time.take() {
            let duration = start.elapsed();
            if self.config.track_request_duration {
                self.metrics.insert(
                    "request_duration_ms".to_string(),
                    duration.as_millis() as f64,
                );
            }
            Some(duration)
        } else {
            None
        }
    }

    pub fn record_response_size(&mut self, size_bytes: usize) {
        if self.config.enabled && self.config.track_response_size {
            self.metrics
                .insert("response_size_bytes".to_string(), size_bytes as f64);
        }
    }

    pub fn record_connection_pool_size(
        &mut self,
        active_connections: usize,
        idle_connections: usize,
    ) {
        if self.config.enabled && self.config.track_connection_pool {
            self.metrics
                .insert("active_connections".to_string(), active_connections as f64);
            self.metrics
                .insert("idle_connections".to_string(), idle_connections as f64);
        }
    }

    pub fn record_custom_metric<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<f64>,
    {
        if self.config.enabled && self.config.metrics_collection {
            self.metrics.insert(key.into(), value.into());
        }
    }

    pub fn get_metrics(&self) -> &HashMap<String, f64> {
        &self.metrics
    }

    pub fn into_metrics(self) -> HashMap<String, f64> {
        self.metrics
    }

    pub fn report_metrics(&self) {
        if self.config.enabled && self.config.metrics_collection && !self.metrics.is_empty() {
            // For now, we'll just suppress since we don't have tracing enabled
            // In a real implementation, this would report to the configured metrics system
            let _ = &self.metrics; // Suppress unused warning
        }
    }
}

impl fmt::Display for PerformanceBenchmark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.metrics.is_empty() {
            write!(f, "No metrics recorded")
        } else {
            let metrics_str: Vec<String> = self
                .metrics
                .iter()
                .map(|(k, v)| format!("{}={:.2}", k, v))
                .collect();
            write!(f, "Metrics: {}", metrics_str.join(", "))
        }
    }
}

/// REST configuration builder with environment variable support
pub struct RestConfigBuilder {
    client_config: Option<RestClientConfig>,
    server_config: Option<RestServerConfig>,
}

impl RestConfigBuilder {
    pub fn new() -> Self {
        Self {
            client_config: None,
            server_config: None,
        }
    }

    pub fn with_client_config(mut self, config: RestClientConfig) -> Self {
        self.client_config = Some(config);
        self
    }

    pub fn with_server_config(mut self, config: RestServerConfig) -> Self {
        self.server_config = Some(config);
        self
    }

    pub fn apply_environment_overrides(mut self) -> Self {
        // Apply environment variable overrides to client config
        if let Some(ref mut client_config) = self.client_config {
            if let Ok(timeout) = std::env::var("QOLLECTIVE_REST_TIMEOUT") {
                if let Ok(timeout_ms) = timeout.parse::<u64>() {
                    client_config.timeout_ms = timeout_ms;
                }
            }

            if let Ok(max_conn) = std::env::var("QOLLECTIVE_REST_MAX_CONNECTIONS") {
                if let Ok(max_connections) = max_conn.parse::<usize>() {
                    client_config.max_connections = max_connections;
                }
            }

            if let Ok(base_url) = std::env::var("QOLLECTIVE_REST_BASE_URL") {
                client_config.base_url = Some(base_url);
            }

            if let Ok(user_agent) = std::env::var("QOLLECTIVE_REST_USER_AGENT") {
                client_config.user_agent = user_agent;
            }

            // Apply TLS environment overrides
            if let Ok(tls_enabled) = std::env::var("QOLLECTIVE_REST_TLS_ENABLED") {
                if let Ok(enabled) = tls_enabled.parse::<bool>() {
                    client_config.tls.enabled = enabled;
                }
            }

            if let Ok(cert_path) = std::env::var("QOLLECTIVE_REST_TLS_CERT_PATH") {
                client_config.tls.cert_path = Some(cert_path.into());
            }

            if let Ok(key_path) = std::env::var("QOLLECTIVE_REST_TLS_KEY_PATH") {
                client_config.tls.key_path = Some(key_path.into());
            }

            if let Ok(ca_path) = std::env::var("QOLLECTIVE_REST_TLS_CA_PATH") {
                client_config.tls.ca_cert_path = Some(ca_path.into());
            }

            if let Ok(verify) = std::env::var("QOLLECTIVE_REST_TLS_VERIFY") {
                if let Ok(verify_certs) = verify.parse::<bool>() {
                    client_config.tls.verification_mode = if verify_certs {
                        crate::config::tls::VerificationMode::SystemCa
                    } else {
                        crate::config::tls::VerificationMode::Skip
                    };
                }
            }
        }

        // Apply environment variable overrides to server config
        if let Some(ref mut server_config) = self.server_config {
            if let Ok(bind_addr) = std::env::var("QOLLECTIVE_REST_BIND_ADDRESS") {
                server_config.bind_address = bind_addr;
            }

            if let Ok(port) = std::env::var("QOLLECTIVE_REST_PORT") {
                if let Ok(port_num) = port.parse::<u16>() {
                    server_config.port = port_num;
                }
            }

            if let Ok(max_conn) = std::env::var("QOLLECTIVE_REST_SERVER_MAX_CONNECTIONS") {
                if let Ok(max_connections) = max_conn.parse::<usize>() {
                    server_config.max_connections = max_connections;
                }
            }

            if let Ok(timeout) = std::env::var("QOLLECTIVE_REST_REQUEST_TIMEOUT") {
                if let Ok(timeout_ms) = timeout.parse::<u64>() {
                    server_config.request_timeout_ms = timeout_ms;
                }
            }
        }

        self
    }

    pub fn build_client_config(&self) -> Option<RestClientConfig> {
        self.client_config.clone()
    }

    pub fn build_server_config(&self) -> Option<RestServerConfig> {
        self.server_config.clone()
    }
}

impl Default for RestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_manager_basic_url_building() {
        let manager = UrlManager::with_base_url("https://api.example.com");
        let url = manager.build_url("users").unwrap();
        assert_eq!(url, "https://api.example.com/users");
    }

    #[test]
    fn test_url_manager_with_path_parameters() {
        let manager =
            UrlManager::with_base_url("https://api.example.com").add_path_parameter("id", "123");
        let url = manager.build_url("users/{id}").unwrap();
        assert_eq!(url, "https://api.example.com/users/123");
    }

    #[test]
    fn test_url_manager_with_query_parameters() {
        let manager = UrlManager::with_base_url("https://api.example.com")
            .add_query_parameter("page", "1")
            .add_query_parameter("limit", "10");
        let url = manager.build_url("users").unwrap();
        assert!(url.contains("page=1"));
        assert!(url.contains("limit=10"));
        assert!(url.contains("?"));
    }

    #[test]
    fn test_header_manager_basic_operations() {
        let manager = HeaderManager::new()
            .add_content_type("application/json")
            .add_authorization_bearer("token123");

        assert_eq!(
            manager.get_header("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            manager.get_header("Authorization"),
            Some(&"Bearer token123".to_string())
        );
    }

    #[test]
    fn test_header_manager_validation() {
        let manager = HeaderManager::new().add_header("Valid-Header", "valid-value");
        assert!(manager.validate_headers().is_ok());

        let invalid_manager =
            HeaderManager::new().add_header("Invalid-Header", "value\nwith\nnewlines");
        assert!(invalid_manager.validate_headers().is_err());
    }

    #[test]
    fn test_performance_benchmark() {
        let config = PerformanceConfig {
            enabled: true,
            track_request_duration: true,
            track_response_size: true,
            track_connection_pool: false,
            benchmarking_enabled: true,
            metrics_collection: true,
        };

        let mut benchmark = PerformanceBenchmark::new(config);
        benchmark.start();
        std::thread::sleep(Duration::from_millis(1));
        let duration = benchmark.end();

        assert!(duration.is_some());
        assert!(duration.unwrap().as_millis() >= 1);

        benchmark.record_response_size(1024);
        assert!(benchmark.get_metrics().contains_key("response_size_bytes"));
        assert_eq!(benchmark.get_metrics()["response_size_bytes"], 1024.0);
    }
}
