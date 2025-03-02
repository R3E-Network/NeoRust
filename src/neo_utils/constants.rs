//! # Neo N3 Network Constants
//!
//! This module contains constant values for Neo N3 networks, including RPC endpoints
//! and commonly used contract addresses for both MainNet and TestNet.

/// Neo N3 Network Magic Numbers
pub mod network_magic {
    /// MainNet magic number
    pub const MAINNET: u32 = 860833102;
    /// TestNet magic number
    pub const TESTNET: u32 = 894710606;
}

/// Neo N3 RPC Endpoints
pub mod rpc_endpoints {
    /// Neo N3 MainNet RPC Endpoints
    pub mod mainnet {
        /// Official Neo MainNet endpoints
        pub const NEO_OFFICIAL: &[&str] = &[
            "https://mainnet1.neo.org:443",
            "https://mainnet2.neo.org:443",
            "https://mainnet3.neo.org:443",
            "https://mainnet4.neo.org:443",
            "https://mainnet5.neo.org:443",
        ];

        /// NGD MainNet endpoints
        pub const NGD: &[&str] = &[
            "https://neo3-mainnet.ngd.network:443",
        ];

        /// COZ MainNet endpoints
        pub const COZ: &[&str] = &[
            "https://rpc.coz.io:443",
            "https://mainnet1.neo.coz.io:443",
            "https://mainnet2.neo.coz.io:443",
        ];
        
        /// NeoTube MainNet endpoints
        pub const NEOTUBE: &[&str] = &[
            "https://mainnet.neotube.io:443",
        ];

        /// All known MainNet endpoints
        pub const ALL: &[&str] = &[
            "https://mainnet1.neo.org:443",
            "https://mainnet2.neo.org:443",
            "https://mainnet3.neo.org:443",
            "https://mainnet4.neo.org:443",
            "https://mainnet5.neo.org:443",
            "https://neo3-mainnet.ngd.network:443",
            "https://rpc.coz.io:443",
            "https://mainnet1.neo.coz.io:443",
            "https://mainnet2.neo.coz.io:443",
            "https://mainnet.neotube.io:443",
        ];
    }

    /// Neo N3 TestNet RPC Endpoints
    pub mod testnet {
        /// Official Neo TestNet endpoints
        pub const NEO_OFFICIAL: &[&str] = &[
            "https://testnet1.neo.org:443",
            "https://testnet2.neo.org:443",
            "https://testnet3.neo.org:443",
            "https://testnet4.neo.org:443",
            "https://testnet5.neo.org:443",
        ];

        /// NGD TestNet endpoints
        pub const NGD: &[&str] = &[
            "https://neo3-testnet.ngd.network:443",
        ];

        /// COZ TestNet endpoints
        pub const COZ: &[&str] = &[
            "https://testnet.coz.io:443",
            "https://testnet1.neo.coz.io:443",
            "https://testnet2.neo.coz.io:443",
        ];
        
        /// NeoTube TestNet endpoints
        pub const NEOTUBE: &[&str] = &[
            "https://testnet.neotube.io:443",
        ];

        /// All known TestNet endpoints
        pub const ALL: &[&str] = &[
            "https://testnet1.neo.org:443",
            "https://testnet2.neo.org:443",
            "https://testnet3.neo.org:443",
            "https://testnet4.neo.org:443",
            "https://testnet5.neo.org:443",
            "https://neo3-testnet.ngd.network:443",
            "https://testnet.coz.io:443",
            "https://testnet1.neo.coz.io:443",
            "https://testnet2.neo.coz.io:443",
            "https://testnet.neotube.io:443",
        ];
    }

    /// NeoFS RPC endpoints
    pub mod neofs {
        /// Main NeoFS endpoint
        pub const MAIN_ENDPOINT: &str = "https://rest.fs.neo.org";
        /// Testnet NeoFS endpoint
        pub const TESTNET_ENDPOINT: &str = "https://testnet.fs.neo.org";
    }
}

