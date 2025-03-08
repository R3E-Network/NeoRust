use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::neo_types::ScriptHash;
use std::str::FromStr;

/// Represents different Neo blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeoNetworkType {
    /// Neo N3 MainNet
    N3MainNet,
    /// Neo N3 TestNet
    N3TestNet,
    /// Neo X MainNet
    XMainNet,
    /// Neo X TestNet
    XTestNet,
    /// Private Network (for local development)
    PrivateNet,
}

impl NeoNetworkType {
    /// Returns the magic number associated with a network
    pub fn to_magic(&self) -> u32 {
        match self {
            NeoNetworkType::N3MainNet => 0x334f454e, // 860833102
            NeoNetworkType::N3TestNet => 0x74746e41, // 1951616833
            NeoNetworkType::XMainNet => 0x584f454e,  // 1481850446
            NeoNetworkType::XTestNet => 0x58746e41,  // 1484633665
            NeoNetworkType::PrivateNet => 0x4e454e,  // 5139278
        }
    }
    
    /// Converts a magic number to a NeoNetworkType
    pub fn from_magic(magic: u32) -> Option<Self> {
        match magic {
            0x334f454e => Some(NeoNetworkType::N3MainNet),
            0x74746e41 => Some(NeoNetworkType::N3TestNet),
            0x584f454e => Some(NeoNetworkType::XMainNet),
            0x58746e41 => Some(NeoNetworkType::XTestNet),
            0x4e454e => Some(NeoNetworkType::PrivateNet),
            _ => None,
        }
    }
    
    /// Returns true if this is a Neo N3 network
    pub fn is_neo_n3(&self) -> bool {
        matches!(self, 
            NeoNetworkType::N3MainNet | 
            NeoNetworkType::N3TestNet)
    }
    
    /// Returns true if this is a Neo X network
    pub fn is_neo_x(&self) -> bool {
        matches!(self, 
            NeoNetworkType::XMainNet | 
            NeoNetworkType::XTestNet)
    }
    
    /// Returns true if this is a mainnet (either N3 or X)
    pub fn is_mainnet(&self) -> bool {
        matches!(self, 
            NeoNetworkType::N3MainNet | 
            NeoNetworkType::XMainNet)
    }
    
    /// Returns true if this is a testnet (either N3 or X)
    pub fn is_testnet(&self) -> bool {
        matches!(self, 
            NeoNetworkType::N3TestNet | 
            NeoNetworkType::XTestNet)
    }
    
    /// Returns the RPC URL for this network type
    pub fn get_default_rpc_url(&self) -> &'static str {
        match self {
            NeoNetworkType::N3MainNet => "https://mainnet1.neo.org:443",
            NeoNetworkType::N3TestNet => "https://testnet1.neo.org:443",
            NeoNetworkType::XMainNet => "https://rpc.neo-x.org",
            NeoNetworkType::XTestNet => "https://testnet.rpc.neo-x.org",
            NeoNetworkType::PrivateNet => "http://localhost:10332",
        }
    }
}

/// NEP-17 Token Constants for both Neo N3 and Neo X
pub struct TokenConstants;

impl TokenConstants {
    // Neo N3 MainNet Token Hashes
    pub const NEO_N3_MAINNET: &'static str = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"; // Neo N3 MainNet
    pub const GAS_N3_MAINNET: &'static str = "d2a4cff31913016155e38e474a2c06d08be276cf"; // Gas N3 MainNet
    
    // Neo N3 TestNet Token Hashes
    pub const NEO_N3_TESTNET: &'static str = "0a46e2e37c9987f570b4af253fb77e7eef0f72b6"; // Neo N3 TestNet
    pub const GAS_N3_TESTNET: &'static str = "74f2dc36a68fdc4682034178eb2220729231db76"; // Gas N3 TestNet
    
    // Neo X MainNet Token Addresses
    pub const NEO_X_MAINNET: &'static str = "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"; // Neo X MainNet (EVM format)
    pub const GAS_X_MAINNET: &'static str = "0xd2a4cff31913016155e38e474a2c06d08be276cf"; // Gas X MainNet (EVM format)
    
    // Neo X TestNet Token Addresses
    pub const NEO_X_TESTNET: &'static str = "0x0a46e2e37c9987f570b4af253fb77e7eef0f72b6"; // Neo X TestNet (EVM format)
    pub const GAS_X_TESTNET: &'static str = "0x74f2dc36a68fdc4682034178eb2220729231db76"; // Gas X TestNet (EVM format)
    
