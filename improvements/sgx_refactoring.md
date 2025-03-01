# SGX Integration Refactoring

This document outlines a proposal for refactoring the Intel Software Guard Extensions (SGX) integration in the NeoRust SDK to make it more modular and maintainable.

## Current Issues

- SGX dependencies are tightly coupled with the main codebase
- SGX-specific code and non-SGX code are not clearly separated
- The build process becomes complex due to SGX requirements
- SGX dependencies lead to compatibility issues on systems without SGX support
- Users who don't need SGX functionality still have to deal with its complexity

## Proposed Solution

Move the SGX integration into a separate crate that can be optionally included, following these principles:

1. **Separation of Concerns**: Core SDK functionality should not depend on SGX
2. **Plugin Architecture**: SGX support should be a plugin to the core SDK
3. **Clear Interfaces**: Define clean interfaces between core SDK and SGX components

## Implementation Plan

### 1. Create a Separate Crate Structure

```
neo-ecosystem/
├── neo3/                 # Core SDK (renamed from current NeoRust)
├── neo3-sgx/             # SGX extension crate
├── neo3-aws/             # AWS extension crate
├── neo3-ledger/          # Ledger hardware wallet extension crate
└── neo3-examples/        # Examples using various extensions
```

### 2. Define Clear Interfaces in Core SDK

In the core SDK (neo3), define trait interfaces that SGX components can implement:

```rust
// In neo3/src/neo_crypto/secure_enclave.rs
pub trait SecureEnclave {
    fn generate_key_pair(&self) -> Result<KeyPair, Error>;
    fn sign_data(&self, data: &[u8], key_id: &str) -> Result<Signature, Error>;
    fn encrypt_data(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>, Error>;
    fn decrypt_data(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>, Error>;
}

// In neo3/src/neo_wallets/secure_wallet.rs
pub trait SecureWallet {
    fn create_secure_account(&self) -> Result<Account, Error>;
    fn sign_transaction(&self, tx: &Transaction) -> Result<Witness, Error>;
}
```

### 3. Implement SGX-specific Components in neo3-sgx

In the neo3-sgx crate, implement the interfaces defined in the core SDK:

```rust
// In neo3-sgx/src/enclave.rs
pub struct SgxEnclave {
    enclave_id: sgx_enclave_id_t,
    // Other SGX-specific fields
}

impl SecureEnclave for SgxEnclave {
    fn generate_key_pair(&self) -> Result<KeyPair, Error> {
        // SGX-specific implementation
    }
    
    fn sign_data(&self, data: &[u8], key_id: &str) -> Result<Signature, Error> {
        // SGX-specific implementation
    }
    
    fn encrypt_data(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>, Error> {
        // SGX-specific implementation
    }
    
    fn decrypt_data(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>, Error> {
        // SGX-specific implementation
    }
}

// In neo3-sgx/src/wallet.rs
pub struct SgxWallet {
    enclave: SgxEnclave,
    // Other SGX-specific fields
}

impl SecureWallet for SgxWallet {
    fn create_secure_account(&self) -> Result<Account, Error> {
        // SGX-specific implementation
    }
    
    fn sign_transaction(&self, tx: &Transaction) -> Result<Witness, Error> {
        // SGX-specific implementation
    }
}
```

### 4. Update Cargo.toml Files

#### neo3/Cargo.toml
```toml
[package]
name = "neo3"
version = "0.1.1"
# ... other metadata

[dependencies]
# ... existing dependencies without SGX

[features]
default = ["http-client", "wallet-standard"]
secure-enclave = [] # Enable trait definitions for secure enclaves, but no implementations
```

#### neo3-sgx/Cargo.toml
```toml
[package]
name = "neo3-sgx"
version = "0.1.0"
# ... other metadata

[dependencies]
neo3 = { version = "0.1.1", features = ["secure-enclave"] }
sgx_types = { version = "=1.1.1", default-features = false }
sgx_urts = { version = "=1.1.1", default-features = false }
sgx_tstd = { version = "=1.1.1", default-features = false }
sgx_tcrypto = { version = "=1.1.1", default-features = false }
# ... other SGX-specific dependencies

[features]
default = []
simulation = [] # For development without SGX hardware
```

### 5. Example Usage

```rust
// Using core SDK without SGX
use neo3::prelude::*;

async fn standard_usage() -> Result<(), Box<dyn std::error::Error>> {
    let provider = HttpProvider::new("https://mainnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Standard wallet operations
    let wallet = Wallet::new();
    let account = Account::create()?;
    wallet.add_account(account);
    
    Ok(())
}

// Using core SDK with SGX extension
use neo3::prelude::*;
use neo3_sgx::SgxWallet;

async fn sgx_usage() -> Result<(), Box<dyn std::error::Error>> {
    let provider = HttpProvider::new("https://mainnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Initialize SGX enclave
    let enclave = neo3_sgx::init_enclave("enclave.signed.so")?;
    let sgx_wallet = SgxWallet::new(enclave);
    
    // Create account securely inside SGX enclave
    let secure_account = sgx_wallet.create_secure_account()?;
    
    // Build transaction (using core SDK)
    let mut tx_builder = TransactionBuilder::new();
    tx_builder.script(/* ... */);
    
    // Sign transaction securely inside SGX enclave
    let witness = sgx_wallet.sign_transaction(&tx_builder.build())?;
    
    Ok(())
}
```

## Migration Path

1. **Stage 1**: Create trait interfaces in core SDK with feature flag
2. **Stage 2**: Move SGX implementations to separate crate
3. **Stage 3**: Update examples and documentation
4. **Stage 4**: Release both crates together

## Benefits

1. **Simplified Core SDK**: Core functionality is not cluttered with SGX-specific code
2. **Better User Experience**: Users only need to import SGX support if they need it
3. **Reduced Dependency Issues**: Fewer potential dependency conflicts
4. **Clearer Architecture**: Well-defined interfaces between components
5. **Easier Maintenance**: SGX-specific changes don't affect core SDK
6. **Better Testing**: Components can be tested independently

## Compatibility Concerns

To maintain backward compatibility, the core SDK can include a compatibility layer that:

1. Provides deprecated re-exports of SGX functionality with warnings
2. Guides users to migrate to the new modular approach
3. Will be removed in a future major version update 