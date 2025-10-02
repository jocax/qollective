//! # OpenAPI 3.1 Integration for Qollective Framework
//!
//! This module provides comprehensive OpenAPI 3.1 schema generation and utilities for the 
//! Qollective envelope-first architecture. It enables automatic API documentation generation
//! with complete metadata support, security schemas, and enterprise-grade examples.
//!
//! ## Features
//!
//! - **Automatic Schema Generation**: Uses `utoipa` to generate OpenAPI 3.1 schemas from Rust types
//! - **Envelope-First Documentation**: Complete documentation of the unified envelope architecture
//! - **Metadata Schema Support**: Full schemas for security, performance, tracing, and debugging metadata
//! - **Example Generation**: Comprehensive examples for common enterprise use cases
//! - **Swagger UI Integration**: Ready-to-use Swagger UI configuration
//! - **Validation Utilities**: Schema validation helpers for envelope conformance
//!
//! ## Quick Start
//!
//! ### Basic OpenAPI Specification Generation
//!
//! ```rust
//! use qollective::openapi::OpenApiUtils;
//!
//! // Generate complete OpenAPI 3.1 specification
//! let spec = OpenApiUtils::generate_spec();
//! println!("API Title: {}", spec["info"]["title"]);
//!
//! // Generate as formatted JSON string
//! let spec_json = OpenApiUtils::generate_spec_string();
//! std::fs::write("api-spec.json", spec_json).unwrap();
//! ```
//!
//! ### Example Envelope Generation
//!
//! ```rust
//! use qollective::openapi::OpenApiUtils;
//!
//! // Generate example envelope with enterprise metadata
//! let envelope = OpenApiUtils::generate_example_envelope();
//! assert_eq!(envelope.meta.tenant.as_deref(), Some("enterprise_starfleet"));
//! assert_eq!(envelope.payload.message, "Bridge to Engineering: Warp core online");
//!
//! // Generate error envelope example
//! let error_envelope = OpenApiUtils::generate_example_error_envelope();
//! assert!(error_envelope.has_error());
//! ```
//!
//! ### Envelope Validation
//!
//! ```rust
//! use qollective::openapi::OpenApiUtils;
//!
//! let envelope = OpenApiUtils::generate_example_envelope();
//! match OpenApiUtils::validate_envelope_schema(&envelope) {
//!     Ok(()) => println!("Envelope is valid"),
//!     Err(e) => eprintln!("Validation error: {}", e),
//! }
//! ```
//!
//! ## Architecture Integration
//!
//! This module integrates seamlessly with the Qollective envelope-first architecture:
//!
//! - **`Envelope<T>`**: Core envelope wrapper with comprehensive metadata
//! - **`Meta`**: Unified metadata structure with security, performance, and tracing info
//! - **`EnvelopeError`**: Standardized error handling with context and tracing
//! - **`EnvelopeBuilder`**: Fluent API for constructing envelopes with validation
//!
//! ## Enterprise Examples
//!
//! The module provides enterprise-grade examples demonstrating real-world usage:
//!
//! - Multi-tenant security with JWT authentication
//! - Performance monitoring with database and cache metrics
//! - Distributed tracing with OpenTelemetry integration
//! - Error handling with detailed context and recommendations
//!
//! ## Schema Extensions
//!
//! All Qollective types include comprehensive OpenAPI schema annotations:
//!
//! ```rust
//! #[derive(Serialize, Deserialize, ToSchema)]
//! #[schema(
//!     title = "Enterprise Message",
//!     description = "Example payload demonstrating Qollective envelope usage",
//!     example = json!({
//!         "message": "Bridge to Engineering: Warp core online",
//!         "status": "operational",
//!         "priority": 1
//!     })
//! )]
//! pub struct EnterpriseMessage {
//!     /// The main message content
//!     pub message: String,
//!     
//!     /// Current operational status
//!     pub status: String,
//!     
//!     /// Priority level (1=high, 3=low)
//!     pub priority: Option<i32>,
//! }
//! ```
//!
//! ## Configuration
//!
//! Enable OpenAPI support using the `openapi` feature flag:
//!
//! ```toml
//! [dependencies]
//! qollective = { version = "0.0.1", features = ["openapi"] }
//! ```

