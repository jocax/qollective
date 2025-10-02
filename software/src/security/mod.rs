// ABOUTME: Main security module providing token propagation and validation functionality
// ABOUTME: Includes JWT validation, OAuth integration, token scopes, and secure storage

//! Security Module
//!
//! This module provides comprehensive token propagation security functionality including:
//! - JWT token validation and refresh mechanisms
//! - OAuth 2.0 and OIDC integration support
//! - Token scope validation per service
//! - Secure token storage and transmission
//! - Token expiration and refresh handling
//! - Audit logging for security events
//! - Identity provider integration
//! - Cross-language security consistency

pub mod audit;
pub mod config;
pub mod expiration;
pub mod jwt;
pub mod oauth;
pub mod scopes;
pub mod storage;
pub mod transmission;

// Re-export commonly used security types
pub use audit::{
    FileSecurityAuditLogger, InMemorySecurityAuditLogger, SecurityAuditError, SecurityAuditEvent,
    SecurityAuditLogger, SecurityEventResult, SecurityEventSeverity, SecurityEventType,
};
pub use config::{
    AuditConfig, ExpirationConfig, JwtValidationConfig, ScopeValidationConfig, SecurityConfig,
    SecurityConfigBuilder, StorageConfig, TransmissionConfig,
};
pub use expiration::TokenExpirationChecker;
pub use jwt::{
    DefaultJwtValidator, JwtTokenRefresher, JwtValidator, SimpleJwtValidator, Token,
    TokenValidationError, ValidatedToken,
};
pub use oauth::{OAuth2Config, OAuth2Validator, OidcConfig, OidcValidator};
pub use scopes::{
    DefaultTokenScopeValidator, RoleBasedScopeValidator, ScopeValidationError, TokenScopeValidator,
};
pub use storage::{InMemoryTokenStorage, RedisTokenStorage, SecureTokenStorage, StorageError};
pub use transmission::SecureTokenTransmitter;
