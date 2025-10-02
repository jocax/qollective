# OpenAPI Specification Examples

> Comprehensive examples demonstrating Qollective's OpenAPI 3.1 integration
> Last Updated: 2025-08-25
> Version: 1.0.0

## Overview

This document provides practical examples of using Qollective's OpenAPI 3.1 integration with utoipa for various common use cases. Each example demonstrates different aspects of schema generation, validation, and API documentation.

## Table of Contents

1. [Basic Data Structures](#1-basic-data-structures)
2. [Enterprise Command APIs](#2-enterprise-command-apis) 
3. [Multi-Tenant Resource Management](#3-multi-tenant-resource-management)
4. [Error Handling Patterns](#4-error-handling-patterns)
5. [Security and Authentication](#5-security-and-authentication)
6. [Performance Monitoring](#6-performance-monitoring)
7. [Agent-to-Agent Communication](#7-agent-to-agent-communication)
8. [Distributed Tracing Integration](#8-distributed-tracing-integration)
9. [WASM Browser Integration](#9-wasm-browser-integration)
10. [Complete API Service Example](#10-complete-api-service-example)

## 1. Basic Data Structures

### Simple Resource with Validation

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "User Profile",
    description = "User profile information with validation constraints",
    example = json!({
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "username": "enterprise_user",
        "email": "user@starfleet.org",
        "full_name": "Jean-Luc Picard",
        "age": 45,
        "department": "Command",
        "active": true,
        "created_at": "2024-01-15T10:30:00Z"
    })
))]
pub struct UserProfile {
    /// Unique user identifier
    #[cfg_attr(feature = "openapi", schema(example = "123e4567-e89b-12d3-a456-426614174000"))]
    pub id: Uuid,
    
    /// Username (3-50 characters, alphanumeric and underscore only)
    #[cfg_attr(feature = "openapi", schema(
        min_length = 3,
        max_length = 50,
        pattern = r"^[a-zA-Z0-9_]+$",
        example = "enterprise_user"
    ))]
    pub username: String,
    
    /// Email address
    #[cfg_attr(feature = "openapi", schema(
        format = "email",
        example = "user@starfleet.org"
    ))]
    pub email: String,
    
    /// Full display name
    #[cfg_attr(feature = "openapi", schema(
        min_length = 1,
        max_length = 100,
        example = "Jean-Luc Picard"
    ))]
    pub full_name: String,
    
    /// User age (18-120 years)
    #[cfg_attr(feature = "openapi", schema(minimum = 18, maximum = 120, example = 45))]
    pub age: u8,
    
    /// Department or division
    #[cfg_attr(feature = "openapi", schema(example = "Command"))]
    pub department: Option<String>,
    
    /// Account status
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub active: bool,
    
    /// Account creation timestamp
    #[cfg_attr(feature = "openapi", schema(example = "2024-01-15T10:30:00Z"))]
    pub created_at: DateTime<Utc>,
}
```

### Enumerated Types with Documentation

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Resource Status",
    description = "Current status of a system resource"
))]
#[serde(rename_all = "snake_case")]
pub enum ResourceStatus {
    /// Resource is active and operational
    Active,
    /// Resource is temporarily inactive but can be reactivated
    Inactive,
    /// Resource is under maintenance
    Maintenance,
    /// Resource has failed and requires intervention
    Failed,
    /// Resource is being decommissioned
    Decommissioning,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "System Resource",
    description = "Represents a system resource with status tracking"
))]
pub struct SystemResource {
    #[cfg_attr(feature = "openapi", schema(example = "warp-core-1"))]
    pub id: String,
    
    #[cfg_attr(feature = "openapi", schema(example = "Warp Core Primary"))]
    pub name: String,
    
    pub status: ResourceStatus,
    
    /// CPU utilization percentage (0.0-100.0)
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, maximum = 100.0, example = 45.7))]
    pub cpu_usage: f32,
    
    /// Memory utilization percentage (0.0-100.0)
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, maximum = 100.0, example = 62.3))]
    pub memory_usage: f32,
}
```

## 2. Enterprise Command APIs

### Command Request/Response Pattern

