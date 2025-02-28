# Error Handling

This reference provides information about error handling in the NeoRust SDK, including error types, error propagation, and best practices for handling errors.

## Error Types

The NeoRust SDK uses a comprehensive error handling system based on Rust's `Result` type. The main error types in the SDK include:

### NeoError

The `NeoError` is the primary error type used throughout the SDK. It encompasses various error categories:

```rust
pub enum NeoError {
    // RPC errors
    RpcError(RpcError),
    
    // Wallet errors
    WalletError(WalletError),
    
    // Cryptographic errors
    CryptoError(CryptoError),
    
    // Transaction errors
    TransactionError(TransactionError),
    
    // Contract errors
    ContractError(ContractError),
    
    // Serialization errors
    SerializationError(SerializationError),
    
    // IO errors
    IoError(std::io::Error),
    
    // Other errors
    Other(String),
}
```

### RpcError

The `RpcError` represents errors that occur during RPC communication with Neo nodes:

```rust
pub enum RpcError {
    // HTTP errors
    HttpError(reqwest::Error),
    
    // JSON-RPC errors
    JsonRpcError {
        code: i64,
        message: String,
        data: Option<serde_json::Value>,
    },
    
    // WebSocket errors
    WebSocketError(String),
    
    // Timeout errors
    TimeoutError,
    
    // Other errors
    Other(String),
}
```

### WalletError

The `WalletError` represents errors related to wallet operations:

```rust
pub enum WalletError {
    // Password errors
    InvalidPassword,
    
    // Account errors
    AccountNotFound,
    InvalidAccount,
    
    // Key errors
    InvalidPrivateKey,
    InvalidPublicKey,
    
    // File errors
    FileError(std::io::Error),
    
    // Other errors
    Other(String),
}
```

### CryptoError

The `CryptoError` represents errors related to cryptographic operations:

```rust
pub enum CryptoError {
    // Signature errors
    SignatureError,
    VerificationError,
    
    // Key errors
    InvalidKey,
    
    // Hash errors
    HashError,
    
    // Other errors
    Other(String),
}
```

### TransactionError

The `TransactionError` represents errors related to transaction operations:

```rust
pub enum TransactionError {
    // Validation errors
    InvalidTransaction,
    InvalidSignature,
    
    // Fee errors
    InsufficientFunds,
    
    // Network errors
    NetworkError,
    
    // Other errors
    Other(String),
}
```

### ContractError

The `ContractError` represents errors related to smart contract operations:

```rust
pub enum ContractError {
    // Invocation errors
    InvocationError,
    
    // Parameter errors
    InvalidParameter,
    
    // Execution errors
    ExecutionError,
    
    // Other errors
    Other(String),
}
```

## Error Propagation

The NeoRust SDK uses Rust's `?` operator for error propagation. This allows for concise error handling code:

```rust
use neo::prelude::*;
use std::path::Path;

fn load_wallet_and_get_balance(
    wallet_path: &Path,
    password: &str,
    provider: &Provider,
    token_hash: ScriptHash,
) -> Result<u64, NeoError> {
    // Load the wallet
    let wallet = Wallet::load(wallet_path, password)?;
    
    // Get the default account
    let account = wallet.default_account()?;
    
    // Create a NEP-17 token instance
    let token = Nep17Contract::new(token_hash, provider.clone());
    
    // Get the token balance
    let balance = token.balance_of(account.address())?;
    
    Ok(balance)
}
```

In this example, if any of the operations fail, the error is propagated up the call stack.

## Converting Between Error Types

The NeoRust SDK provides `From` implementations for converting between different error types:

```rust
impl From<std::io::Error> for NeoError {
    fn from(error: std::io::Error) -> Self {
        NeoError::IoError(error)
    }
}

impl From<WalletError> for NeoError {
    fn from(error: WalletError) -> Self {
        NeoError::WalletError(error)
    }
}

// ... and so on for other error types
```

This allows for easy conversion between error types using the `?` operator.

## Handling RPC Errors

