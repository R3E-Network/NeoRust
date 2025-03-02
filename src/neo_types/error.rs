#[cfg(feature = "thiserror")]
use thiserror::Error;

#[cfg(not(feature = "thiserror"))]
pub use std::fmt;

#[cfg(not(feature = "thiserror"))]
#[derive(Debug, Clone)]
pub struct ErrorWrapper {
    pub kind: String,
    pub message: String,
}

#[cfg(not(feature = "thiserror"))]
impl std::error::Error for ErrorWrapper {}

#[cfg(not(feature = "thiserror"))]
impl fmt::Display for ErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

// Define the TypeError enum with conditional attributes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "thiserror", derive(Error))]
pub enum TypeError {
    #[cfg_attr(feature = "thiserror", error("Invalid script: {0}"))]
    InvalidScript(String),
    
    #[cfg_attr(feature = "thiserror", error("Invalid format: {0}"))]
    InvalidFormat(String),
    
    #[cfg_attr(feature = "thiserror", error("neo-rs not initialized"))]
    NotInitialized,
    
    #[cfg_attr(feature = "thiserror", error("Unexpected returned type: {0}"))]
    UnexpectedReturnType(String),
    
    #[cfg_attr(feature = "thiserror", error("Invalid private key"))]
    InvalidPrivateKey,
    
    #[cfg_attr(feature = "thiserror", error("Invalid public key"))]
    InvalidPublicKey,
    
    #[cfg_attr(feature = "thiserror", error("Invalid address"))]
    InvalidAddress,
    
    #[cfg_attr(feature = "thiserror", error("Invalid signature"))]
    InvalidSignature,
    
    #[cfg_attr(feature = "thiserror", error("Invalid encoding {0}"))]
    InvalidEncoding(String),
    
    #[cfg_attr(feature = "thiserror", error("Invalid op code"))]
    InvalidOpCode,
    
    #[cfg_attr(feature = "thiserror", error("Invalid argument {0}"))]
    InvalidArgument(String),
    
    #[cfg_attr(feature = "thiserror", error("Invalid neo name {0}"))]
    InvalidNeoName(String),
    
    #[cfg_attr(feature = "thiserror", error("Numeric overflow"))]
    NumericOverflow,
    
    #[cfg_attr(feature = "thiserror", error("Wif error {0}"))]
    WifError(String),
    
    #[cfg_attr(feature = "thiserror", error("Codec error: {0}"))]
    CodecError(String),
    
    #[cfg_attr(feature = "thiserror", error("Index out of bounds: {0}"))]
    IndexOutOfBounds(String),
    
    #[cfg_attr(feature = "thiserror", error("Invalid data: {0}"))]
    InvalidData(String),
    
    #[cfg_attr(feature = "thiserror", error("Invalid argument error: {0}"))]
    InvalidArgError(String),
}

// Implement Display for TypeError when thiserror is not enabled
#[cfg(not(feature = "thiserror"))]
impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::InvalidScript(s) => write!(f, "Invalid script: {}", s),
            TypeError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            TypeError::NotInitialized => write!(f, "neo-rs not initialized"),
            TypeError::UnexpectedReturnType(s) => write!(f, "Unexpected returned type: {}", s),
            TypeError::InvalidPrivateKey => write!(f, "Invalid private key"),
            TypeError::InvalidPublicKey => write!(f, "Invalid public key"),
            TypeError::InvalidAddress => write!(f, "Invalid address"),
            TypeError::InvalidSignature => write!(f, "Invalid signature"),
            TypeError::InvalidEncoding(s) => write!(f, "Invalid encoding: {}", s),
            TypeError::InvalidOpCode => write!(f, "Invalid op code"),
            TypeError::InvalidArgument(s) => write!(f, "Invalid argument: {}", s),
            TypeError::InvalidNeoName(s) => write!(f, "Invalid neo name: {}", s),
            TypeError::NumericOverflow => write!(f, "Numeric overflow"),
            TypeError::WifError(s) => write!(f, "Wif error: {}", s),
            TypeError::CodecError(s) => write!(f, "Codec error: {}", s),
            TypeError::IndexOutOfBounds(s) => write!(f, "Index out of bounds: {}", s),
            TypeError::InvalidData(s) => write!(f, "Invalid data: {}", s),
            TypeError::InvalidArgError(s) => write!(f, "Invalid argument error: {}", s),
        }
    }
}

// Implement Error trait for TypeError
#[cfg(not(feature = "thiserror"))]
impl std::error::Error for TypeError {}
