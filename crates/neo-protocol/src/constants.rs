//! Neo N3 protocol limits
//!
//! These bounds are consensus rules, not tuning knobs: a node rejects anything
//! exceeding them, so decoding enforces them rather than allocating first.

/// Maximum serialized transaction size, in bytes.
pub const MAX_TRANSACTION_SIZE: usize = 102_400;

/// Maximum number of attributes on a transaction (signers included).
pub const MAX_TRANSACTION_ATTRIBUTES: usize = 16;

/// Maximum number of allowed contracts, groups or rules per signer.
pub const MAX_SIGNER_SUBITEMS: usize = 16;

/// Maximum nesting depth for witness rule conditions.
pub const MAX_NESTING_DEPTH: usize = 2;

/// Maximum size of a single witness invocation script, in bytes.
pub const MAX_INVOCATION_SCRIPT_SIZE: usize = 1_024;

/// Maximum size of a single witness verification script, in bytes.
pub const MAX_VERIFICATION_SCRIPT_SIZE: usize = 1_024;

/// Number of blocks a transaction stays valid for by default.
pub const MAX_VALID_UNTIL_BLOCK_INCREMENT: u32 = 5_760;
