# WASM Envelope MCP Adapter

## Overview

The MCP Adapter bridges the envelope-aware Qollective framework with standard MCP (Model Context Protocol) servers. It handles envelope-to-MCP protocol translation, context propagation, error policy enforcement, and maintains compatibility with existing MCP implementations.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   A2A Agent     │────│  MCP Client     │────│ Envelope Adapter│
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
                                              ┌─────────┼─────────┐
                                              │         │         │
                                      ┌───────▼───┐ ┌───▼───┐ ┌───▼───┐
                                      │MCP Server │ │MCP    │ │MCP    │
                                      │(Envelope  │ │Server │ │Server │
                                      │ Aware)    │ │(Std)  │ │(Std)  │
                                      └───────────┘ └───────┘ └───────┘
```

## Core Responsibilities

### 1. Protocol Translation
- **Envelope → JSON-RPC**: Extract data payload and convert to standard MCP calls
- **JSON-RPC → Envelope**: Wrap MCP responses in envelope structure
- **Context Injection**: Inject tenant and security context into MCP parameters
- **Error Mapping**: Convert MCP errors to QollectiveError variants

### 2. Context Propagation
- **Tenant Context**: Ensure MCP tools operate within correct tenant scope
- **Security Context**: Pass authentication and authorization data
- **Tracing Context**: Maintain request correlation across MCP boundaries
- **Session Context**: Preserve session state for stateful MCP operations

### 3. Error Policy Enforcement
- **Fail Fast**: Stop on first MCP error (default behavior)
- **Allow Partial**: Continue processing despite individual tool failures
- **Best Effort**: Provide degraded functionality when tools unavailable

## Implementation

### Core Adapter Structure
```rust
use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct EnvelopeAdapter {
    mcp_servers: HashMap<String, McpServerConnection>,
    error_policies: ErrorPolicyConfig,
    context_injectors: HashMap<String, Box<dyn ContextInjector>>,
}

impl EnvelopeAdapter {
    pub fn new() -> Self {
        Self {
            mcp_servers: HashMap::new(),
            error_policies: ErrorPolicyConfig::default(),
            context_injectors: HashMap::new(),
        }
    }
    
    pub async fn register_mcp_server(&mut self, 
        service_name: String, 
        connection: McpServerConnection
    ) -> Result<(), QollectiveError> {
        // Validate MCP server capabilities
        let capabilities = connection.get_capabilities().await?;
        
        // Register server
        self.mcp_servers.insert(service_name.clone(), connection);
        
        // Register default context injector
        self.register_context_injector(
            service_name,
            Box::new(DefaultContextInjector::new())
        );
        
        Ok(enhanced_params)
    }
    
    fn supports_tool(&self, tool_name: &str) -> bool {
        // Default injector supports all tools
        true
    }
}
```

### Database Context Injector
```rust
pub struct DatabaseContextInjector {
    tenant_column: String,
    security_filters: HashMap<String, Vec<String>>,
}

#[async_trait]
impl ContextInjector for DatabaseContextInjector {
    async fn inject_context(&self, 
        params: &Value, 
        meta: &Meta
    ) -> Result<Value, QollectiveError> {
        let mut enhanced_params = params.clone();
        
        // Inject tenant filtering for database queries
        if let Some(tenant) = &meta.tenant {
            // Add WHERE clause for tenant isolation
            if let Some(query) = enhanced_params.get("query").and_then(|q| q.as_str()) {
                let tenant_filter = format!(" AND {} = '{}'", self.tenant_column, tenant);
                enhanced_params["query"] = Value::String(
                    if query.to_uppercase().contains("WHERE") {
                        format!("{}{}", query, tenant_filter)
                    } else {
                        format!("{} WHERE {}", query, &tenant_filter[5..]) // Remove " AND "
                    }
                );
            }
            
            // Add tenant parameter for prepared statements
            enhanced_params["tenant_id"] = Value::String(tenant.clone());
        }
        
        // Add security-based row filtering
        if let Some(security) = &meta.security {
            if let Some(user_id) = &security.user_id {
                enhanced_params["_security_context"] = json!({
                    "user_id": user_id,
                    "roles": security.roles,
                    "permissions": security.permissions
                });
            }
        }
        
        Ok(enhanced_params)
    }
    
    fn supports_tool(&self, tool_name: &str) -> bool {
        matches!(tool_name, "database_query" | "database_execute" | "database_transaction")
    }
}
```

### Error Policy Configuration
```rust
#[derive(Debug, Clone)]
pub struct ErrorPolicy {
    pub mode: ErrorMode,
    pub retry_count: u32,
    pub timeout_ms: u64,
    pub fallback_enabled: bool,
    pub cache_on_error: bool,
}

