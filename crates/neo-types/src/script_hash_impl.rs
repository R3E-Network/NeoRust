//! Implementation of script hash operations
//!
//! This module provides implementation of script hash operations.

use byte_slice_cast::AsByteSlice;
use primitive_types::H160;
use sha2::{Digest, Sha256};
use std::str::FromStr;

// Implementation was removed to fix conflict with implementation in script_hash.rs