#[cfg(feature = "openapi")]
use utoipa::{OpenApi, ToSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::envelope::{Envelope, EnvelopeBuilder, EnvelopeError};
use crate::envelope::meta::{Meta, SecurityMeta, PerformanceMeta, TracingMeta, DebugMeta, MonitoringMeta, ExtensionsMeta, OnBehalfOfMeta};
use crate::error::EnhancedQollectiveError;

/// Enterprise message payload demonstrating Qollective envelope usage
///
/// This struct serves as an example payload for demonstrating how custom data types
/// integrate with the Qollective envelope architecture and OpenAPI schema generation.
/// It represents a typical enterprise communication message with operational status,
/// priority levels, and audit timestamps.
///
/// # Examples
///
/// ## Basic Message Creation
///
/// ```rust
/// use qollective::openapi::EnterpriseMessage;
/// use chrono::Utc;
///
/// let message = EnterpriseMessage {
///     message: "Bridge to Engineering: Warp core online".to_string(),
///     status: "operational".to_string(),
///     priority: Some(1),
///     created_at: Some(Utc::now()),
/// };
///
/// assert_eq!(message.status, "operational");
/// assert_eq!(message.priority, Some(1));
/// ```
///
/// ## JSON Serialization
///
/// ```rust
/// use qollective::openapi::EnterpriseMessage;
/// use serde_json;
///
/// let message = EnterpriseMessage {
///     message: "System status: All systems nominal".to_string(),
///     status: "ready".to_string(),
///     priority: Some(2),
///     created_at: None,
/// };
///
/// let json = serde_json::to_string(&message).unwrap();
/// let deserialized: EnterpriseMessage = serde_json::from_str(&json).unwrap();
/// assert_eq!(message.message, deserialized.message);
/// ```
///
/// ## OpenAPI Schema Integration
///
/// When the `openapi` feature is enabled, this struct automatically generates
/// comprehensive OpenAPI 3.1 schema documentation including examples, descriptions,
/// and validation rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Enterprise Message",
    description = "Example message payload for demonstrating Qollective envelope usage in enterprise applications"
))]
pub struct EnterpriseMessage {
    /// The main message content
    /// 
    /// Contains the primary communication text. In enterprise systems, this typically
    /// represents operational status updates, command confirmations, or system notifications.
    #[cfg_attr(feature = "openapi", schema(example = "Bridge to Engineering: Warp core online"))]
    pub message: String,
    
    /// Current operational status
    /// 
    /// Indicates the current state of the system or operation. Common values include:
    /// - "operational" - System functioning normally
    /// - "error" - System experiencing issues
    /// - "maintenance" - System undergoing scheduled maintenance
    /// - "ready" - System ready for operation
    #[cfg_attr(feature = "openapi", schema(example = "operational"))]
    pub status: String,
    
    /// Priority level for the message
    /// 
    /// Optional priority indicator where:
    /// - 1 = High priority (critical operations)
    /// - 2 = Medium priority (standard operations)  
    /// - 3 = Low priority (informational)
    /// 
    /// Omitted for routine communications where priority is not relevant.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(example = 1))]
    pub priority: Option<i32>,
    
