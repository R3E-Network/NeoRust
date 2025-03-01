# Feature Flag Examples

This directory contains examples demonstrating how to use NeoRust with different feature flag combinations.

## Overview

The NeoRust SDK uses a comprehensive feature flag system that allows you to include only the functionality you need. This helps reduce compile times, decrease binary sizes, and simplify dependency management.

These examples demonstrate how to use different feature flag combinations for various use cases.

## Examples

### 1. Minimal Wallet

**File:** `minimal_wallet.rs`

**Required Features:** `wallet`, `crypto-standard`

**Description:** This example demonstrates how to create a wallet with minimal features, focusing on basic account management and cryptographic operations without the overhead of network communication or smart contract functionality.

**Run with:**
```bash
cargo run --example feature_flags/minimal_wallet --no-default-features --features="wallet,crypto-standard"
```

### 2. NEP-17 Token Interaction

**File:** `nep17_token.rs`

**Required Features:** `nep17`, `http-client`, `transaction`, `wallet`

**Description:** This example shows how to interact with NEP-17 fungible tokens on the Neo N3 blockchain, including connecting to a node, retrieving token information, checking balances, and preparing transactions.

**Run with:**
```bash
cargo run --example feature_flags/nep17_token --features="nep17,http-client,transaction,wallet"
```

## Using Feature Flags in Your Own Projects

To use feature flags in your own projects, specify exactly which features you need in your `Cargo.toml` file:

```toml
[dependencies]
neo3 = { version = "0.1.1", features = ["wallet", "transaction", "http-client"] }
```

Or for a minimal build:

```toml
[dependencies]
neo3 = { version = "0.1.1", default-features = false, features = ["wallet", "crypto-standard"] }
```

See the main [README.md](../../README.md) for a complete list of available features and recommended combinations.

## Performance Benefits

By selecting only the features you need, you can significantly reduce compile times and binary sizes. For example:

- **All features:** ~2m 15s compile time, ~8.2 MB binary size
- **Minimal wallet:** ~38s compile time, ~1.1 MB binary size

## Creating Your Own Examples

When creating your own examples, be sure to:

1. Document the required features at the top of your file
2. Include clear instructions on how to run the example with the correct feature flags
3. Only use functionality that's available with the specified features
4. Handle potential errors gracefully, as some functionality might not be available

For more information on available features, see the [Feature Flag System](../../README.md#feature-flag-system) section in the main README. 