// ABOUTME: Global crypto provider initialization for TLS operations across the framework
// ABOUTME: Provides safe, once-only initialization to avoid conflicts with other libraries

//! Crypto provider initialization for Qollective framework.
//!
//! This module provides global crypto provider initialization that ensures all TLS operations
//! across the framework (NATS, gRPC, REST, A2A) use a consistent crypto provider without conflicts.
//!
//! # Quick Start
//!
//! Call [`ensure_crypto_provider()`] early in your application:
//!
//! ```rust
//! use qollective::ensure_crypto_provider;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize crypto provider once for all TLS operations
//!     ensure_crypto_provider()?;
//!     
//!     // Now create NATS clients, gRPC clients, etc.
//!     Ok(())
//! }
//! ```
//!
//! # Conflict Avoidance
//!
//! This module is designed to work safely when your application uses other libraries
//! that also need crypto providers. The default strategy is [`CryptoProviderStrategy::AutoInstall`]
//! which installs a provider only if none exists, ignoring errors if one is already installed.
//!
//! # Advanced Usage
//!
//! For more control over crypto provider installation:
//!
//! ```rust
//! use qollective::crypto::{init_with_strategy, CryptoProviderStrategy};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Skip installation entirely (your app handles it)
//! init_with_strategy(CryptoProviderStrategy::Skip)?;
//!
//! // Require no existing provider (strict mode)
//! init_with_strategy(CryptoProviderStrategy::RequireEmpty)?;
//! # Ok(())
//! # }
//! ```

use crate::error::{QollectiveError, Result};
use std::sync::Once;

/// Crypto provider installation strategy for handling conflicts with other libraries
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CryptoProviderStrategy {
    /// Auto-install if none exists, ignore if already installed (default, safest)
    AutoInstall,
    /// Skip installation entirely (application handles crypto provider)
    Skip,
    /// Return error if provider already exists (strict mode)
    RequireEmpty,
}

impl Default for CryptoProviderStrategy {
    fn default() -> Self {
        Self::AutoInstall
    }
}

/// Global crypto provider initialization state
static CRYPTO_INIT: Once = Once::new();

/// Ensures that a crypto provider is installed for rustls TLS operations.
///
/// This function is safe to call multiple times across the entire process.
/// It uses [`std::sync::Once`] to ensure the crypto provider is only installed
/// once, avoiding conflicts between multiple clients or libraries.
///
/// Uses [`CryptoProviderStrategy::AutoInstall`] by default, which is the safest
/// option when your application may use other libraries that also need crypto providers.
///
/// # Examples
///
/// ```rust
/// use qollective::ensure_crypto_provider;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Call early in your application
/// ensure_crypto_provider()?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`QollectiveError`] if crypto provider installation fails and the
/// failure cannot be safely ignored based on the strategy.
pub fn ensure_crypto_provider() -> Result<()> {
    init_with_strategy(CryptoProviderStrategy::default())
}

/// Initialize crypto provider with a specific strategy for handling conflicts.
///
/// This provides more control over how crypto provider installation is handled,
/// particularly when your application uses other libraries that may also install
/// crypto providers.
///
/// # Examples
///
/// ```rust
/// use qollective::crypto::{init_with_strategy, CryptoProviderStrategy};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Skip installation (your app handles it)
/// init_with_strategy(CryptoProviderStrategy::Skip)?;
///
/// // Strict mode (error if provider already exists)
/// init_with_strategy(CryptoProviderStrategy::RequireEmpty)?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`QollectiveError`] based on the strategy:
/// - [`CryptoProviderStrategy::Skip`]: Never returns an error
/// - [`CryptoProviderStrategy::AutoInstall`]: Only errors on unexpected failures
/// - [`CryptoProviderStrategy::RequireEmpty`]: Errors if a provider is already installed
pub fn init_with_strategy(strategy: CryptoProviderStrategy) -> Result<()> {
    match strategy {
        CryptoProviderStrategy::Skip => {
            #[cfg(feature = "tracing")]
            tracing::debug!("Skipping crypto provider initialization (application managed)");
            Ok(())
        }
        CryptoProviderStrategy::AutoInstall => {
            let result = Ok(());

            CRYPTO_INIT.call_once(|| {
                #[cfg(feature = "tls")]
                match rustls::crypto::aws_lc_rs::default_provider().install_default() {
                    Ok(_) => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Crypto provider installed successfully");
                    }
                    Err(_existing_provider) => {
                        // Another library already set a provider - this is fine for AutoInstall
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Crypto provider already installed by another library");
                    }
                }
            });

            result
        }
        CryptoProviderStrategy::RequireEmpty => {
            let mut result = Ok(());

            CRYPTO_INIT.call_once(|| {
                #[cfg(feature = "tls")]
                match rustls::crypto::aws_lc_rs::default_provider().install_default() {
                    Ok(_) => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Crypto provider installed successfully (strict mode)");
                    }
                    Err(_existing_provider) => {
                        // In strict mode, this is an error
                        #[cfg(feature = "tracing")]
                        tracing::error!(
                            "Crypto provider already installed (strict mode violation)"
                        );
                        result = Err(QollectiveError::internal(
                            "Crypto provider already installed (RequireEmpty strategy violated)"
                                .to_string(),
                        ));
                    }
                }
            });

            result
        }
    }
}

/// Check if crypto provider initialization has been attempted.
///
/// Note: This only indicates whether [`ensure_crypto_provider()`] or [`init_with_strategy()`]
/// has been called, not whether a crypto provider is actually installed (which might have
/// been done by another library).
///
/// # Examples
///
/// ```rust
/// use qollective::crypto::{ensure_crypto_provider, is_initialized};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// assert!(!is_initialized());
/// ensure_crypto_provider()?;
/// assert!(is_initialized());
/// # Ok(())
/// # }
/// ```
pub fn is_initialized() -> bool {
    CRYPTO_INIT.is_completed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_provider_strategy_default() {
        assert_eq!(
            CryptoProviderStrategy::default(),
            CryptoProviderStrategy::AutoInstall
        );
    }

    #[test]
    fn test_skip_strategy() {
        // Skip strategy should always succeed
        let result = init_with_strategy(CryptoProviderStrategy::Skip);
        assert!(result.is_ok());
    }

    #[test]
    fn test_auto_install_strategy() {
        // Auto install should succeed (either installs or ignores if already installed)
        let result = init_with_strategy(CryptoProviderStrategy::AutoInstall);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensure_crypto_provider() {
        // Should use default strategy (AutoInstall) and succeed
        let result = ensure_crypto_provider();
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialization_tracking() {
        // Note: This test may show true if other tests already initialized
        // The exact value depends on test execution order
        let _is_init = is_initialized();

        // After calling ensure_crypto_provider, it should be initialized
        let _ = ensure_crypto_provider();
        assert!(is_initialized());
    }
}
