# WASM+Rust Research Documentation for Qollective Framework

## Overview
Comprehensive research findings on WASM+Rust patterns, best practices, and production-ready implementations for the Qollective Framework WASM support.

## 1. WASM-Bindgen Patterns & Async/Await

### Official Documentation & Key Resources
- **Primary**: https://rustwasm.github.io/docs/wasm-bindgen/
- **Async Guide**: https://rustwasm.github.io/wasm-bindgen/reference/js-promises-and-rust-futures.html
- **Examples**: https://github.com/rustwasm/wasm-bindgen/tree/main/examples

### Promise-Based APIs for Async Operations
```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
impl WasmClient {
    // PATTERN: All async methods return Promise<Result<JsValue, JsValue>>
    pub async fn send_envelope(&self, envelope: JsValue) -> Result<JsValue, JsValue> {
        let envelope: Envelope<serde_json::Value> = serde_wasm_bindgen::from_value(envelope)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
        
        let result = self.internal_send(envelope).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}
```

### Type Conversion Best Practices
```rust
// PATTERN: Use serde_wasm_bindgen for complex types
use serde_wasm_bindgen;

// JavaScript -> Rust
let config: WasmClientConfig = serde_wasm_bindgen::from_value(js_config)
    .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

// Rust -> JavaScript  
let response = serde_wasm_bindgen::to_value(&envelope)
    .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))?;

// PATTERN: Simple types can use direct conversion
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

### Error Handling Across WASM Boundary
```rust
// PATTERN: Create error translation layer
pub struct ErrorTranslator;

impl ErrorTranslator {
    pub fn to_js_error(error: &QollectiveError) -> JsValue {
        let error_obj = js_sys::Object::new();
        
        // Set error type for JavaScript code to handle
        js_sys::Reflect::set(&error_obj, &"type".into(), &error.error_type().into()).unwrap();
        js_sys::Reflect::set(&error_obj, &"message".into(), &error.to_string().into()).unwrap();
        js_sys::Reflect::set(&error_obj, &"retryable".into(), &error.is_retryable().into()).unwrap();
        
        error_obj.into()
    }
}

// USAGE in async methods
.map_err(|e| ErrorTranslator::to_js_error(&e))
```

### Memory Management & Optimization
```rust
// PATTERN: Use wee_alloc for smaller WASM bundles
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// PATTERN: Implement Drop for cleanup
#[wasm_bindgen]
impl Drop for WasmClient {
    fn drop(&mut self) {
        // Clean up connections, close WebSockets, etc.
        if let Some(ref mut ws) = self.websocket_client {
            let _ = ws.close();
        }
    }
}
```

### HTTP Requests with Fetch API
```rust
use web_sys::{Request, RequestInit, RequestMode, Headers};
use wasm_bindgen_futures::JsFuture;

async fn fetch_envelope<T>(&self, url: &str, envelope: &Envelope<T>) -> Result<Envelope<serde_json::Value>, QollectiveError>
where T: Serialize
{
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    
    // Set up headers
    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    headers.set("Accept", "application/json")?;
    
    // Add mTLS certificate if available
    if let Some(ref cert) = self.config.tls.client_cert {
        headers.set("X-Client-Cert", cert)?;
    }
    
    opts.headers(&headers);
    
    // Serialize envelope to JSON
    let body = serde_json::to_string(envelope)
        .map_err(|e| QollectiveError::serialization(e.to_string()))?;
    opts.body(Some(&JsValue::from_str(&body)));
    
    let request = Request::new_with_str_and_init(url, &opts)?;
    
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();
    
    if !resp.ok() {
        return Err(QollectiveError::transport(format!("HTTP {}: {}", resp.status(), resp.status_text())));
    }
    
    let json = JsFuture::from(resp.json()?).await?;
    let envelope: Envelope<serde_json::Value> = serde_wasm_bindgen::from_value(json)?;
    
    Ok(envelope)
}
```

## 2. WASM Performance Optimization

### Bundle Size Optimization Techniques
```toml
# Cargo.toml optimizations
[profile.release]
lto = true                    # Link-time optimization
codegen-units = 1            # Better optimization
panic = "abort"              # Smaller binary size
strip = true                 # Remove debug symbols

