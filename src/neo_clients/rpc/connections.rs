use std::fmt::Debug;

use async_trait::async_trait;
use auto_impl::auto_impl;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::neo_clients::errors::ProviderError;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[auto_impl(&, Box, Arc)]
/// Trait which must be implemented by data transports to be used with the Neo
/// JSON-RPC provider.
pub trait JsonRpcProvider: Debug + Send + Sync {
	/// A JSON-RPC Error
	type Error: Into<ProviderError>;

	/// Sends a request with the provided JSON-RPC and parameters serialized as JSON
	async fn fetch<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
	where
		T: Debug + Serialize + Send + Sync,
		R: DeserializeOwned + Send;
        
    /// Sends a request with pre-serialized parameters and deserializes the response
    async fn request<T: for<'de> serde::Deserialize<'de> + Send>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, ProviderError> {
        // Default implementation calls fetch with the value
        self.fetch(method, params)
            .await
            .map_err(Into::into)
    }
}
