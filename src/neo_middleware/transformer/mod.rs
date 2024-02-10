use std::string::ParseError;
mod middleware;
pub use middleware::TransformerMiddleware;
use neo::prelude::Transaction;
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors thrown from the types that implement the `Transformer` trait.
pub enum TransformerError {
	#[error("The field `{0}` is missing")]
	MissingField(String),

	#[error(transparent)]
	AbiParseError(#[from] ParseError),
}

/// `Transformer` is a trait to be implemented by a proxy wallet, eg. [`DsProxy`], that intends to
/// intercept a transaction request and transform it into one that is instead sent via the proxy
/// contract.
pub trait Transformer: Send + Sync + std::fmt::Debug {
	/// Transforms a [`transaction request`] into one that can be broadcasted and execute via the
	/// proxy contract.
	///
	/// [`transaction request`]: struct@neo_types::Transaction
	fn transform(&self, tx: &mut Transaction) -> Result<(), TransformerError>;
}
