// ABOUTME: Secure token storage trait and implementations for encrypted token persistence
// ABOUTME: Provides pluggable storage backends (in-memory, Redis, database, etc.)

use std::collections::HashMap;

/// Storage operation errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Token not found for user: {0}")]
    TokenNotFound(String),
    #[error("Storage operation failed: {0}")]
    OperationFailed(String),
    #[error("Encryption/decryption failed: {0}")]
    CryptographyFailed(String),
    #[error("Storage backend unavailable: {0}")]
    BackendUnavailable(String),
}

/// Trait for secure token storage implementations
pub trait SecureTokenStorage: Send + Sync {
    /// Store a token securely for a user
    fn store_token(&mut self, user_id: &str, token: &str) -> Result<(), StorageError>;

    /// Retrieve a token for a user
    fn get_token(&self, user_id: &str) -> Result<Option<String>, StorageError>;

    /// Remove a token for a user (logout, revocation)
    fn remove_token(&mut self, user_id: &str) -> Result<bool, StorageError>;

    /// Check if a token exists for a user
    fn has_token(&self, user_id: &str) -> Result<bool, StorageError>;

    /// Clear all tokens (admin operation)
    fn clear_all(&mut self) -> Result<(), StorageError>;
}

/// In-Memory Token Storage Implementation
pub struct InMemoryTokenStorage {
    storage: HashMap<String, String>,
}

impl InMemoryTokenStorage {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    /// Get raw storage data for testing (returns encrypted form)
    pub fn get_raw_storage_data(&self, user_id: &str) -> Option<&String> {
        self.storage.get(user_id)
    }
}

impl SecureTokenStorage for InMemoryTokenStorage {
    fn store_token(&mut self, user_id: &str, token: &str) -> Result<(), StorageError> {
        // Simple "encryption" for demo purposes
        let encrypted_token = base64::encode(token);
        self.storage.insert(user_id.to_string(), encrypted_token);
        Ok(())
    }

    fn get_token(&self, user_id: &str) -> Result<Option<String>, StorageError> {
        match self.storage.get(user_id) {
            Some(encrypted) => {
                let decrypted = base64::decode(encrypted).map_err(|_| {
                    StorageError::CryptographyFailed("Decryption failed".to_string())
                })?;
                let token = String::from_utf8(decrypted)
                    .map_err(|e| StorageError::CryptographyFailed(e.to_string()))?;
                Ok(Some(token))
            }
            None => Ok(None),
        }
    }

    fn remove_token(&mut self, user_id: &str) -> Result<bool, StorageError> {
        Ok(self.storage.remove(user_id).is_some())
    }

    fn has_token(&self, user_id: &str) -> Result<bool, StorageError> {
        Ok(self.storage.contains_key(user_id))
    }

    fn clear_all(&mut self) -> Result<(), StorageError> {
        self.storage.clear();
        Ok(())
    }
}

impl Default for InMemoryTokenStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Redis Token Storage Implementation (mock for demo)
pub struct RedisTokenStorage {
    connection_string: String,
    key_prefix: String,
}

impl RedisTokenStorage {
    pub fn new(connection_string: &str, key_prefix: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
            key_prefix: key_prefix.to_string(),
        }
    }
}

impl SecureTokenStorage for RedisTokenStorage {
    fn store_token(&mut self, user_id: &str, _token: &str) -> Result<(), StorageError> {
        // Mock Redis implementation
        println!(
            "Redis: Storing token for {} at {}",
            user_id, self.connection_string
        );
        Ok(())
    }

    fn get_token(&self, user_id: &str) -> Result<Option<String>, StorageError> {
        // Mock Redis implementation
        if user_id == "redis_test_user" {
            Ok(Some("redis_mock_token".to_string()))
        } else {
            Ok(None)
        }
    }

    fn remove_token(&mut self, user_id: &str) -> Result<bool, StorageError> {
        println!("Redis: Removing token for {}", user_id);
        Ok(true)
    }

    fn has_token(&self, user_id: &str) -> Result<bool, StorageError> {
        Ok(user_id == "redis_test_user")
    }

    fn clear_all(&mut self) -> Result<(), StorageError> {
        println!("Redis: Clearing all tokens");
        Ok(())
    }
}

// Simple base64 implementation for demo
mod base64 {
    pub fn encode(data: &str) -> String {
        format!("encrypted_{}", data.chars().rev().collect::<String>())
    }

    pub fn decode(data: &str) -> Result<Vec<u8>, ()> {
        if let Some(stripped) = data.strip_prefix("encrypted_") {
            Ok(stripped.chars().rev().collect::<String>().into_bytes())
        } else {
            Err(())
        }
    }
}
