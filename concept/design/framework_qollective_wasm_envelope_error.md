# WASM Envelope Error Handling Scenario

## Scenario Description

Comprehensive error handling across the entire envelope communication stack, from network failures to business logic errors. This scenario demonstrates how QollectiveError provides consistent error semantics while envelope metadata controls error propagation behavior.

## Flow Pattern

```
Error Source → Component Error Handler → QollectiveError → Envelope → Error Propagation → UI Translation
```

## Transport Protocol Error Analysis

| Error Type | HTTPS Impact | NATS Impact | WebSocket Impact | Recovery Strategy | User Impact |
|------------|--------------|-------------|------------------|-------------------|-------------|
| **Network Timeout** | 🔴 Request Fails | 🔴 Message Lost | 🔴 Connection Drop | ↻ Retry + Fallback | ⚠️ Degraded UX |
| **Authentication** | 🔴 401/403 Response | 🔴 Connection Denied | 🔴 Handshake Fail | 🔑 Re-auth Required | 🚫 Access Denied |
| **Serialization** | 🔴 400 Bad Request | 🔴 Message Rejected | 🔴 Frame Error | 🔧 Client Fix Required | ❌ Invalid Request |
| **Service Unavailable** | 🔴 503 Response | ⚠️ No Responders | ⚠️ Backpressure | ↻ Circuit Breaker | ⏳ Service Down |
| **Rate Limited** | 🟡 429 Response | 🟡 Slow Consumer | 🟡 Flow Control | ⏸️ Backoff Strategy | ⏱️ Please Wait |

## QollectiveError Structure

### Error Categories
```rust
#[derive(Debug, Error, Clone)]
pub enum QollectiveError {
    // Transport layer errors
    Transport(String),          // Network, TLS, connection issues
    Connection(String),         // Connection establishment failures
    NatsConnection(String),     // NATS-specific connection errors
    NatsTimeout(String),        // NATS timeout errors
    
    // Protocol layer errors  
    Serialization(String),      // JSON, envelope parsing errors
    Deserialization(String),    // Response parsing errors
    ProtocolAdapter(String),    // MCP adapter errors
    
    // Security layer errors
    Security(String),           // Authentication, authorization
    TenantExtraction(String),   // Multi-tenant context errors
    
    // Business layer errors
    Validation(String),         // Input validation failures
    AgentNotFound(String),      // Agent discovery errors
    McpToolExecution(String),   // MCP tool failures
    
    // System layer errors
    Internal(String),           // Unexpected system errors
    External(String),           // Third-party service errors
    Config(String),             // Configuration errors
}
```

## Envelope Error Propagation

### Error Policy Configuration
```rust
Envelope<T> {
    meta: Meta {
        error_policy: Some(json!({
            "mode": "fail_fast",        // "fail_fast" | "allow_partial" | "best_effort"
            "retry_count": 3,
            "timeout_ms": 30000,
            "rollback_on_failure": true,
            "propagate_to_user": false
        })),
        // ... other meta fields
    },
    data: T,
    error: Option<QollectiveError>,
}
```

## Rust Code Examples

### WASM Error Translation
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ErrorTranslator;

#[wasm_bindgen]
impl ErrorTranslator {
    pub fn translate_for_user(error: &QollectiveError) -> String {
        match error {
            QollectiveError::Transport(_) => "Connection problem. Please try again.".to_string(),
            QollectiveError::Security(_) => "Access denied. Please check your permissions.".to_string(),
            QollectiveError::Validation(msg) => format!("Invalid input: {}", msg),
            QollectiveError::AgentNotFound(_) => "Service temporarily unavailable.".to_string(),
            QollectiveError::McpToolExecution(_) => "Unable to process request. Please try again.".to_string(),
            QollectiveError::NatsTimeout(_) => "Request timed out. Please try again.".to_string(),
            QollectiveError::TenantExtraction(_) => "Authentication error. Please log in again.".to_string(),
            _ => "An unexpected error occurred. Please contact support.".to_string(),
        }
    }
    
    pub fn should_retry(error: &QollectiveError) -> bool {
        matches!(error, 
            QollectiveError::Transport(_) | 
            QollectiveError::Connection(_) |
            QollectiveError::NatsTimeout(_) |
            QollectiveError::External(_)
        )
    }
    
