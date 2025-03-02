# RESTful API Support

NeoRust SDK provides comprehensive support for interacting with Neo N3 RESTful APIs. This allows applications to easily query blockchain data, check balances, and interact with contracts using the RESTful interface exposed by Neo N3 nodes.

## Overview

The RESTful API provides a simpler, more intuitive way to interact with the Neo N3 blockchain compared to the JSON-RPC API. Some advantages include:

- **Simplified Interface**: Cleaner API endpoints with URL parameters
- **Better Response Structure**: Consistent JSON response formats
- **HTTP Status Codes**: Proper HTTP status codes for error handling
- **Standardized Patterns**: Follows REST best practices
- **Browser-Friendly**: Can be used directly from web applications

## Setting Up RESTful API Support

To use the RESTful API features, enable the `rest-client` feature in your `Cargo.toml`:

```toml
[dependencies]
neo3 = { version = "0.1.3", features = ["rest-client", "crypto-standard"] }
```

## Creating a RESTful Client

To create a RESTful client for Neo N3:

```rust
use neo3::prelude::*;
use neo3::neo_clients::RestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a REST client for Neo N3 TestNet
    let client = RestClient::new("https://testnet1.neo.org:443/api")?;
    
    // Use the client for REST API calls...
    
    Ok(())
}
```

## Basic Blockchain Queries

### Get Current Height

```rust
// Get the current blockchain height
let height = client.get_height().await?;
println!("Current block height: {}", height);
```

### Get Block Information

```rust
// Get a block by index
let block = client.get_block_by_index(12345).await?;
println!("Block hash: {}", block.hash);
println!("Block time: {}", block.time);
println!("Block transactions: {}", block.transactions.len());

// Get a block by hash
let block = client.get_block_by_hash("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").await?;
println!("Block index: {}", block.index);
```

### Get Transaction Information

```rust
// Get a transaction by hash
let transaction = client.get_transaction("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").await?;
println!("Transaction type: {}", transaction.version);
println!("Transaction size: {}", transaction.size);
```

## Contract and Application Log Queries

### Get Contract State

```rust
// Get contract information
let contract_hash = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"; // NEO token
let contract = client.get_contract_state(contract_hash).await?;
println!("Contract name: {}", contract.manifest.name);
println!("Contract ID: {}", contract.id);
```

### Get Application Logs

```rust
// Get application logs for a transaction
let tx_hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
let app_log = client.get_application_log(tx_hash).await?;
println!("Transaction state: {}", app_log.execution.state);
println!("Gas consumed: {}", app_log.execution.gas_consumed);
println!("Notifications: {}", app_log.execution.notifications.len());
```

## Token Balance Queries

```rust
// Get token balances for an address
let address = "NVkg1yRMrTyY6QFnEkpP4WUFaviE1gFa3g";
let balances = client.get_balances(address).await?;

// Process token balances
for token in balances.balances {
    println!("Token: {} ({})", token.name, token.symbol);
    println!("Balance: {}", token.amount);
    println!("Asset hash: {}", token.asset_hash);
}
```

## Contract Invocation

```rust
use serde_json::json;

// Invoke a contract method
let contract_hash = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"; // NEO token
let method = "balanceOf";
let params = vec![
    json!({
        "type": "Hash160",
        "value": "0x5c9c3a340f4c28262e7042b908b5f7e7a4bcd7e7"
    })
];

let result = client.invoke_function(contract_hash, method, params).await?;
println!("Invocation state: {}", result.state);
println!("Gas consumed: {}", result.gas_consumed);
```

## Error Handling

The RESTful client provides detailed error information:

```rust
match client.get_balances("invalid_address").await {
    Ok(balances) => {
        // Process balances...
    },
    Err(e) => {
        match e {
            RestClientError::ApiError(msg) => println!("API error: {}", msg),
            RestClientError::HttpError(msg) => println!("HTTP error: {}", msg),
            RestClientError::SerializationError(e) => println!("Failed to deserialize response: {}", e),
            _ => println!("Other error: {}", e),
        }
    }
}
```

## Available RESTful Endpoints

Neo N3 MainNet and TestNet have RESTful API endpoints:

- **MainNet**: https://mainnet1.neo.org:443/api, https://mainnet2.neo.org:443/api
- **TestNet**: https://testnet1.neo.org:443/api, https://testnet2.neo.org:443/api

## Best Practices

1. **Use Connection Pooling**: The RESTful client reuses connections for better performance
2. **Handle Rate Limiting**: Be aware of rate limits on public nodes
3. **Implement Retries**: Add retry logic for transient failures
4. **Cache Results**: Cache frequently accessed data to reduce API calls
5. **Validate Inputs**: Clean and validate all inputs before making API calls

## Examples

For complete examples of RESTful API usage, see:

- [Token Balances Example](https://github.com/R3E-Network/NeoRust/blob/master/examples/rest_api/examples/token_balances.rs) 