// ABOUTME: Clean REST server implementation following NATS architecture patterns with single file organization
// ABOUTME: Provides configuration, metadata handling, HTTP integration, and core server functionality in unified structure

//! REST/HTTP server implementation with clean architecture.
//!
//! This module provides a simplified REST server following the NATS server's
//! architectural patterns: single construction method, configuration-based design,
//! and native UnifiedEnvelopeReceiver integration.

#[cfg(feature = "rest-server")]
use crate::{
    config::tls::TlsConfig,
    constants::{http::{envelope_headers, envelope_query_params}, metadata::PROTOCOL_EXTENSION_KEY},
    envelope::{Context, Envelope, EnvelopeError, Meta},
    error::{QollectiveError, Result},
    server::common::ServerConfig,
    traits::{handlers::ContextDataHandler, receivers::UnifiedEnvelopeReceiver},
};

#[cfg(feature = "rest-server")]
use async_trait::async_trait;

#[cfg(feature = "rest-server")]
use tokio::net::TcpListener;

#[cfg(feature = "rest-server")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "rest-server")]
use std::{collections::HashMap, sync::Arc, time::Duration};

#[cfg(feature = "rest-server")]
use axum::{
    extract::Query,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{delete, get, options, patch, post, put},
    Json, Router,
};

#[cfg(feature = "rest-server")]
use tower_http::trace::TraceLayer;

#[cfg(feature = "rest-server")]
use base64::prelude::*;

#[cfg(feature = "rest-server")]
use uuid::Uuid;

#[cfg(feature = "rest-server")]
use serde_json::Value;

// =============================================================================
// CONFIGURATION TYPES
// =============================================================================

/// Configuration for REST server behavior
#[cfg(feature = "rest-server")]
#[derive(Debug, Clone)]
pub struct RestServerConfig {
    /// Base server configuration (bind address, port, etc.)
    pub base: ServerConfig,
    /// TLS configuration for secure connections
    pub tls: Option<TlsConfig>,
    /// CORS configuration
    pub cors: Option<CorsConfig>,
    /// HTTP metadata handling configuration
    pub metadata: MetadataHandlingConfig,
    /// Request timeout configuration
    pub request_timeout: Option<Duration>,
}

// TLS configuration is now provided by the unified crate::tls::TlsConfig

/// CORS configuration options
#[cfg(feature = "rest-server")]
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins (empty = any origin)
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    /// Whether credentials are allowed
    pub allow_credentials: bool,
}

/// Configuration for HTTP metadata handling
#[cfg(feature = "rest-server")]
#[derive(Debug, Clone)]
pub struct MetadataHandlingConfig {
    /// Maximum size per HTTP header (configurable)
    pub max_header_size: usize,
    /// Maximum total size of all headers (configurable)
    pub max_total_headers: usize,
    /// Encoding format for metadata in headers
    pub encoding: MetadataEncoding,
}

/// Metadata encoding formats for HTTP headers
#[cfg(feature = "rest-server")]
#[derive(Debug, Clone)]
pub enum MetadataEncoding {
    /// Base64 encoding (default)
    Base64,
    /// JSON encoding (for debugging)
    Json,
}

#[cfg(feature = "rest-server")]
impl Default for RestServerConfig {
    fn default() -> Self {
        Self {
            base: ServerConfig::default(),
            tls: None,
            cors: Some(CorsConfig::permissive()),
            metadata: MetadataHandlingConfig::default(),
            request_timeout: Some(Duration::from_secs(30)),
        }
    }
}

#[cfg(feature = "rest-server")]
impl Default for MetadataHandlingConfig {
    fn default() -> Self {
        Self {
            max_header_size: 4_096,    // 4KB per header (conservative)
            max_total_headers: 65_536, // 64KB total headers
            encoding: MetadataEncoding::Base64,
        }
    }
}

#[cfg(feature = "rest-server")]
impl CorsConfig {
    /// Create permissive CORS configuration for development
    pub fn permissive() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
                "PATCH".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
            allow_credentials: false,
        }
    }

    /// Create strict CORS configuration for production
    pub fn strict(origins: Vec<String>) -> Self {
        Self {
            allowed_origins: origins,
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            allow_credentials: true,
        }
    }
}

/// Helper function to create axum-server RustlsConfig from unified TLS config
#[cfg(all(feature = "rest-server", feature = "tls"))]
async fn create_axum_rustls_config(
    tls_config: &TlsConfig,
) -> Result<axum_server::tls_rustls::RustlsConfig> {
    use axum_server::tls_rustls::RustlsConfig;

    // Use the unified TLS config to create server configuration
    let server_config = tls_config.create_server_config().await?;

    // Convert Arc<rustls::ServerConfig> to axum_server::tls_rustls::RustlsConfig
    Ok(RustlsConfig::from_config(server_config))
}

// =============================================================================
// PROTOCOL EXTENSION TYPES
// =============================================================================

/// REST protocol metadata for envelope extensions
/// 
/// This struct provides type-safe access to REST-specific protocol information
/// that gets injected into envelope extensions. Other teams can import and use
/// this struct to create protocol metadata consistently.
/// 
/// # Example
/// ```rust
/// use qollective::server::rest::RestProtocolMetadata;
/// use std::collections::HashMap;
/// 
/// let protocol = RestProtocolMetadata {
///     protocol_type: "rest".to_string(),
///     method: "GET".to_string(),
///     uri_path: "/api/v1/users".to_string(),
///     query_params: Some({
///         let mut params = HashMap::new();
///         params.insert("page".to_string(), "1".to_string());
///         params
///     }),
///     headers: None,
/// };
/// ```
#[cfg(feature = "rest-server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestProtocolMetadata {
    /// Protocol type identifier, always "rest" for REST protocol extensions
    #[serde(rename = "type")]
    pub protocol_type: String,
    
    /// HTTP method used for the request (GET, POST, PUT, DELETE, etc.)
    pub method: String,
    
    /// URI path component starting with forward slash, excluding query parameters
    pub uri_path: String,
    
    /// Optional query parameters as key-value string pairs from request URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_params: Option<HashMap<String, String>>,
    
    /// Optional subset of HTTP headers relevant to application logic
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