    pub fn is_user_error(error: &QollectiveError) -> bool {
        matches!(error,
            QollectiveError::Validation(_) |
            QollectiveError::Security(_)
        )
    }
}
```

### Error Handling in WASM Client
```rust
#[wasm_bindgen]
impl RestClient {
    pub async fn handle_request_with_retry<T>(&self, request: T) -> Result<JsValue, JsValue> 
    where 
        T: Serialize + Clone,
    {
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 3;
        
        loop {
            match self.send_request(&request).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    let qollective_error: QollectiveError = 
                        serde_wasm_bindgen::from_value(error)?;
                    
                    if ErrorTranslator::should_retry(&qollective_error) && retry_count < MAX_RETRIES {
                        retry_count += 1;
                        let delay = 2_u32.pow(retry_count) * 1000; // Exponential backoff
                        
                        gloo_timers::future::TimeoutFuture::new(delay).await;
                        continue;
                    } else {
                        let user_message = ErrorTranslator::translate_for_user(&qollective_error);
                        return Err(JsValue::from_str(&user_message));
                    }
                }
            }
        }
    }
}
```

### Agent Error Handling
```rust
impl Agent {
    pub async fn handle_request_with_error_policy(&self, 
        envelope: Envelope<JsonRpcRequest>
    ) -> Envelope<JsonRpcResponse> {
        let error_policy = envelope.meta.error_policy
            .as_ref()
            .and_then(|p| p.as_object())
            .cloned()
            .unwrap_or_default();
        
        let mode = error_policy.get("mode")
            .and_then(|m| m.as_str())
            .unwrap_or("fail_fast");
        
        match self.process_request(&envelope).await {
            Ok(response) => response,
            Err(error) => match mode {
                "fail_fast" => self.create_error_response(&envelope, error),
                "allow_partial" => self.handle_partial_failure(&envelope, error).await,
                "best_effort" => self.handle_best_effort(&envelope, error).await,
                _ => self.create_error_response(&envelope, error),
            }
        }
    }
    
    fn create_error_response(&self, 
        request: &Envelope<JsonRpcRequest>, 
        error: QollectiveError
    ) -> Envelope<JsonRpcResponse> {
        Envelope {
            meta: request.meta.clone().with_response_fields(),
            data: JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(json!({
                    "code": self.error_to_jsonrpc_code(&error),
                    "message": error.to_string()
                })),
                id: request.data.id,
            },
            error: Some(error),
        }
    }
    
    async fn handle_partial_failure(&self,
        request: &Envelope<JsonRpcRequest>,
        error: QollectiveError
    ) -> Envelope<JsonRpcResponse> {
        // Attempt to provide partial results despite error
        let partial_results = self.get_partial_results(&request.data).await;
        
        Envelope {
            meta: request.meta.clone().with_response_fields(),
            data: JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({
                    "partial": true,
                    "data": partial_results,
                    "warning": error.to_string()
                })),
                error: None,
                id: request.data.id,
            },
            error: Some(error), // Still include error in envelope
        }
    }
}
```

### MCP Adapter Error Handling
```rust
impl EnvelopeAdapter {
    pub async fn call_mcp_tool(&self, 
        service_name: &str, 
        envelope: Envelope<McpToolCall>
    ) -> Result<McpResult, QollectiveError> {
        let error_policy = self.extract_error_policy(&envelope.meta);
        
        match self.execute_mcp_call(service_name, &envelope).await {
            Ok(result) => Ok(result),
            Err(mcp_error) => {
                if error_policy.errors_allowed {
                    // Log error but don't fail the overall operation
                    self.log_mcp_error(&envelope.meta, &mcp_error);
                    Ok(McpResult::partial_failure(mcp_error.to_string()))
                } else {
                    // Convert MCP error to QollectiveError and fail
                    match mcp_error.error_type() {
                        McpErrorType::ToolNotFound => 
                            Err(QollectiveError::mcp_tool_execution("Tool not found")),
                        McpErrorType::InvalidParams => 
                            Err(QollectiveError::validation("Invalid parameters")),
                        McpErrorType::InternalError => 
                            Err(QollectiveError::mcp_tool_execution("Internal MCP error")),
                        McpErrorType::Timeout => 
                            Err(QollectiveError::nats_timeout("MCP tool timeout")),
                    }
                }
            }
        }
    }
    
