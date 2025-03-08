use std::ops::BitXor;

use bs58::encode as bs58_encode;
use derive_more::{AsRef, Deref, Index, IndexMut, IntoIterator};
use hex::encode as hex_encode;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use neo_common::base64;

/// `Bytes` is a wrapper around a vector of bytes (`Vec<u8>`) that provides
/// utility methods for working with byte data in the Neo N3 blockchain context.
/// 
/// This struct is used to represent binary data in a format that's convenient 
/// for blockchain operations, with methods for serialization and conversion.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    AsRef,
    Deref,
    Index,
    IndexMut,
    IntoIterator,
)]
pub struct Bytes(pub Vec<u8>);