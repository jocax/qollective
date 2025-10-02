// Debug test for OPTIONS query parameter transmission

use std::collections::HashMap;

#[tokio::test]
async fn debug_options_query_params() {
    // Test what the client should be sending
    let test_data = serde_json::json!({
        "message": "test OPTIONS roundtrip",
        "method": "options"
    });

    let data_json = serde_json::to_string(&test_data).unwrap();
    println!("🔍 Client would send envelope_data: {}", data_json);

    // Test server-side parsing logic
    let mut query_params = HashMap::new();
    query_params.insert("envelope_data".to_string(), data_json.clone());

    println!("🔍 Server would receive params: {:?}", query_params);

    // Test the actual parsing logic from route_request_with_registry
    if let Some(envelope_data_str) = query_params.get("envelope_data") {
        println!("✅ envelope_data found: {}", envelope_data_str);
        match serde_json::from_str::<serde_json::Value>(envelope_data_str) {
            Ok(data) => {
                println!("✅ Successfully parsed: {:?}", data);
                assert_eq!(data["method"], "options");
                assert_eq!(data["message"], "test OPTIONS roundtrip");
            }
            Err(e) => {
                println!("❌ Parse error: {}", e);
                panic!("Failed to parse envelope_data");
            }
        }
    } else {
        println!("❌ ERROR: envelope_data is missing from query parameters");
        panic!("envelope_data missing");
    }

    // Test URL encoding/decoding
    let encoded = urlencoding::encode(&data_json);
    println!("🔍 URL encoded: {}", encoded);

    let decoded = urlencoding::decode(&encoded).unwrap();
    println!("🔍 URL decoded: {}", decoded);

    // Verify round-trip
    assert_eq!(decoded.as_ref(), data_json);
}
