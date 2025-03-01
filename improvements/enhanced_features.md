# Enhanced Feature Structure for NeoRust SDK

This proposal outlines an improved feature structure for the NeoRust SDK to make the codebase more modular, easier to maintain, and more user-friendly.

## Current Issues

- Limited use of feature flags for controlling dependencies
- Monolithic design where all dependencies are included regardless of need
- Lack of clear feature grouping by functionality
- SGX and hardware wallet features are tightly coupled with core features

## Proposed Feature Structure

```toml
[features]
# Core functionality groups
default = ["http-client", "wallet-standard", "crypto-standard"]

# Network and client features
http-client = ["reqwest", "tokio/rt"]
ws-client = ["tokio-tungstenite", "tokio/rt", "reqwest"]
full-client = ["http-client", "ws-client", "tokio/full"]

# Crypto features
crypto-standard = ["tiny-keccak", "ripemd", "sha3", "aes", "cipher", "block-modes", "scrypt"]
crypto-advanced = ["crypto-standard", "ring", "blake2"]

# Wallet features
wallet-standard = ["bip39"]
wallet-hardware = ["coins-ledger", "protobuf"]
wallet-secure = ["crypto-standard"]
wallet-all = ["wallet-standard", "wallet-hardware", "wallet-secure"]

# Token standards
nep17 = []
nep11 = []
nep5 = []

# Cloud integrations
aws = ["rusoto_core", "rusoto_kms"]
yubikey = ["yubihsm"]

# SGX support
sgx = []
sgx_deps = ["sgx", "sgx_types", "sgx_urts", "sgx_tstd", "sgx_tcrypto"]

# WASM support
wasm = ["instant", "futures-timer/wasm-bindgen"]

# Async utilities
async-full = [
    "futures-core", 
    "futures-util", 
    "futures-executor", 
    "futures-channel", 
    "futures-locks", 
    "futures-timer",
    "async-stream"
]

# Ethereum compatibility (for Neo X support)
ethereum-compat = [
    "ethereum-types", 
    "open-fastrlp", 
    "rlp", 
    "uint", 
    "impl-codec", 
    "impl-serde", 
    "scale-info"
]
```

## Benefits

1. **Reduced Compile Times**: Users only compile what they need
2. **Clearer Dependencies**: Feature groups make it obvious which dependencies are needed for specific functionality
3. **Better User Experience**: Sensible defaults with easy customization
4. **Smaller Binary Sizes**: Only include necessary code for specific use cases
5. **Separation of Concerns**: SGX and hardware wallet support is properly separated from core functionality

## Implementation Steps

1. Update Cargo.toml with new feature structure
2. Update all related modules to use `#[cfg(feature = "...")]` appropriately
3. Update documentation to explain new feature flags
4. Create examples demonstrating minimal builds for specific use cases
5. Add feature compatibility matrix to documentation

## Example Usage

### Minimal HTTP Client
```toml
neo3 = { version = "0.1.1", features = ["http-client"] }
```

### Full-featured Wallet Application
```toml
neo3 = { version = "0.1.1", features = ["full-client", "wallet-all", "nep17", "nep11"] }
```

### WASM Web Application
```toml
neo3 = { version = "0.1.1", features = ["http-client", "wasm"] }
```

### Secure Enterprise Application
```toml
neo3 = { version = "0.1.1", features = ["full-client", "wallet-secure", "sgx_deps"] }
``` 