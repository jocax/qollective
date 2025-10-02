# WASM Envelope A2A MCP Scenario

## Scenario Description

Agent-to-Agent communication with MCP (Model Context Protocol) integration for AI agent workflows. This scenario uses asynchronous messaging over NATS with WebSocket transport, enabling real-time bidirectional communication between browser, agents, and MCP servers.

## Flow Pattern

```
User → Browser/UI → WASM Client → NATS Gateway → A2A Registry → Agent → MCP Client → MCP Server
```

## Transport Protocol Analysis

| Transport | A2A Support | MCP Support | Real-time | Bidirectional | Content Types | Binary Support | Complexity | Enterprise Ready |
|-----------|-------------|-------------|-----------|---------------|---------------|----------------|------------|------------------|
| **NATS over WebSocket** | ✅ **Primary** | ⚠️ Via Adapter | ✅ Yes | ✅ Full Duplex | 📄 JSON, 📁 Binary | ✅ Native | 🟡 Medium | ✅ Yes |
| **WebSocket Direct** | ⚠️ Custom | ❌ No | ✅ Yes | ✅ Full Duplex | 📄 JSON, 📁 Binary | ✅ Native | 🟡 Medium | ✅ Yes |
| **STDIO (MCP)** | ❌ No | ✅ **Standard** | ❌ Local Only | ✅ Yes | 📄 JSON-Lines | ❌ No | 🟢 Low | ⚠️ Local Only |

## Envelope Structure

### Agent Request Envelope
```rust
Envelope<JsonRpcRequest> {
    meta: Meta {
        timestamp: Some(DateTime::now()),
        request_id: Some(Uuid::new()),
        tenant: Some("abc123"),
        session_id: Some("sess-456"),
        agent_id: Some("chat-agent"),
        tracing: Some(TracingMeta {
            trace_id: Some("trace-789"),
            span_id: Some("span-012"),
            ..Default::default()
        }),
    },
    data: JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "getUserData".to_string(),
        params: json!({"user_id": "12345"}),
        id: 1,
    },
    error: None,
}
```

### Agent Response Envelope
```rust
Envelope<JsonRpcResponse> {
    meta: Meta {
        request_id: Some(original_request_id),
        tenant: Some("abc123"),
        session_id: Some("sess-456"),
        duration: Some(156.7),
        performance: Some(PerformanceMeta {
            db_query_time: Some(45.2),
            external_calls: vec![
                ExternalCall {
                    service: "database".to_string(),
                    duration: 45.2,
                    status: CallStatus::Success,
                    endpoint: Some("user_query".to_string()),
                }
            ],
            ..Default::default()
        }),
    },
    data: JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({"user": {"id": 12345, "name": "John"}})),
        error: None,
        id: 1,
    },
    error: None,
}
```

## Security Requirements

### Transport Security
- **TLS 1.3**: All WebSocket connections encrypted
- **mTLS Authentication**: Client certificates for NATS authentication
- **Message Signing**: Optional message-level signatures for critical operations

### Application Security
- **Session Management**: Persistent session context across messages
- **Agent Authorization**: Agent-specific permissions and capabilities
- **Tenant Isolation**: Strict tenant boundaries in all operations

## Rust Code Examples

### WASM NATS Client
```rust
use async_nats::Client;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub struct AgentClient {
    client: Option<Client>,
    session_id: String,
}

#[wasm_bindgen]
impl AgentClient {
    pub async fn connect(&mut self, nats_url: String) -> Result<(), JsValue> {
        let client = async_nats::connect(&nats_url)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        self.client = Some(client);
        Ok(())
    }
    
    pub async fn send_message(&self, agent_id: String, message: String) -> Result<JsValue, JsValue> {
        let client = self.client.as_ref()
            .ok_or_else(|| JsValue::from_str("Not connected"))?;
        
        let envelope = Envelope {
            meta: Meta {
                session_id: Some(self.session_id.clone()),
                agent_id: Some(agent_id.clone()),
                ..Meta::with_auto_fields()
            },
            data: JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "chat".to_string(),
                params: json!({"message": message}),
                id: 1,
            },
            error: None,
        };
        
        let response = client
            .request(format!("agent.{}", agent_id), serde_json::to_vec(&envelope)?)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let result: Envelope<JsonRpcResponse> = serde_json::from_slice(&response.payload)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(serde_wasm_bindgen::to_value(&result.data)?)
    }
}
```

