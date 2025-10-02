# WASM Envelope Multi-MCP Coordination Scenario

## Scenario Description

Complex workflows requiring coordination across multiple MCP servers within a single agent operation. This scenario demonstrates how envelope metadata enables request correlation, parallel processing, and consistent error handling across distributed MCP tool calls.

## Flow Pattern

```
User → WASM → NATS → Agent → MCP Client → [MCP Server 1, MCP Server 2, MCP Server N]
                              ↓
                        Coordination & Aggregation
                              ↓
                        Response Envelope
```

## Transport Protocol Analysis

| Transport | Coordination | Parallel Ops | Request Correlation | Error Handling | Content Types | Complexity | Enterprise Ready |
|-----------|--------------|--------------|-------------------|----------------|---------------|------------|------------------|
| **NATS + STDIO** | ✅ **Optimal** | ✅ Yes | ✅ Via requestId | ✅ Centralized | 📄 JSON, 📁 Binary | 🟡 Medium | ✅ Yes |
| **Direct STDIO** | ⚠️ Sequential | ❌ No | ⚠️ Manual | ⚠️ Individual | 📄 JSON-Lines | 🟢 Low | ⚠️ Limited |
| **WebSocket + STDIO** | ✅ Yes | ✅ Yes | ✅ Via Headers | ✅ Structured | 📄 JSON, 📁 Binary | 🔴 High | ✅ Yes |

## Envelope Structure

### Coordination Request Envelope
```rust
Envelope<WorkflowRequest> {
    meta: Meta {
        request_id: Some(Uuid::parse_str("coord-12345").unwrap()),
        tenant: Some("abc123"),
        session_id: Some("sess-789"),
        agent_id: Some("workflow-agent"),
        workflow_id: Some("order-process-001"),
        correlation_context: Some(json!({
            "operation": "process_order",
            "steps": ["validate", "charge", "fulfill", "notify"]
        })),
        tracing: Some(TracingMeta {
            trace_id: Some("trace-workflow-001"),
            span_id: Some("span-coord-001"),
            ..Default::default()
        }),
    },
    data: WorkflowRequest {
        operation: "process_order".to_string(),
        parameters: json!({
            "order_id": "ord-123456",
            "customer_id": "cust-789",
            "items": [{"sku": "ITEM001", "quantity": 2}]
        }),
    },
    error: None,
}
```

### Individual MCP Call Envelope
```rust
Envelope<McpToolCall> {
    meta: Meta {
        request_id: Some(Uuid::parse_str("coord-12345").unwrap()), // Same as parent
        parent_request_id: Some(Uuid::parse_str("coord-12345").unwrap()),
        child_request_id: Some(Uuid::now_v7()), // Unique for this tool call
        tenant: Some("abc123"),
        step_context: Some(json!({
            "step_name": "validate_inventory",
            "step_index": 1,
            "total_steps": 4
        })),
        tracing: Some(TracingMeta {
            trace_id: Some("trace-workflow-001"), // Same trace
            span_id: Some("span-validate-001"), // New span
            parent_span_id: Some("span-coord-001"),
            ..Default::default()
        }),
    },
    data: McpToolCall {
        tool: "inventory_check".to_string(),
        parameters: json!({
            "tenant": "abc123",
            "sku": "ITEM001",
            "quantity": 2
        }),
    },
    error: None,
}
```

## Security Requirements

### Transport Security
- **Per-MCP TLS**: Each MCP server connection encrypted
- **Unified mTLS**: Single client certificate for all MCP connections
- **Context Isolation**: Tenant boundaries maintained across all tools

### Application Security
- **Request Correlation**: All related calls share parent request_id
- **Step Authorization**: Each tool call validated against workflow permissions
- **Data Minimization**: Only necessary context passed to each MCP server

## Rust Code Examples