#[derive(Debug, Clone)]
pub enum ErrorMode {
    FailFast,      // Stop immediately on error
    AllowPartial,  // Continue with partial results
    BestEffort,    // Try fallbacks and degraded service
}

impl EnvelopeAdapter {
    fn extract_error_policy(&self, meta: &Meta) -> ErrorPolicy {
        meta.extensions
            .as_ref()
            .and_then(|ext| ext.sections.get("error_policy"))
            .and_then(|policy| serde_json::from_value(policy.clone()).ok())
            .unwrap_or_else(|| self.error_policies.default_policy.clone())
    }
    
    fn convert_mcp_error(&self, mcp_error: McpError) -> QollectiveError {
        match mcp_error.code {
            -32700 => QollectiveError::serialization("Parse error"),
            -32600 => QollectiveError::validation("Invalid request"),
            -32601 => QollectiveError::mcp_tool_execution("Method not found"),
            -32602 => QollectiveError::validation("Invalid params"),
            -32603 => QollectiveError::mcp_tool_execution("Internal error"),
            -32000..=-32099 => QollectiveError::mcp_protocol(mcp_error.message),
            _ => QollectiveError::external(format!("MCP error: {}", mcp_error.message)),
        }
    }
}
```

### MCP Server Connection Management
```rust
pub enum McpServerConnection {
    Stdio(StdioConnection),
    WebSocket(WebSocketConnection),
    Http(HttpConnection),
}

impl McpServerConnection {
    pub async fn call(&self, request: Value) -> Result<Value, McpError> {
        match self {
            Self::Stdio(conn) => conn.call(request).await,
            Self::WebSocket(conn) => conn.call(request).await,
            Self::Http(conn) => conn.call(request).await,
        }
    }
    
    pub async fn get_capabilities(&self) -> Result<McpCapabilities, McpError> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "qollective-mcp-adapter",
                    "version": "1.0.0"
                }
            }
        });
        
        let response = self.call(request).await?;
        Ok(serde_json::from_value(response["result"]["capabilities"].clone())?)
    }
}
```

### Envelope-Aware MCP Server
```rust
/// For MCP servers that are envelope-aware and can handle context directly
pub struct EnvelopeAwareMcpServer {
    server_id: String,
    tools: HashMap<String, Box<dyn EnvelopeAwareTool>>,
}

#[async_trait]
pub trait EnvelopeAwareTool: Send + Sync {
    async fn execute(&self, 
        envelope: Envelope<McpToolCall>
    ) -> Result<Envelope<McpToolResponse>, QollectiveError>;
    
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> Value;
}

impl EnvelopeAwareMcpServer {
    pub async fn handle_envelope_call(&self, 
        envelope: Envelope<McpToolCall>
    ) -> Result<Envelope<McpToolResponse>, QollectiveError> {
        let tool_name = &envelope.data.tool;
        
        let tool = self.tools.get(tool_name)
            .ok_or_else(|| QollectiveError::mcp_tool_execution(
                format!("Tool '{}' not found", tool_name)
            ))?;
        
        // Extract context and execute
        let context = Context::from(envelope.meta.clone());
        context.run_with(async {
            tool.execute(envelope).await
        }).await
    }
}

/// Example envelope-aware database tool
pub struct DatabaseTool {
    pool: sqlx::PgPool,
}

#[async_trait]
impl EnvelopeAwareTool for DatabaseTool {
    async fn execute(&self, 
        envelope: Envelope<McpToolCall>
    ) -> Result<Envelope<McpToolResponse>, QollectiveError> {
        let context = Context::current()
            .ok_or_else(|| QollectiveError::internal("No context available"))?;
        
        // Extract tenant from context
        let tenant_id = context.meta().tenant.as_ref()
            .ok_or_else(|| QollectiveError::tenant_extraction("No tenant in context"))?;
        
        // Extract query parameters
        let query = envelope.data.parameters["query"].as_str()
            .ok_or_else(|| QollectiveError::validation("Missing query parameter"))?;
        
        // Execute tenant-scoped query
        let tenant_scoped_query = format!(
            "{} AND tenant_id = $1", 
            query
        );
        
        let rows = sqlx::query(&tenant_scoped_query)
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| QollectiveError::external(e.to_string()))?;
        
        let result = rows.into_iter()
            .map(|row| {
                // Convert row to JSON
                let mut json_row = json!({});
                // ... row conversion logic
                json_row
            })
            .collect::<Vec<_>>();
        
        Ok(Envelope {
            meta: envelope.meta.with_response_fields(),
            data: McpToolResponse {
                tool: envelope.data.tool.clone(),
                result: json!(result),
                metadata: Some(json!({
                    "tenant_id": tenant_id,
                    "row_count": result.len()
                })),
            },
            error: None,
        })
    }
    
    fn name(&self) -> &str {
        "database_query"
    }
    
    fn description(&self) -> &str {
        "Execute tenant-scoped database queries"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "SQL query to execute (tenant filtering applied automatically)"
                }
            },
            "required": ["query"]
        })
    }
}
```

### Adapter Factory and Configuration
```rust
pub struct AdapterFactory;