#[cfg(feature = "rest-server")]
impl RestProtocolMetadata {
    /// Create a new REST protocol metadata instance
    pub fn new(method: String, uri_path: String) -> Self {
        Self {
            protocol_type: "rest".to_string(),
            method,
            uri_path,
            query_params: None,
            headers: None,
        }
    }

    /// Create REST protocol metadata with query parameters
    pub fn with_query_params(
        method: String,
        uri_path: String,
        query_params: HashMap<String, String>,
    ) -> Self {
        Self {
            protocol_type: "rest".to_string(),
            method,
            uri_path,
            query_params: Some(query_params),
            headers: None,
        }
    }

    /// Create REST protocol metadata with headers
    pub fn with_headers(
        method: String,
        uri_path: String,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            protocol_type: "rest".to_string(),
            method,
            uri_path,
            query_params: None,
            headers: Some(headers),
        }
    }

    /// Create REST protocol metadata with both query parameters and headers
    pub fn with_all(
        method: String,
        uri_path: String,
        query_params: HashMap<String, String>,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            protocol_type: "rest".to_string(),
            method,
            uri_path,
            query_params: Some(query_params),
            headers: Some(headers),
        }
    }
}

// =============================================================================
// METADATA HANDLING
// =============================================================================

/// Errors that can occur during metadata handling
#[cfg(feature = "rest-server")]
#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("Headers too large: {current} bytes exceeds limit of {max} bytes")]
    HeadersTooLarge { current: usize, max: usize },

    #[error("Individual header too large: {current} bytes exceeds limit of {max} bytes")]
    HeaderTooLarge { current: usize, max: usize },

    #[error("Failed to encode metadata: {0}")]
    EncodingError(String),

    #[error("Failed to decode metadata: {0}")]
    DecodingError(String),

    #[error("Invalid header name: {0}")]
    InvalidHeaderName(String),
}

/// Convert Axum HeaderMap to HashMap for protocol metadata storage
/// 
/// This function preserves HTTP headers so they can be accessed by business logic handlers
/// through the envelope's protocol metadata extensions. Essential for JWT authentication
/// and custom header processing.
#[cfg(feature = "rest-server")]
fn convert_headermap_to_hashmap(headers: &HeaderMap) -> HashMap<String, String> {
    let mut header_map = HashMap::new();
    
    for (name, value) in headers.iter() {
        let header_name = name.as_str().to_lowercase();
        
        // Convert header value to string, handling potential encoding issues
        match value.to_str() {
            Ok(header_value) => {
                // For multi-value headers, join with comma (HTTP standard)
                header_map
                    .entry(header_name.clone())
                    .and_modify(|existing| {
                        *existing = format!("{}, {}", existing, header_value);
                    })
                    .or_insert_with(|| header_value.to_string());
            }
            Err(_) => {
                // If header contains non-UTF8 data, base64 encode it
                let encoded_value = BASE64_STANDARD.encode(value.as_bytes());
                header_map.insert(
                    header_name,
                    format!("base64:{}", encoded_value)
                );
            }
        }
    }
    
    header_map
}

/// Extract envelope metadata from HTTP headers and query parameters
#[cfg(feature = "rest-server")]
pub fn extract_metadata_from_http(
    headers: &HeaderMap,
    query_params: &HashMap<String, String>,
    config: &MetadataHandlingConfig,
) -> Result<Meta> {
    // Validate total header size first
    let total_size = calculate_total_header_size(headers);
    if total_size > config.max_total_headers {
        return Err(QollectiveError::envelope(format!(
            "Headers too large: {} bytes exceeds limit of {} bytes",
            total_size, config.max_total_headers
        )));
    }

    let mut meta = Meta::default();

    // Extract request ID from headers or query params (no Base64 decoding needed)
    if let Some(request_id_value) = headers.get(envelope_headers::QOLLECTIVE_REQUEST_ID) {
        if let Ok(request_id_str) = request_id_value.to_str() {
            if let Ok(request_id) = Uuid::parse_str(request_id_str) {
                meta.request_id = Some(request_id);
            }
        }
    } else if let Some(request_id_str) = query_params.get(envelope_query_params::REQUEST_ID) {
        if let Ok(request_id) = Uuid::parse_str(request_id_str) {
            meta.request_id = Some(request_id);
        }
    }

    // Extract tenant from headers or query params
    if let Some(tenant_value) = headers.get(envelope_headers::QOLLECTIVE_TENANT) {
        if let Ok(tenant_str) = tenant_value.to_str() {
            meta.tenant = Some(decode_metadata_value(tenant_str, &config.encoding)?);
        }
    } else if let Some(tenant) = query_params.get(envelope_query_params::TENANT) {
        meta.tenant = Some(tenant.clone());
    }

    // Extract version from headers or query params
    if let Some(version_value) = headers.get(envelope_headers::QOLLECTIVE_VERSION) {
        if let Ok(version_str) = version_value.to_str() {
            meta.version = Some(decode_metadata_value(version_str, &config.encoding)?);
        }
    } else if let Some(version) = query_params.get(envelope_query_params::VERSION) {
        meta.version = Some(version.clone());
    }

    // Extract complex metadata from headers
    if let Some(meta_value) = headers.get(envelope_headers::QOLLECTIVE_META) {
        if let Ok(meta_str) = meta_value.to_str() {
            // Check individual header size
            if meta_str.len() > config.max_header_size {
                return Err(QollectiveError::envelope(format!(
                    "Header too large: {} bytes exceeds limit of {} bytes",
                    meta_str.len(),
                    config.max_header_size
                )));
            }

            if let Ok(additional_meta) = decode_complex_metadata(meta_str, &config.encoding) {
                // Merge additional metadata into meta
                merge_metadata(&mut meta, additional_meta);
            }
        }
    }

    // Set timestamp if not already set
    if meta.timestamp.is_none() {
        meta.timestamp = Some(chrono::Utc::now());
    }

    Ok(meta)
}