### Workflow Coordinator
```rust
use futures::future::try_join_all;
use std::collections::HashMap;

pub struct WorkflowCoordinator {
    mcp_clients: HashMap<String, McpClient>,
    envelope_adapter: EnvelopeAdapter,
}

impl WorkflowCoordinator {
    pub async fn execute_workflow(&self, envelope: Envelope<WorkflowRequest>) -> Result<Envelope<WorkflowResponse>, QollectiveError> {
        let context = Context::from(envelope.meta.clone());
        
        context.run_with(async {
            match envelope.data.operation.as_str() {
                "process_order" => self.process_order_workflow(&envelope).await,
                "onboard_user" => self.onboard_user_workflow(&envelope).await,
                _ => Err(QollectiveError::validation("Unknown workflow")),
            }
        }).await
    }
    
    async fn process_order_workflow(&self, envelope: &Envelope<WorkflowRequest>) -> Result<Envelope<WorkflowResponse>, QollectiveError> {
        let order_data = &envelope.data.parameters;
        let base_meta = envelope.meta.clone();
        
        // Step 1: Parallel validation calls
        let validation_futures = vec![
            self.validate_inventory(&base_meta, order_data),
            self.validate_customer(&base_meta, order_data),
            self.validate_payment(&base_meta, order_data),
        ];
        
        let validation_results = try_join_all(validation_futures).await?;
        
        // Step 2: Sequential processing if validation passes
        if validation_results.iter().all(|r| r.is_success()) {
            let charge_result = self.charge_payment(&base_meta, order_data).await?;
            let fulfill_result = self.fulfill_order(&base_meta, order_data).await?;
            let notify_result = self.notify_customer(&base_meta, order_data).await?;
            
            Ok(Envelope {
                meta: base_meta.with_response_fields(),
                data: WorkflowResponse {
                    status: "completed".to_string(),
                    results: json!({
                        "charge": charge_result,
                        "fulfillment": fulfill_result,
                        "notification": notify_result
                    }),
                },
                error: None,
            })
        } else {
            Err(QollectiveError::validation("Order validation failed"))
        }
    }
    
    async fn validate_inventory(&self, base_meta: &Meta, order_data: &Value) -> Result<McpResult, QollectiveError> {
        let envelope = Envelope {
            meta: base_meta.with_child_context("validate_inventory", 1, 4),
            data: McpToolCall {
                tool: "inventory_check".to_string(),
                parameters: json!({
                    "tenant": base_meta.tenant,
                    "items": order_data["items"]
                }),
            },
            error: None,
        };
        
        self.envelope_adapter.call_mcp_tool("inventory_service", envelope).await
    }
    
    async fn charge_payment(&self, base_meta: &Meta, order_data: &Value) -> Result<McpResult, QollectiveError> {
        let envelope = Envelope {
            meta: base_meta.with_child_context("charge_payment", 2, 4),
            data: McpToolCall {
                tool: "process_payment".to_string(),
                parameters: json!({
                    "tenant": base_meta.tenant,
                    "customer_id": order_data["customer_id"],
                    "amount": order_data["total_amount"]
                }),
            },
            error: None,
        };
        
        self.envelope_adapter.call_mcp_tool("payment_service", envelope).await
    }
}
```

### Context Propagation Helpers
```rust
impl Meta {
    pub fn with_child_context(&self, step_name: &str, step_index: usize, total_steps: usize) -> Self {
        let mut meta = self.clone();
        meta.child_request_id = Some(Uuid::now_v7());
        meta.parent_request_id = self.request_id;
        meta.step_context = Some(json!({
            "step_name": step_name,
            "step_index": step_index,
            "total_steps": total_steps
        }));
        
        // Update tracing context
        if let Some(ref mut tracing) = meta.tracing {
            tracing.parent_span_id = tracing.span_id.clone();
            tracing.span_id = Some(format!("span-{}-{:03}", step_name, step_index));
        }
        
        meta
    }
    
    pub fn with_response_fields(mut self) -> Self {
        self.timestamp = Some(Utc::now());
        self.duration = Some(
            self.timestamp.unwrap().timestamp_millis() as f64 - 
            self.timestamp.unwrap().timestamp_millis() as f64
        );
        self
    }
}
```

