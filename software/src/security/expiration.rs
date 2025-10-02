// ABOUTME: Token expiration handling functionality for lifecycle management
// ABOUTME: Provides expiration detection and refresh scheduling for token security

use crate::security::jwt::Token;
use std::time::SystemTime;

/// Token Expiration Checker
pub struct TokenExpirationChecker {
    // For now, we'll keep this simple for the RED->GREEN phase
}

impl TokenExpirationChecker {
    pub fn new() -> Self {
        Self {}
    }

    /// Check if a token is expired
    pub fn is_expired(&self, token: &Token) -> bool {
        token.is_expired()
    }

    /// Check if a token is near expiration (within 5 minutes)
    pub fn is_near_expiration(&self, token: &Token) -> bool {
        let five_minutes = std::time::Duration::from_secs(300);
        match token.expires_at().duration_since(SystemTime::now()) {
            Ok(remaining) => remaining <= five_minutes,
            Err(_) => true, // Already expired
        }
    }
}

impl Default for TokenExpirationChecker {
    fn default() -> Self {
        Self::new()
    }
}