```rust
use qollective::envelope::{Envelope, EnvelopeBuilder};
use qollective::envelope::meta::Meta;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Starfleet Command Request",
    description = "Request to execute a command on a Starfleet vessel or system"
))]
pub struct CommandRequest {
    /// Command identifier
    #[cfg_attr(feature = "openapi", schema(example = "CMD_001"))]
    pub command_id: String,
    
    /// Command type to execute
    pub command_type: CommandType,
    
    /// Target system or vessel
    #[cfg_attr(feature = "openapi", schema(example = "USS_Enterprise"))]
    pub target: String,
    
    /// Command parameters as JSON object
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "warp_factor": 5,
            "destination": "Wolf 359",
            "engagement_time": "2024-01-15T14:30:00Z"
        })
    ))]
    pub parameters: serde_json::Value,
    
    /// Priority level (1 = highest, 5 = lowest)
    #[cfg_attr(feature = "openapi", schema(minimum = 1, maximum = 5, example = 2))]
    pub priority: u8,
    
    /// Optional timeout in seconds
    #[cfg_attr(feature = "openapi", schema(minimum = 1, maximum = 3600, example = 300))]
    pub timeout_seconds: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Command Type",
    description = "Available command types for Starfleet operations"
))]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    /// Navigation and movement commands
    Navigation,
    /// Tactical and defensive operations
    Tactical,
    /// Engineering and system management
    Engineering,
    /// Communication protocols
    Communications,
    /// Medical and life support
    Medical,
    /// Science and exploration
    Science,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Command Response",
    description = "Response from command execution with status and results"
))]
pub struct CommandResponse {
    /// Original command identifier
    #[cfg_attr(feature = "openapi", schema(example = "CMD_001"))]
    pub command_id: String,
    
    /// Execution status
    pub status: CommandStatus,
    
    /// Execution results or output data
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "current_warp": 5.0,
            "eta": "2024-01-15T16:45:00Z",
            "distance_remaining": "12.7 light-years"
        })
    ))]
    pub result: Option<serde_json::Value>,
    
    /// Execution time in milliseconds
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 1250))]
    pub execution_time_ms: u64,
    
    /// Any warnings or informational messages
    pub warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    /// Command completed successfully
    Success,
    /// Command is still being processed
    InProgress,
    /// Command failed with error
    Failed,
    /// Command was cancelled
    Cancelled,
    /// Command timed out
    TimedOut,
}
```

### API Endpoints with OpenAPI Documentation

```rust
use axum::{extract::Path, response::Json, http::StatusCode};
use utoipa::{OpenApi, path};

/// Execute a command on a Starfleet system
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/api/v1/commands",
    request_body = CommandRequest,
    responses(
        (status = 200, description = "Command executed successfully", body = Envelope<CommandResponse>),
        (status = 400, description = "Invalid command request", body = EnvelopeError),
        (status = 401, description = "Unauthorized access", body = EnvelopeError),
        (status = 500, description = "Internal server error", body = EnvelopeError),
    ),
    tag = "Commands"
))]
pub async fn execute_command(
    Json(request): Json<CommandRequest>
) -> Result<Json<Envelope<CommandResponse>>, StatusCode> {
    // Implementation would go here
    todo!()
}

/// Get command execution status
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/v1/commands/{command_id}",
    params(
        ("command_id" = String, Path, description = "Command identifier")
    ),
    responses(
        (status = 200, description = "Command status retrieved", body = Envelope<CommandResponse>),
        (status = 404, description = "Command not found", body = EnvelopeError),
    ),
    tag = "Commands"
))]
pub async fn get_command_status(
    Path(command_id): Path<String>
) -> Result<Json<Envelope<CommandResponse>>, StatusCode> {
    // Implementation would go here
    todo!()
}
```

## 3. Multi-Tenant Resource Management

### Tenant-Aware Resource Model

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Multi-Tenant Resource",
    description = "Resource that belongs to a specific tenant with access controls"
))]
pub struct TenantResource {
    /// Resource identifier (unique within tenant)
    #[cfg_attr(feature = "openapi", schema(example = "resource_001"))]
    pub id: String,
    
    /// Tenant identifier
    #[cfg_attr(feature = "openapi", schema(example = "starfleet_command"))]
    pub tenant_id: String,
    
    /// Resource display name
    #[cfg_attr(feature = "openapi", schema(example = "Bridge Command Console"))]
    pub name: String,
    
    /// Resource type and category
    pub resource_type: ResourceType,
    
    /// Access control permissions
    pub permissions: ResourcePermissions,
    
    /// Resource configuration as JSON
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "max_concurrent_users": 5,
            "session_timeout": 3600,
            "logging_level": "info"
        })
    ))]
    pub configuration: serde_json::Value,
    
    /// Current usage statistics
    pub usage: ResourceUsage,
    
    /// Resource creation timestamp
    #[cfg_attr(feature = "openapi", schema(example = "2024-01-15T10:30:00Z"))]
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    #[cfg_attr(feature = "openapi", schema(example = "2024-01-16T14:20:00Z"))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// Computing resources (CPU, memory)
    Compute,
    /// Storage resources (databases, file systems)
    Storage,
    /// Network resources (bandwidth, connections)
    Network,
    /// Application services
    Application,
    /// AI/ML model resources
    Model,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Resource Permissions",
    description = "Access control permissions for a resource"
))]
pub struct ResourcePermissions {
    /// Users with read access
    #[cfg_attr(feature = "openapi", schema(example = json!(["picard", "riker", "data"])))]
    pub read_users: Vec<String>,
    
