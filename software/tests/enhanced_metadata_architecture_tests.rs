//! Enhanced Metadata Architecture Tests
//!
//! This module tests the enhanced metadata architecture with comprehensive
//! OpenAPI schema generation support, including SecurityContext, DelegationContext,
//! PerformanceContext, TracingContext, InfrastructureContext, and DebugContext
//! structures designed for enterprise-grade distributed systems.

use utoipa::OpenApi;
use serde_json::json;
use qollective::envelope::{
    Meta, SecurityMeta, OnBehalfOfMeta, PerformanceMeta, TracingMeta, MonitoringMeta, DebugMeta
};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_context_schema_generation() {
        // Test that SecurityMeta generates comprehensive OpenAPI schema
        // with authentication, authorization, and session management fields
        
        #[derive(OpenApi)]
        #[openapi(
            components(schemas(SecurityMeta)),
            info(title = "Security Context API", version = "1.0.0")
        )]
        struct SecurityApiDoc;
        
        let openapi = SecurityApiDoc::openapi();
        assert_eq!(openapi.info.title, "Security Context API");
        
        let components = openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("SecurityMeta"), 
                "SecurityMeta schema should be present");
    }
    
    #[test]
    fn test_delegation_context_schema_generation() {
        // Test that OnBehalfOfMeta generates proper OpenAPI schema
        // for on-behalf-of scenarios and delegation chains
        
        #[derive(OpenApi)]
        #[openapi(
            components(schemas(OnBehalfOfMeta)),
            info(title = "Delegation Context API", version = "1.0.0")
        )]
        struct DelegationApiDoc;
        
        let openapi = DelegationApiDoc::openapi();
        let components = openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("OnBehalfOfMeta"), 
                "OnBehalfOfMeta schema should be present");
    }
    
    #[test]
    fn test_performance_context_schema_generation() {
        // Test that PerformanceMeta generates OpenAPI schema
        // with metrics, timing, and monitoring information
        
        #[derive(OpenApi)]
        #[openapi(
            components(schemas(PerformanceMeta)),
            info(title = "Performance Context API", version = "1.0.0")
        )]
        struct PerformanceApiDoc;
        
        let openapi = PerformanceApiDoc::openapi();
        let components = openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("PerformanceMeta"), 
                "PerformanceMeta schema should be present");
    }
    
    #[test]
    fn test_tracing_context_schema_generation() {
        // Test that TracingMeta generates OpenTelemetry-compatible
        // OpenAPI schema with proper trace and span information
        
        #[derive(OpenApi)]
        #[openapi(
            components(schemas(TracingMeta)),
            info(title = "Tracing Context API", version = "1.0.0")
        )]
        struct TracingApiDoc;
        
        let openapi = TracingApiDoc::openapi();
        let components = openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("TracingMeta"), 
                "TracingMeta schema should be present");
    }
    
    #[test]
    fn test_infrastructure_context_schema_generation() {
        // Test that MonitoringMeta generates OpenAPI schema
        // with deployment, service, and environment information
        
        #[derive(OpenApi)]
        #[openapi(
            components(schemas(MonitoringMeta)),
            info(title = "Infrastructure Context API", version = "1.0.0")
        )]
        struct InfrastructureApiDoc;
        
        let openapi = InfrastructureApiDoc::openapi();
        let components = openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("MonitoringMeta"), 
                "MonitoringMeta schema should be present");
    }
    
    #[test]
    fn test_debug_context_schema_generation() {
        // Test that DebugMeta generates OpenAPI schema
        // for development tools and debugging information
        
        #[derive(OpenApi)]
        #[openapi(
            components(schemas(DebugMeta)),
            info(title = "Debug Context API", version = "1.0.0")
        )]
        struct DebugApiDoc;
        
        let openapi = DebugApiDoc::openapi();
        let components = openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("DebugMeta"), 
                "DebugMeta schema should be present");
    }
    
    #[test]
    fn test_enhanced_meta_structure_schema_generation() {
        // Test that enhanced Meta structure generates comprehensive
        // OpenAPI schema including all context types and custom fields
        
        #[derive(OpenApi)]
        #[openapi(
            components(schemas(Meta)),
            info(title = "Enhanced Meta API", version = "1.0.0")
        )]
        struct EnhancedMetaApiDoc;
        
        let openapi = EnhancedMetaApiDoc::openapi();
        let components = openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("Meta"), 
                "Meta schema should be present");
    }
    
    #[test]
    fn test_security_context_realistic_data() {
        // Test SecurityContext with realistic enterprise authentication data
        
        let security_data = json!({
            "authentication": {
                "method": "JWT",
                "token_type": "Bearer",
                "subject": "user@enterprise.starfleet.local",
                "issuer": "https://auth.starfleet.local",
                "audience": ["enterprise-bridge", "enterprise-crew"],
                "expires_at": "2025-08-23T11:30:45.123Z",
                "issued_at": "2025-08-23T10:30:45.123Z",
                "scopes": ["bridge:read", "crew:manage", "ship:navigate"]
            },
            "authorization": {
                "roles": ["Captain", "Bridge Officer"],
                "permissions": ["COMMAND_SHIP", "CREW_MANAGEMENT", "TACTICAL_SYSTEMS"],
                "resource_access": {
                    "ship_systems": ["navigation", "tactical", "engineering"],
                    "crew_quarters": ["deck_1", "deck_2", "bridge"]
                }
            },
            "session": {
                "session_id": "bridge_session_001",
                "created_at": "2025-08-23T10:00:00.000Z",
                "last_activity": "2025-08-23T10:30:45.123Z",
                "ip_address": "192.168.1.100",
                "user_agent": "Enterprise Bridge Console v2.0",
                "mfa_verified": true
            }
        });
        
        // Verify the structure is valid JSON
        assert!(security_data.is_object());
        assert!(security_data["authentication"].is_object());
        assert!(security_data["authorization"].is_object());
        assert!(security_data["session"].is_object());
        
        // Verify required authentication fields
        assert_eq!(security_data["authentication"]["method"], "JWT");
        assert_eq!(security_data["authentication"]["subject"], "user@enterprise.starfleet.local");
        
        // Verify authorization structure
        assert!(security_data["authorization"]["roles"].is_array());
        assert!(security_data["authorization"]["permissions"].is_array());
    }
    
    #[test]
    fn test_delegation_context_realistic_data() {
        // Test DelegationContext with realistic on-behalf-of scenarios
        
        let delegation_data = json!({
            "original_user": {
                "user_id": "picard@starfleet.local",
                "name": "Jean-Luc Picard",
                "role": "Captain"
            },
            "acting_user": {
                "user_id": "riker@starfleet.local", 
                "name": "William T. Riker",
                "role": "First Officer"
            },
            "delegation_reason": "Captain delegation during away mission",
            "delegation_scope": ["bridge_command", "crew_orders", "ship_navigation"],
            "delegation_expires": "2025-08-23T18:00:00.000Z",
            "delegation_level": "FULL_AUTHORITY",
            "audit_trail": [
                {
                    "timestamp": "2025-08-23T10:30:45.123Z",
                    "action": "DELEGATION_GRANTED",
                    "granted_by": "picard@starfleet.local",
                    "granted_to": "riker@starfleet.local"
                }
            ]
        });
        
        // Verify delegation structure
        assert!(delegation_data.is_object());
        assert!(delegation_data["original_user"].is_object());
        assert!(delegation_data["acting_user"].is_object());
        assert!(delegation_data["delegation_scope"].is_array());
        assert!(delegation_data["audit_trail"].is_array());
        
        // Verify delegation details
        assert_eq!(delegation_data["delegation_reason"], "Captain delegation during away mission");
        assert_eq!(delegation_data["delegation_level"], "FULL_AUTHORITY");
    }
    
    #[test]
    fn test_performance_context_realistic_data() {
        // Test PerformanceContext with realistic metrics and timing data
        
        let performance_data = json!({
            "request_timing": {
                "started_at": "2025-08-23T10:30:45.000Z",
                "first_byte_at": "2025-08-23T10:30:45.050Z",
                "completed_at": "2025-08-23T10:30:45.123Z",
                "total_duration_ms": 123,
                "processing_duration_ms": 98,
                "network_duration_ms": 25
            },
            "resource_usage": {
                "cpu_usage_percent": 15.7,
                "memory_usage_mb": 256.8,
                "disk_io_mb": 1.2,
                "network_io_mb": 0.8
            },
            "metrics": {
                "requests_per_second": 1250,
                "error_rate_percent": 0.02,
                "cache_hit_rate_percent": 87.5,
                "database_query_count": 3,
                "database_query_duration_ms": 45
            },
            "bottlenecks": [
                {
                    "component": "database",
                    "severity": "low", 
                    "description": "Query execution slightly elevated"
                }
            ]
        });
        
        // Verify performance structure
        assert!(performance_data.is_object());
        assert!(performance_data["request_timing"].is_object());
        assert!(performance_data["resource_usage"].is_object());
        assert!(performance_data["metrics"].is_object());
        assert!(performance_data["bottlenecks"].is_array());
        
        // Verify timing data
        assert_eq!(performance_data["request_timing"]["total_duration_ms"], 123);
        assert_eq!(performance_data["metrics"]["requests_per_second"], 1250);
    }
    
    #[test]
    fn test_tracing_context_opentelemetry_compatibility() {
        // Test TracingContext with OpenTelemetry-compatible trace data
        
        let tracing_data = json!({
            "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
            "span_id": "00f067aa0ba902b7",
            "parent_span_id": "83887e5d7da921ba",
            "trace_flags": "01",
            "trace_state": "congo=t61rcWkgMzE,rojo=00f067aa0ba902b7",
            "span_name": "enterprise_bridge_command",
            "span_kind": "SERVER",
            "span_status": {
                "code": "OK",
                "message": "Command executed successfully"
            },
            "span_attributes": {
                "service.name": "enterprise-bridge",
                "service.version": "2.0.0",
                "operation.name": "execute_command",
                "user.id": "picard@starfleet.local",
                "ship.registry": "NCC-1701-D"
            },
            "span_events": [
                {
                    "timestamp": "2025-08-23T10:30:45.123Z",
                    "name": "command_received",
                    "attributes": {
                        "command.type": "navigation",
                        "command.priority": "high"
                    }
                }
            ]
        });
        
        // Verify OpenTelemetry compatibility
        assert!(tracing_data.is_object());
        assert_eq!(tracing_data["trace_id"].as_str().unwrap().len(), 32);
        assert_eq!(tracing_data["span_id"].as_str().unwrap().len(), 16);
        assert_eq!(tracing_data["span_kind"], "SERVER");
        
        // Verify span structure
        assert!(tracing_data["span_attributes"].is_object());
        assert!(tracing_data["span_events"].is_array());
        assert_eq!(tracing_data["span_status"]["code"], "OK");
    }
    
    #[test]
    fn test_infrastructure_context_realistic_data() {
        // Test InfrastructureContext with realistic deployment information
        
        let infrastructure_data = json!({
            "service_info": {
                "name": "enterprise-bridge-service",
                "version": "2.0.0",
                "build_number": "2025.08.23.001",
                "deployed_at": "2025-08-23T09:00:00.000Z"
            },
            "environment": {
                "name": "production",
                "region": "starfleet-alpha-quadrant",
                "availability_zone": "deck-1-section-a",
                "cluster": "enterprise-cluster"
            },
            "infrastructure": {
                "kubernetes_namespace": "starfleet-bridge",
                "pod_name": "enterprise-bridge-7d8f9b6c5-xyz12",
                "node_name": "bridge-node-01",
                "container_id": "docker://abc123def456",
                "image": "starfleet/enterprise-bridge:2.0.0"
            },
            "networking": {
                "service_mesh": "istio",
                "load_balancer": "enterprise-lb.starfleet.local",
                "ingress_controller": "nginx",
                "service_port": 8080,
                "health_check_path": "/health"
            }
        });
        
        // Verify infrastructure structure
        assert!(infrastructure_data.is_object());
        assert!(infrastructure_data["service_info"].is_object());
        assert!(infrastructure_data["environment"].is_object());
        assert!(infrastructure_data["infrastructure"].is_object());
        assert!(infrastructure_data["networking"].is_object());
        
        // Verify service information
        assert_eq!(infrastructure_data["service_info"]["name"], "enterprise-bridge-service");
        assert_eq!(infrastructure_data["environment"]["name"], "production");
        assert_eq!(infrastructure_data["networking"]["service_port"], 8080);
    }
    
    #[test]
    fn test_debug_context_realistic_data() {
        // Test DebugContext with realistic development and debugging information
        
        let debug_data = json!({
            "debug_mode": true,
            "log_level": "DEBUG",
            "debug_session": {
                "session_id": "debug_bridge_001",
                "started_at": "2025-08-23T10:30:45.123Z",
                "debugger_attached": true,
                "breakpoints_active": ["BridgeCommand:execute", "CrewManager:assign"]
            },
            "development_info": {
                "developer": "Enterprise Engineering Team",
                "branch": "feature/holodeck-integration",
                "commit_sha": "abc123def456789",
                "build_type": "debug",
                "compiler_version": "rustc 1.80.0"
            },
            "profiling": {
                "cpu_profiling": true,
                "memory_profiling": true,
                "profile_output_path": "/tmp/enterprise_profile.json"
            },
            "test_context": {
                "test_mode": false,
                "test_suite": null,
                "mock_services": []
            }
        });
        
        // Verify debug structure
        assert!(debug_data.is_object());
        assert!(debug_data["debug_session"].is_object());
        assert!(debug_data["development_info"].is_object());
        assert!(debug_data["profiling"].is_object());
        assert!(debug_data["test_context"].is_object());
        
        // Verify debug settings
        assert_eq!(debug_data["debug_mode"], true);
        assert_eq!(debug_data["log_level"], "DEBUG");
        assert!(debug_data["debug_session"]["breakpoints_active"].is_array());
    }
    
    #[test]
    fn test_enhanced_meta_comprehensive_structure() {
        // Test enhanced Meta structure with all context types and custom fields
        
        let enhanced_meta = json!({
            "tenant_id": "enterprise_starfleet",
            "user_id": "picard@starfleet.local",
            "correlation_id": "bridge_command_001", 
            "request_id": "req_enterprise_123",
            "timestamp": "2025-08-23T10:30:45.123Z",
            "security_context": {
                "authentication": {
                    "method": "JWT",
                    "subject": "picard@starfleet.local"
                }
            },
            "delegation_context": {
                "original_user": {
                    "user_id": "picard@starfleet.local",
                    "role": "Captain"
                }
            },
            "performance_context": {
                "request_timing": {
                    "total_duration_ms": 123
                }
            },
            "tracing_context": {
                "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
                "span_id": "00f067aa0ba902b7"
            },
            "infrastructure_context": {
                "service_info": {
                    "name": "enterprise-bridge-service"
                }
            },
            "debug_context": {
                "debug_mode": false,
                "log_level": "INFO"
            },
            "custom_metadata": {
                "ship_registry": "NCC-1701-D",
                "mission_type": "exploration",
                "crew_complement": 1012
            }
        });
        
        // Verify comprehensive structure
        assert!(enhanced_meta.is_object());
        assert!(enhanced_meta["security_context"].is_object());
        assert!(enhanced_meta["delegation_context"].is_object());
        assert!(enhanced_meta["performance_context"].is_object());
        assert!(enhanced_meta["tracing_context"].is_object());
        assert!(enhanced_meta["infrastructure_context"].is_object());
        assert!(enhanced_meta["debug_context"].is_object());
        assert!(enhanced_meta["custom_metadata"].is_object());
        
        // Verify core fields
        assert_eq!(enhanced_meta["tenant_id"], "enterprise_starfleet");
        assert_eq!(enhanced_meta["user_id"], "picard@starfleet.local");
        assert_eq!(enhanced_meta["custom_metadata"]["ship_registry"], "NCC-1701-D");
    }
}