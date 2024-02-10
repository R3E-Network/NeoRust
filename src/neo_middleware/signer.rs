use async_trait::async_trait;
use neo::prelude::{
	Address, BlockId, Bytes, Middleware, MiddlewareError, PendingTransaction, Signer, Transaction,
};
use rustc_serialize::hex::ToHex;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Clone, Debug)]
/// Middleware used for locally signing transactions, compatible with any implementer
/// of the [`Signer`] trait.
///
/// # Example
///
/// ```no_run
/// use std::convert::TryFrom;
/// use NeoRust::prelude::{Address, Http, LocalWallet, Middleware, Provider, SignerMiddleware, Transaction};
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Provider::<Http>::try_from("http://localhost:8545")
///     .expect("could not instantiate HTTP Provider");
///
/// // Transactions will be signed with the private key below and will be broadcast
/// // via the neo_sendRawTransaction API)
/// let wallet: LocalWallet = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
///     .parse()?;
///
/// let mut client = SignerMiddleware::new(provider, wallet);
///
/// // ...and sign transactions
/// let tx = Transaction::pay("r3e.neo", 100);
/// let pending_tx = client.send_transaction(tx).await?;
///
/// // You can `await` on the pending transaction to get the receipt with a pre-specified
/// // number of confirmations
/// let receipt = pending_tx.confirmations(6).await?;
///
/// // You can connect with other wallets at runtime via the `with_signer` function
/// let wallet2: LocalWallet = "cd8c407233c0560f6de24bb2dc60a8b02335c959a1a17f749ce6c1ccf63d74a7"
///     .parse()?;
///
/// // This call will be made with `wallet2` since `with_signer` takes a mutable reference.
/// let tx2 = Transaction::new()
///     .to("0xd8da6bf26964af9d7eed9e03e53415d37aa96045".parse::<Address>()?)
///     .value(200);
/// let tx_hash2 = client.send_transaction(tx2).await?;
///
/// # Ok(())
/// # }
/// ```
///
/// [`Signer`]: neo_signers::Signer
pub struct SignerMiddleware<M, S> {
	pub(crate) inner: M,
	pub(crate) signer: S,
	pub(crate) address: Address,
}

#[derive(Error, Debug)]
/// Error thrown when the client interacts with the blockchain
pub enum SignerMiddlewareError<M: Middleware, S: Signer> {
	#[error("{0}")]
	/// Thrown when the internal call to the signer fails
	SignerError(S::Error),

	#[error("{0}")]
	/// Thrown when an internal middleware errors
	MiddlewareError(M::Error),

	/// Thrown if the `nonce` field is missing
	#[error("no nonce was specified")]
	NonceMissing,
	/// Thrown if the `gas_price` field is missing
	#[error("no gas price was specified")]
	GasPriceMissing,
	/// Thrown if the `gas` field is missing
	#[error("no gas was specified")]
	GasMissing,
	/// Thrown if a signature is requested from a different address
	#[error("specified from address is not signer")]
	WrongSigner,
	/// Thrown if the signer's network_magic is different than the network_magic of the transaction
	#[error("specified network_magic is different than the signer's network_magic")]
	DifferentChainID,
}

impl<M: Middleware, S: Signer> MiddlewareError for SignerMiddlewareError<M, S> {
	type Inner = M::Error;

	fn from_err(src: M::Error) -> Self {
		SignerMiddlewareError::MiddlewareError(src)
	}

	fn as_inner(&self) -> Option<&Self::Inner> {
		match self {
			SignerMiddlewareError::MiddlewareError(e) => Some(e),
			_ => None,
		}
	}
}