### Context Extraction in Agent
```rust
use qollective_envelope::{Envelope, Context};

pub struct Agent {
    mcp_client: McpClient,
}

impl Agent {
    pub async fn handle_request(&self, envelope: Envelope<JsonRpcRequest>) -> Result<Envelope<JsonRpcResponse>, QollectiveError> {
        // Extract context from envelope
        let context = Context::from(envelope.meta.clone());
        
        // Process with context
        let result = context.run_with(async {
            match envelope.data.method.as_str() {
                "getUserData" => self.get_user_data(&envelope.data.params).await,
                "processOrder" => self.process_order(&envelope.data.params).await,
                _ => Err(QollectiveError::validation("Unknown method")),
            }
        }).await?;
        
        Ok(Envelope {
            meta: envelope.meta.with_response_fields(),
            data: JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: envelope.data.id,
            },
            error: None,
        })
    }
    
    async fn get_user_data(&self, params: &Value) -> Result<Value, QollectiveError> {
        let context = Context::current()
            .ok_or_else(|| QollectiveError::internal("No context available"))?;
        
        // Use tenant from context for database query
        let tenant_id = context.meta().tenant.as_ref()
            .ok_or_else(|| QollectiveError::tenant_extraction("No tenant in context"))?;
        
        // Call MCP server with tenant context
        self.mcp_client.call_tool("database_query", json!({
            "tenant": tenant_id,
            "query": "SELECT * FROM users WHERE id = ?",
            "params": [params["user_id"]]
        })).await
    }
}
```

### MCP Integration
```rust
pub struct McpClient {
    adapter: EnvelopeAdapter,
}

impl McpClient {
    pub async fn call_tool(&self, tool_name: &str, params: Value) -> Result<Value, QollectiveError> {
        let context = Context::current()
            .ok_or_else(|| QollectiveError::internal("No context"))?;
        
        // Create envelope for MCP call
        let envelope = Envelope {
            meta: context.meta().clone(),
            data: McpToolCall {
                tool: tool_name.to_string(),
                parameters: params,
            },
            error: None,
        };
        
        // Send through adapter
        self.adapter.call_tool(envelope).await
    }
}
```

## Use Cases

### Chat with Agent
1. User types message in browser
2. WASM creates `Envelope<JsonRpcRequest>` with session context
3. NATS routes to appropriate agent based on meta.agent_id
4. Agent processes message and may call MCP tools
5. Response flows back through envelope chain

### Database Query via Agent
1. UI requests user data
2. Agent receives envelope with tenant context
3. Agent calls MCP database tool with tenant filtering
4. Database returns tenant-scoped results
5. Agent wraps results in response envelope

### Multi-step Workflow
1. User initiates complex workflow
2. Agent coordinates multiple MCP tools
3. Each tool call maintains context from original envelope
4. Results aggregated and returned to user

## Error Scenarios

### Agent Not Found
```rust
// Registry returns envelope with error when agent doesn't exist
Envelope<()> {
    meta: Meta { ... },
    data: (),
    error: Some(QollectiveError::AgentNotFound("Agent 'unknown-agent' not found")),
}
```

### MCP Tool Failure
```rust
// MCP tool error propagated through envelope
Envelope<JsonRpcResponse> {
    meta: Meta { ... },
    data: JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(json!({"code": -1, "message": "Tool execution failed"})),
        id: 1,
    },
    error: Some(QollectiveError::McpToolExecution("Database connection failed")),
}
```

## Performance Considerations

### NATS Optimization
- **Subject-based Routing**: Use agent_id in subject for efficient routing
- **Connection Pooling**: Reuse NATS connections across requests
- **Message Batching**: Group related operations when possible

### Context Propagation
- **Thread-local Storage**: Efficient context access within agent
- **Async Context**: Maintain context across async boundaries
- **Metadata Optimization**: Only include necessary meta fields