/// Inject envelope metadata into HTTP headers
#[cfg(feature = "rest-server")]
pub fn inject_metadata_into_headers(
    meta: &Meta,
    config: &MetadataHandlingConfig,
) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    // Inject request ID (no encoding needed)
    if let Some(request_id) = &meta.request_id {
        inject_header(
            &mut headers,
            envelope_headers::QOLLECTIVE_REQUEST_ID,
            &request_id.to_string(),
            config,
        )?;
    }

    // Inject tenant
    if let Some(tenant) = &meta.tenant {
        let encoded_tenant = encode_metadata_value(tenant, &config.encoding)?;
        inject_header(
            &mut headers,
            envelope_headers::QOLLECTIVE_TENANT,
            &encoded_tenant,
            config,
        )?;
    }

    // Inject version
    if let Some(version) = &meta.version {
        let encoded_version = encode_metadata_value(version, &config.encoding)?;
        inject_header(
            &mut headers,
            envelope_headers::QOLLECTIVE_VERSION,
            &encoded_version,
            config,
        )?;
    }

    // Inject timestamp (no encoding needed)
    if let Some(timestamp) = &meta.timestamp {
        inject_header(
            &mut headers,
            envelope_headers::QOLLECTIVE_TIMESTAMP,
            &timestamp.to_rfc3339(),
            config,
        )?;
    }

    Ok(headers)
}

/// Extract metadata from POST/PUT request body envelope
#[cfg(feature = "rest-server")]
pub fn extract_metadata_from_envelope<T>(envelope: &Envelope<T>) -> Meta {
    envelope.meta.clone()
}

/// Create envelope with metadata for response
#[cfg(feature = "rest-server")]
pub fn create_response_envelope<T>(data: T, request_meta: Meta) -> Envelope<T> {
    // Create response metadata using the proper preservation utility
    // This follows the same pattern as WebSocket, gRPC and MCP servers for consistency
    let response_meta = crate::envelope::Meta::preserve_for_response(Some(&request_meta));
    Envelope::new(response_meta, data)
}

// Metadata helper functions
#[cfg(feature = "rest-server")]
fn calculate_total_header_size(headers: &HeaderMap) -> usize {
    headers
        .iter()
        .map(|(name, value)| name.as_str().len() + value.len())
        .sum()
}

#[cfg(feature = "rest-server")]
fn inject_header(
    headers: &mut HeaderMap,
    name: &str,
    value: &str,
    config: &MetadataHandlingConfig,
) -> Result<()> {
    // Check individual header size
    if value.len() > config.max_header_size {
        return Err(QollectiveError::envelope(format!(
            "Header value too large: {} bytes exceeds limit of {} bytes",
            value.len(),
            config.max_header_size
        )));
    }

    let header_name = HeaderName::from_bytes(name.as_bytes())
        .map_err(|e| QollectiveError::envelope(format!("Invalid header name {}: {}", name, e)))?;

    let header_value = HeaderValue::from_str(value)
        .map_err(|e| QollectiveError::envelope(format!("Invalid header value: {}", e)))?;

    headers.insert(header_name, header_value);
    Ok(())
}

#[cfg(feature = "rest-server")]
fn encode_metadata_value(value: &str, encoding: &MetadataEncoding) -> Result<String> {
    match encoding {
        MetadataEncoding::Base64 => Ok(base64::prelude::BASE64_STANDARD.encode(value.as_bytes())),
        MetadataEncoding::Json => Ok(value.to_string()),
    }
}

#[cfg(feature = "rest-server")]
fn decode_metadata_value(value: &str, encoding: &MetadataEncoding) -> Result<String> {
    match encoding {
        MetadataEncoding::Base64 => base64::prelude::BASE64_STANDARD
            .decode(value)
            .map_err(|e| QollectiveError::envelope(format!("Base64 decode error: {}", e)))
            .and_then(|bytes| {
                String::from_utf8(bytes)
                    .map_err(|e| QollectiveError::envelope(format!("UTF-8 decode error: {}", e)))
            }),
        MetadataEncoding::Json => Ok(value.to_string()),
    }
}

#[cfg(feature = "rest-server")]
fn decode_complex_metadata(
    value: &str,
    encoding: &MetadataEncoding,
) -> Result<HashMap<String, serde_json::Value>> {
    let decoded_value = decode_metadata_value(value, encoding)?;
    serde_json::from_str(&decoded_value)
        .map_err(|e| QollectiveError::envelope(format!("JSON decode error: {}", e)))
}

#[cfg(feature = "rest-server")]
fn merge_metadata(meta: &mut Meta, additional: HashMap<String, serde_json::Value>) {
    // Merge additional metadata into the Meta struct
    // This is a placeholder for now - we can expand this based on Meta structure
    for (key, value) in additional {
        match key.as_str() {
            "tenant" => {
                if let Ok(tenant) = serde_json::from_value::<String>(value) {
                    meta.tenant = Some(tenant);
                }
            }
            "version" => {
                if let Ok(version) = serde_json::from_value::<String>(value) {
                    meta.version = Some(version);
                }
            }
            _ => {
                // Store in custom fields if available in Meta
                // For now, we skip unknown fields
            }
        }
    }
}

