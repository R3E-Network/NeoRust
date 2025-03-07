# Neo Contract

Smart contract interaction utilities for the NeoRust SDK.

This crate provides utilities for interacting with Neo N3 smart contracts, including:

- Standard contract interfaces (NEP-17, NEP-11)
- Native contract wrappers (NeoToken, GasToken, PolicyContract, etc.)
- Contract management utilities
- Name service integration
- Famous contract implementations

## Usage

```rust
use neo_contract::{NeoToken, GasToken, PolicyContract};
use neo_protocol::account::Account;
use std::str::FromStr;

// Interact with the Neo native token contract
let account = Account::from_str("NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj").unwrap();
let neo_token = NeoToken::new(provider);
let balance = neo_token.balance_of(&account).await?;

// Interact with the Gas native token contract
let gas_token = GasToken::new(provider);
let gas_balance = gas_token.balance_of(&account).await?;

// Interact with the Policy contract
let policy = PolicyContract::new(provider);
let fee_per_byte = policy.get_fee_per_byte().await?;
```

For more information, see the [NeoRust documentation](https://docs.rs/neo3).
