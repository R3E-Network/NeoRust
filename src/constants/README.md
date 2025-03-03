# Neo N3 Constants and Reference Data

This module provides essential constants and reference data for working with the Neo N3 blockchain.

## Native Contracts

The `native_contracts` module provides script hash constants for all native contracts built into the Neo N3 blockchain.

```rust
use neo::constants::native_contracts::{NEO_TOKEN, GAS_TOKEN};

// Use the native contract script hashes directly
println!("NEO Token: {}", NEO_TOKEN);
println!("GAS Token: {}", GAS_TOKEN);
```

## Famous Contracts

To access information about well-known contracts deployed on Neo N3 mainnet and testnet, use the `neo_contract::famous::contracts` module:

```rust
use neo::neo_contract::famous::contracts::{get_famous_contracts, Network};

// Get all mainnet contracts
let mainnet_contracts = get_famous_contracts(Network::Mainnet);

// Get information about Flamingo Finance
let flamingo = flamingo_flamingo_finance();
println!("Flamingo Finance contract: {}", flamingo.script_hash);
```

## NeoFS Endpoints

The NeoFS client provides constants for standard endpoints:

```rust
use neo::fs::client::{
    DEFAULT_MAINNET_ENDPOINT, 
    DEFAULT_TESTNET_ENDPOINT,
    DEFAULT_MAINNET_HTTP_GATEWAY,
    DEFAULT_TESTNET_HTTP_GATEWAY
};

// Use in NeoFS client configuration
let config = NeoFSConfig {
    endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
    // ...other config options
};
```

## Reference Data

For a comprehensive list of network endpoints, contracts, and other reference data, see the [`src/neo_fs/reference_data.md`](../neo_fs/reference_data.md) file.

This file contains:
- Native contract addresses
- Famous contract addresses
- RPC endpoints for mainnet and testnet
- NeoFS endpoints
- Block explorer URLs
- Additional resources