// =============================================================================
// SMART CORS MIDDLEWARE
// =============================================================================

/// Smart CORS middleware that distinguishes between CORS preflight and application OPTIONS
#[cfg(feature = "rest-server")]
async fn smart_cors_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    let method = request.method();
    let uri = request.uri();
    let headers = request.headers();

    // Check if this is an OPTIONS request
    if method == axum::http::Method::OPTIONS {
        let path = uri.path();

        // Check if this is a CORS preflight request
        let is_cors_preflight = headers.contains_key("access-control-request-method")
            || headers.contains_key("access-control-request-headers");

        // Get the route's OPTIONS behavior
        let options_behavior = RestServer::get_options_behavior(path).await;

        if is_cors_preflight || options_behavior == OptionsBehavior::CorsOnly {
            // Handle as CORS preflight
            return create_cors_preflight_response();
        } else {
            // Let the application handler process it
            let response = next.run(request).await;
            return add_cors_headers_to_response(response);
        }
    }

    // For non-OPTIONS requests, just add CORS headers to the response
    let response = next.run(request).await;
    add_cors_headers_to_response(response)
}

/// Create a CORS preflight response
#[cfg(feature = "rest-server")]
fn create_cors_preflight_response() -> axum::response::Response {
    use axum::http::{HeaderMap, StatusCode};

    let mut headers = HeaderMap::new();
    headers.insert("access-control-allow-origin", "*".parse().unwrap());
    headers.insert(
        "access-control-allow-methods",
        "GET, POST, PUT, DELETE, OPTIONS, PATCH".parse().unwrap(),
    );
    headers.insert("access-control-allow-headers", "*".parse().unwrap());
    headers.insert("access-control-max-age", "86400".parse().unwrap());

    (StatusCode::OK, headers, "").into_response()
}

/// Add CORS headers to an existing response
#[cfg(feature = "rest-server")]
fn add_cors_headers_to_response(
    mut response: axum::response::Response,
) -> axum::response::Response {
    let headers = response.headers_mut();
    headers.insert("access-control-allow-origin", "*".parse().unwrap());
    headers.insert(
        "access-control-allow-methods",
        "GET, POST, PUT, DELETE, OPTIONS, PATCH".parse().unwrap(),
    );
    headers.insert("access-control-allow-headers", "*".parse().unwrap());

    response
}

// =============================================================================
// AXUM INTEGRATION
// =============================================================================

/// Create basic Axum router with middleware
#[cfg(feature = "rest-server")]
pub fn create_basic_axum_router(routes: &[String], config: &RestServerConfig) -> Result<Router> {
    let mut router = Router::new();

    // Add placeholder handlers for registered routes
    for route in routes {
        router = router
            .route(route, post(placeholder_post_handler))
            .route(route, put(placeholder_put_handler))
            .route(route, get(placeholder_get_handler))
            .route(route, delete(placeholder_delete_handler))
            .route(route, options(placeholder_options_handler))
            .route(route, patch(placeholder_patch_handler));
    }

    // Add health check endpoint only if not already registered by users
    if !routes.contains(&"/health".to_string()) {
        router = router.route("/health", get(health_check_handler));
    }

    // Add tracing layer for debugging
    router = router.layer(TraceLayer::new_for_http());

    // Add smart CORS middleware if CORS is configured
    if let Some(_cors_config) = &config.cors {
        router = router.layer(axum::middleware::from_fn(smart_cors_middleware));
        // Note: We replace the standard CorsLayer with our smart middleware
        // router = router.layer(create_cors_layer(cors_config)?);
    }

    Ok(router)
}

/// Helper function to inject protocol metadata into envelope extensions
#[cfg(feature = "rest-server")]
fn inject_protocol_metadata_into_meta(
    meta: &mut Meta,
    protocol_metadata: RestProtocolMetadata,
) -> Result<()> {
    // Ensure extensions map exists
    if meta.extensions.is_none() {
        meta.extensions = Some(crate::envelope::meta::ExtensionsMeta {
            sections: HashMap::new(),
        });
    }

    // Serialize protocol metadata to JSON and insert into extensions
    if let Some(ref mut extensions) = meta.extensions {
        let protocol_value = serde_json::to_value(protocol_metadata)
            .map_err(|e| QollectiveError::envelope(format!("Failed to serialize protocol metadata: {}", e)))?;
        
        extensions.sections.insert(PROTOCOL_EXTENSION_KEY.to_string(), protocol_value);
    }

    Ok(())
}

