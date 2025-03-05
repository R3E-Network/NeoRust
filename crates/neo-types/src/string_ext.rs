//! Extension traits for string operations
//!
//! This module provides extension traits for string operations.

/// Extension trait for string operations
pub trait StringExt {
    /// Checks if the string is a valid hexadecimal string
    fn is_valid_hex(&self) -> bool;
    
    /// Checks if the string is a valid Base58 string
    fn is_valid_base58(&self) -> bool;
    
    /// Checks if the string is a valid Base64 string
    fn is_valid_base64(&self) -> bool;
}

impl StringExt for String {
    fn is_valid_hex(&self) -> bool {
        if self.len() % 2 != 0 {
            return false;
        }
        
        let hex_chars = "0123456789abcdefABCDEF";
        self.chars().all(|c| hex_chars.contains(c))
    }
    
    fn is_valid_base58(&self) -> bool {
        let base58_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        self.chars().all(|c| base58_chars.contains(c))
    }
    
    fn is_valid_base64(&self) -> bool {
        let base64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
        self.chars().all(|c| base64_chars.contains(c))
    }
}

impl StringExt for &str {
    fn is_valid_hex(&self) -> bool {
        self.to_string().is_valid_hex()
    }
    
    fn is_valid_base58(&self) -> bool {
        self.to_string().is_valid_base58()
    }
    
    fn is_valid_base64(&self) -> bool {
        self.to_string().is_valid_base64()
    }
}