# Enable wee_alloc for smaller allocator
[dependencies]
wee_alloc = { version = "0.4", optional = true }

[features]
default = ["console_error_panic_hook"]
wee_alloc = ["dep:wee_alloc"]
```

```bash
# Build optimization pipeline
wasm-pack build --target web --release
wasm-opt -Oz pkg/qollective_bg.wasm -o pkg/qollective_bg.wasm  # Ultra optimization
```

### Bundle Size Analysis Tools
- **twiggy**: https://github.com/rustwasm/twiggy - Analyze WASM binary size
```bash
cargo install twiggy
twiggy top pkg/qollective_bg.wasm
```

### Memory Management Strategies
```rust
// PATTERN: Minimize allocations in hot paths
impl WasmClient {
    // Pre-allocate buffers for common operations
    fn new(config: WasmClientConfig) -> Self {
        Self {
            buffer: Vec::with_capacity(8192),  // Pre-allocate 8KB buffer
            config,
        }
    }
    
    // Reuse allocations where possible
    pub async fn send_multiple_envelopes(&mut self, envelopes: &[Envelope<serde_json::Value>]) -> Result<Vec<JsValue>, JsValue> {
        self.buffer.clear();  // Reuse existing allocation
        
        for envelope in envelopes {
            // Batch operations to reduce allocation overhead
        }
        
        Ok(results)
    }
}
```

### Zero-Copy Operations
```rust
// PATTERN: Use references where possible to avoid copying
pub fn process_envelope_metadata(envelope: &Envelope<serde_json::Value>) -> JsValue {
    // Work with references to avoid unnecessary cloning
    let meta_info = js_sys::Object::new();
    
    if let Some(ref tenant) = envelope.meta.tenant {
        js_sys::Reflect::set(&meta_info, &"tenant".into(), &tenant.as_str().into()).unwrap();
    }
    
    meta_info.into()
}
```

## 3. WASM Security Best Practices

### Certificate Embedding Patterns
```rust
// PATTERN: Embed certificates at compile time
const CLIENT_CERT: &str = include_str!("../certs/client.pem");
const CLIENT_KEY: &str = include_str!("../certs/client.key");
const CA_CERT: &str = include_str!("../certs/ca.pem");

#[wasm_bindgen]
pub struct TlsCredentials {
    client_cert: String,
    client_key: String,
    ca_cert: String,
}

impl TlsCredentials {
    pub fn embedded() -> Self {
        Self {
            client_cert: CLIENT_CERT.to_string(),
            client_key: CLIENT_KEY.to_string(), 
            ca_cert: CA_CERT.to_string(),
        }
    }
}
```

### Secure Token Handling
```rust
// PATTERN: Store tokens securely, clear on drop
#[wasm_bindgen]
pub struct SecureTokenStore {
    tokens: std::collections::HashMap<String, String>,
}

impl Drop for SecureTokenStore {
    fn drop(&mut self) {
        // Clear sensitive data on drop
        for (_, token) in self.tokens.iter_mut() {
            token.clear();
            token.shrink_to_fit();
        }
        self.tokens.clear();
    }
}

impl SecureTokenStore {
    pub fn store_token(&mut self, key: &str, token: String) {
        self.tokens.insert(key.to_string(), token);
    }
    
    pub fn get_token(&self, key: &str) -> Option<&str> {
        self.tokens.get(key).map(|s| s.as_str())
    }
}
```

### Cross-Origin Security
```rust
// PATTERN: Validate origins and implement CORS properly
pub fn validate_origin(origin: &str) -> bool {
    const ALLOWED_ORIGINS: &[&str] = &[
        "https://app.qollective.com",
        "https://staging.qollective.com",
        // Development
        "http://localhost:8080",
        "http://127.0.0.1:8080",
    ];
    
    ALLOWED_ORIGINS.contains(&origin)
}
```

## 4. WASM+WebSockets Implementation

### WebSocket Connection Patterns
```rust
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent};
use wasm_bindgen::closure::Closure;

