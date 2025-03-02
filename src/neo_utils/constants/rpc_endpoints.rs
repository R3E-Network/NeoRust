//! RPC endpoints for Neo N3 networks

/// MainNet RPC endpoints
pub mod mainnet {
    /// All MainNet RPC endpoints
    pub const ALL: &[&str] = &[
        "https://mainnet1.neo.org:443",
        "https://mainnet2.neo.org:443",
        "https://mainnet3.neo.org:443",
    ];
}

/// TestNet RPC endpoints
pub mod testnet {
    /// All TestNet RPC endpoints
    pub const ALL: &[&str] = &[
        "https://testnet1.neo.org:443",
        "https://testnet2.neo.org:443",
        "https://testnet3.neo.org:443",
    ];
}

/// NeoFS endpoints
pub mod neofs {
    /// MainNet NeoFS endpoint
    pub const MAIN_ENDPOINT: &str = "https://fs.neo.org:443";
    /// TestNet NeoFS endpoint
    pub const TESTNET_ENDPOINT: &str = "https://fs-testnet.neo.org:443";
}
