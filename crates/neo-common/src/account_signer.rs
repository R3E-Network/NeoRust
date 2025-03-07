use std::hash::{Hash, Hasher};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use getset::{Getters, Setters};

use crate::{
    crypto_adapter::Secp256r1PublicKey,
    witness_scope::WitnessScope,
    witness_rule::WitnessRule,
    serde_utils::{
        deserialize_script_hash, deserialize_vec_script_hash,
        deserialize_vec_public_key, serialize_vec_public_key,
        serialize_script_hash, serialize_vec_script_hash
    },
};

/// A signer that represents an account.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters, Setters, Serialize, Deserialize)]
pub struct AccountSigner {
    #[serde(serialize_with = "serialize_script_hash", deserialize_with = "deserialize_script_hash")]
    #[getset(get = "pub")]
    pub(crate) signer_hash: H160,

    #[getset(get = "pub")]
    pub(crate) scopes: Vec<WitnessScope>,

    #[serde(
        serialize_with = "serialize_vec_script_hash",
        deserialize_with = "deserialize_vec_script_hash",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    #[getset(get = "pub")]
    pub(crate) allowed_contracts: Vec<H160>,

    #[serde(
        serialize_with = "serialize_vec_script_hash",
        deserialize_with = "deserialize_vec_script_hash",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    #[getset(get = "pub")]
    pub(crate) allowed_groups: Vec<Secp256r1PublicKey>,

    #[getset(get = "pub")]
    pub(crate) rules: Vec<WitnessRule>,
}

impl AccountSigner {
    /// Creates a new account signer with no witness scope.
    ///
    /// # Arguments
    ///
    /// * `script_hash` - The script hash of the account.
    pub fn none(script_hash: H160) -> Self {
        Self::new(script_hash, WitnessScope::None)
    }

    /// Creates a new account signer with CalledByEntry witness scope.
    ///
    /// # Arguments
    ///
    /// * `script_hash` - The script hash of the account.
    pub fn called_by_entry(script_hash: H160) -> Self {
        Self::new(script_hash, WitnessScope::CalledByEntry)
    }

    /// Creates a new account signer with Global witness scope.
    ///
    /// # Arguments
    ///
    /// * `script_hash` - The script hash of the account.
    pub fn global(script_hash: H160) -> Self {
        Self::new(script_hash, WitnessScope::Global)
    }

    /// Creates a new account signer.
    ///
    /// # Arguments
    ///
    /// * `script_hash` - The script hash of the account.
    /// * `scope` - The witness scope.
    pub fn new(script_hash: H160, scope: WitnessScope) -> Self {
        Self {
            signer_hash: script_hash,
            scopes: vec![scope],
            allowed_contracts: Vec::new(),
            allowed_groups: Vec::new(),
            rules: Vec::new(),
        }
    }
}
