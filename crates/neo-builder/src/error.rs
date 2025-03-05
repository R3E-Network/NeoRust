use neo_codec::CodecError;
use neo_crypto::CryptoError;
use neo_clients::ProviderError;
use crate::transaction::TransactionError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum BuilderError {
	#[error("Invalid operation")]
	InvalidScript(String),
	#[error("Invalid operation")]
	InvalidOperation,
	#[error("Invalid argument")]
	InvalidArgument,
	#[error("Invalid state")]
	InvalidState,
	#[error("Invalid invocation")]
	InvalidInvocation,
	#[error("Stack overflow")]
	StackOverflow,
	#[error("Out of gas")]
	OutOfGas,
	#[error("Out of memory")]
	OutOfMemory,
	#[error("Out of cycles")]
	OutOfCycles,
	#[error("UnknownError")]
	UnknownError,
	#[error("Unsupported operation: {0}")]
	UnsupportedOperation(String),
	#[error("Invalid signer configuration: {0}")]
	SignerConfiguration(String),
	#[error("Invalid transaction configuration: {0}")]
	TransactionConfiguration(String),
	#[error("Invalid configuration: {0}")]
	InvalidConfiguration(String),
	#[error("Too many signers: {0}")]
	TooManySigners(String),
	#[error("Illegal state: {0}")]
	IllegalState(String),
	#[error("Illegal argument: {0}")]
	IllegalArgument(String),
	#[error("Invalid public key: {0}")]
	CodecError(#[from] CodecError),
	#[error("Crypto error: {0}")]
	CryptoError(#[from] CryptoError),
	#[error(transparent)]
	ProviderError(#[from] ProviderError),
	#[error(transparent)]
	TransactionError(Box<TransactionError>),
}

impl From<TransactionError> for BuilderError {
	fn from(err: TransactionError) -> Self {
		BuilderError::TransactionError(Box::new(err))
	}
}
