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

/// Trait for providers that can retrieve NEP-17 token balances
#[async_trait]
pub trait Nep17BalanceProvider {
    /// Get NEP-17 balances for a given script hash
    async fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17BalancesResponse, ProviderError>;
}
