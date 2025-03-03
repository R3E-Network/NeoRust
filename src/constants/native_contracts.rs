//! Definitions of Native Contract script hashes for Neo N3.
//! 
//! This module provides constants for all native contracts available on the Neo N3 blockchain.
//! These script hashes are the same across both Mainnet and Testnet.

/// Contract Management native contract script hash
pub const CONTRACT_MANAGEMENT: &str = "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd";

/// Standard Library native contract script hash
pub const STD_LIB: &str = "0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0";

/// Cryptography Library native contract script hash
pub const CRYPTO_LIB: &str = "0x726cb6e0cd8628a1350a611384688911ab75f51b";

/// Ledger native contract script hash
pub const LEDGER: &str = "0xda65b600f7124ce6c79950c1772a36403104f2be";

/// NEO Token native contract script hash
pub const NEO_TOKEN: &str = "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5";

/// GAS Token native contract script hash
pub const GAS_TOKEN: &str = "0xd2a4cff31913016155e38e474a2c06d08be276cf";

/// Policy native contract script hash
pub const POLICY: &str = "0xcc5e4edd9f5f8dba8bb65734541df7a1c081c67b";

/// Role Management native contract script hash
pub const ROLE_MANAGEMENT: &str = "0x49cf4e5378ffcd4dec034fd98a174c5491e395e2";

/// Oracle native contract script hash
pub const ORACLE: &str = "0xfe924b7cfe89ddd271abaf7210a80a7e11178758";

/// Name Service native contract script hash
pub const NAME_SERVICE: &str = "0x7a8fcf0392cd625647907afa8e45cc66872b596b";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_contract_addresses() {
        // This test doesn't validate the addresses, just ensures they're defined
        assert!(!CONTRACT_MANAGEMENT.is_empty());
        assert!(!STD_LIB.is_empty());
        assert!(!CRYPTO_LIB.is_empty());
        assert!(!LEDGER.is_empty());
        assert!(!NEO_TOKEN.is_empty());
        assert!(!GAS_TOKEN.is_empty());
        assert!(!POLICY.is_empty());
        assert!(!ROLE_MANAGEMENT.is_empty());
        assert!(!ORACLE.is_empty());
        assert!(!NAME_SERVICE.is_empty());
    }
}
