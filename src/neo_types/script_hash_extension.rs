//! Extension trait for ScriptHash (H160)

use primitive_types::H160;
use crate::neo_types::{
    address::Address,
    error::TypeError,
};

#[cfg(feature = "crypto-standard")]
use crate::neo_types::script_hash::HashableForVec;

/// Extension trait for ScriptHash (H160)
pub trait ScriptHashExtension {
    /// Convert a script hash to an address
    fn to_address(&self) -> Address;
    
    /// Convert an address to a script hash
    fn from_address(address: &str) -> Result<H160, TypeError>;
    
    /// Convert a public key to a script hash
    #[cfg(feature = "crypto-standard")]
    fn from_public_key(public_key: &[u8]) -> Result<H160, TypeError>;
}

impl ScriptHashExtension for H160 {
    fn to_address(&self) -> Address {
        let mut data = vec![0x17];
        data.extend_from_slice(self.as_bytes());
        
        #[cfg(all(feature = "utils", feature = "crypto-standard"))]
        {
            use crate::neo_types::script_hash::HashableForVec;
            let sha = &data.hash256().hash256();
            data.extend_from_slice(&sha[..4]);
            bs58::encode(data).into_string()
        }
        
        #[cfg(not(all(feature = "utils", feature = "crypto-standard")))]
        {
            // Fallback implementation when utils feature is not enabled
            format!("0x{:x}", self)
        }
    }
    
    fn from_address(address: &str) -> Result<H160, TypeError> {
        #[cfg(feature = "utils")]
        {
            if address.len() < 1 {
                return Err(TypeError::InvalidFormat("Address is empty".to_string()));
            }
            
            let data = bs58::decode(address)
                .into_vec()
                .map_err(|_| TypeError::InvalidFormat("Invalid base58 encoding".to_string()))?;
                
            if data.len() < 25 {
                return Err(TypeError::InvalidFormat("Address data too short".to_string()));
            }
            
            if data[0] != 0x17 {
                return Err(TypeError::InvalidFormat("Address version incorrect".to_string()));
            }
            
            let script_hash = &data[1..21];
            Ok(H160::from_slice(script_hash))
        }
        
        #[cfg(not(feature = "utils"))]
        {
            // Fallback implementation when utils feature is not enabled
            if address.starts_with("0x") {
                let hex_str = &address[2..];
                if hex_str.len() != 40 {
                    return Err(TypeError::InvalidFormat("Invalid address format".to_string()));
                }
                
                match hex::decode(hex_str) {
                    Ok(bytes) => Ok(H160::from_slice(&bytes)),
                    Err(_) => Err(TypeError::InvalidFormat("Invalid hex in address".to_string())),
                }
            } else {
                Err(TypeError::InvalidFormat("Address conversion not available without utils feature".to_string()))
            }
        }
    }
    
    #[cfg(feature = "crypto-standard")]
    fn from_public_key(public_key: &[u8]) -> Result<H160, TypeError> {
        // Hash the public key with SHA256 and then RIPEMD160
        let hash = public_key.sha256_ripemd160();
        Ok(H160::from_slice(&hash))
    }
}
