use async_trait::async_trait;
use futures_util::lock::Mutex;
use getset::{Getters, Setters};
use primitive_types::{H160, H256};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, value::Value};
use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt::{Debug, Display},
    future::Future,
    pin::Pin,
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use tracing::{debug, trace};
use tracing_futures::Instrument;
use url::{Host, ParseError, Url};

#[cfg(feature = "http-client")]
use crate::neo_clients::http::Http;

use crate::{
    neo_clients::rpc::connections::JsonRpcProvider,
    neo_clients::errors::ProviderError,
};

/// Node Clients
#[derive(Copy, Clone, Debug)]
pub enum NeoClient {
    /// Neo
    NEO,
}

impl FromStr for NeoClient {
    type Err = ProviderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let first_segment = s
            .split('/')
            .next()
            .ok_or(ProviderError::ParseError("Invalid client string format".to_string()))?;
        match first_segment.to_lowercase().as_str() {
            "neo" => Ok(NeoClient::NEO),
            _ => Err(ProviderError::UnsupportedNodeClient),
        }
    }
}

/// Neo blockchain version information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeoVersion {
    /// Protocol configuration
    pub protocol: ProtocolConfig,
    /// TCP port
    pub tcpport: u16,
    /// WebSocket port
    pub wsport: u16,
    /// Nonce value
    pub nonce: u64,
    /// Node user agent
    pub useragent: String,
}

/// Protocol configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// Network ID
    pub network: u32,
    /// Number of validators
    pub validatorscount: u32,
    /// Milliseconds per block
    pub msperblock: u32,
}

#[derive(Clone, Debug, Getters)]
pub struct RpcClient<P> {
    provider: P,
    nns: Option<H160>,
    interval: Option<Duration>,
    from: Option<H160>,
    _node_client: Arc<Mutex<Option<NeoVersion>>>,
}

impl<P: JsonRpcProvider> RpcClient<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            nns: None,
            interval: None,
            from: None,
            _node_client: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn request<T, R>(&self, method: &str, params: T) -> Result<R, ProviderError>
    where
        T: Debug + Serialize + Send + Sync,
        R: Serialize + DeserializeOwned + Debug + Send,
    {
        let fetched = self.provider.fetch(method, params).await;
        let res: R = fetched.map_err(Into::into)?;
        Ok(res)
    }
    
    /// Gets the version information from the Neo node
    pub async fn get_version(&self) -> Result<NeoVersion, ProviderError> {
        self.request::<[(); 0], NeoVersion>("getversion", []).await
    }
}

// Implement AsRef for RpcClient to access the provider
impl<P> AsRef<P> for RpcClient<P> {
    fn as_ref(&self) -> &P {
        &self.provider
    }
}

// HTTP specific implementations
#[cfg(feature = "http-client")]
impl TryFrom<&str> for RpcClient<Http> {
    type Error = ParseError;

    fn try_from(src: &str) -> Result<Self, Self::Error> {
        Ok(RpcClient::new(Http::new(src)?))
    }
}

#[cfg(feature = "http-client")]
impl TryFrom<String> for RpcClient<Http> {
    type Error = ParseError;

    fn try_from(src: String) -> Result<Self, Self::Error> {
        RpcClient::try_from(src.as_str())
    }
}

#[cfg(feature = "http-client")]
impl<'a> TryFrom<&'a String> for RpcClient<Http> {
    type Error = ParseError;

    fn try_from(src: &'a String) -> Result<Self, Self::Error> {
        RpcClient::try_from(src.as_str())
    }
}

#[cfg(feature = "http-client")]
mod sealed {
    use crate::neo_clients::http::Http;
    use crate::neo_clients::rpc::rpc_client::RpcClient;

    /// Private trait to ensure extension trait is not implemented outside of this crate
    pub trait Sealed {}
    impl Sealed for RpcClient<Http> {}
}

#[cfg(feature = "http-client")]
impl RpcClient<Http> {
    /// The Url to which requests are made
    pub fn url(&self) -> &Url {
        self.provider.url()
    }

    /// Mutable access to the Url to which requests are made
    pub fn url_mut(&mut self) -> &mut Url {
        self.provider.url_mut()
    }
}