/// Neo N3 smart contract addresses
pub mod contracts {
    /// MainNet contract addresses (script hashes)
    pub mod mainnet {
        /// NEO native token contract
        pub const NEO_TOKEN: &str = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5";
        /// GAS native token contract
        pub const GAS_TOKEN: &str = "d2a4cff31913016155e38e474a2c06d08be276cf";
        /// NeoFS contract
        pub const NEOFS: &str = "cc47091985d3d4e1d87da3cb089a2e4c77171ccf";
        /// Name service contract
        pub const NEO_NS: &str = "7a8fcf0392cd625647907afa8e45cc66872b596b";
        /// FLM token contract (Flamingo)
        pub const FLM_TOKEN: &str = "f0151f528127558851b39c2cd8aa47da7418ab28";
        /// Oracle contract
        pub const ORACLE: &str = "fe924b7cfe89ddd271abaf7210a80a7e11178758";
        /// NeoLine Swap contract
        pub const NEOLINE_SWAP: &str = "f46719e2d16bf50cddcef9d4bbfece901f73cbb6";
        /// Flamingo Finance Wrapper contract
        pub const FLUND_CONTRACT: &str = "1854b240a3ab901c41b50b8e1fe3a475a83fc2c2";
        /// ONTd token
        pub const ONTD_TOKEN: &str = "6f9673c2dcae818cd602c5bd7cca7e5584fb4770";
        /// BSNd token
        pub const BSND_TOKEN: &str = "b95198bd0d7461db119a717f28362ee7d3b84e26";
    }

    /// TestNet contract addresses (script hashes)
    pub mod testnet {
        /// NEO native token contract
        pub const NEO_TOKEN: &str = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5";
        /// GAS native token contract
        pub const GAS_TOKEN: &str = "d2a4cff31913016155e38e474a2c06d08be276cf";
        /// NeoFS contract
        pub const NEOFS: &str = "cc47091985d3d4e1d87da3cb089a2e4c77171ccf";
        /// Name service contract
        pub const NEO_NS: &str = "7a8fcf0392cd625647907afa8e45cc66872b596b";
        /// Oracle contract
        pub const ORACLE: &str = "fe924b7cfe89ddd271abaf7210a80a7e11178758";
        /// TestNet CNEO token
        pub const CNEO_TOKEN: &str = "e64eb4be6897c6ec98486e9d1c168d22503e8bf1";
        /// TestNet CGAS token
        pub const CGAS_TOKEN: &str = "da65b600f7124ce6c79950c1772a36403104f2be";
    }

    /// Native contracts common to all networks
    pub mod native {
        /// Contract Management contract hash (same on all networks)
        pub const CONTRACT_MANAGEMENT: &str = "fffdc93764dbaddd97c48f252a53ea4643faa3fd";
        /// Ledger contract hash (same on all networks)
        pub const LEDGER: &str = "da65b600f7124ce6c79950c1772a36403104f2be";
        /// Policy contract hash (same on all networks)
        pub const POLICY: &str = "cc5e4edd9f5f8dba8bb65734541df7a1c081c67b";
        /// Role Management contract hash (same on all networks)
        pub const ROLE_MANAGEMENT: &str = "49cf4e5378ffcd4dec034fd98a174c5491e395e2";
        /// StdLib contract hash (same on all networks)
        pub const STD_LIB: &str = "acce6fd80d44e1796aa0c2c625e9e4e0ce39efc0";
        /// CryptoLib contract hash (same on all networks)
        pub const CRYPTO_LIB: &str = "726cb6e0cd8628a1350a611384688911ab75f51b";
        /// Oracle native contract hash (same on all networks)
        pub const ORACLE: &str = "fe924b7cfe89ddd271abaf7210a80a7e11178758";
        /// NEO token native contract (same on all networks)
        pub const NEO_TOKEN: &str = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5";
        /// GAS token native contract (same on all networks)
        pub const GAS_TOKEN: &str = "d2a4cff31913016155e38e474a2c06d08be276cf";
    }
} 