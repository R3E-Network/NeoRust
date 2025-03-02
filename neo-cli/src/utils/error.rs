use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Wallet error: {0}")]
    Wallet(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("RPC error: {0}")]
    Rpc(String),
    
    #[error("Input error: {0}")]
    Input(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("SDK error: {0}")]
    Sdk(String),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Transaction builder error: {0}")]
    TransactionBuilder(String),
    
    #[error("Builder error: {0}")]
    Builder(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Provider error: {0}")]
    Provider(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Contract error: {0}")]
    Contract(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
    
    #[error("Anyhow error: {0}")]
    Anyhow(String),
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        CliError::Anyhow(err.to_string())
    }
}

impl From<std::num::ParseFloatError> for CliError {
    fn from(err: std::num::ParseFloatError) -> Self {
        CliError::Input(format!("Invalid number: {}", err))
    }
}

impl From<std::num::ParseIntError> for CliError {
    fn from(err: std::num::ParseIntError) -> Self {
        CliError::Input(format!("Invalid integer: {}", err))
    }
}

impl From<hex::FromHexError> for CliError {
    fn from(err: hex::FromHexError) -> Self {
        CliError::Input(format!("Invalid hex string: {}", err))
    }
}

pub type CliResult<T> = Result<T, CliError>;
