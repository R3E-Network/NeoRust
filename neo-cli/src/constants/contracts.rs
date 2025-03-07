// DeFi Contract Constants
//
// This module centralizes all DeFi contract addresses across different Neo networks
// (Neo N3 and Neo X) for both mainnet and testnet.
//
// All contract script hashes should be defined here to ensure consistency
// across the SDK and make updates easier to maintain.

use neo3::neo_types::ScriptHash;
use std::str::FromStr;

/// Neo X contract script hashes - MainNet
pub mod neo_x_mainnet {
    // Token registry contract - used to resolve token symbols to addresses
    pub const TOKEN_REGISTRY: &str = "5a9222225f92159afe7c0e9be8e61e87f82133ca";
    
    // DEX contract
    pub const DEX: &str = "acce6fd80d44e1796aa0c2c625e9e4e0ce39efc0";
}

/// Neo X contract script hashes - TestNet
pub mod neo_x_testnet {
    // Token registry contract - used to resolve token symbols to addresses
    pub const TOKEN_REGISTRY: &str = "e7f1b7d23fcfa6e58d65c1369d49b5100d2e2ebc";
    
    // DEX contract 
    pub const DEX: &str = "a0b1bd76570bd8df0aa60d77822f91ef2b57fc40";
}

/// Bridge contract script hashes
pub mod bridge {
    use super::*;
    
    // Neo N3 Mainnet bridge contract
    pub const NEO_N3_MAINNET: &str = "f46719e2d16bf50cddcef9d4bbfece901f73cbb6"; 
    
    // Neo N3 Testnet bridge contract
    pub const NEO_N3_TESTNET: &str = "a576df2ccadf47d782fc8e370f7a4f410f1a6424";
    
    /// Get bridge contract hash for Neo N3 network
    pub fn get_contract_hash(is_testnet: bool) -> Option<ScriptHash> {
        let hash_str = if is_testnet { NEO_N3_TESTNET } else { NEO_N3_MAINNET };
        ScriptHash::from_str(hash_str).ok()
    }
}

/// Flamingo Finance contract script hashes on Neo N3
pub mod flamingo {
    use super::*;
    
    // Flamingo contracts for Neo N3 Mainnet
    pub mod mainnet {
        use super::*;
        
        pub const SWAP_FACTORY: &str = "1d36178c8d9e396637b82968c70c8078a5a313a4";
        pub const SWAP_ROUTER: &str = "f970f4ccecd765b63732b821775dc38c25d74f23";
        pub const LIQUIDITY_POOL: &str = "0f46dc4f244014e9bbd3d1bb05c0262e7136b760";
        pub const STAKE_MANAGER: &str = "a08cea3dd3198a2a278fe2bfc8bcbb104120461b";
        
        /// Get Flamingo contract hash by contract name
        pub fn get_contract_hash(contract_name: &str) -> Option<ScriptHash> {
            match contract_name.to_lowercase().as_str() {
                "swap_factory" => ScriptHash::from_str(SWAP_FACTORY).ok(),
                "swap_router" => ScriptHash::from_str(SWAP_ROUTER).ok(),
                "liquidity_pool" => ScriptHash::from_str(LIQUIDITY_POOL).ok(),
                "stake_manager" => ScriptHash::from_str(STAKE_MANAGER).ok(),
                _ => None,
            }
        }
    }
    
    // Flamingo contracts for Neo N3 Testnet
    pub mod testnet {
        use super::*;
        
        pub const SWAP_FACTORY: &str = "c529e41223a013c7a394ab9f37ae4e59082a1d47";
        pub const SWAP_ROUTER: &str = "0b9707d678a6c0e65657eae3c866ce57c627f2bd";
        pub const LIQUIDITY_POOL: &str = "dbb8971858ea7487425eac77f6002b435944370c";
        pub const STAKE_MANAGER: &str = "43cf98eddbe047e198a3e5d57006311442a0ca15";
        
        /// Get Flamingo contract hash by contract name
        pub fn get_contract_hash(contract_name: &str) -> Option<ScriptHash> {
            match contract_name.to_lowercase().as_str() {
                "swap_factory" => ScriptHash::from_str(SWAP_FACTORY).ok(),
                "swap_router" => ScriptHash::from_str(SWAP_ROUTER).ok(),
                "liquidity_pool" => ScriptHash::from_str(LIQUIDITY_POOL).ok(),
                "stake_manager" => ScriptHash::from_str(STAKE_MANAGER).ok(),
                _ => None,
            }
        }
    }
    
    /// Get Flamingo contract hash based on network
    pub fn get_contract_hash(contract_name: &str, is_testnet: bool) -> Option<ScriptHash> {
        if is_testnet {
            testnet::get_contract_hash(contract_name)
        } else {
            mainnet::get_contract_hash(contract_name)
        }
    }
}

/// NeoBurger contract script hashes on Neo N3
pub mod neoburger {
    use super::*;
    
    // NeoBurger contracts for Neo N3 Mainnet
    pub mod mainnet {
        use super::*;
        
        pub const BURGER_TOKEN: &str = "48c40d4666f93408be1bef038b6722404d9a4c2a"; // bNEO token
        pub const BURGER_CONTRACT: &str = "d2c9be6d9c40d0b1ce5b029ec7c7d389593bdd9e";
        
        /// Get NeoBurger contract hash by contract name
        pub fn get_contract_hash(contract_name: &str) -> Option<ScriptHash> {
            match contract_name.to_lowercase().as_str() {
                "burger_token" => ScriptHash::from_str(BURGER_TOKEN).ok(),
                "burger_contract" => ScriptHash::from_str(BURGER_CONTRACT).ok(),
                _ => None,
            }
        }
    }
    
    // NeoBurger contracts for Neo N3 Testnet
    pub mod testnet {
        use super::*;
        
        pub const BURGER_TOKEN: &str = "cc74bf4ae86c1bd168b1f950fb71a521a9c687df"; // test bNEO token
        pub const BURGER_CONTRACT: &str = "659ac14dcfaaf8a95b52bfe3e814c46d4cc5116a";
        
        /// Get NeoBurger contract hash by contract name
        pub fn get_contract_hash(contract_name: &str) -> Option<ScriptHash> {
            match contract_name.to_lowercase().as_str() {
                "burger_token" => ScriptHash::from_str(BURGER_TOKEN).ok(),
                "burger_contract" => ScriptHash::from_str(BURGER_CONTRACT).ok(),
                _ => None,
            }
        }
    }
    
    /// Get NeoBurger contract hash based on network
    pub fn get_contract_hash(contract_name: &str, is_testnet: bool) -> Option<ScriptHash> {
        if is_testnet {
            testnet::get_contract_hash(contract_name)
        } else {
            mainnet::get_contract_hash(contract_name)
        }
    }
}
