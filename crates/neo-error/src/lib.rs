//! # Neo Error
//!
//! Error types and handling for the NeoRust SDK.
//!
//! This crate provides error types and handling utilities for the Neo N3 blockchain SDK, including:
//!
//! - Common error types used across the SDK
//! - Error conversion traits
//! - Error handling utilities
//! - Integration with standard Rust error handling

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

use std::error::Error as StdError;
use std::fmt;
use thiserror::Error;

/// Errors that can occur when working with providers.
#[derive(Error, Debug)]
pub enum ProviderError {
    /// An error occurred with a JSON-RPC request.
    #[error("JSON-RPC error: {0}")]
    JsonRpcError(String),
    
    /// An error occurred with a WebSocket connection.
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    /// An error occurred with an HTTP request.
    #[error("HTTP error: {0}")]
    HttpError(String),
    
    /// An error occurred during serialization or deserialization.
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// An error occurred with a contract.
    #[error("Contract error: {0}")]
    ContractError(String),
    
    /// An error occurred with a transaction.
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    /// An error occurred with a network connection.
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// An error occurred with authentication.
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    /// An error occurred with a rate limit.
    #[error("Rate limit error: {0}")]
    RateLimitError(String),
    
    /// An error occurred with a subscription.
    #[error("Subscription error: {0}")]
    SubscriptionError(String),
    
    /// An error occurred with a timeout.
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    /// An error occurred with a middleware.
    #[error("Middleware error: {0}")]
    MiddlewareError(String),
    
    /// An unknown error occurred.
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

/// Errors that can occur during encoding and decoding operations.
#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CodecError {
    /// Invalid passphrase provided
    #[error("Invalid passphrase: {0}")]
    InvalidPassphrase(String),
    
    /// Invalid format encountered
    #[error("Invalid format")]
    InvalidFormat,
    
    /// Index out of bounds
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(String),
    
    /// Invalid encoding
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),
    
    /// Invalid operation code
    #[error("Invalid op code")]
    InvalidOpCode,
    
    /// Error converting from primitive
    #[error("Error converting from primitive: {0}")]
    TryFromPrimitiveError(String),
}

/// Custom error type for contract-related errors
#[derive(Error, Debug)]
pub enum ContractError {
    /// Error indicating an invalid Neo name
    #[error("Invalid NNS name {0}")]
    InvalidNeoName(String),
    
    /// Error indicating an invalid Neo Name Service root
    #[error("Invalid NNS root {0}")]
    InvalidNeoNameServiceRoot(String),
    
    /// Error indicating an unexpected return type
    #[error("Unexpected return type {0}")]
    UnexpectedReturnType(String),
    
    /// Error indicating an unresolvable domain name
    #[error("Unresolvable domain name {0}")]
    UnresolvableDomainName(String),
    
    /// Error indicating that a domain name is not available
    #[error("Domain name {0} is not available")]
    DomainNameNotAvailable(String),
    
    /// Error indicating that a domain name is not registered
    #[error("Domain name {0} is not registered")]
    DomainNameNotRegistered(String),
    
    /// Error indicating a runtime error
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    /// Error indicating an invalid state error
    #[error("Invalid state error: {0}")]
    InvalidStateError(String),
    
    /// Error indicating an invalid argument error
    #[error("Invalid argument error: {0}")]
    InvalidArgError(String),
    
    /// Error indicating a provider error, transparently wrapped
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    
    /// Error indicating that a provider is not set
    #[error("Provider not set: {0}")]
    ProviderNotSet(String),
    
    /// Error indicating that an invocation failed
    #[error("Invocation failed: {0}")]
    InvocationFailed(String),
    
    /// Error indicating an invalid response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    /// Error indicating an invalid account
    #[error("Invalid account: {0}")]
    InvalidAccount(String),
    
    /// Error indicating an invalid script hash
    #[error("Invalid script hash: {0}")]
    InvalidScriptHash(String),
}

/// Errors that may occur within wallet operations.
#[derive(Error, Debug)]
pub enum WalletError {
    /// Error indicating an issue with the account's state
    #[error("Account state error: {0}")]
    AccountState(String),

    /// Indicates that no key pair is available
    #[error("No key pair")]
    NoKeyPair,

    /// Wraps errors from ECDSA signature operations
    #[error("ECDSA error: {0}")]
    EcdsaError(String),

    /// Represents errors during hex encoding/decoding
    #[error("Hex error: {0}")]
    HexError(String),

    /// Encapsulates errors from IO operations
    #[error("IO error: {0}")]
    IoError(String),

    /// Signifies that the wallet does not have a default account
    #[error("No default account")]
    NoDefaultAccount,

    /// Used when a key pair is found to be invalid
    #[error("Invalid key pair")]
    SignHashError,

    /// Wraps generic cryptographic errors
    #[error("Crypto error: {0}")]
    CryptoError(String),

    /// Covers errors related to transactions
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Indicates issues during construction
    #[error("Builder error: {0}")]
    BuilderError(String),

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

