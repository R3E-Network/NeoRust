// Temporarily comment out to avoid circular dependency
// use neo_builder::Witness;
use serde::{Deserialize, Serialize};
use neo_types::Base64Encode;

// Define a local struct for Witness to avoid dependency
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    pub invocation_script: Vec<u8>,
    pub verification_script: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct NeoWitness {
	pub invocation: String,
	pub verification: String,
}

impl NeoWitness {
	pub fn new(invocation: String, verification: String) -> Self {
		Self { invocation, verification }
	}

	pub fn from_witness(witness: &Witness) -> Self {
		Self {
			invocation: Base64Encode::to_base64(&witness.invocation_script),
			verification: Base64Encode::to_base64(&witness.verification_script),
		}
	}
}
