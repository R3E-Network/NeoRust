//! # Neo Protocol
//!
//! Core protocol types and interfaces for the Neo N3 blockchain.
//!
//! ## Overview
//!
//! The neo_protocol module provides the core types and interfaces for interacting with
//! the Neo N3 blockchain protocol. It includes:
//!
//! - Account management and address handling
//! - NEP-2 password-protected key format support
//! - Protocol error definitions and handling
//! - Response types for Neo N3 RPC calls
//! - Role-based access control definitions
//!
//! This module forms the foundation for blockchain interactions, defining the data structures
//! and interfaces that represent the Neo N3 protocol.
//!
//! ## Feature Flags
//!
//! This module supports the following feature flags:
//!
//! - **transaction**: Core transaction functionality
//! - **crypto-standard**: Required for account management and cryptographic operations
//! - **wallet**: Enables advanced account features for wallet management
//! - **http-client**: Required for response types from RPC calls
//!
//! ## Examples
//!
//! ### Working with Neo N3 accounts
//!
//! ```rust
//! use neo::prelude::*;
//!
//! // Create a new account
//! let account = Account::create().unwrap();
//! println!("Address: {}", account.address());
//! println!("Script Hash: {}", account.script_hash());
//!
//! // Create an account from a WIF (Wallet Import Format) string
//! let wif = "KwYgW8gcxj1JWJXhPSu4Fqwzfhp5Yfi42mdYmMa4XqK7NJxXUSK7";
//! let account = Account::from_wif(wif).unwrap();
//!
//! // Sign a message
//! let message = b"Hello, Neo!";
//! let signature = account.sign(message).unwrap();
//!
//! // Verify the signature
//! let is_valid = account.verify(message, &signature).unwrap();
//! assert!(is_valid);
//! ```
//!
//! ### Using NEP-2 password-protected keys
//!
//! ```rust
//! use neo::prelude::*;
//!
//! // Encrypt a private key with a password (NEP-2 format)
//! let private_key = PrivateKey::from_slice(&[/* 32 bytes */]).unwrap();
//! let password = "mySecurePassword";
//! let nep2_string = encrypt_to_nep2(&private_key, password).unwrap();
//!
//! // Decrypt a NEP-2 string back to a private key
//! let nep2_string = "6PYVPVe1fQznphjbUxXP9KZJqPMVnVwCx5s5pr5axRJ8uHkMtZg97eT5kL";
//! let password = "mySecurePassword";
//! let private_key = decrypt_from_nep2(nep2_string, password).unwrap();
//! ```

// Core protocol errors are always available
pub use protocol_error::*;

// Account management requires crypto-standard feature
#[cfg(feature = "crypto-standard")]
#[cfg_attr(docsrs, doc(cfg(feature = "crypto-standard")))]
pub use account::*;

// NEP-2 functionality requires crypto-standard feature
#[cfg(feature = "crypto-standard")]
#[cfg_attr(docsrs, doc(cfg(feature = "crypto-standard")))]
pub use nep2::*;

// Response types from RPC calls require http-client feature
#[cfg(feature = "http-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "http-client")))]
pub use responses::*;

// Role-based access control requires transaction feature
#[cfg(feature = "transaction")]
#[cfg_attr(docsrs, doc(cfg(feature = "transaction")))]
pub use role::*;

// Core protocol error is always available
mod protocol_error;

// Account management requires crypto-standard feature
#[cfg(feature = "crypto-standard")]
mod account;

// NEP-2 functionality requires crypto-standard feature
#[cfg(feature = "crypto-standard")]
mod nep2;

// Response types from RPC calls require http-client feature
#[cfg(feature = "http-client")]
mod responses;

// Role-based access control requires transaction feature
#[cfg(feature = "transaction")]
mod role;
