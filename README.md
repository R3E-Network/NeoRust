# NeoRust

[![Rust](https://github.com/R3E-Network/NeoRust/actions/workflows/rust.yml/badge.svg)](https://github.com/R3E-Network/NeoRust/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

NeoRust is a comprehensive Rust SDK for interacting with the Neo N3 blockchain. It provides a complete set of tools and utilities for building applications on the Neo ecosystem, including wallet management, transaction creation, smart contract interaction, and more.

## Features

- **Complete Neo N3 Support**: Full compatibility with the Neo N3 blockchain protocol
- **Wallet Management**: Create, manage, and secure Neo wallets with support for NEP-6 standard
- **Transaction Building**: Construct and sign various transaction types
- **Smart Contract Interaction**: Deploy and interact with smart contracts
- **RPC Client**: Connect to Neo nodes via JSON-RPC
- **NEP-17 Token Support**: Interact with NEP-17 compatible tokens
- **Neo Name Service (NNS)**: Resolve and manage domain names on the Neo blockchain
- **Cryptographic Operations**: Secure key management and cryptographic functions
- **Modular Architecture**: Well-organized codebase with clear separation of concerns

## Installation

Add NeoRust to your `Cargo.toml`:

```toml
[dependencies]
NeoRust = "0.1.0"
```

For the latest development version, you can use the Git repository:

```toml
[dependencies]
NeoRust = { git = "https://github.com/R3E-Network/NeoRust.git" }
```

## Quick Start

```rust
use NeoRust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo N3 TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443");
    let client = RpcClient::new(provider);
    
    // Get blockchain information
    let block_count = client.get_block_count().await?;
    println!("Current block height: {}", block_count);
    
    // Create a new wallet
    let wallet = Wallet::new();
    println!("New wallet created with address: {}", 
             wallet.default_account().address_or_scripthash().address());
    
    Ok(())
}
```

## Usage Examples

### Connecting to Neo Nodes

```rust
use neo::{
    neo_clients::{HttpProvider, JsonRpcProvider},
    prelude::RpcClient,
};

async fn connect_to_nodes() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to MainNet
    let mainnet_provider = HttpProvider::new("https://mainnet1.neo.org:443");
    let mainnet_client = RpcClient::new(mainnet_provider);
    
    // Connect to TestNet
    let testnet_provider = HttpProvider::new("https://testnet1.neo.org:443");
    let testnet_client = RpcClient::new(testnet_provider);
    
    // Get blockchain information
    let block_count = testnet_client.get_block_count().await?;
    let latest_block_hash = testnet_client.get_best_block_hash().await?;
    
    Ok(())
}
```

### Wallet Management

```rust
use neo::{
    neo_protocol::account::Account,
    neo_wallets::{Wallet, WalletBackup, WalletTrait},
    prelude::{NeoNetwork, ScryptParamsDef},
};

async fn manage_wallets() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new wallet
    let mut wallet = Wallet::new();
    
    // Configure wallet properties
    wallet.set_name("MyNeoWallet".to_string());
    let wallet = wallet.with_network(NeoNetwork::TestNet.to_magic());
    
    // Create and add a new account
    let new_account = Account::create()?;
    let mut wallet = wallet;
    wallet.add_account(new_account.clone());
    
    // Encrypt accounts in the wallet
    wallet.encrypt_accounts("password123");
    
    // Backup and recover wallet
    let backup_path = std::path::PathBuf::from("wallet_backup.json");
    WalletBackup::backup(&wallet, backup_path.clone())?;
    let recovered_wallet = WalletBackup::recover(backup_path)?;
    
    Ok(())
}
```

### Creating and Sending Transactions

```rust
use neo::{
    neo_clients::{HttpProvider, JsonRpcProvider},
    neo_builder::transaction::{Transaction, TransactionBuilder},
    neo_protocol::account::Account,
    prelude::{RpcClient, Signer, WalletTrait},
};

async fn create_transaction() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443");
    let client = RpcClient::new(provider);
    
    // Create accounts
    let sender = Account::create()?;
    let receiver = Account::create()?;
    
    // Build a transfer transaction
    let tx = TransactionBuilder::new()
        .version(0)
        .nonce(1234)
        .valid_until_block(client.get_block_count().await? + 100)
        .sender(sender.get_script_hash())
        .receiver(receiver.get_script_hash())
        .system_fee(1000000)
        .network_fee(1000000)
        .build();
    
    // Sign and send the transaction
    let signed_tx = tx.sign(&sender).await?;
    let tx_hash = client.send_raw_transaction(signed_tx).await?;
    
    Ok(())
}
```

### Interacting with Smart Contracts

```rust
use neo::{
    neo_clients::{HttpProvider, JsonRpcProvider},
    neo_contract::{ContractManagement, SmartContract},
    neo_types::contract::{ContractParameter, ContractParameterType},
    prelude::{RpcClient, Signer},
};

async fn interact_with_contract() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443");
    let client = RpcClient::new(provider);
    
    // Load a contract by its script hash
    let contract_hash = "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5".parse()?;
    let contract = SmartContract::new(contract_hash, client.clone());
    
    // Call a read-only method
    let result = contract.call_function("balanceOf", vec![
        ContractParameter::new_hash160("NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj".parse()?)
    ]).await?;
    
    // Invoke a contract method that changes state
    let account = Account::create()?;
    let invoke_result = contract.invoke(
        "transfer",
        vec![
            ContractParameter::new_hash160(account.get_script_hash()),
            ContractParameter::new_hash160("NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj".parse()?),
            ContractParameter::new_integer(1000),
            ContractParameter::new_any(None),
        ],
        account,
    ).await?;
    
    Ok(())
}
```

### Working with NEP-17 Tokens

```rust
use neo::{
    neo_clients::{HttpProvider, JsonRpcProvider},
    neo_contract::Nep17Contract,
    neo_protocol::account::Account,
    prelude::{RpcClient, Signer},
};

async fn work_with_nep17_tokens() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443");
    let client = RpcClient::new(provider);
    
    // Create an account
    let account = Account::create()?;
    
    // Connect to a NEP-17 token contract (e.g., NEO)
    let neo_token_hash = "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5".parse()?;
    let neo_token = Nep17Contract::new(neo_token_hash, client.clone());
    
    // Get token information
    let symbol = neo_token.symbol().await?;
    let decimals = neo_token.decimals().await?;
    let total_supply = neo_token.total_supply().await?;
    
    // Get account balance
    let balance = neo_token.balance_of(account.get_script_hash()).await?;
    
    // Transfer tokens
    let recipient = "NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj".parse()?;
    let transfer_result = neo_token.transfer(
        account.clone(),
        recipient,
        1000,
        None,
    ).await?;
    
    Ok(())
}
```

### Using Neo Name Service (NNS)

```rust
use neo::{
    neo_clients::{HttpProvider, JsonRpcProvider},
    neo_contract::NameService,
    prelude::{RpcClient, Signer},
};

async fn use_neo_name_service() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443");
    let client = RpcClient::new(provider);
    
    // Create a NameService instance
    let nns = NameService::new(client);
    
    // Resolve a domain name to a script hash
    let domain = "example.neo";
    let script_hash = nns.resolve(domain).await?;
    println!("Domain {} resolves to: {}", domain, script_hash);
    
    // Get the owner of a domain
    let owner = nns.get_owner(domain).await?;
    println!("Domain {} is owned by: {}", domain, owner);
    
    // Check if a domain is available
    let is_available = nns.is_available("newdomain.neo").await?;
    println!("Domain is available: {}", is_available);
    
    Ok(())
}
```

## Configuration

NeoRust provides configuration options for different network environments and blockchain parameters:

```rust
use neo::{
    neo_config::{NeoConfig, NeoConstants, NEOCONFIG},
    prelude::NeoNetwork,
};

fn configure_neo() {
    // Access global configuration
    let mut config = NEOCONFIG.lock().unwrap();
    
    // Set network to MainNet
    config.set_network(NeoNetwork::MainNet.to_magic()).unwrap();
    
    // Configure transaction behavior
    config.allows_transmission_on_fault = true;
    
    // Use predefined constants
    let max_tx_size = NeoConstants::MAX_TRANSACTION_SIZE;
    let mainnet_magic = NeoConstants::MAGIC_NUMBER_MAINNET;
    let testnet_magic = NeoConstants::MAGIC_NUMBER_TESTNET;
    
    // Create a custom configuration for MainNet
    let mainnet_config = NeoConfig::mainnet();
}
```

## Project Structure

NeoRust is organized into several modules:

- **neo_builder**: Transaction and script building utilities
- **neo_clients**: Neo node interaction clients (RPC and WebSocket)
- **neo_codec**: Encoding/decoding for Neo-specific data structures
- **neo_config**: Network and client configuration
- **neo_contract**: Smart contract interaction
- **neo_crypto**: Neo-specific cryptographic operations
- **neo_protocol**: Neo network protocol implementation
- **neo_types**: Core Neo ecosystem data types
- **neo_wallets**: Neo asset and account management

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Supported by [R3E Network](https://github.com/R3E-Network) and [GrantShares](https://grantshares.io/app/details/155b825697b61f9f95292c8e466f6891). Additional support is welcome.
