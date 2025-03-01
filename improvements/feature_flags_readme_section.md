# Feature Flag System

NeoRust uses a comprehensive feature flag system to allow you to include only the functionality you need. This helps reduce compile times, decrease binary sizes, and simplify dependency management.

## Core Feature Groups

The following core feature groups are available:

- **std**: Standard library support with basic serialization (default)
- **transaction**: Transaction creation and signing capabilities
- **wallet**: Wallet and account management
- **contract**: Smart contract interaction 
- **http-client**: HTTP-based RPC client for Neo N3 nodes
- **ws-client**: WebSocket client for real-time updates

## Token Standards

Support for Neo N3 token standards:

- **nep17**: NEP-17 fungible token standard support
- **nep11**: NEP-11 non-fungible token standard support

## Integration Features

Features for integrating with external systems:

- **sgx**: Intel SGX secure enclave integration
- **wasm**: WebAssembly support for browser environments
- **ledger**: Ledger hardware wallet support
- **aws**: Amazon Web Services KMS integration
- **ethereum-compat**: Ethereum compatibility for Neo X integration

## Cryptography Levels

Choose your required level of cryptographic functionality:

- **crypto-standard**: Basic cryptographic functions (default)
- **crypto-advanced**: Advanced cryptography with additional algorithms

## Wallet Features

Enhanced wallet capabilities:

- **wallet-standard**: Standard wallet with file I/O support (default with wallet)
- **wallet-hardware**: Hardware wallet support
- **wallet-secure**: Advanced security features for wallets

## Serialization Support

Data format support:

- **serde**: Serialization/deserialization support
- **json**: JSON format support
- **binary-format**: Binary format utilities

## Example Usage

In your `Cargo.toml`, specify exactly what features you need:

```toml
# Basic wallet application
neo3 = { version = "0.5.0", features = ["wallet", "crypto-standard"] }

# Full-featured dApp backend
neo3 = { version = "0.5.0", features = ["wallet", "transaction", "contract", "http-client", "nep17"] }

# Minimal transaction builder
neo3 = { version = "0.5.0", default-features = false, features = ["transaction"] }

# WebAssembly application
neo3 = { version = "0.5.0", features = ["wallet", "http-client", "wasm"] }

# Secure application with hardware wallet support
neo3 = { version = "0.5.0", features = ["wallet-hardware", "wallet-secure"] }
```

## Common Feature Combinations

Here are some recommended feature combinations for common use cases:

| Use Case | Recommended Features |
|----------|---------------------|
| Simple Wallet Tool | `wallet`, `crypto-standard` |
| Block Explorer | `http-client` |
| Token Transfer dApp | `wallet`, `transaction`, `http-client`, `nep17` |
| NFT Marketplace | `wallet`, `transaction`, `http-client`, `nep11` |
| Smart Contract Development | `wallet`, `transaction`, `contract`, `http-client` |
| Web dApp (WASM) | `wallet`, `transaction`, `http-client`, `wasm` |
| Hardware Wallet Integration | `wallet-hardware`, `transaction` |
| Secure Enterprise Application | `wallet-secure`, `crypto-advanced`, `sgx` |

## Performance Benefits

By selecting only the features you need, you can significantly reduce compile times and binary sizes:

| Feature Combination | Approximate Compile Time | Approximate Binary Size |
|---------------------|--------------------------|-------------------------|
| All features | 2m 15s | 8.2 MB |
| Default features | 1m 45s | 6.5 MB |
| Minimal wallet | 38s | 1.1 MB |
| HTTP client only | 42s | 1.5 MB |

The specific improvements will vary based on your hardware and build configuration. 