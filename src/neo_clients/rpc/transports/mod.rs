pub use common::*;
pub use http_provider::{ClientError, HttpProvider};
#[cfg(all(feature = "ipc", any(unix, windows)))]
pub use ipc::{Ipc, IpcError};
#[cfg(feature = "legacy-ws")]
pub use legacy_ws::{ClientError as WsClientError, Ws};
// pub use mock::{MockError, MockProvider, MockResponse};
pub use retry::*;
pub use rw::{RwClient, RwClientError};
#[cfg(all(feature = "ws", not(feature = "legacy-ws")))]
pub use ws::{ConnectionDetails, WsClient as Ws, WsClientError};

pub use self::http_provider::{ClientError as HttpClientError, HttpProvider as Http};

pub mod http_provider;
#[cfg(all(feature = "ipc", any(unix, windows)))]
mod ipc;
// mod quorum;
// pub use quorum::{JsonRpcClientWrapper, Quorum, QuorumError, QuorumProvider, WeightedProvider};

pub mod common;
/// archival websocket
#[cfg(feature = "legacy-ws")]
pub mod legacy_ws;
// mod mock;
pub mod retry;
pub mod rw;
#[cfg(all(feature = "ws", not(feature = "legacy-ws")))]
mod ws;
