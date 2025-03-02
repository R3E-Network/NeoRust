# Using Neo N3 Networks

NeoRust provides utilities to easily work with different Neo N3 networks (MainNet, TestNet, PrivateNet). This document explains how to use these features to build applications that can seamlessly switch between networks.

## Network Constants

The SDK includes network constants with important information:

```rust
use neo::neo_utils::constants;

// Network magic numbers
let mainnet_magic = constants::network_magic::MAINNET; // 860833102
let testnet_magic = constants::network_magic::TESTNET; // 894710606

// RPC endpoints
let mainnet_endpoints = constants::rpc_endpoints::mainnet::ALL;
let testnet_endpoints = constants::rpc_endpoints::testnet::ALL;

// Contract addresses
let neo_token_mainnet = constants::contracts::mainnet::NEO_TOKEN;
let gas_token_mainnet = constants::contracts::mainnet::GAS_TOKEN;

let neo_token_testnet = constants::contracts::testnet::NEO_TOKEN;
let gas_token_testnet = constants::contracts::testnet::GAS_TOKEN;
```

## Network Utilities

The SDK provides network utilities to simplify working with different networks:

```rust
use neo::prelude::*;
use neo::neo_utils::network::NeoNetwork;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client for MainNet
    let mainnet_client = NeoNetwork::MainNet.create_client()?;
    
    // Create a client for TestNet
    let testnet_client = NeoNetwork::TestNet.create_client()?;
    
    // Get contract hash for a known contract
    let neo_contract_hash = NeoNetwork::MainNet.get_contract_hash("neo")
        .expect("NEO contract should exist");
    
    // Get network magic
    let mainnet_magic = NeoNetwork::MainNet.get_magic();
    
    Ok(())
}
```

## Switching Between Networks

You can create network-aware applications that can switch between networks:

```rust
use neo::prelude::*;
use neo::neo_utils::network::{NeoNetwork, NetworkToken};

async fn check_balance(network: NeoNetwork, address: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create network-specific GAS token
    let gas_token = NetworkToken::new(network, "gas")?;
    
    // Get token info
    let token_info = gas_token.token_info().await?;
    println!("Token: {} ({})", token_info.name, token_info.symbol);
    
    // Check balance
    let (balance, symbol, decimals) = gas_token.balance_of(address).await?;
    let formatted_balance = gas_token.format_balance(balance, decimals);
    println!("Balance: {} {}", formatted_balance, symbol);
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check balance on MainNet
    check_balance(NeoNetwork::MainNet, "NVkg1yRMrTyY6QFnEkpP4WUFaviE1gFa3g").await?;
    
    // Check balance on TestNet
    check_balance(NeoNetwork::TestNet, "NVkg1yRMrTyY6QFnEkpP4WUFaviE1gFa3g").await?;
    
    Ok(())
}
```

## Network Builder

For more complex scenarios, you can use the `NetworkBuilder` to customize network settings:

```rust
use neo::prelude::*;
use neo::neo_utils::network::{NeoNetwork, NetworkBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a custom network configuration
    let custom_builder = NetworkBuilder::new(NeoNetwork::MainNet)
        .endpoints(vec![
            "https://my-custom-node.example.com:10332".to_string(),
            "https://backup-node.example.com:10332".to_string(),
        ])
        .magic(860833102); // MainNet magic
    
    // Build a client with the custom configuration
    let client = custom_builder.build_client()?;
    
    // Use the client
    let block_count = client.get_block_count().await?;
    println!("Current block: {}", block_count);
    
    Ok(())
}
```

## Smart Contract Interactions Across Networks

You can interact with smart contracts on different networks:

