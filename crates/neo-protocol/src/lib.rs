//! Neo protocol - transaction, witness and block types for Neo N3
//!
//! This crate builds on [`neo_primitives`] and provides the wire format:
//! binary serialization, transactions, witnesses and signers.
//!
//! # Features
//! - `std` (default): standard library support
//!
//! # Byte order
//!
//! Neo encodes integers little-endian and prefixes variable-length data with a
//! canonical VarInt. Decoding rejects non-canonical VarInts, since accepting
//! them would let distinct byte strings produce the same transaction hash.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

pub mod attribute;
pub mod constants;
pub mod errors;
pub mod opcode;
pub mod script_builder;
pub mod serialization;
pub mod signer;
pub mod transaction;
pub mod transaction_builder;
pub mod witness;

pub use attribute::{OracleResponseCode, TransactionAttribute};
pub use errors::{Error, Result};
pub use opcode::{CallFlags, OpCode};
pub use script_builder::{ContractParameter, ScriptBuilder};
pub use serialization::{var_int_size, Reader, Serializable, Writer};
pub use signer::{Signer, WitnessScope};
pub use transaction::Transaction;
pub use transaction_builder::TransactionBuilder;
pub use witness::Witness;
