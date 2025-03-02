/// Neo N3 blockchain constants
///
/// This module contains constants used throughout the Neo N3 blockchain,
/// including network IDs, limits, and default values.

/// Constants related to transaction and blockchain limits
pub struct NeoConstants;

impl NeoConstants {
    /// Maximum number of signers allowed in a transaction
    pub const MAX_SIGNER_SUBITEMS: u8 = 16;
    
    /// Maximum number of transaction attributes
    pub const MAX_TRANSACTION_ATTRIBUTES: u8 = 16;
    
    /// Maximum number of witnesses
    pub const MAX_WITNESSES: u8 = 16;
    
    /// Maximum script length
    pub const MAX_SCRIPT_LENGTH: usize = 65536;
    
    /// Maximum number for VM values
    pub const MAX_STACK_SIZE: usize = 2048;
    
    /// Default transaction gas limit
    pub const DEFAULT_GAS_LIMIT: i64 = 20_00000000;
    
    /// VM fault state
    pub const VM_FAULT_STATE: &'static str = "FAULT";
    
    /// VM halt state
    pub const VM_HALT_STATE: &'static str = "HALT";
    
    /// Neo N3 MainNet magic number
    pub const MAGIC_NUMBER_MAINNET: u32 = 5195086;
    
    /// Neo N3 TestNet magic number
    pub const MAGIC_NUMBER_TESTNET: u32 = 1951352142;
} 