### Error Handling with Coordination
```rust
pub enum WorkflowError {
    ValidationFailed(Vec<String>),
    PartialFailure { completed: Vec<String>, failed: Vec<String> },
    CriticalFailure(QollectiveError),
}

impl WorkflowCoordinator {
    async fn handle_partial_failure(&self, 
        envelope: &Envelope<WorkflowRequest>,
        completed_steps: Vec<String>,
        failed_step: String,
        error: QollectiveError
    ) -> Envelope<WorkflowResponse> {
        // Check error policy from envelope meta
        let allow_partial = envelope.meta.extensions
            .as_ref()
            .and_then(|ext| ext.sections.get("error_policy"))
            .and_then(|policy| policy.as_str())
            .map(|p| p == "allow_partial")
            .unwrap_or(false);
        
        if allow_partial {
            Envelope {
                meta: envelope.meta.clone().with_response_fields(),
                data: WorkflowResponse {
                    status: "partial_success".to_string(),
                    results: json!({
                        "completed": completed_steps,
                        "failed": failed_step
                    }),
                },
                error: Some(QollectiveError::external(format!("Step failed: {}", failed_step))),
            }
        } else {
            // Rollback completed steps
            self.rollback_steps(&envelope.meta, &completed_steps).await;
            
            Envelope {
                meta: envelope.meta.clone().with_response_fields(),
                data: WorkflowResponse {
                    status: "failed".to_string(),
                    results: json!({"error": "Workflow failed and rolled back"}),
                },
                error: Some(error),
            }
        }
    }
}
```

## Use Cases

### Order Processing Workflow
1. **Parallel Validation**: Inventory, customer, payment method
2. **Sequential Execution**: Charge → Fulfill → Notify
3. **Error Handling**: Rollback on failure, partial success option
4. **Audit Trail**: Complete request correlation across all steps

### User Onboarding Workflow
1. **Account Creation**: User service MCP tool
2. **Email Verification**: Email service MCP tool
3. **Welcome Package**: Multiple parallel MCP calls
4. **Analytics Event**: Tracking service MCP tool

### Data Migration Workflow
1. **Extract**: Source system MCP tool
2. **Transform**: Multiple transformation MCP tools in parallel
3. **Validate**: Data quality MCP tools
4. **Load**: Target system MCP tool

## Error Scenarios

### Validation Failure
```rust
// One or more validation steps fail
Envelope<WorkflowResponse> {
    meta: Meta { ... },
    data: WorkflowResponse {
        status: "validation_failed".to_string(),
        results: json!({
            "failed_validations": ["inventory_check", "payment_method"]
        }),
    },
    error: Some(QollectiveError::validation("Validation failed")),
}
```

### Partial Success with Rollback
```rust
// Payment succeeded but fulfillment failed
Envelope<WorkflowResponse> {
    meta: Meta { 
        workflow_id: Some("order-process-001"),
        correlation_context: Some(json!({
            "rollback_actions": ["refund_payment"]
        })),
        ...
    },
    data: WorkflowResponse {
        status: "failed_with_rollback".to_string(),
        results: json!({
            "completed": ["payment_charged"],
            "failed": "fulfillment",
            "rollback": ["payment_refunded"]
        }),
    },
    error: Some(QollectiveError::external("Fulfillment service unavailable")),
}
```

## Performance Considerations

### Parallel Processing
- **Concurrent MCP Calls**: Use `try_join_all` for independent operations
- **Resource Pooling**: Reuse MCP client connections
- **Batch Operations**: Group related tool calls when possible

### Request Correlation
- **Unique IDs**: Parent/child request ID hierarchy
- **Trace Propagation**: OpenTelemetry-compatible tracing
- **Context Size**: Minimize envelope metadata size

### Error Recovery
- **Circuit Breakers**: Fail fast on repeated MCP failures
- **Retry Logic**: Exponential backoff for transient errors
- **Rollback Strategies**: Compensating transactions for partial failures
