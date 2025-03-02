//! Contract addresses for Neo N3 networks

/// MainNet contract addresses
pub mod mainnet {
    /// NEO token contract hash on MainNet
    pub const NEO_TOKEN: &str = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5";
    
    /// GAS token contract hash on MainNet
    pub const GAS_TOKEN: &str = "d2a4cff31913016155e38e474a2c06d08be276cf";
    
    /// NeoFS contract hash on MainNet
    pub const NEOFS: &str = "d7c6c07cb93d76c1b40c545449a7c18be5e7c3c4";
    
    /// Neo Name Service contract hash on MainNet
    pub const NEO_NS: &str = "7a8fcf0392cd625647907afa8e45cc66872b596b";
    
    /// Flamingo (FLM) token contract hash on MainNet
    pub const FLM_TOKEN: &str = "4d9eab13620fe3569ba3b0e56e2877739e4145e3";
    
    /// Oracle contract hash on MainNet
    pub const ORACLE: &str = "fe924b7cfe89ddd271abaf7210a80a7e11178758";
}

/// TestNet contract addresses
pub mod testnet {
    /// NEO token contract hash on TestNet
    pub const NEO_TOKEN: &str = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5";
    
    /// GAS token contract hash on TestNet
    pub const GAS_TOKEN: &str = "d2a4cff31913016155e38e474a2c06d08be276cf";
    
    /// NeoFS contract hash on TestNet
    pub const NEOFS: &str = "d7c6c07cb93d76c1b40c545449a7c18be5e7c3c4";
    
    /// Neo Name Service contract hash on TestNet
    pub const NEO_NS: &str = "7a8fcf0392cd625647907afa8e45cc66872b596b";
    
    /// Oracle contract hash on TestNet
    pub const ORACLE: &str = "fe924b7cfe89ddd271abaf7210a80a7e11178758";
    
    /// CNEO token contract hash on TestNet
    pub const CNEO_TOKEN: &str = "1a70eac53f5882e40d33f2f583096c6f852442e7";
    
    /// CGAS token contract hash on TestNet
    pub const CGAS_TOKEN: &str = "74f2dc36a68fdc4682034178eb2220729231db76";
}

/// Native contract addresses (same on all networks)
pub mod native {
    /// Contract Management contract hash
    pub const CONTRACT_MANAGEMENT: &str = "fffdc93764dbaddd97c48f252a53ea4643faa3fd";
    
    /// Ledger contract hash
    pub const LEDGER: &str = "da65b600f7124ce6c79950c1772a36403104f2be";
    
    /// Policy contract hash
    pub const POLICY: &str = "cc5e4edd9f5f8dba8bb65734541df7a1c081c67b";
    
    /// Role Management contract hash
    pub const ROLE_MANAGEMENT: &str = "49cf4e5378ffcd4dec034fd98a174c5491e395e2";
}
