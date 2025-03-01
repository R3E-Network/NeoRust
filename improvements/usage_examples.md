# NeoRust SDK Feature Usage Examples

This document provides practical examples of how to use the new feature flag system in the NeoRust SDK.

## Basic Usage Examples

### Minimal Wallet Application

For a simple application that only needs to manage wallets and doesn't need to interact with the blockchain:

```toml
# Cargo.toml
[dependencies]
neo3 = { version = "0.5.0", default-features = false, features = ["wallet"] }
```

```rust
// main.rs
use neo3::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new wallet
    let wallet = Wallet::new("my-wallet.json")?;
    
    // Generate a new account
    let account = Account::create()?;
    wallet.add_account(account)?;
    
    // Save the wallet
    wallet.save()?;
    
    println!("Created new wallet with address: {}", wallet.default_account()?.address());
    Ok(())
}
```

### Read-Only Blockchain Client

For an application that only needs to read data from the blockchain:

```toml
# Cargo.toml
[dependencies]
neo3 = { version = "0.5.0", default-features = false, features = ["http-client"] }
async-std = { version = "1.12.0", features = ["attributes"] }
```

```rust
// main.rs
use neo3::prelude::*;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client to interact with a Neo N3 node
    let client = HttpClient::new("https://mainnet1.neo.coz.io:443")?;
    
    // Get the current blockchain height
    let height = client.get_block_count().await?;
    println!("Current blockchain height: {}", height);
    
    // Get a block by height
    let block = client.get_block_by_index(height - 1).await?;
    println!("Latest block hash: {}", block.hash());
    
    Ok(())
}
```

### Full-Featured dApp

For a decentralized application that needs to interact with smart contracts, create transactions, and manage wallets:

```toml
# Cargo.toml
[dependencies]
neo3 = { version = "0.5.0", features = ["wallet", "transaction", "contract", "http-client", "nep17"] }
async-std = { version = "1.12.0", features = ["attributes"] }
```

```rust
// main.rs
use neo3::prelude::*;
use std::str::FromStr;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load wallet from file
    let wallet = Wallet::load("my-wallet.json", Some("password"))?;
    let account = wallet.default_account()?;
    
    // Create a client to interact with a Neo N3 node
    let client = HttpClient::new("https://mainnet1.neo.coz.io:443")?;
    
    // Create a NEP-17 token instance for GAS
    let gas_token = Nep17Token::new(
        Hash160::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?,
        client.clone(),
    );
    
    // Check the wallet's GAS balance
    let gas_balance = gas_token.balance_of(&account.address()).await?;
    println!("GAS balance: {}", gas_balance);
    
    // Create and send a transaction to invoke a contract
    let contract_hash = Hash160::from_str("85a33c53f896c15bd817295964c5803b76efb8c8")?;
    let builder = TransactionBuilder::new(client.clone());
    
    let tx = builder
        .script(
            ScriptBuilder::new()
                .contract_call(
                    contract_hash,
                    "transfer",
                    &[
                        ContractParameter::hash160(account.address()),
                        ContractParameter::hash160(
                            Address::from_str("NbnjKGMBJzJ7y5K7DgGZEMqnTEkVrKs8xC")?.script_hash(),
                        ),
                        ContractParameter::integer(1000000), // 0.01 GAS (8 decimals)
                        ContractParameter::any(None),
                    ],
                )
                .to_array()
        )
        .signers(vec![
            Signer::caller()
                .with_scope(WitnessScope::CALLED_BY_ENTRY)
        ])
        .sign(&account)?;
    
    // Send the transaction
    let tx_hash = client.send_raw_transaction(&tx).await?;
    println!("Transaction sent: {}", tx_hash);
    
    Ok(())
}
```

## Advanced Usage Examples

### Web Assembly (WASM) Application

For a web application using WebAssembly:

```toml
# Cargo.toml
[dependencies]
neo3 = { version = "0.5.0", default-features = false, features = ["wallet", "transaction", "http-client", "wasm"] }
wasm-bindgen = "0.2"
```

```rust
// lib.rs
use wasm_bindgen::prelude::*;
use neo3::prelude::*;

#[wasm_bindgen]
pub async fn create_wallet() -> Result<String, JsValue> {
    // Create a new wallet
    let wallet = Wallet::new("browser-wallet.json").map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    // Generate a new account
    let account = Account::create().map_err(|e| JsValue::from_str(&e.to_string()))?;
    wallet.add_account(account).map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    // Return the address
    Ok(wallet.default_account()
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .address()
        .to_string())
}
```

### Secure Enclave Integration

For applications requiring Intel SGX secure enclave integration:

```toml
# Cargo.toml
[dependencies]
neo3 = { version = "0.5.0", default-features = false, features = ["sgx", "wallet", "transaction"] }
```

```rust
// main.rs
use neo3::prelude::*;
use neo3::sgx::SgxEnclave;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the SGX enclave
    let enclave = SgxEnclave::init("enclave.so")?;
    
    // Create a secure wallet inside the enclave
    let secure_wallet = enclave.create_wallet("sgx-wallet.json")?;
    
    // Generate a new account inside the enclave
    let account_id = secure_wallet.create_account()?;
    
    // Get the public address (private key remains in the enclave)
    let address = secure_wallet.get_address(account_id)?;
    println!("Created secure wallet with address: {}", address);
    
    // Sign a transaction using the enclave (private key never leaves the enclave)
    let script = ScriptBuilder::new()
        // ... build transaction script
        .to_array();
    
    let signature = secure_wallet.sign_data(account_id, &script)?;
    println!("Generated signature: {}", hex::encode(signature));
    
    Ok(())
}
```

## Feature Combinations

Here are some common feature combinations for different types of applications:

| Application Type | Recommended Features |
|------------------|---------------------|
| Simple wallet tool | `wallet`, `crypto-standard` |
| Block explorer | `http-client` |
| Token transfer dApp | `wallet`, `transaction`, `http-client`, `nep17` |
| NFT marketplace | `wallet`, `transaction`, `http-client`, `nep11` |
| Contract development | `wallet`, `transaction`, `contract`, `http-client` |
| Web dApp (WASM) | `wallet`, `transaction`, `http-client`, `wasm` |
| Secure wallet | `wallet`, `transaction`, `sgx` |

## Compile Time Optimization

By selecting only the features you need, you can significantly reduce compile times and binary sizes. Here are some examples:

| Feature Set | Compile Time | Binary Size |
|-------------|--------------|-------------|
| All features | 2m 15s | 8.2 MB |
| Only wallet | 38s | 1.1 MB |
| wallet + transaction | 1m 5s | 2.3 MB |
| http-client only | 42s | 1.5 MB |

*Note: These are example values. Actual times and sizes will vary based on hardware and other factors.* 