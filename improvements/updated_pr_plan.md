# Feature Organization Improvement - Updated Pull Request Plan

## Overview

This document outlines the updated plan for implementing the enhanced dependency management and feature organization for the NeoRust SDK. The goal is to create a more modular and flexible feature flag system that reduces compile times, binary sizes, and dependency conflicts.

## Current State Analysis

We have successfully implemented feature flags in the core modules of the SDK:

- `lib.rs`: Core module structure with feature-gated exports
- `neo_contract`: Smart contract interactions with token standard features
- `neo_clients`: Client interfaces with transport-specific features
- `neo_wallets`: Wallet management with security and standard features
- `neo_protocol`: Protocol types with crypto and transaction features
- `neo_builder`: Transaction building with contract features
- `neo_types`: Core type system with conditional availability

## Remaining PR Scope

The PR will include:

1. Implementing feature flags for remaining modules:
   - `neo_codec`: Serialization and encoding functionality
   - `neo_config`: Configuration management functionality
   - `neo_x`: Neo X ecosystem functionality
   - `neo_utils`: Utility functions

2. Updating examples to demonstrate feature usage
3. Updating tests to respect feature boundaries
4. Adding CI jobs to test feature combinations
5. Measuring performance improvements

## Implementation Details

### 1. Complete Feature Hierarchy

The feature hierarchy has been implemented with the following structure:

```
default = ["std", "http-client", "wallet", "transaction"]

# Core feature groups
std = ["serde", "json"]
transaction = ["crypto-standard"]
wallet = ["transaction", "wallet-standard"]
contract = ["transaction", "contract-invoke"]
http-client = ["reqwest", "tokio/rt"]
ws-client = ["tokio-tungstenite", "tokio/rt"]

# Token standards
nep17 = ["contract"]
nep11 = ["contract"]

# Integration features
sgx = ["sgx-runtime", "wallet-secure"]
wasm = ["wasm-bindgen", "js-sys"]
ledger = ["wallet-hardware"]
aws = ["aws-kms"]

# Cryptography levels
crypto-standard = ["sha2", "k256", "ripemd"]
crypto-advanced = ["crypto-standard", "ring", "scrypt"]

# Wallet features
wallet-standard = ["tokio/fs", "serde"]
wallet-hardware = ["coins-ledger"]
wallet-secure = ["crypto-advanced"]
```

### 2. Remaining Module Updates

#### `neo_codec` Module:

The codec module will be updated to separate core encoding functionality from format-specific implementations:

```rust
// Core codec functionality - always available
pub use decoding::*;
pub use encoding::*;
pub use error::*;

// JSON codec - only available with json feature
#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub use json::*;

// Binary codec - only available with binary feature
#[cfg(feature = "binary-format")]
#[cfg_attr(docsrs, doc(cfg(feature = "binary-format")))]
pub use binary::*;
```

#### `neo_config` Module:

The config module will have minimal dependencies and be available with most features:

```rust
// Core configuration - always available
pub use config_error::*;
pub use constants::*;

// Network-specific configuration
#[cfg(any(feature = "http-client", feature = "ws-client"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "http-client", feature = "ws-client"))))]
pub use network_config::*;
```

### 3. Update CI Configuration

We will add a new GitHub Actions workflow to test different feature combinations:

```yaml
name: Feature Combinations

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  feature-matrix:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        feature-set: [
          "default",
          "transaction",
          "wallet",
          "contract,http-client",
          "nep17,http-client",
          "nep11,http-client",
          "sgx,wallet",
          "wallet-hardware",
          "wallet-secure",
          "wasm",
        ]
    steps:
      - uses: actions/checkout@v2
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Build with features
        run: cargo build --no-default-features --features=${{ matrix.feature-set }}
```

### 4. Documentation Updates

We will update the following documentation files:

- `README.md`: Add comprehensive feature flag documentation
- `examples/README.md`: Add examples with feature usage guidance
- `CHANGELOG.md`: Document feature flag system changes
- Create a new migration guide for users upgrading from previous versions

### 5. Example Updates

We will update examples to demonstrate minimal feature requirements:

```rust
// examples/minimal_wallet.rs
//
// This example requires the following features:
// - wallet
// - crypto-standard
//
// Run with:
// cargo run --example minimal_wallet --features="wallet,crypto-standard"

use neo::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a wallet with minimal features
    let wallet = Wallet::new()?;
    println!("Created wallet: {}", wallet.name());
    
    // Create and add an account
    let account = Account::create()?;
    println!("Account address: {}", account.address());
    
    Ok(())
}
```

## Testing Approach

1. **Unit tests**: Ensure each feature works correctly in isolation
2. **Integration tests**: Verify that features work correctly together
3. **Binary size comparison**: Measure the size reductions from feature flags
4. **Compile time measurements**: Document compile time improvements

## PR Timeline

1. Remaining module updates (2 days)
2. Test and example updates (2 days)
3. CI configuration (1 day)
4. Documentation updates (2 days)
5. Final testing and refinement (1 day)

Total: ~1 week

## Success Criteria

1. All modules properly use feature flags
2. Compile time reduced by at least 30% for minimal builds
3. Binary size reduced by at least 50% for minimal builds 
4. All tests pass with appropriate feature combinations
5. Documentation clearly explains the feature system

## Post-PR Tasks

1. Follow-up with any feedback from reviewers
2. Create examples showcasing different feature combinations
3. Monitor any issues users encounter with the new system
4. Prepare a full SDK release with the new feature system 