    /// Users with write access  
    #[cfg_attr(feature = "openapi", schema(example = json!(["picard", "riker"])))]
    pub write_users: Vec<String>,
    
    /// Users with admin access
    #[cfg_attr(feature = "openapi", schema(example = json!(["picard"])))]
    pub admin_users: Vec<String>,
    
    /// Roles with read access
    #[cfg_attr(feature = "openapi", schema(example = json!(["bridge_crew", "engineering"])))]
    pub read_roles: Vec<String>,
    
    /// Roles with write access
    #[cfg_attr(feature = "openapi", schema(example = json!(["bridge_crew"])))]
    pub write_roles: Vec<String>,
    
    /// Roles with admin access
    #[cfg_attr(feature = "openapi", schema(example = json!(["captain"])))]
    pub admin_roles: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Resource Usage",
    description = "Current usage statistics for a resource"
))]
pub struct ResourceUsage {
    /// Number of active users/connections
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 3))]
    pub active_connections: u32,
    
    /// CPU utilization percentage
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, maximum = 100.0, example = 25.5))]
    pub cpu_usage_percent: f32,
    
    /// Memory utilization percentage
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, maximum = 100.0, example = 45.2))]
    pub memory_usage_percent: f32,
    
    /// Network bandwidth utilization in Mbps
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, example = 15.7))]
    pub bandwidth_mbps: f32,
    
    /// Total requests processed in the last hour
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 1250))]
    pub requests_per_hour: u32,
}
```

## 4. Error Handling Patterns

### Comprehensive Error Types

```rust
use qollective::envelope::builder::EnvelopeError;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "API Error",
    description = "Comprehensive error information for API responses"
))]
pub struct ApiError {
    /// Error code for programmatic handling
    #[cfg_attr(feature = "openapi", schema(example = "VALIDATION_FAILED"))]
    pub code: String,
    
    /// Human-readable error message
    #[cfg_attr(feature = "openapi", schema(example = "The provided command parameters are invalid"))]
    pub message: String,
    
    /// Detailed error context and debugging information
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "field": "warp_factor",
            "value": 15,
            "constraint": "maximum value is 10",
            "suggestion": "Use a warp factor between 1 and 10"
        })
    ))]
    pub details: Option<serde_json::Value>,
    
    /// Stack trace for debugging (only in development)
    #[cfg_attr(feature = "openapi", schema(
        example = "at command_validator.rs:45 -> warp_drive.rs:123"
    ))]
    pub trace: Option<String>,
    
    /// Timestamp when error occurred
    #[cfg_attr(feature = "openapi", schema(example = "2024-01-15T14:30:00Z"))]
    pub timestamp: DateTime<Utc>,
    
    /// Request ID for tracing and support
    #[cfg_attr(feature = "openapi", schema(example = "req_123e4567e89b12d3"))]
    pub request_id: Option<String>,
    
    /// Suggested remediation actions
    #[cfg_attr(feature = "openapi", schema(
        example = json!([
            "Check the command parameters documentation",
            "Verify your authorization level",
            "Contact support if the issue persists"
        ])
    ))]
    pub suggestions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Validation Error",
    description = "Specific validation error with field-level details"
))]
pub struct ValidationError {
    /// Field name that failed validation
    #[cfg_attr(feature = "openapi", schema(example = "email"))]
    pub field: String,
    
    /// Provided value that failed
    #[cfg_attr(feature = "openapi", schema(example = "invalid-email"))]
    pub value: serde_json::Value,
    
    /// Validation rule that was violated
    #[cfg_attr(feature = "openapi", schema(example = "must be a valid email address"))]
    pub constraint: String,
    
    /// Error code for this specific validation
    #[cfg_attr(feature = "openapi", schema(example = "INVALID_EMAIL_FORMAT"))]
    pub code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Batch Validation Error",
    description = "Multiple validation errors from batch operations"
))]
pub struct BatchValidationError {
    /// Overall error message
    #[cfg_attr(feature = "openapi", schema(example = "Multiple validation errors occurred"))]
    pub message: String,
    
    /// Individual field validation errors
    pub errors: Vec<ValidationError>,
    
    /// Number of items that passed validation
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 5))]
    pub valid_count: u32,
    
    /// Total number of items processed
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 8))]
    pub total_count: u32,
}
```

## 5. Security and Authentication

### JWT Authentication and Authorization

```rust
use qollective::envelope::meta::{SecurityMeta, AuthMethod};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Authentication Request",
    description = "Request to authenticate and receive access tokens"
))]
pub struct AuthRequest {
    /// Username or email
    #[cfg_attr(feature = "openapi", schema(example = "picard@starfleet.org"))]
    pub identifier: String,
    
