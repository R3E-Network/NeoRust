use super::{Bytes, ScriptHashExtension};
use primitive_types::{H160, H256};
use serde_json::Value;

// Temporarily define a stub for Secp256r1PublicKey until we properly integrate neo-crypto
#[derive(Debug, Clone)]
pub struct Secp256r1PublicKey {
    encoded: Vec<u8>,
}

impl Secp256r1PublicKey {
    pub fn get_encoded(&self, _compressed: bool) -> Vec<u8> {
        self.encoded.clone()
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != 33 && bytes.len() != 65 {
            return Err("Invalid public key length");
        }
        Ok(Self { encoded: bytes.to_vec() })
    }
}

pub trait ValueExtension {
	fn to_value(&self) -> Value;
}

impl ValueExtension for Bytes {
	fn to_value(&self) -> Value {
		Value::String(hex::encode(&self.to_vec()))
	}
}

impl ValueExtension for String {
	fn to_value(&self) -> Value {
		Value::String(self.clone())
	}
}

impl ValueExtension for &str {
	fn to_value(&self) -> Value {
		Value::String(self.to_string())
	}
}

impl ValueExtension for H160 {
	fn to_value(&self) -> Value {
		Value::String(ScriptHashExtension::to_hex(self))
	}
}

impl ValueExtension for Secp256r1PublicKey {
	fn to_value(&self) -> Value {
		Value::String(hex::encode(self.get_encoded(true)))
	}
}

impl ValueExtension for H256 {
	fn to_value(&self) -> Value {
		Value::String(hex::encode(self))
	}
}

impl ValueExtension for u32 {
	fn to_value(&self) -> Value {
		Value::Number(serde_json::Number::from(*self))
	}
}

impl ValueExtension for u64 {
	fn to_value(&self) -> Value {
		Value::Number(serde_json::Number::from(*self))
	}
}

impl ValueExtension for i32 {
	fn to_value(&self) -> Value {
		Value::Number(serde_json::Number::from(*self))
	}
}

impl ValueExtension for i64 {
	fn to_value(&self) -> Value {
		Value::Number(serde_json::Number::from(*self))
	}
}

impl ValueExtension for bool {
	fn to_value(&self) -> Value {
		Value::Bool(*self)
	}
}
