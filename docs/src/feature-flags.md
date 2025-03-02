# Feature Flags

NeoRust uses feature flags to control which components are included in your build. This allows you to only include the features you need, reducing build times and binary sizes.

## Available Features

Here's a list of available features:

| Feature Flag | Description |
|--------------|-------------|
| `std` | Standard library support (default) |
| `crypto-standard` | Core cryptographic operations (default) |
| `transaction` | Transaction building and signing capabilities |
| `websocket` | WebSocket support for real-time notifications |
| `rest-client` | RESTful API client for Neo N3 nodes |
| `ethereum-compat` | Neo X / Ethereum compatibility |
| `ledger` | Hardware wallet support |
| `digest` | Hashing functionality |
| `sha2` | SHA-2 hashing algorithms |
| `ripemd160` | RIPEMD-160 hashing algorithm |
| `nightly` | Features only available in nightly Rust |

## Default Features

The default feature set includes:

```toml
[dependencies]
neo3 = "0.1.3" # This includes std and crypto-standard features
```

## Common Feature Combinations

### Minimal Build

A minimal build with only standard library support:

```toml
[dependencies]
neo3 = { version = "0.1.3", default-features = false, features = ["std"] }
```

### Full Build

A complete build with all features:

```toml
[dependencies]
neo3 = { version = "0.1.3", features = [
    "crypto-standard", 
    "transaction", 
    "websocket", 
    "rest-client", 
    "ethereum-compat", 
    "ledger"
] }
```

### Transaction-focused Build

For applications that need to build and sign transactions:

```toml
[dependencies]
neo3 = { version = "0.1.3", features = ["crypto-standard", "transaction"] }
```

### Real-time Applications

For applications that need real-time blockchain events:

```toml
[dependencies]
neo3 = { version = "0.1.3", features = ["crypto-standard", "websocket"] }
```

### API-focused Applications

For applications primarily accessing Neo N3 APIs:

```toml
[dependencies]
neo3 = { version = "0.1.3", features = ["crypto-standard", "rest-client"] }
```

## Feature Details

### WebSocket Support

The `websocket` feature enables real-time notifications and subscriptions from Neo N3 nodes via WebSocket connections. This is useful for event-driven applications that need immediate updates about blockchain events.

Dependencies added:
- tokio-tungstenite
- futures-util
- uuid
- tokio

### RESTful API Support

The `rest-client` feature provides a RESTful client for interacting with Neo N3 REST APIs. This offers a simpler, more intuitive way to query blockchain data compared to the JSON-RPC API.

Dependencies added:
- reqwest

### Ethereum Compatibility (Neo X)

The `ethereum-compat` feature adds support for Neo X, which is Neo's Ethereum Virtual Machine (EVM) compatibility layer. This allows NeoRust to work with Ethereum-compatible features within the Neo ecosystem.

### Hardware Wallet Support

The `ledger` feature adds support for Ledger hardware wallets, allowing NeoRust to securely sign transactions using Ledger devices.

## Conditional Compilation

You can use conditional compilation in your code to handle different feature sets:

```rust
use neo3::prelude::*;

#[cfg(feature = "websocket")]
use neo3::neo_clients::WsProvider;

#[cfg(feature = "rest-client")]
use neo3::neo_clients::RestClient;

fn main() {
    // This code runs with any feature set
    println!("NeoRust SDK");
    
    #[cfg(feature = "websocket")]
    {
        println!("WebSocket support is enabled");
        // WebSocket-specific code
    }
    
    #[cfg(feature = "rest-client")]
    {
        println!("RESTful API support is enabled");
        // RESTful API-specific code
    }
}
``` 