// Helper functions for locally signing transactions
impl<M, S> SignerMiddleware<M, S>
where
	M: Middleware,
	S: Signer,
{
	/// Creates a new client from the provider and signer.
	/// Sets the address of this middleware to the address of the signer.
	/// The network_magic of the signer will not be set to the network magic of the provider. If the signer
	/// passed here is initialized with a different network magic, then the client may throw errors, or
	/// methods like `sign_transaction` may error.
	/// To automatically set the signer's network magic, see `new_with_provider_chain`.
	///
	/// [`Middleware`] neo_providers::Middleware
	/// [`Signer`] neo_signers::Signer
	pub fn new(inner: M, signer: S) -> Self {
		let address = signer.address();
		SignerMiddleware { inner, signer, address }
	}

	/// Signs and returns the RLP encoding of the signed transaction.
	/// If the transaction does not have a network magic set, it sets it to the signer's network magic.
	/// Returns an error if the transaction's existing network magic does not match the signer's chain
	/// id.
	// async fn sign_transaction(
	// 	&self,
	// 	mut tx: Transaction,
	// ) -> Result<Bytes, SignerMiddlewareError<M, S>> {
	// 	// compare network_magic and use signer's network_magic if the tranasaction's network_magic is None,
	// 	// return an error if they are not consistent
	// 	let network_magic = self.signer.network_magic();
	// 	match tx.network_magic() {
	// 		Some(id) if id != network_magic => return Err(SignerMiddlewareError::DifferentChainID),
	// 		None => {
	// 			tx.set_network_magic(network_magic);
	// 		},
	// 		_ => {},
	// 	}
	//
	// 	let witness = self
	// 		.signer
	// 		.get_witness(&tx)
	// 		.await
	// 		.map_err(SignerMiddlewareError::SignerError)?;
	//
	// 	// Return the raw encoded signed transaction
	// 	let witness = Witness::create(tx.get_hash_data());
	// 	Ok(tx.add_witness(&signature))
	// }

	/// Returns the client's address
	pub fn address(&self) -> Address {
		self.address.clone()
	}

	/// Returns a reference to the client's signer
	pub fn signer(&self) -> &S {
		&self.signer
	}

	/// Builds a SignerMiddleware with the given Signer.
	#[must_use]
	pub fn with_signer(&self, signer: S) -> Self
	where
		S: Clone,
		M: Clone,
	{
		let mut this = self.clone();
		this.address = signer.address();
		this.signer = signer;
		this
	}

	/// Creates a new client from the provider and signer.
	/// Sets the address of this middleware to the address of the signer.
	/// Sets the network magic of the signer to the network magic of the inner [`Middleware`] passed in,
	/// using the [`Signer`]'s implementation of with_network_magic.
	///
	/// [`Middleware`] neo_providers::Middleware
	/// [`Signer`] neo_signers::Signer
	pub async fn new_with_provider_chain(
		inner: M,
		signer: S,
	) -> Result<Self, SignerMiddlewareError<M, S>> {
		let address = signer.address();
		let network_magic = inner
			.get_network_magic()
			.await
			.map_err(|e| SignerMiddlewareError::MiddlewareError(e))?;
		let signer = signer.with_network_magic(network_magic);
		Ok(SignerMiddleware { inner, signer, address })
	}
}

#[async_trait]
impl<M, S> Middleware for SignerMiddleware<M, S>
where
	M: Middleware,
	S: Signer,
{
	type Error = SignerMiddlewareError<M, S>;
	type Provider = M::Provider;
	type Inner = M;

	fn inner(&self) -> &M {
		&self.inner
	}

	/// Returns the client's address
	fn default_sender(&self) -> Option<Address> {
		Some(self.address.clone())
	}

	/// Helper for filling a transaction's nonce using the wallet
	async fn fill_transaction(&self, tx: &mut Transaction) -> Result<(), Self::Error> {
		// get the signer's network_magic if the transaction does not set it
		let network_magic = self.signer.network_magic();
		if tx.network_magic().is_none() {
			tx.set_network_magic(network_magic);
		}

		tx.nonce = 0;
		self.inner()
			.fill_transaction(tx)
			.await
			.map_err(SignerMiddlewareError::MiddlewareError)?;
		Ok(())
	}

	/// Signs and broadcasts the transaction.
	async fn send_transaction<T: Into<Transaction> + Send + Sync>(
		&self,
		tx: T,
	) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
		let mut tx = tx.into();

		// fill any missing fields
		self.fill_transaction(&mut tx).await?;

		// if we have a nonce manager set, we should try handling the result in
		// case there was a nonce mismatch
		let signed_tx = tx.to_array(); // self.sign_transaction(tx).await?;

		// Submit the raw transaction
		self.inner
			.send_raw_transaction(signed_tx.to_hex())
			.await
			.map(|tx| PendingTransaction::new(tx.hash, self.provider()))
			.map_err(SignerMiddlewareError::MiddlewareError)
	}

	async fn call(&self, tx: &Transaction, block: Option<BlockId>) -> Result<Bytes, Self::Error> {
		self.inner()
			.call(&tx, block)
			.await
			.map_err(SignerMiddlewareError::MiddlewareError)
	}
}
