// Token Constants
//
// This module centralizes all token addresses and contract script hashes
// across different Neo networks (Neo N3 and Neo X) for both mainnet and testnet.
//
// All token-related constants should be defined here to ensure consistency
// across the SDK and make updates easier to maintain.

use neo3::neo_types::ScriptHash;
use std::str::FromStr;

/// Neo N3 Mainnet token script hashes
pub mod neo_n3_mainnet {
    use super::*;

    pub const NEO: &str = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5";
    pub const GAS: &str = "d2a4cff31913016155e38e474a2c06d08be276cf";
    pub const FLM: &str = "f0151f528127558851b39c2cd8aa47da7418ab28";
    pub const FLUND: &str = "a0cc577f795414329cef84d3519b5d770c9ee06c";
    pub const BNEO: &str = "48c40d4666f93408be1bef038b6722404d9a4c2a";
    
    /// Get a token's script hash by symbol
    pub fn get_script_hash(symbol: &str) -> Option<ScriptHash> {
        match symbol.to_uppercase().as_str() {
            "NEO" => ScriptHash::from_str(NEO).ok(),
            "GAS" => ScriptHash::from_str(GAS).ok(),
            "FLM" => ScriptHash::from_str(FLM).ok(),
            "FLUND" => ScriptHash::from_str(FLUND).ok(),
            "BNEO" => ScriptHash::from_str(BNEO).ok(),
            _ => None,
        }
    }
}

/// Neo N3 Testnet token script hashes
pub mod neo_n3_testnet {
    use super::*;

    pub const NEO: &str = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5";
    pub const GAS: &str = "d2a4cff31913016155e38e474a2c06d08be276cf";
    pub const TEST: &str = "fe9e1c42ca6c61bcdfeea66def6d5a42db550cd8";
    pub const USDT: &str = "d7c6f00f2a5a38ad50a7115074745b106d5ff985";
    
    /// Get a token's script hash by symbol
    pub fn get_script_hash(symbol: &str) -> Option<ScriptHash> {
        match symbol.to_uppercase().as_str() {
            "NEO" => ScriptHash::from_str(NEO).ok(),
            "GAS" => ScriptHash::from_str(GAS).ok(),
            "TEST" => ScriptHash::from_str(TEST).ok(),
            "USDT" => ScriptHash::from_str(USDT).ok(),
            _ => None,
        }
    }
}

/// Neo X Mainnet token script hashes
pub mod neo_x_mainnet {
    use super::*;

    pub const NEO: &str = "8c36dc61d88882ff09800627f6dbf96cc8d18f2e";
    pub const GAS: &str = "4846a9d425805d28a34e785b594b10e896553bad";
    pub const NEOX: &str = "4846a9d425805d28a34e785b594b10e896553bad"; // NEOX is same as GAS on Neo X
    
    /// Get a token's script hash by symbol
    pub fn get_script_hash(symbol: &str) -> Option<ScriptHash> {
        match symbol.to_uppercase().as_str() {
            "NEO" => ScriptHash::from_str(NEO).ok(),
            "GAS" => ScriptHash::from_str(GAS).ok(),
            "NEOX" => ScriptHash::from_str(NEOX).ok(),
            _ => None,
        }
    }
}

/// Neo X Testnet token script hashes
pub mod neo_x_testnet {
    use super::*;

    pub const NEO: &str = "69ecca587e7992ce1f3b82b0d2cd6ce271219c4a";
    pub const GAS: &str = "da65b600f7124ce6c79950c1772a36403104f2be";
    pub const NEOX: &str = "da65b600f7124ce6c79950c1772a36403104f2be"; // NEOX is same as GAS on Neo X
    
    /// Get a token's script hash by symbol
    pub fn get_script_hash(symbol: &str) -> Option<ScriptHash> {
        match symbol.to_uppercase().as_str() {
            "NEO" => ScriptHash::from_str(NEO).ok(),
            "GAS" => ScriptHash::from_str(GAS).ok(),
            "NEOX" => ScriptHash::from_str(NEOX).ok(),
            _ => None,
        }
    }
}
