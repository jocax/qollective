# Qollective Schema Definition Guide

> **AI Agent Instructions**: This file contains complete schema definition patterns for projects using Qollective's envelope-first architecture. Start with the "Quick Start" section, then reference specific patterns as needed.

## Quick Start - Essential Pattern

**CRITICAL**: All projects using Qollective MUST extend the core envelope schema and define their data types in `$defs`. Never redefine the envelope structure.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://your-domain.com/schemas/your-service.json",
  "allOf": [
    {
      "$ref": "https://schemas.qollective.io/core/envelope.json"
    },
    {
      "properties": {
        "payload": {
          "oneOf": [
            { "$ref": "#/$defs/ApiRequest" },
            { "$ref": "#/$defs/ApiResponse" },
            { "$ref": "#/$defs/ApiError" }
          ]
        }
      }
    }
  ],
  "$defs": {
    "ApiRequest": { /* Your request schema */ },
    "ApiResponse": { /* Your response schema */ },
    "ApiError": { /* Your error schema */ }
  },
  "qollective": {
    "version": "1.0.0",
    "envelope": { "enabled": true }
  }
}
```

## Core Principles

1. **Envelope-First Architecture**: All communication wraps data payload in a standardized envelope with metadata
2. **Schema Extension**: Extend `https://schemas.qollective.io/core/envelope.json`, never redefine it
3. **Payload Field Targeting**: Your schemas define what goes in the `payload` field of the envelope
4. **Multi-Protocol Support**: Same schema works across REST, gRPC, WebSocket, NATS, MCP, A2A protocols
5. **Tenant Context**: Built-in multi-tenant support through envelope metadata

