# NeoRust Wallet Feature Structure

## Overview of Issues and Solutions

Through our analysis, we've identified several key issues in the wallet feature implementation:

1. **Circular Dependencies:** The wallet module has circular imports between different submodules and the core library.
2. **Inconsistent Error Types:** Multiple WalletError definitions with different methods and fields.
3. **Private Module Access:** Some modules that should be public are defined as private.
4. **Missing Feature Flags:** Some functionality doesn't have proper feature gates.
5. **Dependency Conflicts:** Dependencies like Signature are imported from both yubikey and p256.

## Recommended Structure Changes

To address these issues, we recommend the following changes:

### 1. Error Handling Hierarchy

Create a clear three-layer error hierarchy:
```
neo_error.rs                    → Contains a generic WalletError(String)
↓
neo_wallets/wallet_error.rs     → Contains detailed WalletError enum with variants
↓
neo_wallets/wallet_detailed_error.rs → Contains specific error cases for wallet operations
```

With proper conversion implementations between all levels.

### 2. Module Visibility

Make sure all necessary modules are public:
```rust
// In neo_wallets/wallet/mod.rs
pub mod wallet;
pub mod wallet_error;
pub mod wallet_detailed_error;
pub mod nep6wallet;
pub mod backup;

#[cfg(feature = "wallet-standard")]
pub mod nep6account;

#[cfg(feature = "wallet-standard")]
pub mod nep6contract;
```

### 3. Type Definitions

Have a single authoritative definition for types like `ScryptParamsDef`:
```rust
// In neo_wallets/wallet/wallet.rs - primary definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScryptParamsDef { ... }

// In neo_types/mod.rs - re-export only
#[cfg(feature = "wallet")]
pub use crate::neo_wallets::wallet::wallet::ScryptParamsDef;
```

### 4. Feature Gates for Optional Features

Add proper feature gates for optional functionality:
```rust
// For transaction support
#[cfg(feature = "transaction")]
pub fn sign_transaction(...) { ... }

// For hardware wallet support  
#[cfg(feature = "wallet-hardware")]
pub fn connect_ledger(...) { ... }

// For standard wallet features
#[cfg(feature = "wallet-standard")]
pub fn save_nep6(...) { ... }
```

### 5. Generic Parameters

Remove unnecessary generic parameters like `NistP256` from the Signature type:
```rust
// Instead of
pub fn sign_hash(&self, hash: H256) -> Result<Signature<NistP256>, WalletError>

// Use
pub fn sign_hash(&self, hash: H256) -> Result<Signature, WalletError>
```

### 6. Import Path Consistency

Use consistent import paths:
```rust
// Prefer
use crate::neo_wallets::wallet_error::WalletError;

// Instead of 
use neo::prelude::WalletError;
```

Unless you're specifically importing from the prelude.

## Import Structure Best Practices

For neo_wallets modules:
```rust
// Internal wallet imports
use crate::neo_wallets::{
    account_trait::AccountTrait,
    wallet::{
        wallet::Wallet,
        wallet_error::WalletError,
    },
    wallet_signer::WalletSigner,
};

// Imports from other crate modules
use crate::{
    neo_crypto::key_pair::KeyPair,
    neo_types::{Address, H160},
    prelude::ProviderError,
};

// External crate imports (group by purpose)
// Crypto & primitives
use p256::ecdsa::Signature;
use primitive_types::H256;

// Serialization
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Standard library
use std::{
    collections::HashMap,
    fmt::{self, Display},
    str::FromStr,
    sync::Arc,
};
```

## Feature Flag Structure

We recommend a hierarchical feature structure:

```toml
# Core wallet functionality with minimal dependencies
wallet = [
    "crypto-standard", 
    "scrypt", 
    "serde_derive"
]

# Enhanced wallet with standard formats (includes wallet)
wallet-standard = [
    "wallet", 
    "ethereum-types"
]

# Hardware wallet support (includes wallet)
wallet-hardware = [
    "wallet", 
    "ledger"
]

# Enhanced security features (includes wallet)
wallet-secure = [
    "wallet", 
    "aes", 
    "block-modes"
]
```

This structure allows users to select only the wallet features they need, minimizing dependencies while maintaining compatibility.

## Testing Strategy

To verify wallet functionality:
```bash
# Test basic wallet functionality
cargo test --no-default-features --features="wallet"

# Test wallet-standard features  
cargo test --no-default-features --features="wallet-standard" 

# Test wallet with specific transactions
cargo test --no-default-features --features="wallet transaction"

# Test full wallet suite
cargo test --features="wallet-standard wallet-hardware wallet-secure"
```
