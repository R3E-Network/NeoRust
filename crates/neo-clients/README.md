# Neo Clients

Client implementations for interacting with Neo N3 blockchain nodes.

This crate provides various client implementations for connecting to and interacting with Neo N3 blockchain nodes, including:

- JSON-RPC client for communicating with Neo nodes
- WebSocket client for real-time event subscriptions
- HTTP client for standard API requests
- Provider abstractions for different connection types
- Middleware support for request/response processing

## Usage

```rust
use neo_clients::{JsonRpcProvider, RpcClient};
use std::str::FromStr;

// Create a JSON-RPC provider
let provider = JsonRpcProvider::new("https://mainnet.neoline.io:443");

// Get the latest block height
let block_height = provider.get_block_count().await?;

// Create a WebSocket client for subscriptions
let ws_client = RpcClient::new_websocket("wss://mainnet.neoline.io:4443/ws");

// Subscribe to new blocks
let subscription = ws_client.subscribe_to_new_blocks().await?;
```

For more information, see the [NeoRust documentation](https://docs.rs/neo3).
