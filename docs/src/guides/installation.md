# Installation Guide

This guide provides detailed instructions for installing and configuring the NeoRust SDK.

## Requirements

- Rust 1.70.0 or later
- Cargo package manager
- Optional: Intel SGX SDK (for SGX features)

## Standard Installation

Add the NeoRust SDK to your Cargo.toml:

```toml
[dependencies]
neo3 = "0.1.3"
```

## Feature Flags

NeoRust supports various feature flags to enable specific functionality:

```toml
[dependencies]
neo3 = { version = "0.1.3", features = ["ledger", "crypto-standard"] }
```

### Core Feature Groups
- `std`: Standard library support (enabled by default)
- `crypto-standard`: Cryptographic functionality using standard libraries (enabled by default)
- `utils`: Utility functions and helpers
- `serde-support`: Serialization/deserialization support

### Optional Features
- `contract`: Smart contract interaction and deployment
- `ledger`: Hardware wallet support
- `nightly`: Documentation features for nightly Rust
- `tokio-support`: Asynchronous runtime support
- `transaction`: Transaction creation and signing
- `aws-nitro-tee`: AWS Nitro Trusted Execution Environment support

### Cryptography Features
- `ripemd160`: RIPEMD-160 hash function for address generation
- `sha2`: SHA2 hash functions
- `digest`: Core digest traits for hash functions

## SGX Support

To use the SGX features, you need to install the Intel SGX SDK. See the [SGX Setup Guide](../tutorials/sgx.md) for detailed instructions.

## Verifying Installation

To verify that the SDK is installed correctly, create a simple test program:

```rust
use neo::prelude::*;

fn main() {
    println!("NeoRust SDK installed successfully!");
}
```

Compile and run the program:

```bash
cargo run
```

If the program compiles and runs without errors, the SDK is installed correctly.

<!-- toc -->