    /// Timestamp when the message was created
    /// 
    /// Optional ISO 8601 timestamp indicating when this message was generated.
    /// Useful for audit trails and debugging distributed system communications.
    /// When omitted, the envelope's metadata timestamp should be used instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(example = "2025-08-23T10:30:45.123Z"))]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Comprehensive OpenAPI 3.1 specification for the Qollective framework
///
/// This struct generates a complete OpenAPI 3.1 specification that documents the entire
/// Qollective envelope-first architecture, including all metadata types, error handling
/// patterns, and security schemas. It serves as the foundation for automatic API
/// documentation generation and client SDK generation.
///
/// # Features
///
/// - **Complete Schema Coverage**: Documents all envelope structures, metadata types, and error patterns
/// - **Enterprise Security**: Includes comprehensive security metadata schemas for JWT auth and multi-tenancy
/// - **Performance Monitoring**: Documents performance metadata for observability and monitoring
/// - **Distributed Tracing**: Includes tracing metadata schemas compatible with OpenTelemetry
/// - **Error Handling**: Complete error response schemas with context and debugging information
///
/// # Generated Schemas
///
/// This specification includes schemas for:
///
/// - **Core Types**: `Envelope<T>`, `EnvelopeBuilder<T>`, `Meta`, `EnvelopeError`
/// - **Metadata**: `SecurityMeta`, `PerformanceMeta`, `TracingMeta`, `DebugMeta`, `MonitoringMeta`
/// - **Extensions**: `ExtensionsMeta`, `OnBehalfOfMeta` for advanced use cases
/// - **Errors**: `EnhancedQollectiveError` for comprehensive error reporting
/// - **Examples**: `EnterpriseMessage` demonstrating payload integration
///
/// # Usage
///
/// ```rust
/// use qollective::openapi::QollectiveApiDoc;
/// use utoipa::OpenApi;
///
/// // Generate the OpenAPI specification
/// let openapi = QollectiveApiDoc::openapi();
/// println!("Title: {}", openapi.info.title);
/// println!("Version: {}", openapi.info.version);
///
/// // Convert to JSON for serving or file export
/// let json = serde_json::to_string_pretty(&openapi).unwrap();
/// std::fs::write("qollective-api.json", json).unwrap();
/// ```
///
/// # Integration with Web Frameworks
///
/// The generated specification integrates seamlessly with web frameworks:
///
/// ```rust
/// use axum::{routing::get, Router, Json};
/// use qollective::openapi::QollectiveApiDoc;
/// use utoipa::OpenApi;
///
/// async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
///     Json(QollectiveApiDoc::openapi())
/// }
///
/// let app = Router::new()
///     .route("/api-docs/openapi.json", get(openapi_spec));
/// ```
#[cfg(feature = "openapi")]
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Qollective Enterprise API", 
        version = "1.0.0",
        description = "Comprehensive OpenAPI specification for Qollective framework demonstrating envelope-first architecture with enterprise-grade metadata support",
        contact(
            name = "Qollective Team",
            email = "support@qollective.dev"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    paths(),
    components(schemas(
        // Core envelope structures
        Envelope<EnterpriseMessage>,
        EnvelopeBuilder<EnterpriseMessage>,
        EnvelopeError,
        
        // Example payloads
        EnterpriseMessage,
        
        // Metadata structures
        Meta,
        SecurityMeta,
        PerformanceMeta, 
        TracingMeta,
        DebugMeta,
        MonitoringMeta,
        ExtensionsMeta,
        OnBehalfOfMeta,
        
        // Error handling
        EnhancedQollectiveError,
    )),
    tags(
        (name = "envelope", description = "Envelope operations and metadata"),
        (name = "security", description = "Security and authentication"),
        (name = "monitoring", description = "Performance and monitoring"),
        (name = "errors", description = "Error handling and validation")
    ),
    servers(
        (url = "https://api.example.com/v1", description = "Production server"),
        (url = "https://staging-api.example.com/v1", description = "Staging server"),
        (url = "http://localhost:8080/v1", description = "Development server")
    )
)]
pub struct QollectiveApiDoc;

/// Utility functions for OpenAPI schema generation and example creation
///
/// This struct provides a collection of utility functions for working with OpenAPI
/// specifications in the Qollective framework. It includes methods for generating
/// complete specifications, creating example envelopes, validating schemas, and
/// producing comprehensive API documentation examples.
///
/// # Key Features
///
/// - **Specification Generation**: Create complete OpenAPI 3.1 specifications
/// - **Example Creation**: Generate realistic envelope examples for documentation
/// - **Schema Validation**: Validate envelopes against expected schema requirements
/// - **API Examples**: Produce comprehensive examples for different use cases
///
/// # Examples
///
/// ## Generate Complete API Specification
///
/// ```rust
/// use qollective::openapi::OpenApiUtils;
///
/// // Generate the complete OpenAPI specification as JSON
/// let spec = OpenApiUtils::generate_spec();
/// assert_eq!(spec["info"]["title"], "Qollective Enterprise API");
/// assert_eq!(spec["info"]["version"], "1.0.0");
///
/// // Generate as formatted JSON string for file output
/// let spec_string = OpenApiUtils::generate_spec_string();
/// std::fs::write("qollective-openapi.json", spec_string).unwrap();
/// ```
///
/// ## Create Example Envelopes
///
/// ```rust
/// use qollective::openapi::OpenApiUtils;
///
/// // Generate success envelope example
/// let success_envelope = OpenApiUtils::generate_example_envelope();
/// assert!(success_envelope.is_success());
/// assert_eq!(success_envelope.meta.tenant.as_deref(), Some("enterprise_starfleet"));
///
/// // Generate error envelope example  
/// let error_envelope = OpenApiUtils::generate_example_error_envelope();
/// assert!(error_envelope.has_error());
/// assert_eq!(error_envelope.error.as_ref().unwrap().code, "WARP_CORE_FAILURE");
/// ```
///
/// ## Schema Validation
///
/// ```rust
/// use qollective::openapi::OpenApiUtils;
///
/// let envelope = OpenApiUtils::generate_example_envelope();
/// match OpenApiUtils::validate_envelope_schema(&envelope) {
///     Ok(()) => println!("Envelope passes validation"),
///     Err(msg) => eprintln!("Validation failed: {}", msg),
/// }
/// ```
#[cfg(feature = "openapi")]
pub struct OpenApiUtils;

