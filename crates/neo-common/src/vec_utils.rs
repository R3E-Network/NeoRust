//! Vector utility functions
//!
//! This module provides utility functions for working with vectors.

/// Converts a vector to a fixed-size array of 32 bytes.
///
/// # Arguments
///
/// * `vec` - The vector to convert
///
/// # Returns
///
/// A Result containing the 32-byte array or an error message if the vector is not 32 bytes
pub fn vec_to_array32(vec: Vec<u8>) -> Result<[u8; 32], String> {
    if vec.len() != 32 {
        return Err(format!("Expected 32 bytes, got {}", vec.len()));
    }
    
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&vec);
    Ok(arr)
}
