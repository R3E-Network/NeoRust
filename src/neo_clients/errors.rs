use std::{error::Error, fmt::Debug};

use thiserror::Error;

use neo::prelude::{CryptoError, JsonRpcError, TypeError};

use crate::prelude::APITrait;

#[derive(Debug, Error)]
/// An error thrown when making a call to the provider
pub enum ProviderError {
	/// An error during NNS name resolution
	#[error("nns name not found: {0}")]
	NnsError(String),
	/// Invalid reverse NNS name
	#[error("reverse nns name not pointing to itself: {0}")]
	NnsNotOwned(String),
	/// Error in underlying lib `serde_json`
	#[error(transparent)]
	SerdeJson(#[from] serde_json::Error),
	/// Error in underlying lib `hex`
	#[error(transparent)]
	HexError(#[from] hex::FromHexError),
	/// Error in underlying lib `reqwest`
	#[error(transparent)]
	HTTPError(#[from] reqwest::Error),
	/// Custom error from unknown source
	#[error("custom error: {0}")]
	CustomError(String),
	/// RPC method is not supported by this provider
	#[error("unsupported RPC")]
	UnsupportedRPC,
	/// Node is not supported by this provider
	#[error("unsupported node client")]
	UnsupportedNodeClient,
	/// Signer is not available to this provider.
	#[error("Attempted to sign a transaction with no available signer. Hint: did you mean to use a SignerMiddleware?"
    )]
	SignerUnavailable,
	#[error("Illegal state: {0}")]
	IllegalState(String),
	#[error("Invalid address")]
	InvalidAddress,
	#[error(transparent)]
	CryptoError(#[from] CryptoError),
	#[error(transparent)]
	TypeError(#[from] TypeError),
	#[error("Invalid password")]
	InvalidPassword,
}

impl PartialEq for ProviderError {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(ProviderError::SerdeJson(a), ProviderError::SerdeJson(b)) =>
				a.to_string() == b.to_string(),
			(ProviderError::HTTPError(a), ProviderError::HTTPError(b)) => a.status() == b.status(),
			(ProviderError::CustomError(a), ProviderError::CustomError(b)) => a == b,
			(ProviderError::UnsupportedRPC, ProviderError::UnsupportedRPC) => true,
			(ProviderError::UnsupportedNodeClient, ProviderError::UnsupportedNodeClient) => true,
			(ProviderError::SignerUnavailable, ProviderError::SignerUnavailable) => true,
			(ProviderError::IllegalState(a), ProviderError::IllegalState(b)) => a == b,
			(ProviderError::InvalidAddress, ProviderError::InvalidAddress) => true,
			(ProviderError::CryptoError(a), ProviderError::CryptoError(b)) => a == b,
			(ProviderError::TypeError(a), ProviderError::TypeError(b)) => a == b,
			(ProviderError::InvalidPassword, ProviderError::InvalidPassword) => true,
			_ => false,
		}
	}
}