    /// Password or other credential
    #[cfg_attr(feature = "openapi", schema(example = "••••••••"))]
    pub credential: String,
    
    /// Authentication method
    pub auth_method: AuthMethod,
    
    /// Requested token lifetime in seconds
    #[cfg_attr(feature = "openapi", schema(minimum = 300, maximum = 86400, example = 3600))]
    pub token_lifetime_seconds: Option<u32>,
    
    /// Requested scopes/permissions
    #[cfg_attr(feature = "openapi", schema(
        example = json!(["command:read", "navigation:write", "tactical:admin"])
    ))]
    pub requested_scopes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Authentication Response",
    description = "Successful authentication response with tokens and user info"
))]
pub struct AuthResponse {
    /// JWT access token
    #[cfg_attr(feature = "openapi", schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."))]
    pub access_token: String,
    
    /// Refresh token for obtaining new access tokens
    #[cfg_attr(feature = "openapi", schema(example = "rt_123e4567e89b12d3a456426614174000"))]
    pub refresh_token: String,
    
    /// Token type (always "Bearer")
    #[cfg_attr(feature = "openapi", schema(example = "Bearer"))]
    pub token_type: String,
    
    /// Token expiration time in seconds
    #[cfg_attr(feature = "openapi", schema(example = 3600))]
    pub expires_in: u32,
    
    /// Granted scopes/permissions
    #[cfg_attr(feature = "openapi", schema(
        example = json!(["command:read", "navigation:write"])
    ))]
    pub granted_scopes: Vec<String>,
    
    /// User information
    pub user: AuthenticatedUser,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Authenticated User",
    description = "User information included in authentication response"
))]
pub struct AuthenticatedUser {
    /// User identifier
    #[cfg_attr(feature = "openapi", schema(example = "user_001"))]
    pub id: String,
    
    /// Username
    #[cfg_attr(feature = "openapi", schema(example = "picard"))]
    pub username: String,
    
    /// Display name
    #[cfg_attr(feature = "openapi", schema(example = "Captain Jean-Luc Picard"))]
    pub display_name: String,
    
    /// Email address
    #[cfg_attr(feature = "openapi", schema(example = "picard@starfleet.org"))]
    pub email: String,
    
    /// User roles
    #[cfg_attr(feature = "openapi", schema(example = json!(["captain", "bridge_crew"])))]
    pub roles: Vec<String>,
    
    /// User permissions
    #[cfg_attr(feature = "openapi", schema(
        example = json!(["command:all", "navigation:all", "tactical:all"])
    ))]
    pub permissions: Vec<String>,
    
    /// Tenant identifier
    #[cfg_attr(feature = "openapi", schema(example = "starfleet_enterprise"))]
    pub tenant_id: String,
}
```

## 6. Performance Monitoring

### Performance Metrics and Monitoring

```rust
use qollective::envelope::meta::{PerformanceMeta, ExternalCall, CallStatus, CacheOperations};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Performance Report",
    description = "Comprehensive performance metrics for system monitoring"
))]
pub struct PerformanceReport {
    /// Time period for this report
    pub period: TimePeriod,
    
    /// System performance metrics
    pub system_metrics: SystemMetrics,
    
    /// Application-specific metrics
    pub application_metrics: ApplicationMetrics,
    
    /// Database performance
    pub database_metrics: DatabaseMetrics,
    
    /// External service call statistics
    pub external_calls: Vec<ExternalServiceMetrics>,
    
