# WebSocket Support

NeoRust SDK provides WebSocket support for real-time notifications and events from Neo N3 nodes. This enables applications to receive immediate updates about new blocks, transaction confirmations, and smart contract notifications.

## Overview

WebSocket connections are persistent, bidirectional communication channels that allow for efficient real-time data exchange between clients and servers. In the context of Neo N3, WebSockets offer several advantages over traditional HTTP requests:

- **Real-time Updates**: Receive immediate notifications for new blocks and transactions
- **Reduced Latency**: Eliminate polling overhead for faster responses
- **Lower Network Traffic**: More efficient than repeated HTTP requests
- **Event-Driven Architecture**: Build responsive applications that react to blockchain events

## Setting Up WebSocket Support

To use WebSocket features, enable the `websocket` feature in your `Cargo.toml`:

```toml
[dependencies]
neo3 = { version = "0.1.3", features = ["websocket", "crypto-standard"] }
```

## Connecting to a WebSocket Endpoint

To establish a WebSocket connection to a Neo N3 node:

```rust
use neo3::prelude::*;
use neo3::neo_clients::WsProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a Neo N3 WebSocket endpoint
    let ws_url = "wss://testnet1.neo.org:60002/ws"; // Use an actual Neo N3 WebSocket endpoint
    let provider = WsProvider::connect(ws_url).await?;
    
    // Use the provider for subscriptions or RPC calls...
    
    Ok(())
}
```

## Subscribing to Events

### Block Subscriptions

To subscribe to new blocks:

```rust
// Create a subscription for new blocks
let mut block_subscription = provider.subscribe_blocks().await?;

// Process incoming blocks
while let Some(block) = block_subscription.next().await {
    println!("New block received: {}", block.hash);
    println!("Block height: {}", block.index);
    // Process block...
}
```

### Contract Notifications

To subscribe to contract notifications:

```rust
// Contract hash to monitor
let contract_hash = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"; // NEO token

// Create a subscription for contract notifications
let mut notification_subscription = provider.subscribe_contract_notifications(contract_hash).await?;

// Process incoming notifications
while let Some(notification) = notification_subscription.next().await {
    println!("Notification from contract: {}", notification.contract);
    println!("Event name: {}", notification.event_name);
    // Process notification...
}
```

### Transaction Subscriptions

To subscribe to new transactions:

```rust
// Create a subscription for new transactions
let mut tx_subscription = provider.subscribe_transactions().await?;

// Process incoming transactions
while let Some(tx) = tx_subscription.next().await {
    println!("New transaction received: {}", tx.hash);
    // Process transaction...
}
```

## Using WebSocket for RPC Calls

The `WsProvider` implements the `JsonRpcProvider` trait, which means you can use it for regular RPC calls as well:

```rust
// Create an RPC client with WebSocket transport
let client = RpcClient::new(provider);

// Use the client as usual
let block_count = client.get_block_count().await?;
println!("Current block height: {}", block_count);
```

## Error Handling and Reconnection

WebSocket connections may sometimes disconnect due to network issues or server maintenance. It's important to handle these scenarios gracefully:

```rust
use std::time::Duration;

// Example of connection handling with retry logic
async fn connect_with_retry(url: &str, max_retries: usize) -> Result<WsProvider, Box<dyn std::error::Error>> {
    let mut retries = 0;
    
    loop {
        match WsProvider::connect(url).await {
            Ok(provider) => return Ok(provider),
            Err(e) => {
                if retries >= max_retries {
                    return Err(Box::new(e));
                }
                
                retries += 1;
                println!("Connection failed, retrying ({}/{}): {}", retries, max_retries, e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}
```

## Available WebSocket Endpoints

Neo N3 MainNet and TestNet have WebSocket endpoints that support real-time notifications:

- **MainNet**: wss://mainnet1.neo.org:60002/ws, wss://mainnet2.neo.org:60002/ws
- **TestNet**: wss://testnet1.neo.org:60002/ws, wss://testnet2.neo.org:60002/ws

Note that not all public Neo N3 nodes may have WebSocket enabled. Check with the node provider for WebSocket support details.

## Best Practices

1. **Implement reconnection logic** to handle disconnections
2. **Add timeout handling** for operations that may not complete
3. **Set up error handling** to recover from failed subscriptions
4. **Consider message buffering** for high-volume notifications
5. **Use separate subscriptions** for different event types

## Examples

For complete examples of WebSocket usage, see:

- [Block Subscription Example](https://github.com/R3E-Network/NeoRust/blob/master/examples/websocket/examples/block_subscription.rs) 