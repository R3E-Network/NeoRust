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
neo = { git = "https://github.com/R3E-Network/NeoRust" }
```

## Feature Flags

NeoRust supports various feature flags to enable specific functionality:

```toml
[dependencies]
neo = { git = "https://github.com/R3E-Network/NeoRust", features = ["ledger", "aws"] }
```

Available features:
- `ledger`: Support for Ledger hardware wallets
- `aws`: AWS integration
- `sgx`: Intel SGX support (requires additional setup)

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