    /// Get the Neo token hash for the specified network
    pub fn get_neo_hash(network_type: NeoNetworkType) -> ScriptHash {
        let hash_str = match network_type {
            NeoNetworkType::N3MainNet => Self::NEO_N3_MAINNET,
            NeoNetworkType::N3TestNet => Self::NEO_N3_TESTNET,
            NeoNetworkType::XMainNet => &Self::NEO_X_MAINNET[2..], // Remove 0x prefix
            NeoNetworkType::XTestNet => &Self::NEO_X_TESTNET[2..], // Remove 0x prefix
            NeoNetworkType::PrivateNet => Self::NEO_N3_TESTNET, // Default to testnet for private networks
        };
        
        ScriptHash::from_str(hash_str).expect("Invalid NEO token hash")
    }
    
    /// Get the GAS token hash for the specified network
    pub fn get_gas_hash(network_type: NeoNetworkType) -> ScriptHash {
        let hash_str = match network_type {
            NeoNetworkType::N3MainNet => Self::GAS_N3_MAINNET,
            NeoNetworkType::N3TestNet => Self::GAS_N3_TESTNET,
            NeoNetworkType::XMainNet => &Self::GAS_X_MAINNET[2..], // Remove 0x prefix
            NeoNetworkType::XTestNet => &Self::GAS_X_TESTNET[2..], // Remove 0x prefix
            NeoNetworkType::PrivateNet => Self::GAS_N3_TESTNET, // Default to testnet for private networks
        };
        
        ScriptHash::from_str(hash_str).expect("Invalid GAS token hash")
    }
}

/// Bridge Contract Constants
pub struct BridgeConstants;

impl BridgeConstants {
    // Neo N3 Bridge Contract Hashes
    pub const N3_MAINNET_BRIDGE: &'static str = "f18bc3133a0523132c1f6b6ed1a8c1a7865d906a"; // Neo N3 MainNet Bridge
    pub const N3_TESTNET_BRIDGE: &'static str = "a6a6c15dcdc882f82a3a1bc17d7e3bd3105b4c87"; // Neo N3 TestNet Bridge
    
    // Neo X Bridge Contract Addresses
    pub const X_MAINNET_BRIDGE: &'static str = "0xf18bc3133a0523132c1f6b6ed1a8c1a7865d906a"; // Neo X MainNet Bridge (EVM format)
    pub const X_TESTNET_BRIDGE: &'static str = "0xa6a6c15dcdc882f82a3a1bc17d7e3bd3105b4c87"; // Neo X TestNet Bridge (EVM format)
    
    /// Get the bridge contract hash for the specified network
    pub fn get_bridge_hash(network_type: NeoNetworkType) -> ScriptHash {
        let hash_str = match network_type {
            NeoNetworkType::N3MainNet => Self::N3_MAINNET_BRIDGE,
            NeoNetworkType::N3TestNet => Self::N3_TESTNET_BRIDGE,
            NeoNetworkType::XMainNet => &Self::X_MAINNET_BRIDGE[2..], // Remove 0x prefix
            NeoNetworkType::XTestNet => &Self::X_TESTNET_BRIDGE[2..], // Remove 0x prefix
            NeoNetworkType::PrivateNet => Self::N3_TESTNET_BRIDGE, // Default to testnet for private networks
        };
        
        ScriptHash::from_str(hash_str).expect("Invalid bridge contract hash")
    }
}

/// DeFi Contracts Constants
pub struct DefiConstants;

impl DefiConstants {
    // Neo N3 DeFi Contracts
    pub const FLAMINGO_SWAP_N3_MAINNET: &'static str = "a0d1421903a4a5c43bf8b2ce5432a97d3148ff9b";
    pub const FLAMINGO_SWAP_N3_TESTNET: &'static str = "78e8e249a70ba451ff6e43e2e6d995e9b0d9adf0";
    
    // Neo X DeFi Contracts
    pub const FLUND_DEX_X_MAINNET: &'static str = "0xa0d1421903a4a5c43bf8b2ce5432a97d3148ff9b";
    pub const FLUND_DEX_X_TESTNET: &'static str = "0x78e8e249a70ba451ff6e43e2e6d995e9b0d9adf0";
    
