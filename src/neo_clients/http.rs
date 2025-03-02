//! HTTP provider module for Neo N3 RPC clients

#[cfg(feature = "http-client")]
use crate::neo_clients::rpc::transports::http_provider::HttpProvider;

/// Type alias for HTTP provider
#[cfg(feature = "http-client")]
pub type Http = HttpProvider;

#[cfg(feature = "http-client")]
pub use crate::neo_clients::rpc::transports::http_provider::HttpClientError; 