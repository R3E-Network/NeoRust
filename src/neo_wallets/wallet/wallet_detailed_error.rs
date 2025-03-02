use p256::ecdsa;
use thiserror::Error;

/// Detailed errors that may occur within wallet operations.
/// This is used when the wallet feature is enabled.
#[derive(Error, Debug)]
pub enum WalletDetailedError {
    /// Error indicating an issue with the account's state
    #[error("Account state error: {0}")]
    AccountState(String),

    /// Indicates that no key pair is available
    #[error("No key pair")]
    NoKeyPair,

    /// Wraps errors from the `ecdsa` crate
    #[error("ECDSA error")]
    EcdsaError(ecdsa::Error),

    /// Represents errors encountered during hex encoding or decoding
    #[error(transparent)]
    HexError(hex::FromHexError),

    /// Encapsulates errors arising from IO operations
    #[error("IO error")]
    IoError(std::io::Error),

    /// No default account
    #[error("No default account")]
    NoDefaultAccount,

    /// Invalid key pair
    #[error("Invalid key pair")]
    SignHashError,

    /// Indicates an invalid signature
    #[error("Invalid signature")]
    VerifyError,

    /// Errors related to Ledger hardware wallet operations
    #[error("Ledger error: {0}")]
    LedgerError(String),

    /// Error indicating no accounts in wallet
    #[error("No accounts in wallet")]
    NoAccounts,

    /// Errors related to YubiHSM operations
    #[error("YubiHSM error: {0}")]
    YubiHsmError(String),

    /// Errors during account decryption
    #[error("Decryption error: {0}")]
    DecryptionError(String),

    /// Errors during transaction signing
    #[error("Signing error: {0}")]
    SigningError(String),

    /// Errors during file operations
    #[error("File error: {0}")]
    FileError(String),

    /// Errors during parsing operations
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Errors during key import operations
    #[error("Import error: {0}")]
    ImportError(String),

    /// Invalid password provided
    #[error("Invalid password")]
    InvalidPassword,

    /// Errors during deserialization
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

// Manual implementation of Clone to handle non-Clone types like std::io::Error
impl Clone for WalletDetailedError {
    fn clone(&self) -> Self {
        match self {
            Self::AccountState(s) => Self::AccountState(s.clone()),
            Self::NoKeyPair => Self::NoKeyPair,
            Self::EcdsaError(_) => Self::EcdsaError(ecdsa::Error::new()),
            Self::HexError(e) => Self::HexError(e.clone()),
            Self::IoError(_) => Self::IoError(std::io::Error::new(std::io::ErrorKind::Other, "IO Error cloned")),
            Self::NoDefaultAccount => Self::NoDefaultAccount,
            Self::SignHashError => Self::SignHashError,
            Self::VerifyError => Self::VerifyError,
            Self::LedgerError(s) => Self::LedgerError(s.clone()),
            Self::NoAccounts => Self::NoAccounts,
            Self::YubiHsmError(s) => Self::YubiHsmError(s.clone()),
            Self::DecryptionError(s) => Self::DecryptionError(s.clone()),
            Self::SigningError(s) => Self::SigningError(s.clone()),
            Self::FileError(s) => Self::FileError(s.clone()),
            Self::ParseError(s) => Self::ParseError(s.clone()),
            Self::ImportError(s) => Self::ImportError(s.clone()),
            Self::InvalidPassword => Self::InvalidPassword,
            Self::DeserializationError(s) => Self::DeserializationError(s.clone()),
        }
    }
}

impl From<String> for WalletDetailedError {
    fn from(s: String) -> Self {
        WalletDetailedError::AccountState(s)
    }
}

impl From<&str> for WalletDetailedError {
    fn from(s: &str) -> Self {
        WalletDetailedError::AccountState(s.to_string())
    }
}

impl From<ecdsa::Error> for WalletDetailedError {
    fn from(e: ecdsa::Error) -> Self {
        WalletDetailedError::EcdsaError(e)
    }
}

impl From<hex::FromHexError> for WalletDetailedError {
    fn from(e: hex::FromHexError) -> Self {
        WalletDetailedError::HexError(e)
    }
}

impl From<std::io::Error> for WalletDetailedError {
    fn from(e: std::io::Error) -> Self {
        WalletDetailedError::IoError(e)
    }
} 