//! The NeoRust prelude.
//!
//! This module includes the most commonly used types and traits from the Neo SDK.
//! Import this module to quickly get started with Neo Rust development.

// Re-export common traits, types and functions from the SDK
pub use neo_builder::{ScriptBuilder, TransactionBuilder};
pub use neo_clients::{HttpProvider, JsonRpcProvider, Provider, RpcClient, WebSocketProvider};
pub use neo_codec::{Decoder, Encoder, NeoSerializable};
pub use neo_config::{ConfigBuilder, NetworkType, Protocol};
pub use neo_contract::{
    nep11::Nep11Contract,
    nep17::{GasToken, Nep17Contract, NeoToken},
    ContractInvocation, SmartContract,
};
pub use neo_crypto::{
    KeyPair, NEP2, Secp256r1PrivateKey, Secp256r1PublicKey, WIF,
    base58_helper::{Base58CheckDecode, Base58CheckEncode, Base58Decode, Base58Encode},
    HashableForBytes, HashableForVec,
};
pub use neo_types::{
    address::{Address, AddressExtension},
    block::{Block, BlockHeader, MerkleProof},
    contract::{ContractParameter, ContractParameterType, StackItem, VMState},
    hash::{ScriptHash, ScriptHashExtension},
    nep17::{Nep17Balance, Nep17Transfer},
    transaction::{
        Signer, SignerScope, Transaction, TransactionAttribute, TransactionAttributeType, Witness
    },
    wallet::{Account, Wallet, WalletAccount},
    Base64Decode, Base64Encode, Bytes, BytesExtension, Hash160, Hash256, Secp256r1Signature,
};
pub use neo_utils::{SliceExtension, StringExtension};
pub use neo_wallets::{WalletError, WalletTrait};

// Re-export standard library types commonly used with the SDK
pub use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    error::Error,
    fmt,
    str::FromStr,
};

// Re-export external crates that are frequently used
pub use base64;
pub use hex;
pub use serde::{self, Deserialize, Serialize};
pub use serde_json;

// Error handling
pub use neo_error::{Error as NeoError, Result as NeoResult};