#[cfg(feature = "openapi")]
impl OpenApiUtils {
    /// Generate the complete OpenAPI 3.1 specification as JSON
    ///
    /// Creates a complete OpenAPI specification document that includes schemas for all
    /// Qollective envelope structures, metadata types, and error handling patterns.
    /// The specification follows OpenAPI 3.1 standards and includes comprehensive
    /// examples and descriptions.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` containing the complete OpenAPI specification with:
    /// - API information (title, version, description, contact, license)
    /// - Server configurations for different environments  
    /// - Component schemas for all envelope and metadata types
    /// - Tag definitions for organizing API operations
    ///
    /// # Example
    ///
    /// ```rust
    /// use qollective::openapi::OpenApiUtils;
    ///
    /// let spec = OpenApiUtils::generate_spec();
    /// 
    /// // Access API metadata
    /// assert_eq!(spec["info"]["title"], "Qollective Enterprise API");
    /// assert_eq!(spec["info"]["version"], "1.0.0");
    /// 
    /// // Check for core schemas
    /// let schemas = &spec["components"]["schemas"];
    /// assert!(schemas["Meta"].is_object());
    /// assert!(schemas["SecurityMeta"].is_object());
    /// assert!(schemas["EnvelopeError"].is_object());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if the OpenAPI specification cannot be serialized to JSON,
    /// which should only occur if there are internal inconsistencies in the schema definitions.
    pub fn generate_spec() -> serde_json::Value {
        let openapi = QollectiveApiDoc::openapi();
        serde_json::to_value(openapi).expect("Failed to serialize OpenAPI spec")
    }
    
    /// Generate the OpenAPI specification as a formatted JSON string
    ///
    /// This method creates a pretty-printed JSON string representation of the complete
    /// OpenAPI specification, suitable for writing to files or serving over HTTP.
    /// The output is formatted with proper indentation for human readability.
    ///
    /// # Returns
    ///
    /// A formatted JSON string containing the complete OpenAPI 3.1 specification.
    ///
    /// # Example
    ///
    /// ```rust
    /// use qollective::openapi::OpenApiUtils;
    /// use std::fs;
    ///
    /// // Generate and save specification to file
    /// let spec_string = OpenApiUtils::generate_spec_string();
    /// fs::write("qollective-api-spec.json", &spec_string).unwrap();
    ///
    /// // Verify the content is valid JSON
    /// let parsed: serde_json::Value = serde_json::from_str(&spec_string).unwrap();
    /// assert_eq!(parsed["info"]["title"], "Qollective Enterprise API");
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **File Export**: Save specification to file for version control or distribution
    /// - **HTTP Serving**: Serve specification from web endpoints for API documentation
    /// - **Client Generation**: Input for OpenAPI client code generators
    /// - **Documentation**: Human-readable API documentation generation
    ///
    /// # Panics
    ///
    /// This function panics if the OpenAPI specification cannot be formatted as JSON,
    /// which should only occur if there are internal serialization issues.
    pub fn generate_spec_string() -> String {
        let spec = Self::generate_spec();
        serde_json::to_string_pretty(&spec).expect("Failed to format OpenAPI spec")
    }
    
