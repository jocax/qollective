# WASM Envelope REST Scenario

## Scenario Description

Traditional REST API communication for CRUD operations like authentication, user management, and data retrieval. This scenario uses synchronous request/response patterns over HTTPS with mTLS authentication.

## Flow Pattern

```
User → Browser/UI → WASM Client → HTTPS Server → Backend Services
```

## Transport Protocol Analysis

| Transport | REST Support | Security | Request/Response | Streaming | Content Types | Binary Support | Complexity | Enterprise Ready |
|-----------|--------------|----------|------------------|-----------|---------------|----------------|------------|------------------|
| **HTTPS/1.1** | ✅ **Primary** | 🔒 TLS + mTLS | ✅ Yes | ❌ No | 📄 JSON, 📊 XML, 🌐 HTML | ⚠️ Via Base64 | 🟢 Low | ✅ Yes |
| **HTTPS/2** | ✅ **Optimal** | 🔒 TLS + mTLS | ✅ Yes | ✅ Multiplexed | 📄 JSON, 📊 XML, 📁 Any MIME | ✅ Native Binary | 🟡 Medium | ✅ Yes |

## Envelope Structure

### Request Envelope
```rust
Envelope<LoginRequest> {
    meta: Meta {
        timestamp: Some(DateTime::now()),
        request_id: Some(Uuid::new()),
        tenant: Some("abc123"),
        version: Some("1.0"),
        security: Some(SecurityMeta { ... }),
    },
    data: LoginRequest {
        username: "user@domain.com",
        password: "hashed_password",
    },
    error: None,
}
```

### Response Envelope
```rust
Envelope<User> {
    meta: Meta {
        request_id: Some(original_request_id),
        tenant: Some("abc123"),
        duration: Some(45.2), // milliseconds
    },
    data: User {
        id: 12345,
        username: "user@domain.com",
        token: "jwt_token_here",
        roles: vec!["user".to_string()],
    },
    error: None,
}
```

## Security Requirements

### Transport Security
- **TLS 1.3**: All HTTPS communications encrypted
- **mTLS Authentication**: Client certificates embedded in WASM
- **Certificate Validation**: Server certificate verification required

### Application Security
- **Tenant Isolation**: Tenant ID required in meta section
- **Request Correlation**: Unique request ID for tracing
- **Token Management**: JWT tokens handled by WASM layer

## Rust Code Examples

### WASM Client Implementation
```rust
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
pub struct RestClient {
    base_url: String,
    client: reqwest::Client,
}

#[wasm_bindgen]
impl RestClient {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .build()
            .unwrap();
        
        Self { base_url, client }
    }
    
    pub async fn login(&self, username: String, password: String) -> Result<JsValue, JsValue> {
        let envelope = Envelope {
            meta: Meta::with_auto_fields(),
            data: LoginRequest { username, password },
            error: None,
        };
        
        let response = self.client
            .post(&format!("{}/auth/login", self.base_url))
            .json(&envelope)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let result: Envelope<User> = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        Ok(serde_wasm_bindgen::to_value(&result.data)?)
    }
}
```

### Meta Auto-Population
```rust
impl Meta {
    pub fn with_auto_fields() -> Self {
        Self {
            timestamp: Some(Utc::now()),
            request_id: Some(Uuid::now_v7()),
            version: Some("1.0".to_string()),
            tenant: Self::extract_tenant_from_token(),
            security: Some(SecurityMeta::from_current_session()),
            ..Default::default()
        }
    }
}
```

### Error Handling
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn handle_login_response(envelope: Envelope<User>) -> Result<User, QollectiveError> {
    match envelope.error {
        Some(error) => Err(error),
        None => Ok(envelope.data),
    }
}
```

## Use Cases

### Authentication Flow
1. User enters credentials in UI
2. WASM creates `Envelope<LoginRequest>` with auto-populated meta
3. HTTPS POST to `/auth/login` with mTLS
4. Server validates and returns `Envelope<User>`
5. WASM extracts user data and token for session

### CRUD Operations
1. **Create**: POST with `Envelope<CreateEntityRequest>`
2. **Read**: GET with tenant context in headers
3. **Update**: PUT with `Envelope<UpdateEntityRequest>`
4. **Delete**: DELETE with confirmation envelope

### Password Reset
1. UI submits email
2. WASM sends `Envelope<PasswordResetRequest>`
3. Server processes and returns `Envelope<PasswordResetResponse>`
4. UI displays confirmation message

## Error Scenarios

### Network Errors
```rust
// Transport layer errors become QollectiveError::Transport
match reqwest_error.kind() {
    ErrorKind::Timeout => QollectiveError::transport("Request timeout"),
    ErrorKind::Connect => QollectiveError::connection("Connection failed"),
    _ => QollectiveError::transport(reqwest_error.to_string()),
}
```

### Authentication Errors
```rust
// Server returns envelope with error field populated
Envelope<()> {
    meta: Meta { ... },
    data: (),
    error: Some(QollectiveError::Security("Invalid credentials")),
}
```

## Performance Considerations

### HTTP/2 Benefits
- **Multiplexing**: Multiple requests over single connection
- **Header Compression**: Reduced overhead for repeated headers
- **Server Push**: Proactive resource delivery

### Caching Strategy
- **Envelope Metadata**: Include cache directives in meta section
- **ETags**: Use request_id for cache validation
- **Conditional Requests**: If-Modified-Since with timestamp from meta
