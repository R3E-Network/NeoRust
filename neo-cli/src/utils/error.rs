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

pub type CliResult<T> = Result<T, CliError>;