    fn extract_error_policy(&self, meta: &Meta) -> ErrorPolicy {
        meta.extensions
            .as_ref()
            .and_then(|ext| ext.sections.get("error_policy"))
            .and_then(|policy| serde_json::from_value(policy.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
struct ErrorPolicy {
    #[serde(default)]
    errors_allowed: bool,
    #[serde(default = "default_retry_count")]
    retry_count: u32,
    #[serde(default = "default_timeout")]
    timeout_ms: u64,
}

impl Default for ErrorPolicy {
    fn default() -> Self {
        Self {
            errors_allowed: false,
            retry_count: 3,
            timeout_ms: 30000,
        }
    }
}
```

## Error Scenarios

### Network Connectivity Issues
```rust
// Transport error with retry logic
Envelope<()> {
    meta: Meta {
        retry_context: Some(json!({
            "attempt": 3,
            "max_attempts": 3,
            "last_error": "Connection timeout"
        })),
        ..Default::default()
    },
    data: (),
    error: Some(QollectiveError::Transport("Failed after 3 retries")),
}
```

### Authentication Failures
```rust
// Security error requiring re-authentication
Envelope<()> {
    meta: Meta {
        security_context: Some(json!({
            "auth_required": true,
            "token_expired": true,
            "redirect_to": "/login"
        })),
        ..Default::default()
    },
    data: (),
    error: Some(QollectiveError::Security("Token expired")),
}
```

### Multi-Tenant Data Access Violation
```rust
// Tenant isolation violation
Envelope<()> {
    meta: Meta {
        tenant: Some("tenant-A".to_string()),
        security_violation: Some(json!({
            "attempted_tenant": "tenant-B",
            "access_denied": true
        })),
        ..Default::default()
    },
    data: (),
    error: Some(QollectiveError::TenantExtraction("Cross-tenant access denied")),
}
```

### MCP Tool Chain Failure
```rust
// Partial workflow failure with rollback
Envelope<WorkflowResponse> {
    meta: Meta {
        workflow_context: Some(json!({
            "completed_steps": ["validate", "reserve"],
            "failed_step": "charge_payment",
            "rollback_steps": ["release_reservation"]
        })),
        ..Default::default()
    },
    data: WorkflowResponse {
        status: "failed_with_rollback".to_string(),
        partial_results: json!({"validation": "passed", "reservation": "released"}),
    },
    error: Some(QollectiveError::McpToolExecution("Payment service unavailable")),
}
```

## Error Recovery Strategies

### Circuit Breaker Pattern
```rust
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: AtomicU64,
    state: AtomicU8, // 0=Closed, 1=Open, 2=HalfOpen
}

impl CircuitBreaker {
    pub fn call_with_breaker<F, T>(&self, f: F) -> Result<T, QollectiveError>
    where
        F: FnOnce() -> Result<T, QollectiveError>,
    {
        match self.state() {
            CircuitState::Open => Err(QollectiveError::external("Circuit breaker open")),
            CircuitState::HalfOpen => {
                match f() {
                    Ok(result) => {
                        self.reset();
                        Ok(result)
                    },
                    Err(error) => {
                        self.record_failure();
                        Err(error)
                    }
                }
            },
            CircuitState::Closed => {
                match f() {
                    Ok(result) => Ok(result),
                    Err(error) => {
                        self.record_failure();
                        Err(error)
                    }
                }
            }
        }
    }
}
```

### Graceful Degradation
```rust
impl Agent {
    async fn handle_service_degradation(&self, 
        request: &Envelope<JsonRpcRequest>
    ) -> Result<JsonValue, QollectiveError> {
        // Try primary service first
        match self.primary_service.call(&request.data).await {
            Ok(result) => Ok(result),
            Err(primary_error) => {
                // Fall back to cached data or alternative service
                if let Some(cached_result) = self.get_cached_result(&request.data).await {
                    Ok(json!({
                        "data": cached_result,
                        "source": "cache",
                        "warning": "Primary service unavailable"
                    }))
                } else if let Ok(fallback_result) = self.fallback_service.call(&request.data).await {
                    Ok(json!({
                        "data": fallback_result,
                        "source": "fallback",
                        "warning": "Using alternative service"
                    }))
                } else {
                    Err(primary_error)
                }
            }
        }
    }
}
```

## User Experience Considerations

### Progressive Error Disclosure
- **Immediate**: Show user-friendly message
- **Expandable**: Technical details available on request
- **Actionable**: Clear next steps when possible
- **Contextual**: Error severity affects UI behavior

### Error State Management
- **Retry Buttons**: For transient errors
- **Fallback Content**: When partial data available
- **Offline Indicators**: For connectivity issues
- **Support Links**: For persistent problems
