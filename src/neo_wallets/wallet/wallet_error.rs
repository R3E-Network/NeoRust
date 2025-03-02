use p256::ecdsa;
use thiserror::Error;

use crate::neo_error::{BuilderError, ProviderError};
#[cfg(feature = "crypto-standard")]
use crate::neo_crypto::error::CryptoError;
#[cfg(feature = "transaction")]
use crate::neo_error::TransactionError;

use super::wallet_detailed_error::WalletDetailedError;

/// Errors that may occur within wallet operations.
///
/// This enum encompasses a range of errors that can arise when interacting with
/// cryptocurrency wallets, including but not limited to account state issues, cryptographic
/// errors, and IO operations. It is designed to be comprehensive, covering errors
/// from underlying libraries (such as ECDSA operations, hex encoding/decoding) as well
/// as wallet-specific errors like missing key pairs or default accounts.
///
/// # Variants
///
/// - `AccountState`: Represents errors related to an account's state, such as when an account
///   cannot be found or is in an invalid state for the requested operation.
/// - `NoKeyPair`: Indicates that a key pair was expected but none was found.
/// - `EcdsaError`: Wraps errors from the `ecdsa` crate, typically related to signature operations.
/// - `HexError`: Wraps errors from the `hex` crate, often arising during hex encoding or decoding.
/// - `IoError`: Wraps standard IO errors that might occur during file operations.
/// - `NoDefaultAccount`: Signals that a default account was expected but not set.
/// - `InvalidKeyPair`: General error for when a key pair is invalid or fails validation.
/// - `CryptoError`: Wraps cryptographic errors, potentially from operations like hashing or encryption.
/// - `TransactionError`: Encapsulates errors that may occur during transaction creation or processing.
/// - `BuilderError`: Wraps errors that occur during the construction of complex objects, possibly due to invalid parameters.
/// - `DecryptionError`: Indicates an error occurred during account decryption.
/// - `SigningError`: Indicates an error occurred during transaction signing.
/// - `FileError`: Indicates an error occurred during file operations.
/// - `ParseError`: Indicates an error occurred during parsing operations.
/// - `ImportError`: Indicates an error occurred during key import operations.
/// - `InvalidPassword`: Indicates that an invalid password was provided.
/// - `NoAccounts`: Indicates that no accounts were found in the wallet.
/// - `YubiHsmError`: Wraps errors related to YubiHSM operations.
/// - `ProviderError`: Wraps errors from the RPC provider.
///
/// # Examples
///
/// Handling a `WalletError` might look like this:
///
/// ```
/// # use NeoRust::prelude::WalletError;
/// # fn main() -> Result<(), WalletError> {
/// let result = some_wallet_operation();
///     match result {
///         Ok(_) => println!("Operation successful"),
///         Err(WalletError::NoKeyPair) => println!("Key pair missing"),
///         Err(e) => println!("An error occurred: {:?}", e),
///     }
/// #    Ok(())
/// # }
/// # fn some_wallet_operation() -> Result<(), WalletError> {
/// #    Err(WalletError::NoKeyPair)
/// # }
/// ```
///
/// This approach allows for precise error handling and reporting, facilitating debugging and user feedback.
#[derive(Error, Debug, Clone)]
pub enum WalletError {
	#[error(transparent)]
	Detailed(#[from] WalletDetailedError),
	
	/// Wraps cryptographic errors
	#[cfg(feature = "crypto-standard")]
	#[error("Crypto error: {0}")]
	CryptoError(String),

	/// Covers errors related to transaction operations
	#[cfg(feature = "transaction")]
	#[error("Transaction error: {0}")]
	TransactionError(String),

	/// Indicates issues during construction
	#[error("Builder error: {0}")]
	BuilderError(String),
	
	/// Errors from the RPC provider
	#[error("Provider error: {0}")]
	ProviderError(String),
	
	/// General wallet error
	#[error("General wallet error: {0}")]
	General(String),
}

// Implement conversion from the top-level WalletError
impl From<crate::neo_error::WalletError> for WalletError {
	fn from(err: crate::neo_error::WalletError) -> Self {
		WalletError::General(format!("{}", err))
	}
}

impl From<String> for WalletError {
	fn from(s: String) -> Self {
		WalletError::Detailed(WalletDetailedError::AccountState(s))
	}
}

impl From<&str> for WalletError {
	fn from(s: &str) -> Self {
		WalletError::Detailed(WalletDetailedError::AccountState(s.to_string()))
	}
}

impl From<ecdsa::Error> for WalletError {
	fn from(err: ecdsa::Error) -> Self {
		WalletError::Detailed(WalletDetailedError::EcdsaError(err))
	}
}

impl From<hex::FromHexError> for WalletError {
	fn from(err: hex::FromHexError) -> Self {
		WalletError::Detailed(WalletDetailedError::HexError(err))
	}
}

impl From<std::io::Error> for WalletError {
	fn from(err: std::io::Error) -> Self {
		WalletError::Detailed(WalletDetailedError::IoError(err))
	}
}

// Add conversions for external error types
#[cfg(feature = "crypto-standard")]
impl From<CryptoError> for WalletError {
    fn from(err: CryptoError) -> Self {
        WalletError::CryptoError(err.to_string())
    }
}

#[cfg(feature = "transaction")]
impl From<TransactionError> for WalletError {
    fn from(err: TransactionError) -> Self {
        WalletError::TransactionError(err.to_string())
    }
}

impl From<ProviderError> for WalletError {
    fn from(err: ProviderError) -> Self {
        WalletError::ProviderError(err.to_string())
    }
}

impl From<BuilderError> for WalletError {
    fn from(err: BuilderError) -> Self {
        WalletError::BuilderError(format!("{:?}", err))
    }
}