#[wasm_bindgen]
pub struct WasmWebSocketClient {
    socket: Option<WebSocket>,
    on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    on_error: Option<Closure<dyn FnMut(ErrorEvent)>>,
    on_close: Option<Closure<dyn FnMut(CloseEvent)>>,
}

impl WasmWebSocketClient {
    pub async fn connect(&mut self, url: &str) -> Result<(), JsValue> {
        let ws = WebSocket::new(url)?;
        
        // Set up message handler
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let envelope_text = txt.as_string().unwrap();
                // Process envelope here
                web_sys::console::log_1(&format!("Received envelope: {}", envelope_text).into());
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        
        // Store closure to prevent it from being dropped
        self.on_message = Some(on_message);
        self.socket = Some(ws);
        
        Ok(())
    }
    
    pub fn send_envelope(&self, envelope: &Envelope<serde_json::Value>) -> Result<(), JsValue> {
        if let Some(ref ws) = self.socket {
            let json = serde_json::to_string(envelope)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            ws.send_with_str(&json)?;
        }
        Ok(())
    }
}
```

### Connection Pooling and Management
```rust
// PATTERN: Implement connection pooling for WebSockets
pub struct WebSocketPool {
    connections: std::collections::HashMap<String, WasmWebSocketClient>,
    max_connections: usize,
}

impl WebSocketPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: std::collections::HashMap::new(),
            max_connections,
        }
    }
    
    pub async fn get_or_create_connection(&mut self, url: &str) -> Result<&mut WasmWebSocketClient, JsValue> {
        if !self.connections.contains_key(url) {
            if self.connections.len() >= self.max_connections {
                // Close oldest connection
                if let Some((old_url, _)) = self.connections.iter().next() {
                    let old_url = old_url.clone();
                    self.connections.remove(&old_url);
                }
            }
            
            let mut client = WasmWebSocketClient::new();
            client.connect(url).await?;
            self.connections.insert(url.to_string(), client);
        }
        
        Ok(self.connections.get_mut(url).unwrap())
    }
}
```

## 5. WASM Testing Strategies

### Browser Testing with wasm-bindgen-test
```rust
// tests/wasm_tests.rs
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_envelope_creation() {
    let envelope = Envelope {
        meta: Meta::with_auto_fields(),
        data: serde_json::json!({"test": "data"}),
        error: None,
    };
    
    assert_eq!(envelope.data["test"], "data");
}

#[wasm_bindgen_test]
async fn test_async_envelope_send() {
    let config = WasmClientConfig::default();
    let client = WasmClient::new(serde_wasm_bindgen::to_value(&config).unwrap()).unwrap();
    
    // This would require mocking fetch API for full test
    // Use web_sys::window().unwrap().fetch_with_str() pattern
}
```

### Unit Testing Patterns for WASM Code
```bash
# Run WASM tests in browser
wasm-pack test --headless --firefox

# Run with Chrome
wasm-pack test --headless --chrome

# Run with specific test
wasm-pack test --headless --firefox -- test_envelope_creation
```

### Integration Testing Approaches
```javascript
// tests/integration.test.js - JavaScript integration tests
import init, { WasmClient } from '../pkg/qollective.js';

describe('WASM Envelope Integration', () => {
    beforeAll(async () => {
        await init();
    });
    
    test('should create client and send envelope', async () => {
        const config = {
            rest: {
                base_url: 'https://api.example.com',
                timeout_ms: 30000
            }
        };
        
        const client = new WasmClient(config);
        
        const envelope = {
            meta: {
                tenant: 'test-tenant',
                request_id: '123-456-789'
            },
            data: { message: 'test' },
            error: null
        };
        
        // Mock fetch for testing
        global.fetch = jest.fn().mockResolvedValue({
            ok: true,
            json: () => Promise.resolve({
                meta: envelope.meta,
                data: { response: 'success' },
                error: null
            })
        });
        
        const result = await client.send_rest_envelope(envelope);
        expect(result.data.response).toBe('success');
    });
});
```

## 6. Production Deployment Considerations

### Common Gotchas and Pitfalls
```rust
// GOTCHA: WASM doesn't support all std library features
// AVOID: std::thread, std::process, file I/O
// USE: web-sys APIs, wasm-bindgen for browser integration