impl AdapterFactory {
    pub fn create_database_adapter() -> EnvelopeAdapter {
        let mut adapter = EnvelopeAdapter::new();
        
        // Register database context injector
        adapter.register_context_injector(
            "database_service".to_string(),
            Box::new(DatabaseContextInjector {
                tenant_column: "tenant_id".to_string(),
                security_filters: HashMap::new(),
            })
        );
        
        adapter
    }
    
    pub fn create_email_adapter() -> EnvelopeAdapter {
        let mut adapter = EnvelopeAdapter::new();
        
        adapter.register_context_injector(
            "email_service".to_string(),
            Box::new(EmailContextInjector::new())
        );
        
        adapter
    }
    
    pub async fn create_multi_service_adapter() -> Result<EnvelopeAdapter, QollectiveError> {
        let mut adapter = EnvelopeAdapter::new();
        
        // Register multiple MCP servers
        adapter.register_mcp_server(
            "database".to_string(),
            McpServerConnection::Stdio(StdioConnection::new("./database-mcp-server"))
        ).await?;
        
        adapter.register_mcp_server(
            "email".to_string(),
            McpServerConnection::Http(HttpConnection::new("http://email-service:8080/mcp"))
        ).await?;
        
        adapter.register_mcp_server(
            "analytics".to_string(),
            McpServerConnection::WebSocket(WebSocketConnection::new("ws://analytics:9090/mcp"))
        ).await?;
        
        Ok(adapter)
    }
}
```

## Usage Examples

### Agent Integration
```rust
impl Agent {
    async fn handle_database_request(&self, 
        envelope: Envelope<JsonRpcRequest>
    ) -> Result<Envelope<JsonRpcResponse>, QollectiveError> {
        // Create MCP tool call from JSON-RPC request
        let tool_call = McpToolCall {
            tool: "database_query".to_string(),
            parameters: envelope.data.params.clone(),
        };
        
        let mcp_envelope = Envelope {
            meta: envelope.meta.clone(),
            data: tool_call,
            error: None,
        };
        
        // Call through adapter
        let result = self.mcp_adapter.call_mcp_tool("database", mcp_envelope).await?;
        
        // Convert back to JSON-RPC response
        Ok(Envelope {
            meta: result.meta,
            data: JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result.data.result),
                error: None,
                id: envelope.data.id,
            },
            error: result.error,
        })
    }
}
```

### Error Handling Integration
```rust
async fn handle_mcp_workflow(adapter: &EnvelopeAdapter) -> Result<Value, QollectiveError> {
    let envelope = Envelope {
        meta: Meta {
            tenant: Some("tenant-123".to_string()),
            error_policy: Some(json!({
                "mode": "allow_partial",
                "fallback_enabled": true
            })),
            ..Meta::with_auto_fields()
        },
        data: McpToolCall {
            tool: "complex_workflow".to_string(),
            parameters: json!({"data": "some_input"}),
        },
        error: None,
    };
    
    match adapter.call_mcp_tool("workflow_service", envelope).await {
        Ok(response) => {
            if let Some(error) = response.error {
                // Partial success case
                log::warn!("Partial workflow success: {}", error);
                Ok(json!({
                    "status": "partial",
                    "data": response.data.result,
                    "warning": error.to_string()
                }))
            } else {
                Ok(response.data.result)
            }
        },
        Err(error) => {
            // Complete failure
            Err(error)
        }
    }
}
```

## Benefits

### For Existing MCP Servers
- **Zero Code Changes**: Standard MCP servers work unchanged
- **Automatic Context**: Tenant and security context injected transparently
- **Enhanced Observability**: Tracing and monitoring added automatically

### For Envelope-Aware Servers
- **Rich Context Access**: Full envelope metadata available
- **Type Safety**: Compile-time guarantees for envelope handling
- **Consistent Error Handling**: Unified error semantics

### For System Integration
- **Protocol Bridging**: Seamless integration between envelope and MCP worlds
- **Security Enforcement**: Automatic tenant isolation and access control
- **Operational Excellence**: Built-in monitoring, tracing, and error handling(())
  }

  pub fn register_context_injector(&mut self,
  service_name: String,
  injector: Box<dyn ContextInjector>
  ) {
  self.context_injectors.insert(service_name, injector);
  }
  }
```

