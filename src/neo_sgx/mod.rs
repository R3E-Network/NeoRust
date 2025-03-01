//! # Neo SGX
//!
//! Intel SGX (Software Guard Extensions) support for the Neo N3 blockchain.
//!
//! ## Overview
//!
//! The neo_sgx module provides a secure enclave implementation for the Neo N3 blockchain
//! using Intel SGX technology. It includes:
//!
//! - Trusted enclave environment for sensitive operations
//! - Secure key management and storage
//! - Protected transaction signing
//! - Secure RPC client communications
//! - Isolated cryptographic operations
//! - Secure wallet management
//!
//! This module enables developers to build applications that leverage the security
//! properties of Intel SGX to protect sensitive operations and data when interacting
//! with the Neo N3 blockchain.
//!
//! ## Feature Flags
//!
//! This module requires the `sgx` feature to be enabled. Additional functionality
//! is available with the following feature combinations:
//!
//! - **sgx**: Core SGX integration (required for all SGX functionality)
//! - **wallet-secure**: Enhanced wallet security with SGX
//! - **crypto-advanced**: Advanced cryptography within secure enclaves
//!
//! ## Examples
//!
//! ### Creating and using a wallet in an SGX enclave
//!
//! ```rust
//! use neo::prelude::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the SGX environment
//!     let enclave = SgxEnclave::init("path/to/enclave.signed.so")?;
//!     
//!     // Create a wallet inside the secure enclave
//!     let wallet_id = enclave.create_wallet("my_secure_wallet", "password123")?;
//!     
//!     // Generate a new account inside the enclave
//!     let account_id = enclave.create_account(wallet_id)?;
//!     
//!     // Get the public address (this doesn't expose private keys)
//!     let address = enclave.get_account_address(account_id)?;
//!     println!("Secure account address: {}", address);
//!     
//!     // Sign a transaction inside the enclave
//!     let transaction_data = vec![/* transaction bytes */];
//!     let signature = enclave.sign_transaction(account_id, &transaction_data)?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Using the secure RPC client
//!
//! ```rust
//! use neo::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the SGX environment
//!     let enclave = SgxEnclave::init("path/to/enclave.signed.so")?;
//!     
//!     // Create a secure RPC client that protects API keys and connection data
//!     let client_id = enclave.create_rpc_client("https://mainnet1.neo.org:443")?;
//!     
//!     // Make RPC calls through the secure enclave
//!     let block_count = enclave.get_block_count(client_id).await?;
//!     println!("Current block count: {}", block_count);
//!     
//!     // Get network information
//!     let version = enclave.get_version(client_id).await?;
//!     println!("Node version: {}", version);
//!     
//!     Ok(())
//! }
//! ```

// App-side SGX functionality - requires sgx feature
#[cfg(feature = "sgx")]
#[cfg_attr(docsrs, doc(cfg(feature = "sgx")))]
pub mod app;

// Enclave-side SGX functionality - requires sgx feature
#[cfg(feature = "sgx")]
#[cfg_attr(docsrs, doc(cfg(feature = "sgx")))]
pub mod enclave;

// EDL definitions - requires sgx feature
#[cfg(feature = "sgx")]
#[cfg_attr(docsrs, doc(cfg(feature = "sgx")))]
pub mod edl;

// Re-export app-side SGX functionality
#[cfg(feature = "sgx")]
#[cfg_attr(docsrs, doc(cfg(feature = "sgx")))]
pub use app::*;

// Advanced cryptography in SGX - requires crypto-advanced and sgx features
#[cfg(all(feature = "sgx", feature = "crypto-advanced"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "sgx", feature = "crypto-advanced"))))]
pub mod crypto;

// Secure wallet functionality - requires wallet-secure and sgx features
#[cfg(all(feature = "sgx", feature = "wallet-secure"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "sgx", feature = "wallet-secure"))))]
pub mod wallet;
