#![doc = include_str!("../README.md")]
#![deny(unsafe_code, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// The [TransformerMiddleware] is used to intercept transactions
/// and transform them to be sent via various supported transformers, e.g.,
/// [DSProxy](crate::transformer::DsProxy).
pub mod transformer;
pub use transformer::TransformerMiddleware;

/// The [SignerMiddleware] is used to locally sign transactions and messages instead of using
/// `neo_sendTransaction` and `neo_sign`.
pub mod signer;
pub use signer::SignerMiddleware;

/// The [Policy] is used to ensure transactions comply with the rules configured in the
/// [`PolicyMiddleware`] before sending them.
pub mod policy;
pub use policy::{
	AllowEverything, Policy, PolicyMiddleware, PolicyMiddlewareError, RejectEverything,
};

/// [MiddlewareBuilder] provides a way to compose many [`Middleware`]s in a concise way.
pub mod builder;
pub use builder::MiddlewareBuilder;

pub use neo::prelude::{Middleware, MiddlewareError};

// For macro expansions only, not public API.
// See: [#2235](https://github.com/gakonst/neo-rs/pull/2235)

// #[doc(hidden)]
// #[allow(unused_extern_crates)]
// extern crate self as neo;

// #[doc(hidden)]
// pub use neo_contract as contract;
//
// #[doc(hidden)]
// pub use neo::prelude as providers;
