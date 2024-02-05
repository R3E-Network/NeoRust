#![feature(inherent_associated_types)]
#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]
#![warn(missing_docs)]
#![deny(unsafe_code, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod core;
pub mod protocol_error;

mod ext;

pub use ext::*;
use lazy_static::lazy_static;

mod rpc;
pub use rpc::*;

mod toolbox;
pub use toolbox::*;

/// Crate utilities and type aliases
mod utils;
pub use utils::*;

/// Errors
mod errors;
pub use errors::{MiddlewareError, ProviderError, RpcError};

mod middleware;

pub use middleware::Middleware;

#[allow(deprecated)]
pub use test_provider::{MAINNET, TESTNET};

lazy_static! {
	pub static ref HTTP_PROVIDER: Provider<Http> =
		Provider::<Http>::try_from(std::env::var("ENDPOINT").unwrap().as_str()).unwrap();
}

#[allow(missing_docs)]
/// Pre-instantiated Infura HTTP clients which rotate through multiple API keys
/// to prevent rate limits
pub mod test_provider {
	use super::*;
	use crate::Http;
	use once_cell::sync::Lazy;
	use std::{convert::TryFrom, iter::Cycle, slice::Iter, sync::Mutex};

	// List of infura keys to rotate through so we don't get rate limited
	const INFURA_KEYS: &[&str] = &["15e8aaed6f894d63a0f6a0206c006cdd"];

	pub static MAINNET: Lazy<TestProvider> =
		Lazy::new(|| TestProvider::new(INFURA_KEYS, "mainnet"));
	pub static TESTNET: Lazy<TestProvider> =
		Lazy::new(|| TestProvider::new(INFURA_KEYS, "testnet"));

	#[derive(Debug)]
	pub struct TestProvider {
		network: String,
		keys: Mutex<Cycle<Iter<'static, &'static str>>>,
	}

	impl TestProvider {
		pub fn new(keys: &'static [&'static str], network: impl Into<String>) -> Self {
			Self { keys: keys.iter().cycle().into(), network: network.into() }
		}

		pub fn url(&self) -> String {
			let Self { network, keys } = self;
			let key = keys.lock().unwrap().next().unwrap();
			format!("https://{network}.infura.io/v3/{key}")
		}

		pub fn provider(&self) -> Provider<Http> {
			Provider::try_from(self.url().as_str()).unwrap()
		}

		#[cfg(feature = "ws")]
		pub async fn ws(&self) -> Provider<crate::Ws> {
			let url = format!(
				"wss://{}.infura.io/ws/v3/{}",
				self.network,
				self.keys.lock().unwrap().next().unwrap()
			);
			Provider::connect(url.as_str()).await.unwrap()
		}
	}
}