    /// Generate a comprehensive example envelope with enterprise metadata
    ///
    /// Creates a fully populated envelope example demonstrating successful operation
    /// in an enterprise environment. The envelope includes tenant information,
    /// versioning, timestamps, and operational status data. This example is ideal
    /// for documentation, testing, and demonstrating the envelope-first architecture.
    ///
    /// # Returns
    ///
    /// An `Envelope<EnterpriseMessage>` containing:
    /// - **Metadata**: Request ID, timestamp, tenant, and version information
    /// - **Payload**: Enterprise message with operational status
    /// - **Success State**: No error information (successful operation)
    ///
    /// # Example
    ///
    /// ```rust
    /// use qollective::openapi::OpenApiUtils;
    ///
    /// let envelope = OpenApiUtils::generate_example_envelope();
    ///
    /// // Verify envelope structure
    /// assert!(envelope.is_success());
    /// assert!(!envelope.has_error());
    ///
    /// // Check metadata
    /// assert_eq!(envelope.meta.tenant.as_deref(), Some("enterprise_starfleet"));
    /// assert_eq!(envelope.meta.version.as_deref(), Some("1.0"));
    /// assert!(envelope.meta.request_id.is_some());
    /// assert!(envelope.meta.timestamp.is_some());
    ///
    /// // Check payload
    /// assert_eq!(envelope.payload.message, "Bridge to Engineering: Warp core online");
    /// assert_eq!(envelope.payload.status, "operational");
    /// assert_eq!(envelope.payload.priority, Some(1));
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **API Documentation**: Example responses in OpenAPI specifications
    /// - **Client Testing**: Realistic test data for client implementations
    /// - **Integration Testing**: Validation of envelope processing logic
    /// - **Schema Validation**: Reference implementation for envelope structure
    pub fn generate_example_envelope() -> Envelope<EnterpriseMessage> {
        let mut meta = Meta::for_new_request();
        meta.tenant = Some("enterprise_starfleet".to_string());
        meta.version = Some("1.0".to_string());
        
        let message = EnterpriseMessage {
            message: "Bridge to Engineering: Warp core online".to_string(),
            status: "operational".to_string(),
            priority: Some(1),
            created_at: Some(chrono::Utc::now()),
        };
        
        Envelope::new(meta, message)
    }
    
