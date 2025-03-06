//! Implementation of the Nep17BalanceProvider trait for RpcClient
//!
//! This module provides the implementation of the Nep17BalanceProvider trait
//! for the RpcClient struct, allowing it to fetch NEP-17 balances.

use async_trait::async_trait;
use primitive_types::H160;

use crate::rpc::rpc_client::RpcClient;
use crate::JsonRpcProvider;
use neo_common::{Nep17BalanceProvider, Nep17BalancesResponse, Nep17Balance, ProviderError};

#[async_trait]
impl<P> Nep17BalanceProvider for RpcClient<P>
where
    P: JsonRpcProvider + Send + Sync,
{
    async fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17BalancesResponse, ProviderError> {
        // Call the existing implementation
        let response = self.request("getnep17balances", vec![script_hash.to_string()]).await?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HttpProvider;
    use std::str::FromStr;

    #[tokio::test]
    #[ignore] // Ignore by default as it requires a network connection
    async fn test_get_nep17_balances() {
        let provider = RpcClient::new(HttpProvider::new("http://seed1.neo.org:10332"));
        let script_hash = H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap();
        
        let result = provider.get_nep17_balances(script_hash).await;
        assert!(result.is_ok());
    }
}
