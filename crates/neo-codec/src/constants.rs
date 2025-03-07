//! Constants used in the Neo codec module

/// Neo blockchain constants
pub struct NeoConstants;

impl NeoConstants {
    /// Size of a HASH160 in bytes
    pub const HASH160_SIZE: usize = 20;
    
    /// Size of a HASH256 in bytes
    pub const HASH256_SIZE: usize = 32;
}
