//! Constants used throughout the Neo N3 SDK.
//! 
//! This module provides access to various constants used in the Neo N3 blockchain,
//! including native contract addresses, network endpoints, and other configuration constants.

pub mod native_contracts;
pub mod network_constants;

// Re-export commonly used constants
pub use native_contracts::{
    CONTRACT_MANAGEMENT,
    STD_LIB,
    CRYPTO_LIB,
    LEDGER,
    NEO_TOKEN,
    GAS_TOKEN,
    POLICY,
    ROLE_MANAGEMENT,
    ORACLE,
    NAME_SERVICE,
};

pub use network_constants::{
    NeoNetworkType,
    TokenConstants,
    BridgeConstants,
    DefiConstants,
    NNSConstants,
    NFSConstants,
};