    /// Cache performance
    pub cache_metrics: CacheMetrics,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TimePeriod {
    /// Start of measurement period
    #[cfg_attr(feature = "openapi", schema(example = "2024-01-15T14:00:00Z"))]
    pub start: DateTime<Utc>,
    
    /// End of measurement period
    #[cfg_attr(feature = "openapi", schema(example = "2024-01-15T15:00:00Z"))]
    pub end: DateTime<Utc>,
    
    /// Duration in seconds
    #[cfg_attr(feature = "openapi", schema(example = 3600))]
    pub duration_seconds: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SystemMetrics {
    /// Average CPU utilization percentage
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, maximum = 100.0, example = 35.7))]
    pub cpu_usage_avg: f32,
    
    /// Peak CPU utilization percentage
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, maximum = 100.0, example = 78.2))]
    pub cpu_usage_max: f32,
    
    /// Average memory utilization percentage
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, maximum = 100.0, example = 62.1))]
    pub memory_usage_avg: f32,
    
    /// Peak memory utilization in MB
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 1024))]
    pub memory_peak_mb: u32,
    
    /// Average network I/O in MB/s
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, example = 15.3))]
    pub network_io_avg_mbps: f32,
    
    /// Number of garbage collections
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 25))]
    pub gc_collections: u32,
    
    /// Total GC time in milliseconds
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 450))]
    pub gc_time_ms: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DatabaseMetrics {
    /// Total number of queries executed
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 15420))]
    pub query_count: u32,
    
    /// Average query execution time in milliseconds
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, example = 12.5))]
    pub query_time_avg_ms: f32,
    
    /// Slowest query time in milliseconds
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, example = 1250.0))]
    pub query_time_max_ms: f32,
    
    /// Number of slow queries (>1000ms)
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 5))]
    pub slow_query_count: u32,
    
    /// Database connection pool statistics
    pub connection_pool: ConnectionPoolMetrics,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConnectionPoolMetrics {
    /// Current active connections
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 8))]
    pub active_connections: u32,
    
    /// Maximum connections allowed
    #[cfg_attr(feature = "openapi", schema(minimum = 1, example = 50))]
    pub max_connections: u32,
    
    /// Average connection wait time in milliseconds
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, example = 5.2))]
    pub avg_wait_time_ms: f32,
    
    /// Number of connection timeouts
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 2))]
    pub timeout_count: u32,
}
```

## 7. Agent-to-Agent Communication

### MCP Tool Registration and Execution

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "MCP Tool",
    description = "Model Context Protocol tool definition for agent communication"
))]
pub struct McpTool {
    /// Tool identifier
    #[cfg_attr(feature = "openapi", schema(example = "navigate_to_coordinates"))]
    pub name: String,
    
    /// Human-readable description
    #[cfg_attr(feature = "openapi", schema(
        example = "Navigate the starship to specified galactic coordinates"
    ))]
    pub description: String,
    
    /// JSON schema for tool parameters
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "type": "object",
            "properties": {
                "x": {"type": "number", "description": "X coordinate"},
                "y": {"type": "number", "description": "Y coordinate"},
                "z": {"type": "number", "description": "Z coordinate"},
                "warp_factor": {"type": "number", "minimum": 1, "maximum": 10}
            },
            "required": ["x", "y", "z"]
        })
    ))]
    pub parameters_schema: serde_json::Value,
    
    /// Tool category for organization
    #[cfg_attr(feature = "openapi", schema(example = "navigation"))]
    pub category: String,
    
    /// Required permissions to execute this tool
    #[cfg_attr(feature = "openapi", schema(example = json!(["navigation:write", "command:execute"])))]
    pub required_permissions: Vec<String>,
    
    /// Estimated execution time in seconds
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 30))]
    pub estimated_duration_seconds: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Tool Execution Request",
    description = "Request to execute an MCP tool with specific parameters"
))]
pub struct ToolExecutionRequest {
    /// Tool name to execute
    #[cfg_attr(feature = "openapi", schema(example = "navigate_to_coordinates"))]
    pub tool_name: String,
    
    /// Tool parameters as JSON object
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "x": 123.45,
            "y": -67.89,
            "z": 234.56,
            "warp_factor": 5
        })
    ))]
    pub parameters: serde_json::Value,
    
    /// Execution context and metadata
    pub context: ToolExecutionContext,
    
    /// Maximum execution time in seconds
    #[cfg_attr(feature = "openapi", schema(minimum = 1, maximum = 3600, example = 300))]
    pub timeout_seconds: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ToolExecutionContext {
    /// Agent or user requesting execution
    #[cfg_attr(feature = "openapi", schema(example = "bridge_command_agent"))]
    pub requester_id: String,
    
    /// Session or conversation ID
    #[cfg_attr(feature = "openapi", schema(example = "session_123e4567"))]
    pub session_id: String,
    
    /// Additional context data
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "priority": "high",
            "mission_id": "EXPLORATION_001",
            "authorization_level": "captain"
        })
    ))]
    pub metadata: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Tool Execution Response",
    description = "Response from tool execution with results and status"
))]
pub struct ToolExecutionResponse {
    /// Original tool name
    #[cfg_attr(feature = "openapi", schema(example = "navigate_to_coordinates"))]
    pub tool_name: String,
    
    /// Execution status
    pub status: ToolExecutionStatus,
    
    /// Tool execution results
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "navigation_started": true,
            "estimated_arrival": "2024-01-15T16:45:00Z",
            "current_position": {"x": 100.0, "y": -50.0, "z": 200.0},
            "distance_to_target": 45.7
        })
    ))]
    pub result: Option<serde_json::Value>,
    
    /// Execution time in milliseconds
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 2500))]
    pub execution_time_ms: u64,
    
    /// Any errors that occurred
    pub error: Option<ToolExecutionError>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ToolExecutionStatus {
    Success,
    Failed,
    Timeout,
    Cancelled,
    PartialSuccess,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ToolExecutionError {
    /// Error code
    #[cfg_attr(feature = "openapi", schema(example = "INVALID_COORDINATES"))]
    pub code: String,
    
    /// Error message
    #[cfg_attr(feature = "openapi", schema(
        example = "The specified coordinates are outside known space"
    ))]
    pub message: String,
    
    /// Additional error details
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "provided_coordinates": {"x": 999999, "y": 999999, "z": 999999},
            "valid_range": {"max_x": 1000, "max_y": 1000, "max_z": 1000}
        })
    ))]
    pub details: Option<serde_json::Value>,
}
```

