# Migration Guide: Adopting the New Feature Flag System

This guide will help you migrate your existing NeoRust SDK applications to use the new feature flag system.

## Overview of Changes

The NeoRust SDK has been reorganized to use a more modular feature flag system. This allows you to include only the features you need, resulting in faster compile times and smaller binaries. However, it may require changes to your `Cargo.toml` file and potentially some code updates.

## Quick Migration Path

For most applications, the migration will be straightforward:

1. Update your dependency in `Cargo.toml`
2. Select the appropriate features for your application
3. Test your application

## Detailed Migration Steps

### Step 1: Update Your Dependency

Update your dependency in `Cargo.toml` to use the new version of the SDK:

```toml
# Before
[dependencies]
neo3 = "0.1.1"

# After
[dependencies]
neo3 = "0.1.2"
```

### Step 2: Select Appropriate Features

If you were using the default features before, you may not need to make any changes. However, to optimize your application, you should explicitly select only the features you need:

```toml
# For a basic wallet application
[dependencies]
neo3 = { version = "0.1.2", features = ["wallet", "crypto-standard"] }

# For a blockchain explorer application
[dependencies]
neo3 = { version = "0.1.2", features = ["http-client"] }

# For a full dApp
[dependencies]
neo3 = { version = "0.1.2", features = ["wallet", "transaction", "contract", "http-client"] }
```

### Step 3: Feature Mapping

Use this table to determine which features you need based on the functionality you were using:

| Previous Usage | New Features to Include |
|----------------|------------------------|
| Wallet management | `wallet`, `crypto-standard` |
| Transactions | `transaction`, `crypto-standard` |
| Smart contracts | `contract`, `transaction` |
| RPC/HTTP client | `http-client` |
| WebSocket client | `ws-client` |
| NEP-17 tokens | `nep17`, `http-client`, `contract` |
| NEP-11 tokens | `nep11`, `http-client`, `contract` |
| SGX integration | `sgx`, `wallet-secure` |
| WASM support | `wasm`, `http-client` |
| Ledger hardware wallet | `wallet-hardware` |
| AWS KMS integration | `aws` |

### Step 4: Update Import Statements

If you were using wildcard imports, you may need to update your import statements to import specific modules based on the features you're using:

```rust
// Before
use neo::prelude::*;

// After - be more specific if not using all features
use neo::prelude::*;
// Or use more specific imports
use neo::types::{Hash160, Address};
use neo::wallets::{Wallet, Account};
use neo::transaction::TransactionBuilder;
```

### Step 5: Test Your Application

After updating your `Cargo.toml` and imports, build and test your application to ensure everything works as expected.

## Common Issues and Solutions

### Missing Functionality

**Problem**: You get compilation errors about missing types or functions.

**Solution**: You may need to enable additional features. Check the error messages to identify which modules are missing, then add the appropriate features to your `Cargo.toml`.

```toml
# Example: Adding missing transaction feature
[dependencies]
neo3 = { version = "0.1.2", features = ["wallet", "transaction", "http-client"] }
```

### Feature Combinations

**Problem**: Some functionality requires multiple features to work correctly.

**Solution**: Ensure you have all the necessary features enabled for your use case. For example, using NEP-17 tokens requires the `nep17`, `contract`, and `http-client` features.

### Legacy Features

**Problem**: You were using a feature that has been renamed or reorganized.

**Solution**: Refer to this mapping of legacy features to new features:

| Legacy Feature | New Feature(s) |
|----------------|---------------|
| `futures` | `async-std` |
| `ledger` | `wallet-hardware` |
| `aws` | `aws` |
| `sgx` | `sgx`, `wallet-secure` |

## Advanced Migration Topics

### Custom Feature Combinations

If your application has specific requirements, you can create custom feature combinations. For example, if you need a minimal WASM build:

```toml
[dependencies]
neo3 = { version = "0.1.2", default-features = false, features = ["wasm", "wallet"] }
```

### Conditional Compilation in Your Code

You can use feature flags in your own code to conditionally compile sections based on which features are enabled:

```rust
#[cfg(feature = "wallet")]
fn create_wallet() {
    // Wallet creation code
}

#[cfg(not(feature = "wallet"))]
fn create_wallet() {
    compile_error!("This function requires the 'wallet' feature");
}
```

### Working with Multiple Feature Sets

If you need different feature sets for different build targets:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
neo3 = { version = "0.1.2", default-features = false, features = ["wasm", "wallet"] }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
neo3 = { version = "0.1.2", features = ["wallet", "transaction", "http-client"] }
```

## Getting Help

If you encounter any issues while migrating to the new feature flag system, please:

1. Check the [documentation](https://docs.rs/neo3) for guidance on specific features
2. Review the [examples](https://github.com/R3E-Network/NeoRust/tree/main/examples/feature_flags) for usage patterns
3. Open an issue on [GitHub](https://github.com/R3E-Network/NeoRust/issues) if you encounter bugs
4. Join the [Neo Discord](https://discord.gg/neo) for community support

## Conclusion

The new feature flag system is designed to make the NeoRust SDK more flexible, efficient, and maintainable. While it may require some initial migration effort, the benefits in terms of compile times, binary sizes, and clarity of dependencies should make it worthwhile.

We appreciate your patience during this transition and welcome your feedback on the new system. 