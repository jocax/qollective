//! NATS NKey authentication helper module
//!
//! Provides functions for connecting to NATS with NKey authentication and TLS

use crate::{Result, TaleTrailError};
use async_nats::ConnectOptions;
use nkeys::KeyPair;
use std::fs;

/// Load NKey seed from a file as String
///
/// # Arguments
/// * `path` - Path to the NKey seed file (usually `.nk` file)
///
/// # Returns
/// * `Result<String>` - The loaded seed string
pub fn load_nkey_seed_from_file(path: &str) -> Result<String> {
    let seed = fs::read_to_string(path)
        .map_err(|e| TaleTrailError::ConfigError(format!("Failed to read NKey from {}: {}", path, e)))?
        .trim()
        .to_string();

    // Validate the seed by parsing it
    KeyPair::from_seed(&seed)
        .map_err(|e| TaleTrailError::ConfigError(format!("Invalid NKey in {}: {}", path, e)))?;

    Ok(seed)
}

/// Load NKey from a file as KeyPair
///
/// # Arguments
/// * `path` - Path to the NKey seed file (usually `.nk` file)
///
/// # Returns
/// * `Result<KeyPair>` - The loaded key pair
pub fn load_nkey_from_file(path: &str) -> Result<KeyPair> {
    let seed = load_nkey_seed_from_file(path)?;
    KeyPair::from_seed(&seed)
        .map_err(|e| TaleTrailError::ConfigError(format!("Invalid NKey in {}: {}", path, e)))
}

/// Connect to NATS with NKey authentication and TLS
///
/// # Arguments
/// * `url` - NATS server URL (e.g., "nats://localhost:5222")
/// * `nkey_path` - Path to the NKey seed file
/// * `ca_cert_path` - Path to the CA certificate for TLS verification
///
/// # Returns
/// * `Result<async_nats::Client>` - Connected NATS client
///
/// # Example
/// ```no_run
/// use shared_types::connect_with_nkey;
///
/// #[tokio::main]
/// async fn main() {
///     let client = connect_with_nkey(
///         "nats://localhost:5222",
///         "./nkeys/story-generator.nk",
///         "./certs/ca.pem"
///     ).await.expect("Failed to connect to NATS");
///
///     println!("Connected to NATS with NKey authentication");
/// }
/// ```
pub async fn connect_with_nkey(
    url: &str,
    nkey_path: &str,
    ca_cert_path: &str,
) -> Result<async_nats::Client> {
    // Load NKey seed
    let seed = load_nkey_seed_from_file(nkey_path)?;

    // Connect to NATS with NKey authentication and TLS
    let client = ConnectOptions::with_nkey(seed)
        .require_tls(true)
        .add_root_certificates(ca_cert_path.into())
        .connect(url)
        .await
        .map_err(|e| TaleTrailError::NetworkError(format!("NATS connection failed to {}: {}", url, e)))?;

    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nkey_invalid_path() {
        let result = load_nkey_from_file("/nonexistent/path/key.nk");
        assert!(result.is_err());
        assert!(matches!(result, Err(TaleTrailError::ConfigError(_))));
    }
}
