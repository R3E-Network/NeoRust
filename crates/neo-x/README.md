# Neo X

Cross-chain and extension functionality for the NeoRust SDK.

This crate provides extended functionality for the Neo N3 blockchain, including:

- EVM compatibility layer
- Cross-chain bridges
- Interoperability with other blockchains
- Extended protocol support
- Advanced transaction types
- Custom extensions

## Usage

```rust
use neo_x::evm::{EvmProvider, EvmTransaction};
use neo_x::bridge::Bridge;
use neo_types::ScriptHash;
use std::str::FromStr;

// Use EVM compatibility layer
let evm_provider = EvmProvider::new("https://neo-evm-rpc.example.com");
let tx_hash = evm_provider.send_transaction(tx).await?;

// Use cross-chain bridge
let bridge = Bridge::new(provider);
let token_hash = ScriptHash::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap();
let bridged_tx = bridge.bridge_token(token_hash, destination_chain, amount).await?;
```

For more information, see the [NeoRust documentation](https://docs.rs/neo3).