/// Check registry for handler and route request appropriately
#[cfg(feature = "rest-server")]
async fn route_request_with_registry(
    method: &str,
    route: String,
    headers: HeaderMap,
    query_params: HashMap<String, String>,
    body: Option<Value>,
    protocol_metadata: Option<RestProtocolMetadata>,
) -> impl IntoResponse {
    let metadata_config = MetadataHandlingConfig::default();

    // Extract handler data based on HTTP method
    let handler_data = if method == "GET" || method == "DELETE" || method == "OPTIONS" {
        // For GET/DELETE/OPTIONS, try to extract envelope data from query parameters
        if let Some(envelope_data_str) = query_params.get("envelope_data") {
            // Deserialize the envelope data from the query parameter
            match serde_json::from_str::<Value>(envelope_data_str) {
                Ok(data) => data,
                Err(_) => Value::Null, // Fall back to null if deserialization fails
            }
        } else {
            Value::Null
        }
    } else {
        // For POST/PUT/PATCH, extract payload from envelope body
        match body {
            Some(envelope_obj) => {
                // Check if this is a full envelope object with "payload" field
                if let Some(payload_field) = envelope_obj.get("payload") {
                    payload_field.clone()
                } else {
                    // If no "payload" field, use the entire body as data
                    envelope_obj
                }
            }
            None => Value::Null,
        }
    };

    // Check for unsupported methods
    if !matches!(
        method,
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "OPTIONS"
    ) {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            format!("HTTP method '{}' not supported", method),
        )
            .into_response();
    }

    // Check if we have a registered handler for this route
    {
        let registry = get_handler_registry().read().await;
        if let Some(handler) = registry.get(&route) {
            // Use the registered handler with method-aware data
            match handler(
                headers,
                query_params,
                Some(handler_data),
                metadata_config.clone(),
                protocol_metadata,
            )
            .await
            {
                Ok((response_data, response_meta)) => {
                    // Create proper envelope response using builder pattern
                    let envelope = create_response_envelope(response_data, response_meta.clone());

                    // Create response headers with metadata
                    match inject_metadata_into_headers(&response_meta, &metadata_config) {
                        Ok(response_headers) => {
                            // Convert HeaderMap to axum Response
                            let mut response = Json(envelope).into_response();

                            // Add metadata headers to response
                            for (name, value) in response_headers.iter() {
                                response.headers_mut().insert(name.clone(), value.clone());
                            }

                            return response;
                        }
                        Err(e) => {
                            let error = EnvelopeError {
                                code: "METADATA_INJECTION_FAILED".to_string(),
                                message: format!("Failed to inject response metadata: {}", e),
                                details: Some(serde_json::json!({"operation": "metadata_injection"})),
                                trace: None,
                                #[cfg(any(
                                    feature = "rest-server", 
                                    feature = "rest-client",
                                    feature = "websocket-server", 
                                    feature = "websocket-client",
                                    feature = "a2a"
                                ))]
                                http_status_code: Some(500),
                            };
                            return create_error_envelope_response(error, None);
                        }
                    }
                }
                Err(e) => {
                    // Check if this is a headers-too-large error and return 413
                    let error_message = e.to_string();
                    if error_message.contains("Headers too large")
                        || error_message.contains("Header too large")
                    {
                        let error = EnvelopeError {
                            code: "REQUEST_HEADERS_TOO_LARGE".to_string(),
                            message: format!("Request headers too large: {}", error_message),
                            details: Some(serde_json::json!({"operation": "header_parsing", "error_type": "size_limit"})),
                            trace: None,
                            #[cfg(any(
                                feature = "rest-server", 
                                feature = "rest-client",
                                feature = "websocket-server", 
                                feature = "websocket-client",
                                feature = "a2a"
                            ))]
                            http_status_code: Some(413),
                        };
                        return create_error_envelope_response(error, None);
                    } else {
                        let error = EnvelopeError {
                            code: "HANDLER_ERROR".to_string(),
                            message: format!("Handler error: {}", error_message),
                            details: Some(serde_json::json!({"operation": "handler_execution"})),
                            trace: None,
                            #[cfg(any(
                                feature = "rest-server", 
                                feature = "rest-client",
                                feature = "websocket-server", 
                                feature = "websocket-client",
                                feature = "a2a"
                            ))]
                            http_status_code: Some(400),
                        };
                        return create_error_envelope_response(error, None);
                    }
                }
            }
        }
    }

    // Fall back to default behavior if no handler is registered
    handle_envelope_request(headers, query_params, Some(handler_data), metadata_config)
        .await
        .into_response()
}

// HTTP handlers with metadata integration and 413 error strategy
#[cfg(feature = "rest-server")]
async fn placeholder_post_handler(
    uri: axum::http::Uri,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let route = uri.path().to_string();
    let headers_map = convert_headermap_to_hashmap(&headers);
    let protocol_metadata = RestProtocolMetadata::with_headers("POST".to_string(), route.clone(), headers_map);
    route_request_with_registry("POST", route, headers, HashMap::new(), Some(body), Some(protocol_metadata)).await
}

#[cfg(feature = "rest-server")]
async fn placeholder_put_handler(
    uri: axum::http::Uri,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let route = uri.path().to_string();
    let headers_map = convert_headermap_to_hashmap(&headers);
    let protocol_metadata = RestProtocolMetadata::with_headers("PUT".to_string(), route.clone(), headers_map);
    route_request_with_registry("PUT", route, headers, HashMap::new(), Some(body), Some(protocol_metadata)).await
}

#[cfg(feature = "rest-server")]
async fn placeholder_get_handler(
    uri: axum::http::Uri,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let route = uri.path().to_string();
    let headers_map = convert_headermap_to_hashmap(&headers);
    let protocol_metadata = RestProtocolMetadata::with_all("GET".to_string(), route.clone(), params.clone(), headers_map);
    route_request_with_registry("GET", route, headers, params, None, Some(protocol_metadata)).await
}

#[cfg(feature = "rest-server")]
async fn placeholder_delete_handler(
    uri: axum::http::Uri,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let route = uri.path().to_string();
    let headers_map = convert_headermap_to_hashmap(&headers);
    let protocol_metadata = RestProtocolMetadata::with_all("DELETE".to_string(), route.clone(), params.clone(), headers_map);
    route_request_with_registry("DELETE", route, headers, params, None, Some(protocol_metadata)).await
}

#[cfg(feature = "rest-server")]
async fn placeholder_options_handler(
    uri: axum::http::Uri,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let route = uri.path().to_string();
    let headers_map = convert_headermap_to_hashmap(&headers);
    let protocol_metadata = RestProtocolMetadata::with_all("OPTIONS".to_string(), route.clone(), params.clone(), headers_map);
    route_request_with_registry("OPTIONS", route, headers, params, None, Some(protocol_metadata)).await
}

#[cfg(feature = "rest-server")]
async fn placeholder_patch_handler(
    uri: axum::http::Uri,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let route = uri.path().to_string();
    let headers_map = convert_headermap_to_hashmap(&headers);
    let protocol_metadata = RestProtocolMetadata::with_headers("PATCH".to_string(), route.clone(), headers_map);
    route_request_with_registry("PATCH", route, headers, HashMap::new(), Some(body), Some(protocol_metadata)).await
}