// GOTCHA: Shared memory requires proper synchronization
// AVOID: Global mutable state without protection
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

static GLOBAL_CONFIG: Lazy<Arc<Mutex<Option<WasmClientConfig>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

// GOTCHA: Error messages cross WASM boundary must be serializable
// AVOID: Complex error types with non-serializable fields
#[derive(Debug, Serialize)]
pub struct WasmError {
    error_type: String,
    message: String,
    retryable: bool,
}
```

### Performance Optimization Warnings
```rust
// WARNING: Frequent allocations in hot paths
// BAD:
pub fn process_many_envelopes(envelopes: &[Envelope<serde_json::Value>]) -> Vec<String> {
    envelopes.iter()
        .map(|e| serde_json::to_string(e).unwrap())  // Many allocations
        .collect()
}

// GOOD:
pub fn process_many_envelopes_optimized(envelopes: &[Envelope<serde_json::Value>], buffer: &mut String) -> Vec<String> {
    let mut results = Vec::with_capacity(envelopes.len());
    
    for envelope in envelopes {
        buffer.clear();  // Reuse allocation
        serde_json::to_writer(unsafe { buffer.as_mut_vec() }, envelope).unwrap();
        results.push(buffer.clone());
    }
    
    results
}
```

### Bundle Size Monitoring
```bash
# Monitor bundle size in CI/CD
#!/bin/bash
BUNDLE_SIZE=$(stat -c%s pkg/qollective_bg.wasm)
MAX_SIZE=512000  # 500KB

if [ $BUNDLE_SIZE -gt $MAX_SIZE ]; then
    echo "Bundle size $BUNDLE_SIZE exceeds maximum $MAX_SIZE"
    exit 1
fi

echo "Bundle size: $BUNDLE_SIZE bytes (OK)"
```

## 7. Specific URLs and Resources

### Official Documentation
- **Rust WASM Book**: https://rustwasm.github.io/docs/book/
- **wasm-bindgen Guide**: https://rustwasm.github.io/wasm-bindgen/
- **web-sys Documentation**: https://rustwasm.github.io/wasm-bindgen/api/web_sys/
- **js-sys Documentation**: https://rustwasm.github.io/wasm-bindgen/api/js_sys/

### GitHub Repositories with Production Examples
- **wasm-bindgen Examples**: https://github.com/rustwasm/wasm-bindgen/tree/main/examples
- **wasm-pack Template**: https://github.com/rustwasm/wasm-pack-template
- **Production WASM Apps**: https://github.com/yewstack/yew (for patterns)

### Performance Tools
- **twiggy**: https://github.com/rustwasm/twiggy - WASM size profiler
- **wasm-opt**: https://github.com/WebAssembly/binaryen - WASM optimizer
- **wasmtime**: https://github.com/bytecodealliance/wasmtime - WASM runtime

### Testing Resources
- **wasm-bindgen-test**: https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/
- **Browser Testing Setup**: https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/browsers.html

### Community Resources
- **Rust WASM Working Group**: https://github.com/rustwasm
- **Mozilla WASM Docs**: https://developer.mozilla.org/en-US/docs/WebAssembly
- **WASM Performance Guide**: https://web.dev/webassembly/

## 8. Critical Implementation Notes for Qollective

### Framework Integration Requirements
1. **Preserve Envelope Pattern**: All WASM communication must maintain `Envelope<T>` structure
2. **Config Inheritance**: Every WASM component must accept and use configuration
3. **Constants Usage**: No hardcoded values - all from `constants.rs`
4. **TLS First**: All remote communication must support mTLS
5. **Error Translation**: QollectiveError must be translated to user-friendly JavaScript

### Performance Targets
- Bundle size < 500KB after wasm-opt
- Initial load time < 2 seconds on 3G
- Memory usage < 10MB for typical operation
- Support for 100+ concurrent envelope operations

### Security Requirements
- mTLS certificate embedding
- Secure token storage with automatic cleanup
- Origin validation for all requests
- No sensitive data in console logs

This research provides the comprehensive foundation needed for implementing WASM support in the Qollective Framework while maintaining all existing patterns and performance requirements.