## Complete Working Example

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://your-company.com/schemas/user-service.json",
  "title": "User Service API Schema",
  "description": "Schema for user management service using Qollective envelope pattern",
  "version": "1.0.0",
  
  "allOf": [
    {
      "$ref": "https://schemas.qollective.io/core/envelope.json"
    },
    {
      "properties": {
        "payload": {
          "oneOf": [
            { "$ref": "#/$defs/ApiRequest" },
            { "$ref": "#/$defs/ApiResponse" },
            { "$ref": "#/$defs/ApiError" }
          ]
        }
      }
    }
  ],

  "$defs": {
    "ApiRequest": {
      "type": "object",
      "title": "API Request Payload",
      "description": "Request payload data structure for user service operations",
      "properties": {
        "operation": {
          "type": "string",
          "enum": ["create_user", "get_user", "update_user", "delete_user", "list_users"],
          "description": "The operation to perform"
        },
        "user_id": {
          "type": "string",
          "format": "uuid",
          "description": "User identifier for operations requiring it"
        },
        "user_data": {
          "$ref": "#/$defs/UserData"
        },
        "filters": {
          "$ref": "#/$defs/UserFilters"
        },
        "pagination": {
          "$ref": "#/$defs/PaginationRequest"
        }
      },
      "required": ["operation"],
      "additionalProperties": false
    },

    "ApiResponse": {
      "type": "object",
      "title": "API Response Payload", 
      "description": "Successful response payload data structure",
      "properties": {
        "user": {
          "$ref": "#/$defs/User"
        },
        "users": {
          "type": "array",
          "items": { "$ref": "#/$defs/User" }
        },
        "total_count": {
          "type": "integer",
          "minimum": 0,
          "description": "Total number of users matching filters"
        },
        "affected_records": {
          "type": "integer",
          "minimum": 0,
          "description": "Number of records affected by the operation"
        },
        "operation_status": {
          "type": "string",
          "enum": ["success", "partial_success"],
          "description": "Status of the requested operation"
        }
      },
      "additionalProperties": false
    },

    "ApiError": {
      "type": "object",
      "title": "API Error Payload",
      "description": "Error response following RFC 7807 Problem Details",
      "properties": {
        "error_code": {
          "type": "string",
          "enum": [
            "USER_NOT_FOUND", 
            "VALIDATION_ERROR", 
            "PERMISSION_DENIED", 
            "DUPLICATE_EMAIL",
            "INVALID_OPERATION",
            "INTERNAL_ERROR"
          ],
          "description": "Machine-readable error code"
        },
        "error_message": {
          "type": "string",
          "description": "Human-readable error description"
        },
        "field_errors": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "field": { 
                "type": "string",
                "description": "Name of the field with validation error"
              },
              "message": { 
                "type": "string",
                "description": "Validation error message for this field"
              },
              "code": {
                "type": "string",
                "description": "Machine-readable validation error code"
              }
            },
            "required": ["field", "message"]
          },
          "description": "Field-specific validation errors"
        },
        "retry_after": {
          "type": "integer",
          "minimum": 0,
          "description": "Suggested retry delay in seconds for retryable errors"
        }
      },
      "required": ["error_code", "error_message"],
      "additionalProperties": false
    },

    "User": {
      "type": "object",
      "title": "User Entity",
      "description": "Complete user data structure",
      "properties": {
        "id": { 
          "type": "string", 
          "format": "uuid",
          "description": "Unique user identifier"
        },
        "email": { 
          "type": "string", 
          "format": "email",
          "description": "User email address (unique)"
        },
        "name": { 
          "type": "string",
          "minLength": 1,
          "maxLength": 255,
          "description": "User display name"
        },
        "status": {
          "type": "string",
          "enum": ["active", "inactive", "suspended"],
          "description": "User account status"
        },
        "created_at": { 
          "type": "string", 
          "format": "date-time",
          "description": "Account creation timestamp"
        },
        "updated_at": { 
          "type": "string", 
          "format": "date-time",
          "description": "Last update timestamp"
        },
        "metadata": {
          "type": "object",
          "description": "Additional user metadata",
          "additionalProperties": true
        }
      },
      "required": ["id", "email", "name", "status", "created_at"],
      "additionalProperties": false
    },

    "UserData": {
      "type": "object",
      "title": "User Input Data",
      "description": "User data for creation and updates",
      "properties": {
        "email": { 
          "type": "string", 
          "format": "email",
          "description": "User email address"
        },
        "name": { 
          "type": "string", 
          "minLength": 1,
          "maxLength": 255,
          "description": "User display name"
        },
        "password": { 
          "type": "string", 
          "minLength": 8,
          "maxLength": 128,
          "description": "User password (for creation/updates)"
        },
        "status": {
          "type": "string",
          "enum": ["active", "inactive"],
          "description": "Initial or updated user status"
        },
        "metadata": {
          "type": "object",
          "description": "Additional user metadata",
          "additionalProperties": true
        }
      },
      "additionalProperties": false
    },

    "UserFilters": {
      "type": "object",
      "title": "User Query Filters",
      "description": "Filtering options for user queries",
      "properties": {
        "email": {
          "type": "string",
          "description": "Filter by email (exact match or pattern)"
        },
        "name": {
          "type": "string", 
          "description": "Filter by name (partial match)"
        },
        "status": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": ["active", "inactive", "suspended"]
          },
          "description": "Filter by user status"
        },
        "created_after": {
          "type": "string",
          "format": "date-time",
          "description": "Users created after this timestamp"
        },
        "created_before": {
          "type": "string", 
          "format": "date-time",
          "description": "Users created before this timestamp"
        }
      },
      "additionalProperties": false
    },

    "PaginationRequest": {
      "type": "object",
      "title": "Pagination Parameters",
      "description": "Pagination options for list operations",
      "properties": {
        "page": {
          "type": "integer",
          "minimum": 1,
          "description": "Page number (1-based)"
        },
        "page_size": {
          "type": "integer",
          "minimum": 1,
          "maximum": 1000,
          "description": "Number of items per page"
        },
        "cursor": {
          "type": "string",
          "description": "Cursor for cursor-based pagination"
        },
        "sort_by": {
          "type": "string",
          "enum": ["name", "email", "created_at", "updated_at"],
          "description": "Field to sort by"
        },
        "sort_order": {
          "type": "string",
          "enum": ["asc", "desc"],
          "description": "Sort order"
        }
      },
      "additionalProperties": false
    }
  },

  "qollective": {
    "version": "1.0.0",
    "envelope": {
      "enabled": true,
      "meta_sections": {
        "security": true,
        "tracing": true,
        "performance": true,
        "monitoring": true,
        "debug": false
      }
    },
    "generation": {
      "targets": ["rust-rest", "rust-grpc", "rust-websocket"],
      "outputDir": "./src/generated",
      "features": ["tenant-extraction", "validation", "tls"]
    },
    "tenant_extraction": {
      "enabled": true,
      "jwt_extraction": {
        "tenant_claim": "tenant_id",
        "user_claim": "sub",
        "permissions_claim": "permissions"
      }
    },
    "validation": {
      "enabled": true,
      "strict_mode": true,
      "additional_properties": false
    }
  }
}
```

## Qollective Configuration Reference

### Basic Configuration
```json
{
  "qollective": {
    "version": "1.0.0",
    "envelope": {
      "enabled": true
    }
  }
}
```

### Full Configuration Options
```json
{
  "qollective": {
    "version": "1.0.0",
    "envelope": {
      "enabled": true,
      "meta_sections": {
        "security": true,      // Include security context
        "tracing": true,       // Include distributed tracing
        "performance": true,   // Include performance metrics
        "monitoring": true,    // Include monitoring data
        "debug": false,        // Include debug information (dev only)
        "extensions": true     // Allow custom meta extensions
      }
    },
    "generation": {
      "targets": [
        "rust-rest",          // Generate REST client/server
        "rust-grpc",          // Generate gRPC client/server
        "rust-websocket",     // Generate WebSocket client/server
        "rust-nats",          // Generate NATS client/server
        "rust-jsonrpc",       // Generate JSON-RPC client/server
        "rust-mcp",           // Generate MCP client/server
        "rust-a2a"            // Generate A2A client/server
      ],
      "outputDir": "./src/generated",
      "features": [
        "tenant-extraction",  // Multi-tenant support
        "validation",         // Schema validation
        "tls",               // TLS support
        "metrics",           // Metrics collection
        "tracing"            // Distributed tracing
      ]
    },
    "tenant_extraction": {
      "enabled": true,
      "jwt_extraction": {
        "tenant_claim": "tenant_id",      // JWT claim for tenant ID
        "user_claim": "sub",              // JWT claim for user ID
        "permissions_claim": "permissions", // JWT claim for permissions
        "roles_claim": "roles"            // JWT claim for user roles
      }
    },
    "validation": {
      "enabled": true,
      "strict_mode": true,              // Strict validation mode
      "additional_properties": false,   // Allow additional properties
      "coerce_types": false            // Coerce types during validation
    },
    "security": {
      "require_auth": true,            // Require authentication
      "require_tenant": true,          // Require tenant context
      "allowed_origins": ["*"],        // CORS allowed origins
      "rate_limiting": {
        "enabled": true,
        "requests_per_minute": 100
      }
    }
  }
}
```

## Protocol-Specific Patterns

### REST API Pattern
```json
{
  "$defs": {
    "ApiRequest": {
      "type": "object",
      "properties": {
        "method": {
          "type": "string",
          "enum": ["GET", "POST", "PUT", "DELETE", "PATCH"]
        },
        "path": { "type": "string" },
        "query_params": { "type": "object" },
        "body": { "type": "object" }
      }
    }
  }
}
```

### gRPC Service Pattern
```json
{
  "$defs": {
    "GrpcRequest": {
      "type": "object",
      "properties": {
        "service": { "type": "string" },
        "method": { "type": "string" },
        "message": { "type": "object" }
      }
    }
  }
}
```

### WebSocket Message Pattern
```json
{
  "$defs": {
    "WebSocketMessage": {
      "type": "object",
      "properties": {
        "message_type": {
          "type": "string",
          "enum": ["command", "event", "response"]
        },
        "payload": { "type": "object" }
      }
    }
  }
}
```

### NATS Subject Pattern
```json
{
  "$defs": {
    "NatsMessage": {
      "type": "object",
      "properties": {
        "subject": { "type": "string" },
        "reply_to": { "type": "string" },
        "payload": { "type": "object" }
      }
    }
  }
}
```

### MCP Tool Pattern
```json
{
  "$defs": {
    "McpToolCall": {
      "type": "object",
      "properties": {
        "tool_name": { "type": "string" },
        "arguments": { "type": "object" },
        "session_id": { "type": "string" }
      }
    },
    "McpToolResponse": {
      "type": "object",
      "properties": {
        "result": { "type": "object" },
        "error": { "type": "string" },
        "is_final": { "type": "boolean" }
      }
    }
  }
}
```

### A2A Agent Communication Pattern
```json
{
  "$defs": {
    "A2aMessage": {
      "type": "object",
      "properties": {
        "agent_id": { "type": "string" },
        "target_agent": { "type": "string" },
        "message_type": {
          "type": "string",
          "enum": ["request", "response", "broadcast", "discovery"]
        },
        "payload": { "type": "object" }
      }
    }
  }
}
```

## Validation and Testing Patterns

### Request Validation Schema
```json
{
  "$defs": {
    "ValidationTest": {
      "type": "object",
      "properties": {
        "valid_requests": {
          "type": "array",
          "items": { "$ref": "#/$defs/ApiRequest" }
        },
        "invalid_requests": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "request": { "$ref": "#/$defs/ApiRequest" },
              "expected_error": { "type": "string" }
            }
          }
        }
      }
    }
  }
}
```

## Error Handling Patterns

### Standard Error Codes
```json
{
  "$defs": {
    "StandardErrors": {
      "type": "string",
      "enum": [
        "VALIDATION_ERROR",      // 400 - Invalid input data
        "AUTHENTICATION_ERROR",  // 401 - Authentication failed
        "AUTHORIZATION_ERROR",   // 403 - Insufficient permissions
        "NOT_FOUND",            // 404 - Resource not found
        "CONFLICT",             // 409 - Resource conflict
        "RATE_LIMITED",         // 429 - Rate limit exceeded
        "INTERNAL_ERROR",       // 500 - Internal server error
        "SERVICE_UNAVAILABLE",  // 503 - Service temporarily unavailable
        "TIMEOUT",              // 504 - Request timeout
        "TENANT_MISMATCH",      // Custom - Tenant context error
        "SCHEMA_VIOLATION"      // Custom - Schema validation error
      ]
    }
  }
}
```

## Multi-Tenant Patterns

### Tenant-Aware Request
```json
{
  "$defs": {
    "TenantRequest": {
      "type": "object",
      "properties": {
        "operation": { "type": "string" },
        "tenant_context": {
          "type": "object",
          "properties": {
            "tenant_id": { "type": "string" },
            "user_id": { "type": "string" },
            "permissions": {
              "type": "array",
              "items": { "type": "string" }
            }
          }
        },
        "payload": { "type": "object" }
      }
    }
  }
}
```

## Schema Generation Commands

After defining your schema, use these commands to generate Qollective-compatible code:

```bash
# Generate Rust code for all protocols
qollective-generator --schema your-service.json --target rust-all