    /// Generate a comprehensive example error envelope with detailed error information
    ///
    /// Creates a fully populated error envelope demonstrating how failures are
    /// communicated in the Qollective framework. The envelope includes detailed
    /// error information with context, structured error details, and debugging
    /// traces. This example shows best practices for error reporting in distributed systems.
    ///
    /// # Returns
    ///
    /// An `Envelope<EnterpriseMessage>` containing:
    /// - **Metadata**: Request ID, timestamp, and tenant information
    /// - **Payload**: Enterprise message describing the error condition
    /// - **Error Information**: Detailed error with code, message, context, and trace
    ///
    /// # Example
    ///
    /// ```rust
    /// use qollective::openapi::OpenApiUtils;
    ///
    /// let envelope = OpenApiUtils::generate_example_error_envelope();
    ///
    /// // Verify error envelope structure
    /// assert!(!envelope.is_success());
    /// assert!(envelope.has_error());
    ///
    /// // Check error details
    /// let error = envelope.error.as_ref().unwrap();
    /// assert_eq!(error.code, "WARP_CORE_FAILURE");
    /// assert!(error.message.contains("dilithium crystals"));
    /// assert!(error.details.is_some());
    /// assert!(error.trace.is_some());
    ///
    /// // Check payload reflects error state
    /// assert_eq!(envelope.payload.status, "error");
    /// assert_eq!(envelope.payload.priority, Some(3)); // High priority for failures
    /// ```
    ///
    /// # Error Structure
    ///
    /// The generated error includes:
    /// - **Error Code**: Machine-readable error identifier (`WARP_CORE_FAILURE`)
    /// - **Human Message**: Descriptive error message for user interfaces
    /// - **Structured Details**: JSON object with diagnostic information
    /// - **Stack Trace**: Debugging information for development environments
    ///
    /// # Use Cases
    ///
    /// - **Error Documentation**: Example error responses in API specifications
    /// - **Client Error Handling**: Test error processing and user feedback
    /// - **Monitoring Integration**: Example error data for alerting systems
    /// - **Debugging Support**: Reference for error investigation workflows
    pub fn generate_example_error_envelope() -> Envelope<EnterpriseMessage> {
        let mut meta = Meta::for_new_request();
        meta.tenant = Some("enterprise_starfleet".to_string());
        
        let message = EnterpriseMessage {
            message: "Error in warp core initialization".to_string(),
            status: "error".to_string(),
            priority: Some(3),
            created_at: Some(chrono::Utc::now()),
        };
        
        let error = EnvelopeError {
            code: "WARP_CORE_FAILURE".to_string(),
            message: "Warp core failed to initialize due to insufficient dilithium crystals".to_string(),
            details: Some(json!({
                "component": "warp_core",
                "expected_crystals": 4,
                "actual_crystals": 2,
                "recommendation": "Replace dilithium crystals and restart initialization"
            })),
            trace: Some("at initialize_warp_core (warp.rs:142)".to_string()),
            #[cfg(any(
                feature = "rest-server", 
                feature = "rest-client",
                feature = "websocket-server", 
                feature = "websocket-client",
                feature = "a2a"
            ))]
            http_status_code: Some(500),
        };
        
        Envelope::error(meta, message, error)
    }
    
    /// Validate envelope conformance to expected schema requirements
    ///
    /// Performs basic validation of envelope structure to ensure it meets the minimum
    /// requirements for proper envelope-first architecture. This validation checks
    /// for required metadata fields and structural integrity. In production systems,
    /// this would typically use JSON Schema validation for comprehensive checking.
    ///
    /// # Arguments
    ///
    /// * `envelope` - The envelope to validate, must implement `ToSchema` and `Serialize`
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Envelope passes validation
    /// * `Err(String)` - Envelope fails validation with error description
    ///
    /// # Validation Rules
    ///
    /// - **Request ID**: Must be present for request tracing and correlation
    /// - **Timestamp**: Must be present for audit trails and debugging
    /// - **Structure**: Envelope must be properly formed and serializable
    ///
    /// # Example
    ///
    /// ```rust
    /// use qollective::openapi::OpenApiUtils;
    ///
    /// // Validate a correct envelope
    /// let valid_envelope = OpenApiUtils::generate_example_envelope();
    /// assert!(OpenApiUtils::validate_envelope_schema(&valid_envelope).is_ok());
    ///
    /// // Create invalid envelope (would fail validation)
    /// // let invalid_envelope = create_envelope_without_request_id();
    /// // assert!(OpenApiUtils::validate_envelope_schema(&invalid_envelope).is_err());
    /// ```
    ///
    /// # Production Usage
    ///
    /// In production systems, extend this validation with:
    /// - JSON Schema validation using the `jsonschema` crate
    /// - Business rule validation for payload content
    /// - Security validation for metadata fields
    /// - Protocol-specific validation requirements
    ///
    /// ```rust
    /// // Example production validation (requires jsonschema feature)
    /// // use jsonschema::JSONSchema;
    /// // let schema = JSONSchema::compile(&openapi_schema)?;
    /// // let envelope_json = serde_json::to_value(&envelope)?;
    /// // schema.validate(&envelope_json)?;
    /// ```
    pub fn validate_envelope_schema<T: ToSchema + Serialize>(envelope: &Envelope<T>) -> Result<(), String> {
        // Basic validation - in a real implementation, you'd use jsonschema crate
        if envelope.meta.request_id.is_none() {
            return Err("Envelope metadata must include request_id".to_string());
        }
        
        if envelope.meta.timestamp.is_none() {
            return Err("Envelope metadata must include timestamp".to_string());
        }
        
        Ok(())
    }
    
    /// Generate comprehensive API examples for documentation and testing
    ///
    /// Creates a collection of diverse envelope examples demonstrating different
    /// use cases, security configurations, and complexity levels. These examples
    /// are ideal for API documentation, client testing, and integration validation.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` containing a JSON object with multiple envelope examples:
    /// - **success_envelope**: Full enterprise envelope with operational status
    /// - **error_envelope**: Complete error envelope with diagnostic information
    /// - **minimal_envelope**: Minimal viable envelope with basic metadata
    /// - **envelope_with_security**: Envelope demonstrating security metadata
    ///
    /// # Example
    ///
    /// ```rust
    /// use qollective::openapi::OpenApiUtils;
    ///
    /// let examples = OpenApiUtils::generate_api_examples();
    ///
    /// // Access different example types
    /// let success = &examples["success_envelope"];
    /// let error = &examples["error_envelope"];
    /// let minimal = &examples["minimal_envelope"];
    /// let secure = &examples["envelope_with_security"];
    ///
    /// // Verify example structure
    /// assert!(success["meta"]["tenant"].is_string());
    /// assert!(error["error"]["code"].is_string());
    /// assert!(minimal["data"]["message"].is_string());
    /// assert!(secure["meta"]["security"]["permissions"].is_array());
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **OpenAPI Documentation**: Examples for API specification files
    /// - **Client Development**: Test data for client SDK development  
    /// - **Integration Testing**: Validation data for system integration
    /// - **Training Materials**: Educational examples for developer onboarding
    /// - **Debugging**: Reference data for troubleshooting envelope issues
    ///
    /// # Example Types Explained
    ///
    /// ## Success Envelope
    /// Demonstrates successful operation with:
    /// - Complete metadata (tenant, version, timestamps)
    /// - Operational status payload
    /// - No error information
    ///
    /// ## Error Envelope  
    /// Shows comprehensive error reporting with:
    /// - Error codes and human-readable messages
    /// - Structured diagnostic details
    /// - Stack traces for debugging
    ///
    /// ## Minimal Envelope
    /// Illustrates minimum viable envelope with:
    /// - Basic required metadata only
    /// - Simple payload structure
    /// - No optional fields
    ///
    /// ## Security Envelope
    /// Demonstrates enterprise security with:
    /// - JWT authentication metadata
    /// - User permissions and roles
    /// - Multi-tenant security context
    pub fn generate_api_examples() -> serde_json::Value {
        json!({
            "success_envelope": Self::generate_example_envelope(),
            "error_envelope": Self::generate_example_error_envelope(),
            "minimal_envelope": {
                "meta": {
                    "timestamp": "2025-08-23T10:30:45.123Z",
                    "request_id": "01912345-1234-5678-9abc-123456789def",
                    "version": "1.0"
                },
                "data": {
                    "message": "Simple bridge communication",
                    "status": "ready"
                }
            },
            "envelope_with_security": {
                "meta": {
                    "timestamp": "2025-08-23T10:30:45.123Z",
                    "request_id": "01912345-1234-5678-9abc-123456789def", 
                    "version": "1.0",
                    "tenant": "enterprise_starfleet",
                    "security": {
                        "user_id": "picard@starfleet.local",
                        "session_id": "bridge_session_001",
                        "auth_method": "Jwt",
                        "permissions": ["COMMAND_SHIP", "CREW_MANAGEMENT"],
                        "roles": ["Captain", "Bridge Officer"],
                        "tenant_id": "enterprise_starfleet"
                    }
                },
                "data": {
                    "message": "Secure command from bridge",
                    "status": "authorized"
                }
            }
        })
    }
}

