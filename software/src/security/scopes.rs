// ABOUTME: Token scope validation functionality for service-level authorization
// ABOUTME: Provides scope checking and validation to ensure tokens have required permissions

use crate::security::jwt::Token;

/// Scope validation errors
#[derive(Debug, thiserror::Error)]
pub enum ScopeValidationError {
    #[error("Insufficient scopes. Missing: {missing:?}")]
    InsufficientScopes { missing: Vec<String> },
    #[error("Invalid scope format: {0}")]
    InvalidScopeFormat(String),
}

/// Trait for token scope validation implementations
pub trait TokenScopeValidator: Send + Sync {
    /// Validate that a token has the required scopes
    fn validate(
        &self,
        token: &Token,
        required_scopes: &[String],
    ) -> Result<(), ScopeValidationError>;
}

/// Default Token Scope Validator
pub struct DefaultTokenScopeValidator {
    // Simple implementation
}

impl DefaultTokenScopeValidator {
    pub fn new() -> Self {
        Self {}
    }
}

impl TokenScopeValidator for DefaultTokenScopeValidator {
    fn validate(
        &self,
        token: &Token,
        required_scopes: &[String],
    ) -> Result<(), ScopeValidationError> {
        let token_scopes = token.scopes();
        let mut missing_scopes = Vec::new();

        for required_scope in required_scopes {
            if !token_scopes.contains(required_scope) {
                missing_scopes.push(required_scope.clone());
            }
        }

        if missing_scopes.is_empty() {
            Ok(())
        } else {
            Err(ScopeValidationError::InsufficientScopes {
                missing: missing_scopes,
            })
        }
    }
}

impl Default for DefaultTokenScopeValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Role-based Scope Validator (enterprise use case)
pub struct RoleBasedScopeValidator {
    role_hierarchy: std::collections::HashMap<String, Vec<String>>,
}

impl RoleBasedScopeValidator {
    pub fn new() -> Self {
        let mut role_hierarchy = std::collections::HashMap::new();

        // Define role hierarchy: admin > manager > user
        role_hierarchy.insert(
            "admin".to_string(),
            vec![
                "admin:all".to_string(),
                "manager:all".to_string(),
                "user:all".to_string(),
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
            ],
        );

        role_hierarchy.insert(
            "manager".to_string(),
            vec![
                "manager:all".to_string(),
                "user:all".to_string(),
                "read".to_string(),
                "write".to_string(),
            ],
        );

        role_hierarchy.insert(
            "user".to_string(),
            vec!["user:all".to_string(), "read".to_string()],
        );

        Self { role_hierarchy }
    }
}

impl TokenScopeValidator for RoleBasedScopeValidator {
    fn validate(
        &self,
        token: &Token,
        required_scopes: &[String],
    ) -> Result<(), ScopeValidationError> {
        let token_scopes = token.scopes();
        let mut effective_scopes = token_scopes.to_vec();

        // Expand role-based scopes
        for scope in token_scopes {
            if let Some(role_scopes) = self.role_hierarchy.get(scope) {
                effective_scopes.extend(role_scopes.clone());
            }
        }

        let mut missing_scopes = Vec::new();
        for required_scope in required_scopes {
            if !effective_scopes.contains(required_scope) {
                missing_scopes.push(required_scope.clone());
            }
        }

        if missing_scopes.is_empty() {
            Ok(())
        } else {
            Err(ScopeValidationError::InsufficientScopes {
                missing: missing_scopes,
            })
        }
    }
}