    /// Errors from the RPC provider
    #[error(transparent)]
    ProviderError(#[from] ProviderError),

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

/// Represents errors that can occur within the signing process.
#[derive(Error, Debug)]
pub enum SignerError {
    /// Represents an error when an invalid passphrase is provided
    #[error("Invalid passphrase: {0}")]
    InvalidPassphrase(String),

    /// Indicates that the provided address is not valid
    #[error("Invalid address")]
    InvalidAddress,

    /// Wraps errors related to building or configuring objects
    #[error("Builder error: {0}")]
    BuilderError(String),

    /// Encapsulates errors that originate from wallet operations
    #[error(transparent)]
    WalletError(#[from] WalletError),

    /// Represents errors during hex encoding/decoding
    #[error("Hex error: {0}")]
    HexError(String),

    /// Covers general cryptographic errors
    #[error("Crypto error: {0}")]
    CryptoError(String),

    /// Error during hex decoding
    #[error("Hex decoding error: {0}")]
    RustcFromHexError(String),

    /// Indicates a failure related to type conversion
    #[error("Type error: {0}")]
    TypeError(String),
}

/// Errors that can occur during transaction operations.
#[derive(Error, Debug, PartialEq, Clone)]
pub enum TransactionError {
    /// Script format error
    #[error("Script format error: {0}")]
    ScriptFormat(String),
    
    /// Signer configuration error
    #[error("Signer configuration error: {0}")]
    SignerConfiguration(String),
    
    /// Invalid nonce
    #[error("Invalid nonce")]
    InvalidNonce,
    
    /// Invalid block
    #[error("Invalid block")]
    InvalidBlock,
    
    /// Invalid transaction
    #[error("Invalid transaction")]
    InvalidTransaction,
    
    /// Invalid witness condition
    #[error("Invalid witness condition")]
    InvalidWitnessCondition,
    
    /// Too many signers
    #[error("Too many signers")]
    TooManySigners,
    
    /// Duplicate signer
    #[error("Duplicate signer")]
    DuplicateSigner,
    
    /// No signers
    #[error("No signers")]
    NoSigners,
    
    /// No script
    #[error("No script")]
    NoScript,
    
    /// Empty script
    #[error("Empty script")]
    EmptyScript,
    
    /// Invalid sender
    #[error("Invalid sender")]
    InvalidSender,
    
    /// Invalid state
    #[error("Invalid state:{0}")]
    IllegalState(String),
    
    /// Transaction too large
    #[error("Transaction too large")]
    TxTooLarge,
    
    /// Transaction configuration error
    #[error("Transaction configuration error: {0}")]
    TransactionConfiguration(String),
    
    /// Codec error
    #[error("Codec error: {0}")]
    CodecError(String),
}

/// Errors that can occur during builder operations.
#[derive(Error, Debug, PartialEq, Clone)]
pub enum BuilderError {
    /// Invalid script
    #[error("Invalid script: {0}")]
    InvalidScript(String),
    
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    /// Missing required parameter
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    
    /// Unsupported feature
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    
    /// Builder state error
    #[error("Builder state error: {0}")]
    BuilderState(String),
    
    /// Codec error
    #[error("Codec error: {0}")]
    CodecError(String),
}

/// Errors that can occur during NeoFS operations.
#[derive(Error, Debug)]
pub enum NeoFSError {
    /// Container error
    #[error("Container error: {0}")]
    ContainerError(String),
    
    /// Object error
    #[error("Object error: {0}")]
    ObjectError(String),
    
    /// ACL error
    #[error("ACL error: {0}")]
    AclError(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    /// Permission error
    #[error("Permission error: {0}")]
    PermissionError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Provider error
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
}

/// Errors that can occur during protocol operations.
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// Invalid message
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    
    /// Invalid format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    /// Unsupported protocol version
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Errors that can occur during type operations.
#[derive(Error, Debug)]
pub enum TypeError {
    /// Invalid type
    #[error("Invalid type: {0}")]
    InvalidType(String),
    
    /// Type conversion error
    #[error("Type conversion error: {0}")]
    ConversionError(String),
    
    /// Invalid format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    /// Invalid value
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    
    /// Overflow error
    #[error("Overflow error: {0}")]
    OverflowError(String),
    
    /// Underflow error
    #[error("Underflow error: {0}")]
    UnderflowError(String),
    
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Invalid address
    #[error("Invalid address")]
    InvalidAddress,
    
    /// Invalid encoding
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),
    
    /// Invalid data
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgError(String),
    
    /// Unexpected return type
    #[error("Unexpected return type: {0}")]
    UnexpectedReturnType(String),
}

/// Errors that can occur during cryptographic operations.
#[derive(Error, Debug)]
pub enum CryptoError {
    /// Invalid key
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    /// Invalid hash
    #[error("Invalid hash: {0}")]
    InvalidHash(String),
    
    /// Invalid algorithm
    #[error("Invalid algorithm: {0}")]
    InvalidAlgorithm(String),
    
    /// Encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    /// Decryption error
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    
    /// P256 error
    #[error("P256 error: {0}")]
    P256Error(String),
}

/// Errors that can occur during NEP-2 operations.
#[derive(Error, Debug)]
pub enum Nep2Error {
    /// Invalid format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    /// Invalid passphrase
    #[error("Invalid passphrase: {0}")]
    InvalidPassphrase(String),
    
    /// Decryption error
    #[error("Decryption error: {0}")]
    DecryptionError(String),
}

/// Errors that can occur during signing operations.
#[derive(Error, Debug)]
pub enum SignError {
    /// Invalid key
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    /// Invalid message
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

/// The main error type for the Neo SDK.
///
/// This enum encompasses all possible errors that can occur within the Neo SDK.
/// It provides a unified error handling approach for applications using the SDK.
#[derive(Error, Debug)]
pub enum NeoError {
    /// Error related to JSON-RPC operations
    #[error("JSON-RPC error: {0}")]
    JsonRpcError(String),
    
    /// Error related to provider operations
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    
    /// Error related to wallet operations
    #[error(transparent)]
    WalletError(#[from] WalletError),
    
    /// Error related to contract operations
    #[error(transparent)]
    ContractError(#[from] ContractError),
    
    /// Error related to transaction operations
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    
    /// Error related to builder operations
    #[error(transparent)]
    BuilderError(#[from] BuilderError),
    
    /// Error related to cryptographic operations
    #[error(transparent)]
    CryptoError(#[from] CryptoError),
    
    /// Error related to type operations
    #[error(transparent)]
    TypeError(#[from] TypeError),
    
    /// Error related to codec operations
    #[error(transparent)]
    CodecError(#[from] CodecError),
    
    /// Error related to IO operations
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Error related to serialization operations
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Error related to deserialization operations
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    /// Error related to network operations
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Error related to timeout operations
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    /// Error related to authentication operations
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    /// Error related to permission operations
    #[error("Permission error: {0}")]
    PermissionError(String),
    
    /// Error related to configuration operations
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    /// Error related to validation operations
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Error related to illegal argument operations
    #[error("Illegal argument: {0}")]
    IllegalArgument(String),
    
    /// Error related to illegal state operations
    #[error("Illegal state: {0}")]
    IllegalState(String),
    
    /// Error related to unsupported operations
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    /// Error related to not implemented operations
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    /// Error related to not found operations
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Error related to already exists operations
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    /// Error related to invalid address operations
    #[error("Invalid address")]
    InvalidAddress,
    
    /// Error related to invalid script hash operations
    #[error("Invalid script hash: {0}")]
    InvalidScriptHash(String),
    
    /// Error related to invalid public key operations
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    
    /// Error related to invalid private key operations
    #[error("Invalid private key")]
    InvalidPrivateKey,
    
    /// Error related to invalid signature operations
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    /// Error related to invalid format operations
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    /// Error related to invalid value operations
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    
    /// Error related to invalid parameter operations
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    /// Error related to invalid response operations
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    /// Error related to invalid state operations
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    /// Error related to invalid operation operations
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Error related to invalid configuration operations
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    /// Error related to invalid token operations
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    /// Error related to invalid contract operations
    #[error("Invalid contract: {0}")]
    InvalidContract(String),
    
    /// Error related to invalid transaction operations
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    /// Error related to invalid block operations
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    
    /// Error related to invalid account operations
    #[error("Invalid account: {0}")]
    InvalidAccount(String),
    
    /// Error related to invalid wallet operations
    #[error("Invalid wallet: {0}")]
    InvalidWallet(String),
    
    /// Error related to invalid password operations
    #[error("Invalid password")]
    InvalidPassword,
    
    /// Error related to invalid passphrase operations
    #[error("Invalid passphrase: {0}")]
    InvalidPassphrase(String),
}

/// Converts an Option to a Result with a custom error message.
///
/// This utility function is useful for converting `Option<T>` to `Result<T, E>`
/// with a custom error message when the option is `None`.
///
/// # Examples
///
/// ```
/// use neo_error::option_to_result;
///
/// let option: Option<i32> = None;
/// let result = option_to_result(option, "Value not found");
/// assert!(result.is_err());
/// ```
pub fn option_to_result<T, E>(option: Option<T>, error: E) -> Result<T, E> {
    option.ok_or(error)
}

/// Adds context to a Result's error.
///
/// This utility function is useful for adding context to a `Result<T, E>`
/// with a custom error message when the result is `Err`.
///
/// # Examples
///
/// ```
/// use neo_error::with_context;
///
/// let result: Result<i32, &str> = Err("Error");
/// let result_with_context = with_context(result, |e| format!("Context: {}", e));
/// assert!(result_with_context.is_err());
/// ```
pub fn with_context<T, E, F, R>(result: Result<T, E>, f: F) -> Result<T, R>
where
    F: FnOnce(E) -> R,
{
    result.map_err(f)
}

/// Converts a Result to an Option.
///
/// This utility function is useful for converting `Result<T, E>` to `Option<T>`,
/// discarding the error information.
///
/// # Examples
///
/// ```
/// use neo_error::result_to_option;
///
/// let result: Result<i32, &str> = Err("Error");
/// let option = result_to_option(result);
/// assert!(option.is_none());
/// ```
pub fn result_to_option<T, E>(result: Result<T, E>) -> Option<T> {
    result.ok()
}
