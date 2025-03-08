use thiserror::Error;
use crate::provider_error::ProviderError;

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
