use std::hash::{Hash, Hasher};

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
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

impl Hash for CodecError {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            CodecError::InvalidPassphrase(s) => {
                0.hash(state);
                s.hash(state);
            },
            CodecError::InvalidFormat => 1.hash(state),
            CodecError::IndexOutOfBounds(s) => {
                2.hash(state);
                s.hash(state);
            },
            CodecError::InvalidEncoding(s) => {
                3.hash(state);
                s.hash(state);
            },
            CodecError::InvalidOpCode => 4.hash(state),
        }
    }
}

// Implement From trait for TryFromPrimitiveError<T> to convert it to CodecError
impl<T: TryFromPrimitive> From<TryFromPrimitiveError<T>> for CodecError {
    fn from(_: TryFromPrimitiveError<T>) -> Self {
        CodecError::InvalidOpCode
    }
}