When working with RPC calls, you may need to handle specific error codes:

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a Neo N3 TestNet node
    let provider = Provider::new_http("https://testnet1.neo.coz.io:443");
    
    // Try to get a transaction
    let tx_hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".parse::<TxHash>()?;
    
    match provider.get_transaction(&tx_hash).await {
        Ok(tx) => {
            println!("Transaction found: {:?}", tx);
        },
        Err(NeoError::RpcError(RpcError::JsonRpcError { code, message, .. })) if code == -100 => {
            println!("Transaction not found: {}", message);
        },
        Err(e) => {
            println!("Error: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
```

## Handling Wallet Errors

When working with wallets, you may need to handle specific wallet errors:

```rust
use neo::prelude::*;
use std::path::Path;

fn open_wallet(wallet_path: &Path, password: &str) -> Result<Wallet, NeoError> {
    match Wallet::load(wallet_path, password) {
        Ok(wallet) => {
            println!("Wallet loaded successfully");
            Ok(wallet)
        },
        Err(NeoError::WalletError(WalletError::InvalidPassword)) => {
            println!("Invalid password");
            Err(NeoError::WalletError(WalletError::InvalidPassword))
        },
        Err(NeoError::WalletError(WalletError::FileError(e))) => {
            println!("File error: {}", e);
            Err(NeoError::WalletError(WalletError::FileError(e)))
        },
        Err(e) => {
            println!("Error: {}", e);
            Err(e)
        }
    }
}
```

## Handling Transaction Errors

When sending transactions, you may need to handle specific transaction errors:

```rust
use neo::prelude::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a Neo N3 TestNet node
    let provider = Provider::new_http("https://testnet1.neo.coz.io:443");
    
    // Load your wallet
    let wallet_path = Path::new("my-wallet.json");
    let password = "my-secure-password";
    let wallet = Wallet::load(wallet_path, password)?;
    
    // Get the account that will send the transaction
    let account = wallet.default_account()?;
    
    // Create a transaction
    let transaction = TransactionBuilder::new()
        .version(0)
        .nonce(rand::random::<u32>())
        .valid_until_block(provider.get_block_count().await? + 100)
        .script(
            ScriptBuilder::new()
                .contract_call(
                    "d2a4cff31913016155e38e474a2c06d08be276cf".parse::<ScriptHash>()?,
                    "transfer",
                    &[
                        ContractParameter::hash160(account.address().script_hash()),
                        ContractParameter::hash160("NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj".parse::<Address>()?),
                        ContractParameter::integer(1_00000000), // 1 GAS
                        ContractParameter::any(None),
                    ],
                )
                .to_array()
        )
        .sign(account)?
        .build();
    
    // Send the transaction
    match provider.send_raw_transaction(&transaction).await {
        Ok(txid) => {
            println!("Transaction sent with ID: {}", txid);
        },
        Err(NeoError::TransactionError(TransactionError::InsufficientFunds)) => {
            println!("Insufficient funds to send the transaction");
        },
        Err(NeoError::RpcError(RpcError::JsonRpcError { code, message, .. })) => {
            println!("RPC error: {} (code: {})", message, code);
        },
        Err(e) => {
            println!("Error: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
```

## Custom Error Types

You can create custom error types for your application that wrap the NeoRust SDK errors:

```rust
use neo::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Neo SDK error: {0}")]
    NeoError(#[from] NeoError),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Database error: {0}")]
    DbError(String),
    
    #[error("User error: {0}")]
    UserError(String),
}

fn app_function() -> Result<(), AppError> {
    // Use the NeoRust SDK
    let wallet = Wallet::new("password").map_err(AppError::NeoError)?;
    
    // Or with the ? operator
    let wallet = Wallet::new("password")?;
    
    Ok(())
}
```

## Error Logging

The NeoRust SDK uses the `tracing` crate for logging errors. You can configure the logging level to see more detailed error information:

```rust
use neo::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    // Initialize the logger with custom configuration
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("neo=debug".parse().unwrap())
            .add_directive("warn".parse().unwrap()))
        .init();
    
    // Now errors will be logged with more detail
}
```

## SGX Error Handling

If you're using the SGX features, there are additional error types for SGX-specific operations:

```rust
pub enum SgxError {
    // Enclave errors
    EnclaveError(sgx_types::sgx_status_t),
    
    // Attestation errors
    AttestationError,
    
    // Sealing errors
    SealingError,
    
    // Other errors
    Other(String),
}
```

Handling SGX errors:

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the enclave shared object
    let enclave_path = "path/to/enclave.so";
    
    // Initialize the SGX enclave
    match SgxEnclaveManager::new(enclave_path) {
        Ok(enclave_manager) => {
            println!("SGX enclave initialized successfully!");
            
            // Use the enclave manager
        },
        Err(NeoError::SgxError(SgxError::EnclaveError(status))) => {
            println!("SGX enclave initialization failed with status: {:?}", status);
        },
        Err(e) => {
            println!("Error: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
```

## Neo X Error Handling

If you're using the Neo X features, there are additional error types for Neo X-specific operations:

```rust
pub enum NeoXError {
    // EVM errors
    EvmError(String),
    
    // Bridge errors
    BridgeError(String),
    
    // Other errors
    Other(String),
}
```

Handling Neo X errors:

```rust
use neo::prelude::*;
use neo::neo_x::evm::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Neo X provider
    let provider = NeoXProvider::new_http("https://rpc.neoX.io");
    
    // Create a transaction
    let transaction = NeoXTransaction::new()
        .to("0x1234567890123456789012345678901234567890")
        .value(1_000_000_000_000_000_000u128) // 1 ETH in wei
        .gas_price(20_000_000_000u64) // 20 Gwei
        .gas_limit(21_000u64)
        .build();
    
    // Send the transaction
    match provider.send_transaction(&transaction).await {
        Ok(txid) => {
            println!("Transaction sent with ID: {}", txid);
        },
        Err(NeoError::NeoXError(NeoXError::EvmError(message))) => {
            println!("EVM error: {}", message);
        },
        Err(e) => {
            println!("Error: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
```

## Best Practices

1. **Use the `?` Operator**: Use the `?` operator for concise error propagation.
2. **Match on Specific Errors**: Match on specific error types when you need to handle them differently.
3. **Custom Error Types**: Create custom error types for your application that wrap the NeoRust SDK errors.
4. **Error Logging**: Configure logging to see more detailed error information.
5. **Error Context**: Add context to errors to make them more informative.
6. **Error Recovery**: Implement recovery strategies for recoverable errors.
7. **Error Testing**: Write tests for error conditions to ensure they're handled correctly.

<!-- toc -->
