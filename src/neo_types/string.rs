#[cfg(feature = "bs58")]
use bs58;
#[cfg(feature = "hex")]
use hex;
#[cfg(all(feature = "crypto-standard", feature = "sha2"))]
use sha2::{Digest, Sha256};

#[cfg(feature = "script-hash-ext")]
use crate::neo_types::script_hash::ScriptHash;

#[cfg(feature = "string-ext")]
pub trait StringExt {
	fn bytes_from_hex(&self) -> Result<Vec<u8>, hex::FromHexError>;

	fn base64_decoded(&self) -> Result<Vec<u8>, base64::DecodeError>;

	fn base64_encoded(&self) -> String;

	fn base58_decoded(&self) -> Option<Vec<u8>>;

	fn base58_check_decoded(&self) -> Option<Vec<u8>>;

	fn base58_encoded(&self) -> String;

	fn var_size(&self) -> usize;

	fn is_valid_address(&self) -> bool;

	fn is_valid_hex(&self) -> bool;

	fn address_to_scripthash(&self) -> Result<ScriptHash, &'static str>;

	fn reversed_hex(&self) -> String;
}

#[cfg(feature = "string-ext")]
impl StringExt for String {
	fn bytes_from_hex(&self) -> Result<Vec<u8>, hex::FromHexError> {
		#[cfg(feature = "hex")]
		{
			hex::decode(self.trim_start_matches("0x"))
		}
		#[cfg(not(feature = "hex"))]
		{
			Err(hex::FromHexError::InvalidHexCharacter { c: '0', index: 0 })
		}
	}

	fn base64_decoded(&self) -> Result<Vec<u8>, base64::DecodeError> {
		#[cfg(feature = "utils")]
		{
			base64::decode(self)
		}
		#[cfg(not(feature = "utils"))]
		{
			Err(base64::DecodeError::InvalidLength)
		}
	}

	fn base64_encoded(&self) -> String {
		#[cfg(feature = "utils")]
		{
			base64::encode(self.as_bytes())
		}
		#[cfg(not(feature = "utils"))]
		{
			String::new()
		}
	}

	fn base58_decoded(&self) -> Option<Vec<u8>> {
		#[cfg(feature = "bs58")]
		{
			bs58::decode(self).into_vec().ok()
		}
		#[cfg(not(feature = "bs58"))]
		{
			None
		}
	}

	fn base58_check_decoded(&self) -> Option<Vec<u8>> {
		#[cfg(feature = "bs58")]
		{
			bs58::decode(self).into_vec().ok()
		}
		#[cfg(not(feature = "bs58"))]
		{
			None
		}
	}

	fn base58_encoded(&self) -> String {
		#[cfg(feature = "bs58")]
		{
			bs58::encode(self.as_bytes()).into_string()
		}
		#[cfg(not(feature = "bs58"))]
		{
			String::new()
		}
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

	fn is_valid_address(&self) -> bool {
		#[cfg(feature = "crypto-standard")]
		{
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
		#[cfg(not(feature = "crypto-standard"))]
		{
			// Placeholder implementation when crypto-standard is not enabled
			false
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

	fn reversed_hex(&self) -> String {
		#[cfg(feature = "hex")]
		{
			let mut bytes = self.bytes_from_hex().unwrap();
			bytes.reverse();
			hex::encode(bytes)
		}
		#[cfg(not(feature = "hex"))]
		{
			String::new()
		}
	}
}
