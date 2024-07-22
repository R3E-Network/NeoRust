//! The Http transport is used to send JSON-RPC requests over HTTP to an Neo node.
//! This is the most basic connection to a node.

use primitive_types::H256;
use reqwest::header::{HeaderMap, HeaderValue};
use std::sync::Arc;
use NeoRust::prelude::*;

const RPC_URL: &str = NeoConstants::SEED_1;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    create_instance().await?;
    share_providers_across_tasks().await?;
    Ok(())
}

async fn create_instance() -> eyre::Result<()> {
    // An Http provider can be created from an http(s) URI.
    // In case of https you must add the "rustls" or "openssl" feature
    // to the library dependency in `Cargo.toml`.
    let _provider = Provider::<Http>::try_from(RPC_URL)?;

    // Instantiate with auth to append basic authorization headers across requests
    let url = reqwest::Url::parse(RPC_URL)?;
    let auth = Authorization::basic("username", "password");
    let _provider = Http::new_with_auth(url, auth)?;

    // Instantiate from custom Http Client if you need
    // finer control over the Http client configuration
    // (TLS, Proxy, Cookies, Headers, etc.)
    let url = reqwest::Url::parse(RPC_URL)?;

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("Bearer my token"));
    headers.insert("X-MY-HEADERS", HeaderValue::from_static("Some value"));

    let http_client = reqwest::Client::builder()
        .default_headers(headers)
        .proxy(reqwest::Proxy::all("http://proxy.example.com:8080")?)
        .build()?;

    let _provider = Http::new_with_client(url, http_client);

    Ok(())
}

/// Providers can be easily shared across tasks using `Arc` smart pointers
async fn share_providers_across_tasks() -> eyre::Result<()> {
    let provider: Provider<Http> = Provider::<Http>::try_from(RPC_URL)?;

    let client_1 = Arc::new(provider);
    let client_2 = Arc::clone(&client_1);

    let handle1 =
        tokio::spawn(async move { client_1.get_best_block_hash().await.unwrap_or(H256::zero()) });

    let handle2 =
        tokio::spawn(async move { client_2.get_best_block_hash().await.unwrap_or(H256::zero()) });

    let block1: H256 = handle1.await?;
    let block2: H256 = handle2.await?;

    println!("{block1:?} {block2:?}");

    Ok(())
}