/// Handle HTTP request with envelope metadata extraction and 413 error strategy
#[cfg(feature = "rest-server")]
async fn handle_envelope_request(
    headers: HeaderMap,
    query_params: HashMap<String, String>,
    body: Option<Value>,
    metadata_config: MetadataHandlingConfig,
) -> impl IntoResponse {
    // Extract metadata from headers and query parameters
    let metadata_result = extract_metadata_from_http(&headers, &query_params, &metadata_config);

    match metadata_result {
        Ok(meta) => {
            // Create a simple response envelope with the extracted metadata
            let response_data = serde_json::json!({
                "message": "Envelope processed successfully",
                "extracted_metadata": {
                    "request_id": meta.request_id,
                    "tenant": meta.tenant,
                    "version": meta.version,
                    "timestamp": meta.timestamp
                },
                "body": body
            });

            // Create proper envelope response using builder pattern
            let envelope = create_response_envelope(response_data, meta.clone());

            // Create response headers with metadata
            let response_headers_result = inject_metadata_into_headers(&meta, &metadata_config);

            match response_headers_result {
                Ok(response_headers) => {
                    // Convert HeaderMap to axum Response
                    let mut response = Json(envelope).into_response();

                    // Add metadata headers to response
                    for (name, value) in response_headers.iter() {
                        response.headers_mut().insert(name.clone(), value.clone());
                    }

                    response
                }
                Err(e) => {
                    // If response metadata injection fails, return error without metadata headers
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to inject response metadata: {}", e),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            // Check if this is a headers-too-large error and return 413
            let error_message = e.to_string();
            if error_message.contains("Headers too large")
                || error_message.contains("Header too large")
            {
                (
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!("Request headers too large: {}", error_message),
                )
                    .into_response()
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid request metadata: {}", error_message),
                )
                    .into_response()
            }
        }
    }
}

#[cfg(feature = "rest-server")]
async fn health_check_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "qollective-rest-server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// =============================================================================
// CORE REST SERVER
// =============================================================================

use std::future::Future;
use std::pin::Pin;

/// Type-erased handler using closures - much simpler than trait objects
#[cfg(feature = "rest-server")]
type ErasedHandler = Box<
    dyn Fn(
            HeaderMap,
            HashMap<String, String>,
            Option<Value>,
            MetadataHandlingConfig,
            Option<RestProtocolMetadata>,
        ) -> Pin<Box<dyn Future<Output = Result<(Value, Meta)>> + Send>>
        + Send
        + Sync,
>;

/// Global handler registry for type-erased handlers
#[cfg(feature = "rest-server")]
static HANDLER_REGISTRY: std::sync::OnceLock<tokio::sync::RwLock<HashMap<String, ErasedHandler>>> =
    std::sync::OnceLock::new();

/// Global registry for route OPTIONS behavior configuration
#[cfg(feature = "rest-server")]
static ROUTE_OPTIONS_REGISTRY: std::sync::OnceLock<
    tokio::sync::RwLock<HashMap<String, OptionsBehavior>>,
> = std::sync::OnceLock::new();

/// Get or initialize the handler registry
#[cfg(feature = "rest-server")]
fn get_handler_registry() -> &'static tokio::sync::RwLock<HashMap<String, ErasedHandler>> {
    HANDLER_REGISTRY.get_or_init(|| tokio::sync::RwLock::new(HashMap::new()))
}

/// Get or initialize the route OPTIONS behavior registry
#[cfg(feature = "rest-server")]
fn get_route_options_registry() -> &'static tokio::sync::RwLock<HashMap<String, OptionsBehavior>> {
    ROUTE_OPTIONS_REGISTRY.get_or_init(|| tokio::sync::RwLock::new(HashMap::new()))
}

/// OPTIONS request behavior configuration
#[cfg(feature = "rest-server")]
#[derive(Debug, Clone, PartialEq)]
pub enum OptionsBehavior {
    /// Let application handler process OPTIONS requests
    Application,
    /// Only handle as CORS preflight (default)
    CorsOnly,
}

#[cfg(feature = "rest-server")]
impl Default for OptionsBehavior {
    fn default() -> Self {
        Self::CorsOnly
    }
}

/// Handler info for tracking registered routes
#[cfg(feature = "rest-server")]
#[derive(Debug, Clone)]
pub struct HandlerInfo {
    pub route: String,
    pub description: String,
    pub options_behavior: OptionsBehavior,
}

/// REST server for HTTP communication following NATS architecture patterns
#[cfg(feature = "rest-server")]
pub struct RestServer {
    config: RestServerConfig,
    routes: Vec<String>,                    // Simple route tracking for now
    handlers: HashMap<String, HandlerInfo>, // Route -> Handler info mapping
    listener: Option<TcpListener>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[cfg(feature = "rest-server")]
impl std::fmt::Debug for RestServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RestServer")
            .field("config", &self.config)
            .field("routes", &self.routes)
            .field("handlers", &self.handlers)
            .field("listener", &"<TcpListener>")
            .field("shutdown_tx", &"<OneShot>")
            .finish()
    }
}

#[cfg(feature = "rest-server")]
impl RestServer {
    /// Create a new REST server with the given configuration
    ///
    /// This is the single, obvious way to create a REST server,
    /// following the NATS pattern of simple construction.
    pub async fn new(config: RestServerConfig) -> Result<Self> {
        // Validate configuration
        if config.base.port == 0 {
            return Err(QollectiveError::config("Port cannot be 0"));
        }

        if config.base.bind_address.is_empty() {
            return Err(QollectiveError::config("Bind address cannot be empty"));
        }

        // Validate metadata configuration
        if config.metadata.max_header_size == 0 {
            return Err(QollectiveError::config("max_header_size cannot be 0"));
        }

        if config.metadata.max_total_headers == 0 {
            return Err(QollectiveError::config("max_total_headers cannot be 0"));
        }

        Ok(Self {
            config,
            routes: Vec::new(),
            handlers: HashMap::new(),
            listener: None,
            shutdown_tx: None,
        })
    }

