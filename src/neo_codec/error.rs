use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Hash, Clone)]
pub enum CodecError {
	#[error("Invalid passphrase: {0}")]
	InvalidPassphrase(String),
	#[error("Invalid format")]
	InvalidFormat,
	#[error("Index out of bounds: {0}")]
	IndexOutOfBounds(String),
	#[error("Invalid encoding: {0}")]
	InvalidEncoding(String),
	#[error("Invalid op code")]
	InvalidOpCode,
}
