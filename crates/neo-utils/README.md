# Neo Utils

Utility functions and helpers for the NeoRust SDK.

This crate provides various utility functions and helpers for working with the Neo N3 blockchain, including:

- Conversion utilities
- Formatting helpers
- Validation functions
- Common patterns and abstractions
- Testing utilities
- Logging and debugging helpers

## Usage

```rust
use neo_utils::{format_neo_amount, validate_address, hex_to_bytes};
use neo_types::ScriptHash;
use std::str::FromStr;

// Format Neo amounts with proper decimal places
let formatted = format_neo_amount(1000000000, 8);
assert_eq!(formatted, "10.00000000");

// Validate Neo addresses
let is_valid = validate_address("NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj");
assert!(is_valid);

// Convert hex strings to byte arrays
let bytes = hex_to_bytes("0123456789abcdef").unwrap();
```

For more information, see the [NeoRust documentation](https://docs.rs/neo3).