## 8. Distributed Tracing Integration

### OpenTelemetry Integration

```rust
use qollective::envelope::meta::{TracingMeta, SpanKind, SpanStatus, SpanStatusCode, TraceValue};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Distributed Trace",
    description = "Distributed tracing information for request correlation"
))]
pub struct DistributedTrace {
    /// Unique trace identifier
    #[cfg_attr(feature = "openapi", schema(example = "trace_12345678abcdef"))]
    pub trace_id: String,
    
    /// Root span information
    pub root_span: SpanInfo,
    
    /// All spans in this trace
    pub spans: Vec<SpanInfo>,
    
    /// Trace duration in milliseconds
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 1250))]
    pub total_duration_ms: u64,
    
    /// Services involved in this trace
    #[cfg_attr(feature = "openapi", schema(
        example = json!(["bridge-service", "navigation-service", "warp-drive-service"])
    ))]
    pub services: Vec<String>,
    
    /// Trace sampling rate
    #[cfg_attr(feature = "openapi", schema(minimum = 0.0, maximum = 1.0, example = 1.0))]
    pub sampling_rate: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SpanInfo {
    /// Span identifier
    #[cfg_attr(feature = "openapi", schema(example = "span_abcdef123456"))]
    pub span_id: String,
    
    /// Parent span ID (if not root)
    #[cfg_attr(feature = "openapi", schema(example = "span_parent_789"))]
    pub parent_span_id: Option<String>,
    
    /// Operation name
    #[cfg_attr(feature = "openapi", schema(example = "execute_navigation_command"))]
    pub operation_name: String,
    
    /// Service name
    #[cfg_attr(feature = "openapi", schema(example = "navigation-service"))]
    pub service_name: String,
    
    /// Span start time
    #[cfg_attr(feature = "openapi", schema(example = "2024-01-15T14:30:00.123Z"))]
    pub start_time: DateTime<Utc>,
    
    /// Span end time
    #[cfg_attr(feature = "openapi", schema(example = "2024-01-15T14:30:01.875Z"))]
    pub end_time: DateTime<Utc>,
    
    /// Span duration in milliseconds
    #[cfg_attr(feature = "openapi", schema(minimum = 0, example = 1752))]
    pub duration_ms: u64,
    
    /// Span kind
    pub kind: SpanKind,
    
    /// Span status
    pub status: SpanStatus,
    
    /// Span tags/attributes
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "http.method": "POST",
            "http.url": "/api/v1/navigation",
            "http.status_code": 200,
            "user.id": "picard_001",
            "component": "navigation"
        })
    ))]
    pub tags: HashMap<String, TraceValue>,
    
    /// Span events/logs
    pub events: Vec<SpanEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SpanEvent {
    /// Event timestamp
    #[cfg_attr(feature = "openapi", schema(example = "2024-01-15T14:30:00.500Z"))]
    pub timestamp: DateTime<Utc>,
    
    /// Event name
    #[cfg_attr(feature = "openapi", schema(example = "warp_drive_engaged"))]
    pub name: String,
    
    /// Event attributes
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "warp_factor": 5,
            "engine_status": "optimal",
            "antimatter_level": 95.5
        })
    ))]
    pub attributes: HashMap<String, TraceValue>,
}
```

## 9. WASM Browser Integration

### Browser-Compatible Types

