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

Available features:
- `std`: Standard library support with basic serialization (enabled by default)
- `crypto-standard`: Cryptographic functionality including hash functions, key pair operations, and signature verification (enabled by default)
- `ledger`: Support for Ledger hardware wallets
- `nightly`: Support for nightly Rust features (used for documentation)
- `ripemd160`: RIPEMD-160 hash function support
- `sha2`: SHA-2 hash function support
- `digest`: Cryptographic digest algorithms

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
