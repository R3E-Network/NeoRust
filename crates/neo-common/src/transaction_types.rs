//! Transaction-related type placeholders
//!
//! This module provides placeholder types for transaction-related structures
//! to help break circular dependencies between crates.

use primitive_types::H160;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Placeholder for TransactionAttribute
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TransactionAttribute {
    /// Type of the attribute
    pub r#type: String,
    /// Value of the attribute
    pub value: String,
}

/// Placeholder for TransactionSigner
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionSigner {
    /// Account of the signer
    pub account: H160,
    /// Scopes of the signer
    pub scopes: Vec<WitnessScope>,
    /// Allowed contracts
    pub allowed_contracts: Option<Vec<String>>,
    /// Allowed groups
    pub allowed_groups: Option<Vec<String>>,
    /// Rules
    pub rules: Option<Vec<WitnessRule>>,
}

impl Serialize for TransactionSigner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("TransactionSigner", 5)?;
        
        // Serialize H160 as hex string
        let account_hex = format!("0x{}", hex::encode(self.account.as_bytes()));
        state.serialize_field("account", &account_hex)?;
        
        state.serialize_field("scopes", &self.scopes)?;
        state.serialize_field("allowed_contracts", &self.allowed_contracts)?;
        state.serialize_field("allowed_groups", &self.allowed_groups)?;
        state.serialize_field("rules", &self.rules)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TransactionSigner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        
        struct TransactionSignerVisitor;
        
        impl<'de> Visitor<'de> for TransactionSignerVisitor {
            type Value = TransactionSigner;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct TransactionSigner")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<TransactionSigner, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut account = None;
                let mut scopes = None;
                let mut allowed_contracts = None;
                let mut allowed_groups = None;
                let mut rules = None;
                
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "account" => {
                            let account_str: String = map.next_value()?;
                            let account_str = account_str.trim_start_matches("0x");
                            let account_bytes = hex::decode(account_str)
                                .map_err(|e| de::Error::custom(format!("Invalid hex: {}", e)))?;
                            account = Some(H160::from_slice(&account_bytes));
                        }
                        "scopes" => {
                            scopes = Some(map.next_value()?);
                        }
                        "allowed_contracts" => {
                            allowed_contracts = Some(map.next_value()?);
                        }
                        "allowed_groups" => {
                            allowed_groups = Some(map.next_value()?);
                        }
                        "rules" => {
                            rules = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }
                
                let account = account.ok_or_else(|| de::Error::missing_field("account"))?;
                let scopes = scopes.ok_or_else(|| de::Error::missing_field("scopes"))?;
                
                Ok(TransactionSigner {
                    account,
                    scopes,
                    allowed_contracts,
                    allowed_groups,
                    rules,
                })
            }
        }
        
        deserializer.deserialize_map(TransactionSignerVisitor)
    }
}

/// Placeholder for WitnessRule
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WitnessRule {
    /// Action of the rule
    pub action: WitnessAction,
    /// Condition of the rule
    pub condition: WitnessCondition,
}

/// Placeholder for WitnessAction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WitnessAction {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
}

/// Placeholder for WitnessCondition
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WitnessCondition {
    /// Boolean condition
    Boolean(bool),
    /// Not condition
    Not(Box<WitnessCondition>),
    /// And condition
    And(Vec<WitnessCondition>),
    /// Or condition
    Or(Vec<WitnessCondition>),
    /// Script hash condition
    ScriptHash(H160),
    /// Group condition
    Group(String),
    /// Called by entry condition
    CalledByEntry,
    /// Called by contract condition
    CalledByContract(H160),
    /// Called by group condition
    CalledByGroup(String),
}