/// Swagger UI integration utilities for web-based API documentation
///
/// This helper struct provides utilities for integrating Swagger UI with web frameworks
/// to serve interactive API documentation. The Swagger UI interface allows developers
/// to explore the API, view schemas, and test endpoints directly from the browser.
///
/// # Features
///
/// - **Interactive Documentation**: Browse API schemas and examples in a web interface
/// - **Endpoint Testing**: Test API endpoints directly from the documentation
/// - **Schema Exploration**: Navigate complex nested schemas with ease
/// - **Framework Integration**: Ready-to-use configurations for popular web frameworks
///
/// # Example Integration
///
/// ## With Axum
///
/// ```rust
/// use axum::{routing::get, Router};
/// use qollective::openapi::{SwaggerUiHelper, QollectiveApiDoc};
/// use utoipa::OpenApi;
/// use utoipa_swagger_ui::SwaggerUi;
///
/// // Create router with Swagger UI
/// let app = Router::new()
///     .merge(SwaggerUi::new("/swagger-ui")
///         .url("/api-docs/openapi.json", QollectiveApiDoc::openapi())
///         .config(SwaggerUiHelper::generate_config())
///     );
/// ```
///
/// ## With Actix Web
///
/// ```rust
/// use actix_web::{web, App, HttpServer};
/// use qollective::openapi::{SwaggerUiHelper, QollectiveApiDoc};
/// use utoipa_swagger_ui::SwaggerUi;
///
/// let app = App::new()
///     .service(
///         SwaggerUi::new("/swagger-ui/{_:.*}")
///             .url("/api-docs/openapi.json", QollectiveApiDoc::openapi())
///             .config(SwaggerUiHelper::generate_config())
///     );
/// ```
#[cfg(feature = "openapi")]
pub struct SwaggerUiHelper;