    /// Start the REST server
    ///
    /// Binds to the configured address and port, then starts serving HTTP requests.
    /// This method will block until the server is shut down.
    pub async fn start(&mut self) -> Result<()> {
        let bind_addr = format!(
            "{}:{}",
            self.config.base.bind_address, self.config.base.port
        );

        // Create basic Axum router
        let app = create_basic_axum_router(&self.routes, &self.config)?;

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Check if TLS is configured
        if let Some(tls_config) = &self.config.tls {
            // TLS-enabled server
            #[cfg(all(feature = "rest-server", feature = "tls"))]
            {
                // Create rustls configuration using unified TLS config
                let rustls_config = create_axum_rustls_config(tls_config).await?;

                // Use axum-server for TLS support
                let addr = bind_addr.parse().map_err(|e| {
                    QollectiveError::transport(format!("Invalid bind address {}: {}", bind_addr, e))
                })?;

                let server =
                    axum_server::bind_rustls(addr, rustls_config).serve(app.into_make_service());

                // Handle graceful shutdown
                tokio::select! {
                    result = server => {
                        if let Err(e) = result {
                            return Err(QollectiveError::transport(format!("TLS server error: {}", e)));
                        }
                    }
                    _ = shutdown_rx => {
                        // Graceful shutdown requested
                    }
                }
            }

            #[cfg(not(all(feature = "rest-server", feature = "tls")))]
            {
                return Err(QollectiveError::feature_not_enabled(
                    "TLS support requires 'tls' feature to be enabled",
                ));
            }
        } else {
            // Plain HTTP server
            // Create TCP listener
            let listener = TcpListener::bind(&bind_addr).await.map_err(|e| {
                QollectiveError::transport(format!("Failed to bind to {}: {}", bind_addr, e))
            })?;

            self.listener = Some(listener);

            // Get the listener (we know it's Some because we just set it)
            let listener = self.listener.take().unwrap();

            // Start the server with graceful shutdown
            let server = axum::serve(listener, app).with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            });