    // Get DEX contracts hash map
    pub fn get_dex_contracts(network_type: NeoNetworkType) -> HashMap<&'static str, ScriptHash> {
        let mut contracts = HashMap::new();
        
        match network_type {
            NeoNetworkType::N3MainNet => {
                contracts.insert("flamingo_swap", ScriptHash::from_str(Self::FLAMINGO_SWAP_N3_MAINNET)
                    .expect("Invalid Flamingo Swap hash"));
                // Add more N3 MainNet DeFi contracts as needed
            },
            NeoNetworkType::N3TestNet => {
                contracts.insert("flamingo_swap", ScriptHash::from_str(Self::FLAMINGO_SWAP_N3_TESTNET)
                    .expect("Invalid Flamingo Swap hash"));
                // Add more N3 TestNet DeFi contracts as needed
            },
            NeoNetworkType::XMainNet => {
                contracts.insert("flund_dex", ScriptHash::from_str(&Self::FLUND_DEX_X_MAINNET[2..])
                    .expect("Invalid FLUND DEX hash"));
                // Add more X MainNet DeFi contracts as needed
            },
            NeoNetworkType::XTestNet => {
                contracts.insert("flund_dex", ScriptHash::from_str(&Self::FLUND_DEX_X_TESTNET[2..])
                    .expect("Invalid FLUND DEX hash"));
                // Add more X TestNet DeFi contracts as needed
            },
            NeoNetworkType::PrivateNet => {
                // Default to N3 TestNet for private networks
                contracts.insert("flamingo_swap", ScriptHash::from_str(Self::FLAMINGO_SWAP_N3_TESTNET)
                    .expect("Invalid Flamingo Swap hash"));
            },
        }
        
        contracts
    }
}

/// Neo Name Service (NNS) Constants
pub struct NNSConstants;

impl NNSConstants {
    // NNS Resolver Contract Hashes
    pub const NNS_RESOLVER_N3_MAINNET: &'static str = "50ac1c37690cc2cfc594472833cf57505d5f46de";
    pub const NNS_RESOLVER_N3_TESTNET: &'static str = "5e2378de1bd47031d0b85d664c0989f97c5e5692";
    pub const NNS_RESOLVER_X_MAINNET: &'static str = "0x50ac1c37690cc2cfc594472833cf57505d5f46de";
    pub const NNS_RESOLVER_X_TESTNET: &'static str = "0x5e2378de1bd47031d0b85d664c0989f97c5e5692";
    
    /// Get the NNS resolver contract hash for the specified network
    pub fn get_nns_resolver_hash(network_type: NeoNetworkType) -> ScriptHash {
        let hash_str = match network_type {
            NeoNetworkType::N3MainNet => Self::NNS_RESOLVER_N3_MAINNET,
            NeoNetworkType::N3TestNet => Self::NNS_RESOLVER_N3_TESTNET,
            NeoNetworkType::XMainNet => &Self::NNS_RESOLVER_X_MAINNET[2..], // Remove 0x prefix
            NeoNetworkType::XTestNet => &Self::NNS_RESOLVER_X_TESTNET[2..], // Remove 0x prefix
            NeoNetworkType::PrivateNet => Self::NNS_RESOLVER_N3_TESTNET, // Default to testnet for private networks
        };
        
        ScriptHash::from_str(hash_str).expect("Invalid NNS resolver hash")
    }
}

/// Neo FS Constants
pub struct NFSConstants;

impl NFSConstants {
    // Neo FS Gateway URLs
    pub const NFS_GATEWAY_N3_MAINNET: &'static str = "https://fs-gw.neo.org";
    pub const NFS_GATEWAY_N3_TESTNET: &'static str = "https://fs-gw-testnet.neo.org";
    
    /// Get the Neo FS gateway URL for the specified network
    pub fn get_nfs_gateway_url(network_type: NeoNetworkType) -> &'static str {
        match network_type {
            NeoNetworkType::N3MainNet | NeoNetworkType::XMainNet => Self::NFS_GATEWAY_N3_MAINNET,
            NeoNetworkType::N3TestNet | NeoNetworkType::XTestNet | NeoNetworkType::PrivateNet => Self::NFS_GATEWAY_N3_TESTNET,
        }
    }
}