# Generate specific protocol implementations
qollective-generator --schema your-service.json --target rust-rest
qollective-generator --schema your-service.json --target rust-grpc

# Generate with specific features
qollective-generator --schema your-service.json --features tenant-extraction,tls,validation
```

## Best Practices

### 1. Schema Organization
- Keep schemas focused on a single service/domain
- Use meaningful names for all definitions
- Include comprehensive descriptions
- Version your schemas appropriately

### 2. Data Type Design
- Use specific, constrained types rather than generic objects
- Include validation constraints (min/max, patterns, enums)
- Design for both request and response scenarios
- Consider pagination for list operations

### 3. Error Handling
- Define comprehensive error codes for your domain
- Include field-level validation errors
- Provide actionable error messages
- Follow RFC 7807 Problem Details standard

### 4. Multi-Protocol Compatibility
- Design schemas that work across all transport protocols
- Avoid protocol-specific assumptions
- Use envelope metadata for protocol-specific payload data
- Test with multiple transport layers

### 5. Performance Considerations
- Include performance metadata sections when needed
- Design for efficient serialization/deserialization
- Consider message size limits for different protocols
- Use appropriate pagination strategies

## Migration from Existing APIs

### From REST-Only APIs
1. Wrap existing request/response schemas in envelope structure
2. Add Qollective configuration section
3. Update error responses to use envelope error field
4. Enable multi-protocol generation

### From gRPC Services
1. Convert protobuf definitions to JSON Schema
2. Wrap in Qollective envelope pattern
3. Maintain message compatibility
4. Add REST and WebSocket support

### From GraphQL APIs
1. Map GraphQL types to JSON Schema definitions
2. Convert queries/mutations to request patterns
3. Wrap in envelope structure
4. Enable real-time subscriptions via WebSocket

## Troubleshooting

### Common Schema Issues
1. **Envelope not extended**: Always use `allOf` with envelope reference
2. **Missing $defs**: Define all payload data types in $defs section
3. **Invalid Qollective config**: Check configuration syntax
4. **Protocol conflicts**: Ensure schemas work across all enabled protocols

### Validation Errors
1. Check JSON Schema syntax with online validators
2. Verify all $ref references resolve correctly
3. Ensure required fields are properly defined
4. Test with sample payload data before deployment

This guide provides everything needed to create Qollective-compatible schemas. The envelope-first architecture ensures consistent behavior across all supported protocols while maintaining the flexibility to define domain-specific data structures.