```rust
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "WASM Client Config",
    description = "Configuration for WASM client initialization in browser"
))]
#[wasm_bindgen]
pub struct WasmClientConfig {
    /// WebSocket endpoint URL
    #[cfg_attr(feature = "openapi", schema(
        format = "uri",
        example = "wss://api.starfleet.org/ws"
    ))]
    pub websocket_url: String,
    
    /// REST API base URL
    #[cfg_attr(feature = "openapi", schema(
        format = "uri", 
        example = "https://api.starfleet.org/api/v1"
    ))]
    pub api_base_url: String,
    
    /// Authentication token
    #[cfg_attr(feature = "openapi", schema(example = "Bearer eyJhbGciOiJIUzI1NiIs..."))]
    pub auth_token: Option<String>,
    
    /// Connection timeout in milliseconds
    #[cfg_attr(feature = "openapi", schema(minimum = 1000, maximum = 60000, example = 5000))]
    pub timeout_ms: u32,
    
    /// Maximum retry attempts
    #[cfg_attr(feature = "openapi", schema(minimum = 0, maximum = 10, example = 3))]
    pub max_retries: u32,
    
    /// Enable debug logging
    #[cfg_attr(feature = "openapi", schema(example = false))]
    pub debug_logging: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(
    title = "Browser Event",
    description = "Event data sent from browser to server via WebSocket"
))]
#[wasm_bindgen]
pub struct BrowserEvent {
    /// Event type identifier
    #[cfg_attr(feature = "openapi", schema(example = "user_interaction"))]
    pub event_type: String,
    
    /// Event timestamp in browser
    #[cfg_attr(feature = "openapi", schema(example = 1705327800000))]
    pub timestamp_ms: u64,
    
    /// Browser session identifier
    #[cfg_attr(feature = "openapi", schema(example = "session_browser_123"))]
    pub session_id: String,
    
    /// User identifier (if authenticated)
    #[cfg_attr(feature = "openapi", schema(example = "user_picard"))]
    pub user_id: Option<String>,
    
    /// Event payload data
    #[cfg_attr(feature = "openapi", schema(
        example = json!({
            "action": "button_click",
            "element_id": "engage_button",
            "page": "/bridge/command",
            "coordinates": {"x": 150, "y": 300}
        })
    ))]
    pub payload: serde_json::Value,
    
    /// Browser information
    pub browser_info: BrowserInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[wasm_bindgen]
pub struct BrowserInfo {
    /// User agent string
    #[cfg_attr(feature = "openapi", schema(
        example = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
    ))]
    pub user_agent: String,
    
    /// Browser language
    #[cfg_attr(feature = "openapi", schema(example = "en-US"))]
    pub language: String,
    
    /// Screen resolution
    #[cfg_attr(feature = "openapi", schema(example = "1920x1080"))]
    pub screen_resolution: String,
    
    /// Current page URL
    #[cfg_attr(feature = "openapi", schema(example = "https://starfleet.org/bridge/command"))]
    pub current_url: String,
    
    /// Referrer URL
    #[cfg_attr(feature = "openapi", schema(example = "https://starfleet.org/bridge"))]
    pub referrer: Option<String>,
}
```

## 10. Complete API Service Example

### Full REST API with OpenAPI Documentation

