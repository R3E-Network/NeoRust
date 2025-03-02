//! # JSON Codec
//!
//! JSON encoding and decoding for Neo N3 blockchain data.
//!
//! This module provides serialization and deserialization functionality for
//! Neo N3 data structures in JSON format, primarily used for RPC requests and responses.

use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::error::Error;

/// Serialize a value to a JSON string
pub fn to_json<T>(value: &T) -> Result<String, Box<dyn Error>>
where
	T: Serialize,
{
	Ok(serde_json::to_string(value)?)
}

/// Serialize a value to a pretty-printed JSON string
pub fn to_json_pretty<T>(value: &T) -> Result<String, Box<dyn Error>>
where
	T: Serialize,
{
	Ok(serde_json::to_string_pretty(value)?)
}

/// Deserialize a JSON string to a value
pub fn from_json<T>(json: &str) -> Result<T, Box<dyn Error>>
where
	T: for<'de> Deserialize<'de>,
{
	Ok(serde_json::from_str(json)?)
}

/// Deserialize a JSON string to a serde_json::Value
pub fn parse_json(json: &str) -> Result<Value, Box<dyn Error>> {
	Ok(serde_json::from_str(json)?)
}

/// Convert a value to a serde_json::Value
pub fn to_value<T>(value: &T) -> Result<Value, Box<dyn Error>>
where
	T: Serialize,
{
	Ok(serde_json::to_value(value)?)
}

/// Convert a serde_json::Value to a value
pub fn from_value<T>(value: Value) -> Result<T, Box<dyn Error>>
where
	T: for<'de> Deserialize<'de>,
{
	Ok(serde_json::from_value(value)?)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde::{Deserialize, Serialize};

	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct TestStruct {
		name: String,
		value: u32,
	}

	#[test]
	fn test_json_serialization() {
		let test = TestStruct { name: "test".to_string(), value: 123 };

		let json = to_json(&test).unwrap();
		let parsed: TestStruct = from_json(&json).unwrap();

		assert_eq!(test, parsed);
	}

	#[test]
	fn test_json_value_conversion() {
		let test = TestStruct { name: "test".to_string(), value: 123 };

		let value = to_value(&test).unwrap();
		let parsed: TestStruct = from_value(value).unwrap();

		assert_eq!(test, parsed);
	}
}
