# Neo Common

Common types and utilities shared across NeoRust SDK crates.

This crate provides shared functionality to avoid circular dependencies between other crates in the NeoRust SDK.

## Features

- Serialization utilities for Neo types
- Common error types
- Shared enums and constants
- Base64 encoding/decoding utilities
- Common traits used across multiple crates

## Usage

```rust
use neo_common::serde_utils::{serialize_h160, deserialize_h160};
use neo_common::base64::{encode, decode};
use neo_common::role::Role;
```

## License

Licensed under the MIT License.