### Main Translation Interface
```rust
impl EnvelopeAdapter {
    pub async fn call_mcp_tool(&self, 
        service_name: &str, 
        envelope: Envelope<McpToolCall>
    ) -> Result<Envelope<McpToolResponse>, QollectiveError> {
        // Extract context from envelope
        let context = Context::from(envelope.meta.clone());
        
        // Get MCP server connection
        let mcp_server = self.mcp_servers.get(service_name)
            .ok_or_else(|| QollectiveError::mcp_server_not_found(service_name))?;
        
        // Get context injector
        let context_injector = self.context_injectors.get(service_name)
            .ok_or_else(|| QollectiveError::protocol_adapter("No context injector found"))?;
        
        // Extract error policy
        let error_policy = self.extract_error_policy(&envelope.meta);
        
        // Execute within context
        context.run_with(async {
            self.execute_mcp_call_with_policy(
                mcp_server,
                context_injector.as_ref(),
                &envelope,
                &error_policy
            ).await
        }).await
    }
    
    async fn execute_mcp_call_with_policy(&self,
        mcp_server: &McpServerConnection,
        context_injector: &dyn ContextInjector,
        envelope: &Envelope<McpToolCall>,
        error_policy: &ErrorPolicy
    ) -> Result<Envelope<McpToolResponse>, QollectiveError> {
        // Inject context into MCP parameters
        let enhanced_params = context_injector.inject_context(
            &envelope.data.parameters,
            &envelope.meta
        )?;
        
        // Create standard MCP JSON-RPC call
        let mcp_request = json!({
            "jsonrpc": "2.0",
            "id": envelope.meta.request_id,
            "method": "tools/call",
            "params": {
                "name": envelope.data.tool,
                "arguments": enhanced_params
            }
        });
        
        // Execute MCP call
        match mcp_server.call(mcp_request).await {
            Ok(mcp_response) => {
                self.wrap_mcp_success(envelope, mcp_response).await
            },
            Err(mcp_error) => {
                self.handle_mcp_error(envelope, mcp_error, error_policy).await
            }
        }
    }
    
    async fn wrap_mcp_success(&self,
        original_envelope: &Envelope<McpToolCall>,
        mcp_response: Value
    ) -> Result<Envelope<McpToolResponse>, QollectiveError> {
        let response_data = McpToolResponse {
            tool: original_envelope.data.tool.clone(),
            result: mcp_response["result"].clone(),
            metadata: Some(json!({
                "execution_time": mcp_response.get("execution_time"),
                "server_version": mcp_response.get("server_version")
            })),
        };
        
        Ok(Envelope {
            meta: original_envelope.meta.clone().with_response_fields(),
            data: response_data,
            error: None,
        })
    }
    
    async fn handle_mcp_error(&self,
        original_envelope: &Envelope<McpToolCall>,
        mcp_error: McpError,
        error_policy: &ErrorPolicy
    ) -> Result<Envelope<McpToolResponse>, QollectiveError> {
        // Convert MCP error to QollectiveError
        let qollective_error = self.convert_mcp_error(mcp_error);
        
        match error_policy.mode {
            ErrorMode::FailFast => Err(qollective_error),
            ErrorMode::AllowPartial => {
                // Return envelope with error but allow processing to continue
                Ok(Envelope {
                    meta: original_envelope.meta.clone().with_response_fields(),
                    data: McpToolResponse {
                        tool: original_envelope.data.tool.clone(),
                        result: Value::Null,
                        metadata: Some(json!({
                            "error_handled": true,
                            "partial_failure": true
                        })),
                    },
                    error: Some(qollective_error),
                })
            },
            ErrorMode::BestEffort => {
                // Try fallback or cached response
                self.attempt_fallback_response(original_envelope, qollective_error).await
            }
        }
    }
}
```

### Context Injection Interface
```rust
#[async_trait]
pub trait ContextInjector: Send + Sync {
    async fn inject_context(&self, 
        params: &Value, 
        meta: &Meta
    ) -> Result<Value, QollectiveError>;
    
    fn supports_tool(&self, tool_name: &str) -> bool;
}

pub struct DefaultContextInjector;

impl DefaultContextInjector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ContextInjector for DefaultContextInjector {
    async fn inject_context(&self, 
        params: &Value, 
        meta: &Meta
    ) -> Result<Value, QollectiveError> {
        let mut enhanced_params = params.clone();
        
        // Inject tenant context
        if let Some(tenant) = &meta.tenant {
            enhanced_params["_context"] = json!({
                "tenant": tenant,
                "request_id": meta.request_id,
                "timestamp": meta.timestamp
            });
        }
        
        // Inject security context
        if let Some(security) = &meta.security {
            enhanced_params["_security"] = json!({
                "user_id": security.user_id,
                "session_id": security.session_id,
                "permissions": security.permissions
            });
        }
        
        // Inject tracing context
        if let Some(tracing) = &meta.tracing {
            enhanced_params["_tracing"] = json!({
                "trace_id": tracing.trace_id,
                "span_id": tracing.span_id,
                "parent_span_id": tracing.parent_span_id
            });
        }
        
        Ok