            if let Err(e) = server.await {
                return Err(QollectiveError::transport(format!("Server error: {}", e)));
            }
        }

        Ok(())
    }

    /// Shutdown the REST server gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        Ok(())
    }

    /// Get the server configuration
    pub fn config(&self) -> &RestServerConfig {
        &self.config
    }

    /// Register a route with handler info
    ///
    /// This is an internal method used by the UnifiedEnvelopeReceiver implementation.
    fn register_route_with_handler(
        &mut self,
        route: &str,
        handler_info: HandlerInfo,
    ) -> Result<()> {
        // Validate route
        if route.is_empty() {
            return Err(QollectiveError::config("Route cannot be empty"));
        }

        if !route.starts_with('/') {
            return Err(QollectiveError::config("Route must start with '/'"));
        }

        // Check for duplicate routes
        if self.routes.contains(&route.to_string()) {
            return Err(QollectiveError::config(format!(
                "Route '{}' is already registered",
                route
            )));
        }

        // Store route and handler info
        self.routes.push(route.to_string());
        self.handlers.insert(route.to_string(), handler_info);
        println!("📝 Registered route: {}", route);
        Ok(())
    }

    /// Get handler info for a route
    pub fn get_handler_info(&self, route: &str) -> Option<&HandlerInfo> {
        self.handlers.get(route)
    }

    /// Get the number of registered routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Check if a route is registered
    pub fn has_route(&self, route: &str) -> bool {
        self.routes.contains(&route.to_string())
    }

    /// List all registered routes
    pub fn routes(&self) -> Vec<String> {
        self.routes.clone()
    }

    /// Configure OPTIONS behavior for a specific route
    pub async fn set_options_behavior(
        &mut self,
        route: &str,
        behavior: OptionsBehavior,
    ) -> Result<()> {
        // Update the route OPTIONS registry
        {
            let mut registry = get_route_options_registry().write().await;
            registry.insert(route.to_string(), behavior.clone());
        }

        // Update the handler info if it exists
        if let Some(handler_info) = self.handlers.get_mut(route) {
            handler_info.options_behavior = behavior;
        }

        Ok(())
    }

    /// Get OPTIONS behavior for a route (static method for middleware access)
    pub async fn get_options_behavior(route: &str) -> OptionsBehavior {
        let registry = get_route_options_registry().read().await;
        registry.get(route).cloned().unwrap_or_default()
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Extract HTTP status code from EnvelopeError with fallback logic
///
/// This function checks if the EnvelopeError has a custom http_status_code set,
/// and if so, validates it's in the valid error range (400-599). If not present
/// or invalid, it falls back to pattern-based status code mapping based on the
/// error code string.
#[cfg(feature = "rest-server")]
pub fn extract_http_status_code(error: &EnvelopeError) -> StatusCode {
    // Check if custom status code is available and valid
    #[cfg(any(
        feature = "rest-server", 
        feature = "rest-client",
        feature = "websocket-server", 
        feature = "websocket-client",
        feature = "a2a"
    ))]
    if let Some(status_code) = error.http_status_code {
        // Validate status code is in error range (400-599)
        if status_code >= 400 && status_code < 600 {
            if let Ok(status) = StatusCode::from_u16(status_code) {
                return status;
            }
        }
        // Log warning for invalid status code but continue with fallback
        tracing::warn!("Invalid HTTP status code {} in EnvelopeError, using fallback mapping", status_code);
    }
    
    // Fallback to pattern-based status code mapping
    let error_code = error.code.to_uppercase();
    match error_code.as_str() {
        // Authentication and authorization errors
        code if code.contains("AUTH") || code.contains("UNAUTHORIZED") => StatusCode::UNAUTHORIZED,
        code if code.contains("FORBIDDEN") || code.contains("PERMISSION") => StatusCode::FORBIDDEN,
        
        // Client errors  
        code if code.contains("VALIDATION") || code.contains("INVALID") => StatusCode::BAD_REQUEST,
        code if code.contains("NOT_FOUND") || code.contains("MISSING") => StatusCode::NOT_FOUND,
        code if code.contains("CONFLICT") || code.contains("EXISTS") => StatusCode::CONFLICT,
        code if code.contains("TOO_LARGE") || code.contains("SIZE") => StatusCode::PAYLOAD_TOO_LARGE,
        code if code.contains("RATE_LIMIT") || code.contains("THROTTLE") => StatusCode::TOO_MANY_REQUESTS,
        
        // Server errors
        code if code.contains("TIMEOUT") || code.contains("DEADLINE") => StatusCode::GATEWAY_TIMEOUT,
        code if code.contains("UNAVAILABLE") || code.contains("SERVICE") => StatusCode::SERVICE_UNAVAILABLE,
        code if code.contains("NOT_IMPLEMENTED") => StatusCode::NOT_IMPLEMENTED,
        
        // Default to 500 for any unrecognized error patterns
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Create JSON envelope error response from EnvelopeError
///
/// This function creates a proper JSON envelope response for errors, replacing
/// plain text error responses. It preserves request metadata when available and
/// uses the custom HTTP status code if specified.
#[cfg(feature = "rest-server")]
pub fn create_error_envelope_response(
    error: EnvelopeError, 
    original_meta: Option<Meta>
) -> axum::response::Response {
    // Extract HTTP status code using helper function
    let status_code = extract_http_status_code(&error);
    
    // Create response metadata preserving original request metadata
    let response_meta = Meta::preserve_for_response(original_meta.as_ref());
    
    // Create error envelope with empty payload (error information is in envelope.error field)
    let error_envelope = Envelope::error(response_meta, (), error);
    
    // Return JSON response with appropriate status code
    (status_code, Json(error_envelope)).into_response()
}

/// Implementation of UnifiedEnvelopeReceiver trait
///
/// This provides native integration with the unified pattern,
/// built into the core architecture rather than bolted on.
#[cfg(feature = "rest-server")]
#[async_trait]
impl UnifiedEnvelopeReceiver for RestServer {
    /// Register a handler for envelopes at the default route ("/envelope")
    async fn receive_envelope<T, R, H>(&mut self, handler: H) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
        H: ContextDataHandler<T, R> + Send + Sync + 'static,
    {
        self.receive_envelope_at("/envelope", handler).await
    }

    /// Register a handler for envelopes at the specified route
    async fn receive_envelope_at<T, R, H>(&mut self, route: &str, handler: H) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
        H: ContextDataHandler<T, R> + Send + Sync + 'static,
    {
        // Create a shared handler that can be moved into the closure
        let handler = Arc::new(handler);

        // Create a type-erased closure that wraps the specific handler
        let erased_handler: ErasedHandler =
            Box::new(move |headers, query_params, body, metadata_config, protocol_metadata| {
                let handler = handler.clone();
                Box::pin(async move {
                    // Extract metadata from headers and query parameters
                    let mut meta =
                        extract_metadata_from_http(&headers, &query_params, &metadata_config)?;

                    // Inject protocol metadata into extensions if available
                    if let Some(protocol_meta) = protocol_metadata {
                        inject_protocol_metadata_into_meta(&mut meta, protocol_meta)?;
                    }

                    // Create context from metadata (now includes protocol info)
                    let context = Some(Context::new(meta.clone()));

                    // Deserialize body data to expected type T
                    let data: T = if let Some(body_value) = body {
                        serde_json::from_value(body_value).map_err(|e| {
                            QollectiveError::envelope(format!(
                                "Failed to deserialize request data: {}",
                                e
                            ))
                        })?
                    } else {
                        // For GET/DELETE requests with no body, deserialize Value::Null to T
                        serde_json::from_value(Value::Null).map_err(|e| {
                            QollectiveError::envelope(format!(
                                "Failed to deserialize empty request data: {}",
                                e
                            ))
                        })?
                    };

                    // Call the actual handler
                    let response_data = handler.handle(context, data).await?;

                    // Serialize response
                    let response_value = serde_json::to_value(&response_data).map_err(|e| {
                        QollectiveError::envelope(format!(
                            "Failed to serialize response data: {}",
                            e
                        ))
                    })?;

                    // Create response metadata using the proper preservation utility
                    // This ensures consistent metadata handling across all transports
                    let response_meta = crate::envelope::Meta::preserve_for_response(Some(&meta));

                    Ok((response_value, response_meta))
                })
            });

        // Create handler info for tracking (moved up to use for both storage mechanisms)
        let handler_info = HandlerInfo {
            route: route.to_string(),
            description: format!("Handler registered for route: {}", route),
            options_behavior: OptionsBehavior::default(),
        };

        // Store the handler in the global registry
        {
            let mut registry = get_handler_registry().write().await;
            registry.insert(route.to_string(), erased_handler);
        }

        // Store default OPTIONS behavior in global registry
        {
            let mut options_registry = get_route_options_registry().write().await;
            options_registry.insert(route.to_string(), OptionsBehavior::default());
        }

        // Register the route with handler info (FIX: This was missing and caused the race condition!)
        self.register_route_with_handler(route, handler_info)
    }
}
