// ABOUTME: Monitoring and observability integrations for Qollective framework
// ABOUTME: Provides essential metrics collection for production visibility

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Simple metrics storage for monitoring
#[derive(Debug, Clone)]
pub(crate) struct MetricsStore {
    pub(crate) http_requests: HashMap<String, u64>,
    pub(crate) grpc_requests: HashMap<String, u64>,
    pub(crate) envelope_operations: HashMap<String, u64>,
    pub(crate) operation_durations: HashMap<String, Vec<Duration>>,
}

impl MetricsStore {
    fn new() -> Self {
        Self {
            http_requests: HashMap::new(),
            grpc_requests: HashMap::new(),
            envelope_operations: HashMap::new(),
            operation_durations: HashMap::new(),
        }
    }

    fn record_http_request(
        &mut self,
        method: &str,
        endpoint: &str,
        status: u16,
        duration: Duration,
    ) {
        let key = format!("{}:{} {}", method, endpoint, status);
        *self.http_requests.entry(key.clone()).or_insert(0) += 1;
        self.operation_durations
            .entry(key)
            .or_insert_with(Vec::new)
            .push(duration);
    }

    fn record_envelope_operation(&mut self, operation: &str, duration: Duration, success: bool) {
        let key = format!(
            "{}:{}",
            operation,
            if success { "success" } else { "error" }
        );
        *self.envelope_operations.entry(key.clone()).or_insert(0) += 1;
        self.operation_durations
            .entry(key)
            .or_insert_with(Vec::new)
            .push(duration);
    }
}

// Global metrics store
static METRICS_STORE: std::sync::OnceLock<Arc<Mutex<MetricsStore>>> = std::sync::OnceLock::new();

pub(crate) fn get_metrics_store() -> Arc<Mutex<MetricsStore>> {
    METRICS_STORE
        .get_or_init(|| Arc::new(Mutex::new(MetricsStore::new())))
        .clone()
}

/// Record HTTP request metrics
pub fn record_http_request(method: &str, endpoint: &str, status: u16, duration: Duration) {
    let store = get_metrics_store();
    if let Ok(mut metrics) = store.lock() {
        metrics.record_http_request(method, endpoint, status, duration);
    };
}

/// Record envelope operation metrics  
pub fn record_envelope_operation(operation: &str, duration: Duration, success: bool) {
    let store = get_metrics_store();
    if let Ok(mut metrics) = store.lock() {
        metrics.record_envelope_operation(operation, duration, success);
    };
}

/// Get metrics summary for testing/debugging
pub fn get_metrics_summary() -> HashMap<String, u64> {
    let store = get_metrics_store();
    let result = if let Ok(metrics) = store.lock() {
        let mut summary = HashMap::new();
        for (key, count) in &metrics.http_requests {
            summary.insert(format!("http:{}", key), *count);
        }
        for (key, count) in &metrics.grpc_requests {
            summary.insert(format!("grpc:{}", key), *count);
        }
        for (key, count) in &metrics.envelope_operations {
            summary.insert(format!("envelope:{}", key), *count);
        }
        summary
    } else {
        HashMap::new()
    };
    result
}

/// Clear all metrics (for testing)
#[cfg(test)]
pub fn clear_metrics() {
    let store = get_metrics_store();
    if let Ok(mut metrics) = store.lock() {
        *metrics = MetricsStore::new();
    };
}

/// Start a timer for measuring operation duration
pub fn start_operation_timer() -> OperationTimer {
    OperationTimer::new()
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    start: Instant,
}

impl OperationTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_should_record_http_request_metrics() {
        // Clear metrics for test isolation
        clear_metrics();

        // Record some HTTP requests
        record_http_request("GET", "/api/users", 200, Duration::from_millis(150));
        record_http_request("POST", "/api/users", 201, Duration::from_millis(300));
        record_http_request("GET", "/api/users/123", 404, Duration::from_millis(75));
        record_http_request("GET", "/api/users", 200, Duration::from_millis(120)); // Same endpoint again

        // Verify metrics were recorded
        let summary = get_metrics_summary();
        assert_eq!(summary.get("http:GET:/api/users 200"), Some(&2)); // Called twice
        assert_eq!(summary.get("http:POST:/api/users 201"), Some(&1));
        assert_eq!(summary.get("http:GET:/api/users/123 404"), Some(&1));
    }

    #[test]
    fn test_should_record_envelope_processing_metrics() {
        // Clear metrics for test isolation
        clear_metrics();

        // Record envelope operations
        record_envelope_operation("serialize", Duration::from_millis(5), true);
        record_envelope_operation("deserialize", Duration::from_millis(8), true);
        record_envelope_operation("validate", Duration::from_millis(2), false);
        record_envelope_operation("serialize", Duration::from_millis(4), true); // Same operation again

        // Verify metrics were recorded
        let summary = get_metrics_summary();
        assert_eq!(summary.get("envelope:serialize:success"), Some(&2)); // Called twice
        assert_eq!(summary.get("envelope:deserialize:success"), Some(&1));
        assert_eq!(summary.get("envelope:validate:error"), Some(&1));
    }

    #[test]
    fn test_should_provide_timing_utilities() {
        let timer = start_operation_timer();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();

        assert!(elapsed >= Duration::from_millis(10));
        assert!(elapsed <= Duration::from_millis(50)); // Give some reasonable upper bound
    }

    #[test]
    fn test_metrics_thread_safety() {
        use std::thread;

        // Clear metrics for test isolation
        clear_metrics();

        // Spawn multiple threads recording metrics concurrently
        let handles: Vec<_> = (0..5)
            .map(|i| {
                thread::spawn(move || {
                    record_http_request(
                        "GET",
                        &format!("/test/{}", i),
                        200,
                        Duration::from_millis(100),
                    );
                    record_envelope_operation("test", Duration::from_millis(10), true);
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all metrics were recorded
        let summary = get_metrics_summary();

        // Should have 5 different endpoints
        for i in 0..5 {
            let key = format!("http:GET:/test/{} 200", i);
            assert_eq!(summary.get(&key), Some(&1));
        }

        // Should have 5 test envelope operations
        assert_eq!(summary.get("envelope:test:success"), Some(&5));
    }
}

pub mod middleware_integration;
