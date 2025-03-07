//! NEP-17 Balance Provider Trait
//!
//! This module defines the trait for retrieving NEP-17 token balances.

use async_trait::async_trait;
use primitive_types::H160;
use crate::ProviderError;

/// Response structure for NEP-17 balances
#[derive(Debug, Clone)]
pub struct Nep17BalancesResponse {
    /// The address of the account
    pub address: String,
    /// The balances of the account
    pub balances: Vec<Nep17Balance>,
}

impl serde::Serialize for Nep17BalancesResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Nep17BalancesResponse", 2)?;
        state.serialize_field("address", &self.address)?;
        state.serialize_field("balances", &self.balances)?;
        state.end()
    }
}

/// NEP-17 balance information
#[derive(Debug, Clone)]
pub struct Nep17Balance {
    /// The asset hash
    pub asset_hash: H160,
    /// The asset name
    pub name: String,
    /// The asset symbol
    pub symbol: String,
    /// The asset decimals
    pub decimals: u8,
    /// The asset amount
    pub amount: String,
}

impl serde::Serialize for Nep17Balance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Nep17Balance", 5)?;
        
        // Serialize H160 as hex string
        let asset_hash_hex = format!("0x{}", hex::encode(self.asset_hash.as_bytes()));
        state.serialize_field("asset_hash", &asset_hash_hex)?;
        
        state.serialize_field("name", &self.name)?;
        state.serialize_field("symbol", &self.symbol)?;
        state.serialize_field("decimals", &self.decimals)?;
        state.serialize_field("amount", &self.amount)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Nep17Balance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        
        struct Nep17BalanceVisitor;
        
        impl<'de> Visitor<'de> for Nep17BalanceVisitor {
            type Value = Nep17Balance;
            
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Nep17Balance")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<Nep17Balance, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut asset_hash = None;
                let mut name = None;
                let mut symbol = None;
                let mut decimals = None;
                let mut amount = None;
                
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "asset_hash" => {
                            let asset_hash_str = map.next_value::<String>()?;
                            // Remove 0x prefix if present
                            let hash_str = asset_hash_str.strip_prefix("0x").unwrap_or(&asset_hash_str);
                            let hash_bytes = hex::decode(hash_str).map_err(de::Error::custom)?;
                            asset_hash = Some(H160::from_slice(&hash_bytes));
                        }
                        "name" => name = Some(map.next_value()?),
                        "symbol" => symbol = Some(map.next_value()?),
                        "decimals" => decimals = Some(map.next_value()?),
                        "amount" => amount = Some(map.next_value()?),
                        _ => { let _ = map.next_value::<serde::de::IgnoredAny>()?; }
                    }
                }
                
                let asset_hash = asset_hash.ok_or_else(|| de::Error::missing_field("asset_hash"))?;
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let symbol = symbol.ok_or_else(|| de::Error::missing_field("symbol"))?;
                let decimals = decimals.ok_or_else(|| de::Error::missing_field("decimals"))?;
                let amount = amount.ok_or_else(|| de::Error::missing_field("amount"))?;
                
                Ok(Nep17Balance {
                    asset_hash,
                    name,
                    symbol,
                    decimals,
                    amount,
                })
            }
        }
        
        deserializer.deserialize_map(Nep17BalanceVisitor)
    }
}

/// Trait for providers that can retrieve NEP-17 token balances
#[async_trait]
pub trait Nep17BalanceProvider {
    /// Get NEP-17 balances for a given script hash
    async fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17BalancesResponse, ProviderError>;
}

impl<'de> serde::Deserialize<'de> for Nep17BalancesResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        
        struct Nep17BalancesResponseVisitor;
        
        impl<'de> Visitor<'de> for Nep17BalancesResponseVisitor {
            type Value = Nep17BalancesResponse;
            
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Nep17BalancesResponse")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<Nep17BalancesResponse, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut address = None;
                let mut balances = None;
                
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "address" => address = Some(map.next_value()?),
                        "balances" => balances = Some(map.next_value()?),
                        _ => { let _ = map.next_value::<serde::de::IgnoredAny>()?; }
                    }
                }
                
                let address = address.ok_or_else(|| de::Error::missing_field("address"))?;
                let balances = balances.ok_or_else(|| de::Error::missing_field("balances"))?;
                
                Ok(Nep17BalancesResponse {
                    address,
                    balances,
                })
            }
        }
        
        deserializer.deserialize_map(Nep17BalancesResponseVisitor)
    }
}