#[cfg(feature = "openapi")]
impl SwaggerUiHelper {
    /// Generate optimized Swagger UI configuration for Qollective APIs
    ///
    /// Creates a pre-configured Swagger UI configuration optimized for displaying
    /// Qollective envelope-first API documentation. The configuration includes
    /// settings for proper display of complex nested schemas and metadata structures.
    ///
    /// # Returns
    ///
    /// A `utoipa_swagger_ui::Config` configured with:
    /// - API specification endpoint path
    /// - Base layout for better schema navigation
    /// - Optimized settings for envelope architecture display
    ///
    /// # Example
    ///
    /// ```rust
    /// use qollective::openapi::SwaggerUiHelper;
    /// use utoipa_swagger_ui::SwaggerUi;
    ///
    /// let config = SwaggerUiHelper::generate_config();
    /// 
    /// // Use with web framework
    /// let swagger_ui = SwaggerUi::new("/docs")
    ///     .config(config);
    /// ```
    ///
    /// # Configuration Details
    ///
    /// The generated configuration:
    /// - **Specification URL**: Points to `/api-docs/openapi.json` by default
    /// - **Base Layout**: Enabled for better navigation of complex schemas
    /// - **Theme**: Uses default Swagger UI theme for consistency
    /// - **Features**: All standard Swagger UI features enabled
    ///
    /// # Customization
    ///
    /// For custom configurations, modify the returned config:
    ///
    /// ```rust
    /// use qollective::openapi::SwaggerUiHelper;
    ///
    /// let mut config = SwaggerUiHelper::generate_config();
    /// // Add custom configuration options
    /// // config.custom_setting = value;
    /// ```
    pub fn generate_config() -> utoipa_swagger_ui::Config<'static> {
        utoipa_swagger_ui::Config::new(["/api-docs/openapi.json"])
            .use_base_layout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(feature = "openapi")]
    #[test]
    fn test_openapi_spec_generation() {
        let spec = OpenApiUtils::generate_spec();
        
        // Verify basic structure
        assert!(spec["info"].is_object());
        assert!(spec["components"].is_object());
        assert!(spec["components"]["schemas"].is_object());
        
        // Verify title and version
        assert_eq!(spec["info"]["title"], "Qollective Enterprise API");
        assert_eq!(spec["info"]["version"], "1.0.0");
        
        // Verify key schemas are present
        let schemas = &spec["components"]["schemas"];
        assert!(schemas["Envelope_EnterpriseMessage"].is_object());
        assert!(schemas["EnvelopeError"].is_object());
        assert!(schemas["Meta"].is_object());
        assert!(schemas["SecurityMeta"].is_object());
        assert!(schemas["EnhancedQollectiveError"].is_object());
    }
    
    #[cfg(feature = "openapi")]
    #[test] 
    fn test_example_envelope_generation() {
        let envelope = OpenApiUtils::generate_example_envelope();
        
        assert!(envelope.meta.tenant.is_some());
        assert_eq!(envelope.meta.tenant.as_ref().unwrap(), "enterprise_starfleet");
        assert!(envelope.meta.request_id.is_some());
        assert!(envelope.meta.timestamp.is_some());
        assert!(envelope.is_success());
    }
    
    #[cfg(feature = "openapi")]
    #[test]
    fn test_example_error_envelope_generation() {
        let envelope = OpenApiUtils::generate_example_error_envelope();
        
        assert!(envelope.has_error());
        assert!(envelope.error.is_some());
        
        let error = envelope.error.as_ref().unwrap();
        assert_eq!(error.code, "WARP_CORE_FAILURE");
        assert!(error.details.is_some());
        assert!(error.trace.is_some());
    }
    
    #[cfg(feature = "openapi")]
    #[test]
    fn test_envelope_schema_validation() {
        let valid_envelope = OpenApiUtils::generate_example_envelope();
        assert!(OpenApiUtils::validate_envelope_schema(&valid_envelope).is_ok());
        
        // Test with invalid envelope (missing required fields)
        let mut invalid_meta = Meta::default();
        invalid_meta.request_id = None; // Missing required field
        
        let message = EnterpriseMessage {
            message: "Test".to_string(),
            status: "test".to_string(),
            priority: None,
            created_at: None,
        };
        
        let invalid_envelope = Envelope::new(invalid_meta, message);
        assert!(OpenApiUtils::validate_envelope_schema(&invalid_envelope).is_err());
    }
    
    #[test]
    fn test_enterprise_message_creation() {
        let message = EnterpriseMessage {
            message: "Test message".to_string(),
            status: "ready".to_string(),
            priority: Some(2),
            created_at: Some(chrono::Utc::now()),
        };
        
        assert_eq!(message.message, "Test message");
        assert_eq!(message.status, "ready");
        assert_eq!(message.priority, Some(2));
        assert!(message.created_at.is_some());
    }
}