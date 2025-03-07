# Neo Error

Error types and handling for the NeoRust SDK.

This crate provides error types and handling utilities for the Neo N3 blockchain SDK, including:

- Common error types used across the SDK
- Error conversion traits
- Error handling utilities
- Integration with standard Rust error handling

## Usage

```rust
use neo_error::{ProviderError, CryptoError, CodecError};
use thiserror::Error;

// Define your own error type that wraps Neo errors
#[derive(Debug, Error)]
pub enum MyAppError {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
    
    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
    
    #[error("Application error: {0}")]
    Application(String),
}

// Use the error types in your code
fn my_function() -> Result<(), MyAppError> {
    // Handle different error types
    Ok(())
}
```

For more information, see the [NeoRust documentation](https://docs.rs/neo3).
