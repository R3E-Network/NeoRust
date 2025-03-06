//! RPC types and utilities for the NeoRust SDK.
//!
//! This module provides common types and traits for RPC communication.

use crate::provider_error::ProviderError;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

/// A trait for JSON-RPC providers.
#[async_trait]
pub trait JsonRpcProvider: Send + Sync + Debug {
    /// Creates a new provider with the given URL.
    fn new_provider(url: &str) -> Self where Self: Sized;
    
    /// Sends a JSON-RPC request and returns the response.
    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, ProviderError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned;
}

/// A basic implementation of a JSON-RPC provider.
#[derive(Debug, Clone)]
pub struct BasicJsonRpcProvider {
    /// The URL of the JSON-RPC provider.
    pub url: String,
}

#[async_trait]
impl JsonRpcProvider for BasicJsonRpcProvider {
    fn new_provider(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
    
    async fn request<T, R>(&self, _method: &str, _params: T) -> Result<R, ProviderError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        Err(ProviderError::NotImplemented("BasicJsonRpcProvider is a placeholder implementation".to_string()))
    }
}
