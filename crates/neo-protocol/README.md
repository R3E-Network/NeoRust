# Neo Protocol

Neo blockchain protocol implementation for the NeoRust SDK.

This crate provides core protocol functionality for interacting with the Neo N3 blockchain, including:

- Account management and wallet functionality
- Transaction construction and signing
- Smart contract interaction
- Network protocol implementations
- Response types for RPC calls
- NEP-2 password-protected key format support

## Usage

```rust
use neo_protocol::{Account, AccountTrait};
use neo_types::ScriptHash;
use std::str::FromStr;

// Create an account from a WIF or address
let account = Account::from_wif("KwkUAF4y4UQwQGY8RkRtddHX8FgDgpwdH2RYKQcnAi7fFkzYQUV3").unwrap();
let address = account.address();

// Get account information
let script_hash = account.script_hash();
let public_key = account.public_key();

// Sign data with the account
let signature = account.sign(b"message to sign").unwrap();
let is_valid = account.verify(b"message to sign", &signature).unwrap();
```

For more information, see the [NeoRust documentation](https://docs.rs/neo3).
