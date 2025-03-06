//! Transaction signer types for the NeoRust SDK.
//!
//! This module provides types for working with transaction signers in the Neo blockchain.

use crate::witness_rule::WitnessRule;
use crate::witness_scope::WitnessScope;
use primitive_types::H160;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::str::FromStr;
use hex;

/// A transaction signer in the Neo blockchain
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionSigner {
    /// The account that is signing
    pub account: H160,
    /// The scopes of the signature
    pub scopes: Vec<WitnessScope>,
    /// Allow fee only if the transaction has no other attributes
    pub allow_only_fee: bool,
    /// The rules for the signature
    pub rules: Vec<WitnessRule>,
}

impl Serialize for TransactionSigner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("TransactionSigner", 4)?;
        state.serialize_field("account", &format!("{:x}", self.account))?;
        state.serialize_field("scopes", &self.scopes)?;
        state.serialize_field("allow_only_fee", &self.allow_only_fee)?;
        state.serialize_field("rules", &self.rules)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TransactionSigner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            account: String,
            scopes: Vec<WitnessScope>,
            allow_only_fee: bool,
            rules: Vec<WitnessRule>,
        }

        let helper = Helper::deserialize(deserializer)?;
        let account = H160::from_str(&helper.account)
            .map_err(|_| serde::de::Error::custom("Invalid H160 format"))?;

        Ok(TransactionSigner {
            account,
            scopes: helper.scopes,
            allow_only_fee: helper.allow_only_fee,
            rules: helper.rules,
        })
    }
}

impl TransactionSigner {
    /// Create a new transaction signer
    pub fn new(account: H160, scopes: Vec<WitnessScope>, allow_only_fee: bool, rules: Vec<WitnessRule>) -> Self {
        Self {
            account,
            scopes,
            allow_only_fee,
            rules,
        }
    }

    /// Convert from another signer type
    pub fn from_signer<T>(signer: &T) -> Self
    where
        T: Signer,
    {
        Self {
            account: signer.account(),
            scopes: signer.scopes().clone(),
            allow_only_fee: signer.allow_only_fee(),
            rules: signer.rules().clone(),
        }
    }
}

/// A trait for signers in the Neo blockchain
pub trait Signer {
    /// Get the account that is signing
    fn account(&self) -> H160;
    
    /// Get the scopes of the signature
    fn scopes(&self) -> &Vec<WitnessScope>;
    
    /// Check if the signer allows fee only if the transaction has no other attributes
    fn allow_only_fee(&self) -> bool;
    
    /// Get the rules for the signature
    fn rules(&self) -> &Vec<WitnessRule>;
}
