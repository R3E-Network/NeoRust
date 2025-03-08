use bs58;
use hex;
use sha2::{Digest, Sha256};
use ::base64::{Engine, DecodeError};
use ::base64::engine::general_purpose::STANDARD;

use crate::script_hash::ScriptHash;

/// String extension methods for Neo functions
pub trait StringExt {
	/// Check if a string is a valid Neo address
	fn is_valid_address(&self) -> bool;
	
	/// Decode a hex encoded string to bytes
	fn from_hex(&self) -> Result<Vec<u8>, hex::FromHexError>;
	
	/// Decode a Base58 encoded string to bytes
	fn base58_decoded(&self) -> Option<Vec<u8>>;
	
	/// Encode bytes to a reverse hex string
	fn reversed_hex(&self) -> String;

	fn bytes_from_hex(&self) -> Result<Vec<u8>, hex::FromHexError>;

	fn base64_decoded(&self) -> Result<Vec<u8>, DecodeError>;

	fn base64_encoded(&self) -> String;

	fn base58_check_decoded(&self) -> Option<Vec<u8>>;

	fn base58_encoded(&self) -> String;

	fn var_size(&self) -> usize;

	fn is_valid_hex(&self) -> bool;

	fn address_to_scripthash(&self) -> Result<ScriptHash, &'static str>;
}

/// Base64 encoding/decoding trait
pub trait Base64String {
	/// Decode a base64 encoded string to bytes
	fn base64_decoded(&self) -> Result<Vec<u8>, base64::DecodeError>;
	
	/// Encode this type to a base64 string
	fn base64_encoded(&self) -> String;
}

impl StringExt for String {
	fn is_valid_address(&self) -> bool {
		if let Some(data) = self.base58_decoded() {
			if data.len() == 25 && data[0] == 0x17 {
				let checksum = &Sha256::digest(&Sha256::digest(&data[..21]))[..4];
				checksum == &data[21..]
			} else {
				false
			}
		} else {
			false
		}
	}
	
	fn from_hex(&self) -> Result<Vec<u8>, hex::FromHexError> {
		hex::decode(self)
	}
	
	fn base58_decoded(&self) -> Option<Vec<u8>> {
		bs58::decode(self).into_vec().ok()
	}
	
	fn reversed_hex(&self) -> String {
		let bytes = match hex::decode(self) {
			Ok(b) => b,
			Err(_) => return String::new(),
		};
		
		let mut reversed = bytes.clone();
		reversed.reverse();
		
		hex::encode(reversed)
	}

	fn bytes_from_hex(&self) -> Result<Vec<u8>, hex::FromHexError> {
		hex::decode(self.trim_start_matches("0x"))
	}

	fn base64_decoded(&self) -> Result<Vec<u8>, DecodeError> {
		STANDARD.decode(self.as_bytes())
	}

	fn base64_encoded(&self) -> String {
		STANDARD.encode(self.as_bytes())
	}

	fn base58_check_decoded(&self) -> Option<Vec<u8>> {
		bs58::decode(self).into_vec().ok()
	}

	fn base58_encoded(&self) -> String {
		bs58::encode(self.as_bytes()).into_string()
	}

	fn var_size(&self) -> usize {
		let bytes = self.as_bytes();
		let len = bytes.len();
		if len < 0xFD {
			1
		} else if len <= 0xFFFF {
			3
		} else if len <= 0xFFFFFFFF {
			5
		} else {
			9
		}
	}

	fn is_valid_hex(&self) -> bool {
		self.len() % 2 == 0 && self.chars().all(|c| c.is_ascii_hexdigit())
	}

	fn address_to_scripthash(&self) -> Result<ScriptHash, &'static str> {
		if self.is_valid_address() {
			let data = self.base58_decoded().ok_or("Invalid address").unwrap();
			let mut scripthash = data[1..21].to_vec();
			scripthash.reverse();
			Ok(ScriptHash::from_slice(&scripthash))
		} else {
			Err("Not a valid address")
		}
	}
}

impl StringExt for str {
	fn is_valid_address(&self) -> bool {
		if let Some(data) = self.base58_decoded() {
			if data.len() == 25 && data[0] == 0x17 {
				let checksum = &Sha256::digest(&Sha256::digest(&data[..21]))[..4];
				checksum == &data[21..]
			} else {
				false
			}
		} else {
			false
		}
	}
	
	fn from_hex(&self) -> Result<Vec<u8>, hex::FromHexError> {
		hex::decode(self)
	}
	
	fn base58_decoded(&self) -> Option<Vec<u8>> {
		bs58::decode(self).into_vec().ok()
	}
	
	fn reversed_hex(&self) -> String {
		let bytes = match hex::decode(self) {
			Ok(b) => b,
			Err(_) => return String::new(),
		};
		
		let mut reversed = bytes.clone();
		reversed.reverse();
		
		hex::encode(reversed)
	}

	fn bytes_from_hex(&self) -> Result<Vec<u8>, hex::FromHexError> {
		hex::decode(self.trim_start_matches("0x"))
	}

	fn base64_decoded(&self) -> Result<Vec<u8>, DecodeError> {
		STANDARD.decode(self.as_bytes())
	}

	fn base64_encoded(&self) -> String {
		STANDARD.encode(self.as_bytes())
	}

	fn base58_check_decoded(&self) -> Option<Vec<u8>> {
		bs58::decode(self).into_vec().ok()
	}

	fn base58_encoded(&self) -> String {
		bs58::encode(self.as_bytes()).into_string()
	}

	fn var_size(&self) -> usize {
		let bytes = self.as_bytes();
		let len = bytes.len();
		if len < 0xFD {
			1
		} else if len <= 0xFFFF {
			3
		} else if len <= 0xFFFFFFFF {
			5
		} else {
			9
		}
	}

	fn is_valid_hex(&self) -> bool {
		self.len() % 2 == 0 && self.chars().all(|c| c.is_ascii_hexdigit())
	}

	fn address_to_scripthash(&self) -> Result<ScriptHash, &'static str> {
		if self.is_valid_address() {
			let data = self.base58_decoded().ok_or("Invalid address").unwrap();
			let mut scripthash = data[1..21].to_vec();
			scripthash.reverse();
			Ok(ScriptHash::from_slice(&scripthash))
		} else {
			Err("Not a valid address")
		}
	}
}

impl Base64String for str {
	fn base64_decoded(&self) -> Result<Vec<u8>, base64::DecodeError> {
		STANDARD.decode(self.as_bytes())
	}
	
	fn base64_encoded(&self) -> String {
		STANDARD.encode(self.as_bytes())
	}
}

impl Base64String for String {
	fn base64_decoded(&self) -> Result<Vec<u8>, base64::DecodeError> {
		STANDARD.decode(self.as_bytes())
	}
	
	fn base64_encoded(&self) -> String {
		STANDARD.encode(self.as_bytes())
	}
}

impl Base64String for [u8] {
	fn base64_decoded(&self) -> Result<Vec<u8>, base64::DecodeError> {
		STANDARD.decode(self)
	}
	
	fn base64_encoded(&self) -> String {
		STANDARD.encode(self)
	}
}

