# Neo Crypto

Cryptographic utilities for the NeoRust SDK.

This crate provides cryptographic functionality for the Neo N3 blockchain, including:

- Elliptic curve cryptography (secp256r1)
- Key pair generation and management
- Digital signatures
- Hashing algorithms (SHA256, RIPEMD160, etc.)
- Encryption and decryption
- Base58 encoding/decoding
- Secure random number generation

## Usage

```rust
use neo_crypto::{KeyPair, Secp256r1PrivateKey, HashableForVec};
use std::str::FromStr;

// Generate a new key pair
let private_key = Secp256r1PrivateKey::random();
let key_pair = KeyPair::from_private_key(&private_key);

// Sign a message
let message = b"Hello, Neo!";
let signature = key_pair.sign(message);

// Verify a signature
let is_valid = key_pair.verify(message, &signature);
assert!(is_valid);

// Hash data
let data = b"Data to hash";
let hash = data.hash_sha256();
```

For more information, see the [NeoRust documentation](https://docs.rs/neo3).