```rust
use neo::prelude::*;
use neo::neo_utils::network::{NeoNetwork, get_network_contract};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get NEO contract on MainNet
    let neo_contract_mainnet = get_network_contract(NeoNetwork::MainNet, "neo")?;
    
    // Get NEO contract on TestNet
    let neo_contract_testnet = get_network_contract(NeoNetwork::TestNet, "neo")?;
    
    // Invoke functions on MainNet
    let mainnet_result = neo_contract_mainnet.test_invoke("totalSupply", vec![]).await?;
    if let Some(item) = mainnet_result.stack.first() {
        if let Some(supply) = item.get_int() {
            println!("MainNet NEO Total Supply: {}", supply);
        }
    }
    
    // Invoke functions on TestNet
    let testnet_result = neo_contract_testnet.test_invoke("totalSupply", vec![]).await?;
    if let Some(item) = testnet_result.stack.first() {
        if let Some(supply) = item.get_int() {
            println!("TestNet NEO Total Supply: {}", supply);
        }
    }
    
    Ok(())
}
```

## Creating Token-Aware Applications

The `NetworkToken` utility simplifies working with tokens across networks:

```rust
use neo::prelude::*;
use neo::neo_utils::network::{NeoNetwork, NetworkToken};
use std::error::Error;

async fn transfer_token(
    network: NeoNetwork,
    token_name: &str,
    from_account: Account,
    to_address: &str,
    amount: f64,
) -> Result<String, Box<dyn Error>> {
    // Get token info
    let token = NetworkToken::new(network, token_name)?;
    let info = token.token_info().await?;
    
    // Get client for network
    let client = network.create_client()?;
    
    // Convert amount to raw value
    let raw_amount = (amount * 10_f64.powi(info.decimals as i32)) as i64;
    
    // Create recipient address hash
    let to_address_obj = Address::from_str(to_address)?;
    let to_script_hash = to_address_obj.script_hash();
    
    // Create parameters for transfer
    let params = vec![
        ContractParameter::hash160(&from_account.script_hash()), // from
        ContractParameter::hash160(&to_script_hash),             // to
        ContractParameter::integer(raw_amount),                 // amount
        ContractParameter::any(),                               // data
    ];
    
    // Build and send the transaction
    // ... (transaction building code) ...
    
    Ok("transaction_hash".to_string())
}
```

## Network-Aware CLI Applications

You can build CLI applications that support different networks:

```rust
use neo::prelude::*;
use neo::neo_utils::network::NeoNetwork;
use std::env;

fn parse_network(network_str: &str) -> Result<NeoNetwork, Box<dyn std::error::Error>> {
    match network_str.to_lowercase().as_str() {
        "mainnet" => Ok(NeoNetwork::MainNet),
        "testnet" => Ok(NeoNetwork::TestNet),
        _ => Err(format!("Unknown network: {}", network_str).into()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        println!("Usage: {} [network] [command]", args[0]);
        println!("Networks: mainnet, testnet");
        return Ok(());
    }
    
    let network = parse_network(&args[1])?;
    let command = &args[2];
    
    match command.as_str() {
        "info" => {
            let client = network.create_client()?;
            let version = client.get_version().await?;
            println!("Connected to {} ({})", network, version.user_agent);
        }
        // ... other commands ...
        _ => println!("Unknown command: {}", command),
    }
    
    Ok(())
}
```

## Private Networks

For private networks, you can use `NeoNetwork::PrivateNet` and customize the settings:

```rust
use neo::prelude::*;
use neo::neo_utils::network::{NeoNetwork, NetworkBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure private network
    let private_net = NetworkBuilder::new(NeoNetwork::PrivateNet)
        .endpoints(vec!["http://localhost:10332".to_string()])
        .magic(5195086); // Custom magic number
    
    // Create client
    let client = private_net.build_client()?;
    
    // Use the client
    let block_count = client.get_block_count().await?;
    println!("Private network block height: {}", block_count);
    
    Ok(())
}
```

## Best Practices

1. **Use Network Enums**: Use the `NeoNetwork` enum to represent networks rather than hardcoding network-specific values.

2. **Graceful Fallbacks**: When connections fail, try multiple endpoints before giving up.

3. **Network-Specific Contracts**: Be aware that some contracts exist only on specific networks.

4. **Environment Configuration**: Use environment variables to configure which network to use in production.

5. **Test on TestNet First**: Always test your application on TestNet before deploying to MainNet.

6. **Error Handling**: Handle network-specific errors properly and provide clear error messages. 