impl Serialize for WitnessCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        
        match self {
            WitnessCondition::Boolean(value) => {
                let mut state = serializer.serialize_struct("WitnessCondition", 2)?;
                state.serialize_field("type", "Boolean")?;
                state.serialize_field("value", value)?;
                state.end()
            }
            WitnessCondition::Not(condition) => {
                let mut state = serializer.serialize_struct("WitnessCondition", 2)?;
                state.serialize_field("type", "Not")?;
                state.serialize_field("condition", condition)?;
                state.end()
            }
            WitnessCondition::And(conditions) => {
                let mut state = serializer.serialize_struct("WitnessCondition", 2)?;
                state.serialize_field("type", "And")?;
                state.serialize_field("conditions", conditions)?;
                state.end()
            }
            WitnessCondition::Or(conditions) => {
                let mut state = serializer.serialize_struct("WitnessCondition", 2)?;
                state.serialize_field("type", "Or")?;
                state.serialize_field("conditions", conditions)?;
                state.end()
            }
            WitnessCondition::ScriptHash(hash) => {
                let mut state = serializer.serialize_struct("WitnessCondition", 2)?;
                state.serialize_field("type", "ScriptHash")?;
                let hash_hex = format!("0x{}", hex::encode(hash.as_bytes()));
                state.serialize_field("hash", &hash_hex)?;
                state.end()
            }
            WitnessCondition::Group(group) => {
                let mut state = serializer.serialize_struct("WitnessCondition", 2)?;
                state.serialize_field("type", "Group")?;
                state.serialize_field("group", group)?;
                state.end()
            }
            WitnessCondition::CalledByEntry => {
                let mut state = serializer.serialize_struct("WitnessCondition", 1)?;
                state.serialize_field("type", "CalledByEntry")?;
                state.end()
            }
            WitnessCondition::CalledByContract(hash) => {
                let mut state = serializer.serialize_struct("WitnessCondition", 2)?;
                state.serialize_field("type", "CalledByContract")?;
                let hash_hex = format!("0x{}", hex::encode(hash.as_bytes()));
                state.serialize_field("hash", &hash_hex)?;
                state.end()
            }
            WitnessCondition::CalledByGroup(group) => {
                let mut state = serializer.serialize_struct("WitnessCondition", 2)?;
                state.serialize_field("type", "CalledByGroup")?;
                state.serialize_field("group", group)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for WitnessCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        
        struct WitnessConditionVisitor;
        
        impl<'de> Visitor<'de> for WitnessConditionVisitor {
            type Value = WitnessCondition;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WitnessCondition")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<WitnessCondition, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut condition_type = None;
                let mut value = None;
                let mut condition = None;
                let mut conditions = None;
                let mut hash = None;
                let mut group = None;
                
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => {
                            condition_type = Some(map.next_value::<String>()?);
                        }
                        "value" => {
                            value = Some(map.next_value::<bool>()?);
                        }
                        "condition" => {
                            condition = Some(map.next_value::<WitnessCondition>()?);
                        }
                        "conditions" => {
                            conditions = Some(map.next_value::<Vec<WitnessCondition>>()?);
                        }
                        "hash" => {
                            let hash_str: String = map.next_value()?;
                            let hash_str = hash_str.trim_start_matches("0x");
                            let hash_bytes = hex::decode(hash_str)
                                .map_err(|e| de::Error::custom(format!("Invalid hex: {}", e)))?;
                            hash = Some(H160::from_slice(&hash_bytes));
                        }
                        "group" => {
                            group = Some(map.next_value::<String>()?);
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }
                
                let condition_type = condition_type.ok_or_else(|| de::Error::missing_field("type"))?;
                
                match condition_type.as_str() {
                    "Boolean" => {
                        let value = value.ok_or_else(|| de::Error::missing_field("value"))?;
                        Ok(WitnessCondition::Boolean(value))
                    }
                    "Not" => {
                        let condition = condition.ok_or_else(|| de::Error::missing_field("condition"))?;
                        Ok(WitnessCondition::Not(Box::new(condition)))
                    }
                    "And" => {
                        let conditions = conditions.ok_or_else(|| de::Error::missing_field("conditions"))?;
                        Ok(WitnessCondition::And(conditions))
                    }
                    "Or" => {
                        let conditions = conditions.ok_or_else(|| de::Error::missing_field("conditions"))?;
                        Ok(WitnessCondition::Or(conditions))
                    }
                    "ScriptHash" => {
                        let hash = hash.ok_or_else(|| de::Error::missing_field("hash"))?;
                        Ok(WitnessCondition::ScriptHash(hash))
                    }
                    "Group" => {
                        let group = group.ok_or_else(|| de::Error::missing_field("group"))?;
                        Ok(WitnessCondition::Group(group))
                    }
                    "CalledByEntry" => {
                        Ok(WitnessCondition::CalledByEntry)
                    }
                    "CalledByContract" => {
                        let hash = hash.ok_or_else(|| de::Error::missing_field("hash"))?;
                        Ok(WitnessCondition::CalledByContract(hash))
                    }
                    "CalledByGroup" => {
                        let group = group.ok_or_else(|| de::Error::missing_field("group"))?;
                        Ok(WitnessCondition::CalledByGroup(group))
                    }
                    _ => Err(de::Error::custom(format!("Unknown condition type: {}", condition_type)))
                }
            }
        }
        
        deserializer.deserialize_map(WitnessConditionVisitor)
    }
}

/// Witness scope enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WitnessScope {
    /// None scope
    None,
    /// Called by entry scope
    CalledByEntry,
    /// Custom contracts scope
    CustomContracts,
    /// Custom groups scope
    CustomGroups,
    /// Global scope
    Global,
}