```rust
use axum::{routing::{get, post, put, delete}, Router};
use utoipa::{OpenApi, path};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_users,
        create_user,
        get_user_by_id,
        update_user,
        delete_user,
        execute_command,
        get_command_status,
        authenticate,
        get_performance_metrics
    ),
    components(
        schemas(
            UserProfile,
            CommandRequest,
            CommandResponse,
            CommandType,
            CommandStatus,
            AuthRequest,
            AuthResponse,
            AuthenticatedUser,
            PerformanceReport,
            ApiError,
            ValidationError,
            EnvelopeError,
            // Add all other schemas from above examples
        )
    ),
    tags(
        (name = "Users", description = "User management operations"),
        (name = "Commands", description = "Command execution system"),
        (name = "Authentication", description = "Authentication and authorization"),
        (name = "Monitoring", description = "Performance and health monitoring")
    ),
    info(
        title = "Starfleet Command API",
        version = "2.0.0",
        description = "Comprehensive API for Starfleet vessel and crew management",
        contact(
            name = "Starfleet Engineering",
            email = "engineering@starfleet.org"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "https://api.starfleet.org/v1", description = "Production server"),
        (url = "https://staging-api.starfleet.org/v1", description = "Staging server"),
        (url = "http://localhost:3000/v1", description = "Development server")
    )
)]
pub struct ApiDoc;

/// Get all users with pagination
#[utoipa::path(
    get,
    path = "/users",
    params(
        ("page" = Option<u32>, Query, description = "Page number (1-based)"),
        ("limit" = Option<u32>, Query, description = "Items per page (1-100)"),
        ("department" = Option<String>, Query, description = "Filter by department"),
        ("active" = Option<bool>, Query, description = "Filter by active status")
    ),
    responses(
        (status = 200, description = "Users retrieved successfully", 
         body = Envelope<Vec<UserProfile>>,
         headers(
             ("X-Total-Count" = u32, description = "Total number of users"),
             ("X-Page-Count" = u32, description = "Total number of pages")
         )
        ),
        (status = 400, description = "Invalid query parameters", body = EnvelopeError),
        (status = 401, description = "Authentication required", body = EnvelopeError),
        (status = 500, description = "Internal server error", body = EnvelopeError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Users"
)]
async fn get_users() -> Json<Envelope<Vec<UserProfile>>> {
    // Implementation
    todo!()
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/users",
    request_body = UserProfile,
    responses(
        (status = 201, description = "User created successfully", body = Envelope<UserProfile>),
        (status = 400, description = "Invalid user data", body = EnvelopeError),
        (status = 401, description = "Authentication required", body = EnvelopeError),
        (status = 403, description = "Permission denied", body = EnvelopeError),
        (status = 409, description = "User already exists", body = EnvelopeError),
        (status = 500, description = "Internal server error", body = EnvelopeError)
    ),
    security(
        ("bearer_auth" = ["users:create"])
    ),
    tag = "Users"
)]
async fn create_user(Json(user): Json<UserProfile>) -> Json<Envelope<UserProfile>> {
    // Implementation
    todo!()
}

pub fn create_router() -> Router {
    Router::new()
        .route("/users", get(get_users).post(create_user))
        .route("/users/:id", get(get_user_by_id).put(update_user).delete(delete_user))
        .route("/commands", post(execute_command))
        .route("/commands/:id", get(get_command_status))
        .route("/auth/login", post(authenticate))
        .route("/metrics/performance", get(get_performance_metrics))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

// Security scheme definitions for OpenAPI
#[derive(utoipa::ToSchema)]
#[schema(
    security_scheme(
        scheme = "Bearer",
        bearer_format = "JWT",
        description = "JWT Bearer token authentication"
    )
)]
struct BearerAuth;

#[tokio::main]
async fn main() {
    let app = create_router();
    
    println!("🚀 Starfleet Command API starting...");
    println!("📚 Swagger UI available at: http://localhost:3000/swagger-ui");
    println!("📋 OpenAPI spec available at: http://localhost:3000/api-docs/openapi.json");
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## Usage Examples

### Example: Generating OpenAPI Specification

```rust
use qollective::openapi::{OpenApiUtils, QollectiveApiDoc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate complete OpenAPI specification
    let spec = OpenApiUtils::generate_spec();
    
    // Convert to JSON
    let spec_json = serde_json::to_string_pretty(&spec)?;
    println!("OpenAPI Specification:\n{}", spec_json);
    
    // Save to file
    std::fs::write("openapi.json", spec_json)?;
    
    // Generate example envelopes
    let example_envelope = OpenApiUtils::generate_example_envelope();
    println!("Example envelope: {:?}", example_envelope);
    
    Ok(())
}
```

### Example: Validating Against Schema

```rust
use jsonschema::{JSONSchema, ValidationError};

fn validate_user_profile(profile: &UserProfile) -> Result<(), Vec<ValidationError>> {
    // Get schema from OpenAPI spec
    let spec = OpenApiUtils::generate_spec();
    let user_schema = spec.components
        .as_ref()
        .and_then(|c| c.schemas.get("UserProfile"))
        .expect("UserProfile schema not found");
    
    // Create validator
    let schema = JSONSchema::compile(&user_schema)
        .expect("Failed to compile schema");
    
    // Validate instance
    let instance = serde_json::to_value(profile)?;
    
    match schema.validate(&instance) {
        Ok(_) => Ok(()),
        Err(errors) => Err(errors.collect())
    }
}
```

## Integration with Axum/Actix Web

### Axum Integration

```rust
use axum::{extract::Query, response::Json};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn create_api_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_users, create_user))
        .routes(routes!(execute_command, get_command_status))
        .routes(routes!(authenticate))
        .layer(cors_layer())
        .layer(auth_middleware())
}
```

## Best Practices

1. **Schema Documentation**: Always include comprehensive descriptions and examples
2. **Validation Constraints**: Use schema constraints (minimum, maximum, pattern) appropriately
3. **Error Handling**: Provide detailed error responses with structured information
4. **Security**: Document authentication requirements and permission levels
5. **Versioning**: Include API version information in paths and schemas
6. **Examples**: Provide realistic, helpful examples for all data structures
7. **Feature Gating**: Use conditional compilation for OpenAPI features
8. **Performance**: Consider the overhead of schema generation in production

## Testing OpenAPI Integration

```bash
# Run OpenAPI demo example
cargo run --bin openapi_demo --features openapi

# Generate and validate OpenAPI specification
cargo test test_openapi_schema_generation -- --test-threads=1

# Test with all features enabled
cargo test --all-features -- --test-threads=1
```

This comprehensive guide demonstrates how to leverage Qollective's OpenAPI 3.1 integration for building well-documented, type-safe APIs with comprehensive schema validation and interactive documentation.