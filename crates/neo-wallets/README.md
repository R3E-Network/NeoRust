# Neo Wallets

Wallet management for the NeoRust SDK.

This crate provides wallet functionality for the Neo N3 blockchain, including:

- Wallet creation and management
- NEP-6 wallet standard support
- Account management
- Key storage and retrieval
- Wallet signing capabilities
- Hardware wallet support (Yubikey)
- Wallet backup and recovery

## Usage

```rust
use neo_wallets::{Wallet, Account, NEP6Account};
use neo_types::ScriptHash;
use std::str::FromStr;

// Create a new wallet
let mut wallet = Wallet::new("my-wallet");

// Add an account to the wallet
let account = Account::from_wif("KwkUAF4y4UQwQGY8RkRtddHX8FgDgpwdH2RYKQcnAi7fFkzYQUV3").unwrap();
wallet.add_account(account);

// Save the wallet to a file
wallet.save_to_file("wallet.json").unwrap();

// Load a wallet from a file
let loaded_wallet = Wallet::load_from_file("wallet.json").unwrap();

// Sign a transaction with the wallet
let signature = wallet.sign_with_account(message, account_address).unwrap();
```

For more information, see the [NeoRust documentation](https://docs.rs/neo3).
