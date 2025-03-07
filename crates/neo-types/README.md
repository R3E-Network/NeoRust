# Neo Types

Core Neo ecosystem data types for the NeoRust SDK.

This crate provides the fundamental data types used in the Neo N3 blockchain ecosystem, including:

- Script hashes and addresses
- Contract parameters and manifests
- Block and transaction types
- Stack items and VM state
- Cryptographic types
- Serialization utilities
- NNS name types

## Usage

```rust
use neo_types::{ScriptHash, Address, ContractParameter};
use std::str::FromStr;

// Create a script hash from a string
let script_hash = ScriptHash::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap();

// Convert between script hash and address
let address = Address::from_script_hash(&script_hash);
let script_hash_from_address = address.to_script_hash();

// Create contract parameters
let param = ContractParameter::integer(42);
let string_param = ContractParameter::string("Hello, Neo!");
```

For more information, see the [NeoRust documentation](https://docs.rs/neo3).
