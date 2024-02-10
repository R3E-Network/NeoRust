use neo::prelude::{Middleware, Signer, SignerMiddleware};

/// A builder trait to compose different [`Middleware`] layers and then build a composed
/// [`Provider`](neo_providers::Provider) architecture.
///
/// [`Middleware`] composition acts in a wrapping fashion. Adding a new layer results in wrapping
/// its predecessor.
///
/// ```rust
///
/// use NeoRust::prelude::{Http, LocalWallet, MiddlewareBuilder, Provider, SignerMiddleware};
/// fn builder_example() {
///     let key = "fdb33e2105f08abe41a8ee3b758726a31abdd57b7a443f470f23efce853af169";
///     let signer = key.parse::<LocalWallet>().unwrap();
///     let address = signer.address();
///
///     let provider = Provider::<Http>::try_from("http://localhost:8545")
///         .unwrap()
///         .with_signer(signer)
///         .nonce_manager(address); // Outermost layer
/// }
///
/// fn builder_example_raw_wrap() {
///     let key = "fdb33e2105f08abe41a8ee3b758726a31abdd57b7a443f470f23efce853af169";
///     let signer = key.parse::<LocalWallet>().unwrap();
///     let address = signer.address();
///
///     let provider = Provider::<Http>::try_from("http://localhost:8545")
///         .unwrap()
///         .wrap_into(|p| SignerMiddleware::new(p, signer)); // Outermost layer
/// }
/// ```
pub trait MiddlewareBuilder: Middleware + Sized + 'static {
	/// Wraps `self` inside a new [`Middleware`].
	///
	/// `f` Consumes `self`. Must be used to return a new [`Middleware`] wrapping `self`.
	fn wrap_into<F, T>(self, f: F) -> T
	where
		F: FnOnce(Self) -> T,
		T: Middleware,
	{
		f(self)
	}

	/// Wraps `self` inside a [`SignerMiddleware`].
	fn with_signer<S>(self, s: S) -> SignerMiddleware<Self, S>
	where
		S: Signer,
	{
		SignerMiddleware::new(self, s)
	}
}

impl<M> MiddlewareBuilder for M where M: Middleware + Sized + 'static {}
