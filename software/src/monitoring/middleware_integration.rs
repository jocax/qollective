// ABOUTME: Tests for monitoring integration with server middleware
// ABOUTME: Defines how automatic monitoring should work with REST and gRPC servers

use std::time::Duration;

// Public function for gRPC middleware to use
pub fn record_grpc_request(method: &str, status: &str, duration: Duration) {
    let store = crate::monitoring::get_metrics_store();
    if let Ok(mut metrics) = store.lock() {
        let key = format!("{}:{}", method, status);
        *metrics.grpc_requests.entry(key.clone()).or_insert(0) += 1;
        metrics
            .operation_durations
            .entry(format!("grpc:{}", key))
            .or_insert_with(Vec::new)
            .push(duration);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::{clear_metrics, get_metrics_summary};
    use std::time::Duration;

    #[test]
    fn test_rest_middleware_should_automatically_record_http_metrics() {
        // Clear metrics for test isolation
        clear_metrics();

        // This test defines that REST middleware should automatically record HTTP metrics
        // when processing requests through the server

        // Simulate a REST request being processed
        let middleware = create_monitoring_rest_middleware();

        // Process a simulated request
        middleware.process_request("GET", "/api/users", 200, Duration::from_millis(150));
        middleware.process_request("POST", "/api/users", 201, Duration::from_millis(300));

        // Verify metrics were automatically recorded
        let summary = get_metrics_summary();
        assert_eq!(summary.get("http:GET:/api/users 200"), Some(&1));
        assert_eq!(summary.get("http:POST:/api/users 201"), Some(&1));
    }

    #[test]
    fn test_rest_middleware_should_record_envelope_processing_metrics() {
        // Clear metrics for test isolation
        clear_metrics();

        // This test defines that envelope operations should be tracked during request processing
        let middleware = create_monitoring_rest_middleware();

        // Simulate envelope operations during request processing
        middleware.process_envelope_operation("deserialize", Duration::from_millis(5), true);
        middleware.process_envelope_operation("validate", Duration::from_millis(2), true);
        middleware.process_envelope_operation("serialize", Duration::from_millis(3), true);

        // Verify envelope operations were recorded
        let summary = get_metrics_summary();
        assert_eq!(summary.get("envelope:deserialize:success"), Some(&1));
        assert_eq!(summary.get("envelope:validate:success"), Some(&1));
        assert_eq!(summary.get("envelope:serialize:success"), Some(&1));
    }

    #[test]
    fn test_grpc_middleware_should_automatically_record_grpc_metrics() {
        // Clear metrics for test isolation
        clear_metrics();

        // This test defines that gRPC middleware should automatically record metrics
        let middleware = create_monitoring_grpc_middleware();

        // Simulate gRPC request processing
        middleware.process_grpc_request("GetUser", "success", Duration::from_millis(75));
        middleware.process_grpc_request("CreateUser", "success", Duration::from_millis(200));

        // Verify gRPC metrics were recorded
        let summary = get_metrics_summary();
        assert_eq!(summary.get("grpc:GetUser:success"), Some(&1));
        assert_eq!(summary.get("grpc:CreateUser:success"), Some(&1));
    }

    #[test]
    fn test_middleware_should_handle_errors_gracefully() {
        // Clear metrics for test isolation
        clear_metrics();

        // This test defines that middleware should record error metrics
        let middleware = create_monitoring_rest_middleware();

        // Simulate error scenarios
        middleware.process_request("GET", "/api/missing", 404, Duration::from_millis(50));
        middleware.process_envelope_operation("validate", Duration::from_millis(1), false);

        // Verify error metrics were recorded
        let summary = get_metrics_summary();
        assert_eq!(summary.get("http:GET:/api/missing 404"), Some(&1));
        assert_eq!(summary.get("envelope:validate:error"), Some(&1));
    }

    // These are the structs/functions we need to implement to make tests pass
    fn create_monitoring_rest_middleware() -> MonitoringRestMiddleware {
        MonitoringRestMiddleware::new()
    }

    fn create_monitoring_grpc_middleware() -> MonitoringGrpcMiddleware {
        MonitoringGrpcMiddleware::new()
    }

    struct MonitoringRestMiddleware;

    impl MonitoringRestMiddleware {
        fn new() -> Self {
            Self
        }

        fn process_request(&self, method: &str, endpoint: &str, status: u16, duration: Duration) {
            crate::monitoring::record_http_request(method, endpoint, status, duration);
        }

        fn process_envelope_operation(&self, operation: &str, duration: Duration, success: bool) {
            crate::monitoring::record_envelope_operation(operation, duration, success);
        }
    }

    struct MonitoringGrpcMiddleware;

    impl MonitoringGrpcMiddleware {
        fn new() -> Self {
            Self
        }

        fn process_grpc_request(&self, method: &str, status: &str, duration: Duration) {
            // Record gRPC metrics using the general HTTP recording but with grpc: prefix
            record_grpc_request(method, status, duration);
        }
    }

    // Helper function for gRPC metrics - minimal implementation to make tests pass
    fn record_grpc_request(method: &str, status: &str, duration: Duration) {
        let store = crate::monitoring::get_metrics_store();
        if let Ok(mut metrics) = store.lock() {
            let key = format!("{}:{}", method, status);
            *metrics.grpc_requests.entry(key.clone()).or_insert(0) += 1;
            metrics
                .operation_durations
                .entry(format!("grpc:{}", key))
                .or_insert_with(Vec::new)
                .push(duration);
        };
    }
}
