# Neo Config

Configuration utilities for the NeoRust SDK.

This crate provides configuration settings and constants for the Neo N3 blockchain, including:

- Network configuration (MainNet, TestNet)
- Protocol constants
- Fee settings
- Default values
- Test utilities

## Usage

```rust
use neo_config::{NeoConstants, NEOCONFIG, TestConstants};

// Access network constants
let gas_token_hash = NeoConstants::GAS_TOKEN_HASH;
let neo_token_hash = NeoConstants::NEO_TOKEN_HASH;

// Use global configuration
let default_network = NEOCONFIG.default_network();

// Use test constants for unit tests
let test_private_key = TestConstants::PRIVATE_KEY;
```

For more information, see the [NeoRust documentation](https://docs.rs/neo3).
