use crate::core::{error::BuilderError, script::script_builder::ScriptBuilder};
use getset::{Getters, Setters};
use neo_codec::{encode::NeoSerializable, Decoder, Encoder};
use neo_crypto::{hash::HashableForVec, key_pair::KeyPair, keys::Secp256r1Signature};
use neo_types::Bytes;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters, Setters, Serialize, Deserialize)]
#[getset(get_copy, set)]
#[derive(educe::Educe)]
// note `new` below: generate `new()` that calls Default
#[educe(Default(new))]
#[derive(
	derive_more::AsRef,
	derive_more::Deref,
	derive_more::IntoIterator,
	derive_more::Index,
	derive_more::IndexMut,
	derive_more::Into,
	derive_more::From,
)]
pub struct InvocationScript {
	script: Bytes,
}

impl InvocationScript {
	pub fn from_signature(signature: &Secp256r1Signature) -> Self {
		let mut builder = ScriptBuilder::new();
		builder
			.push_data(signature.to_bytes().into())
			.expect("TODO: panic message");
		Self { script: builder.to_bytes() }
	}

	pub fn from_message_and_key_pair(
		message: Bytes,
		key_pair: &KeyPair,
	) -> Result<Self, BuilderError> {
		let message_hash = message.hash256();
		let signature = key_pair.private_key.sign_tx(&message_hash)?;
		let mut builder = ScriptBuilder::new();
		// Convert signature to bytes
		let signature_bytes = signature.to_bytes();
		builder.push_data(signature_bytes.to_vec()).expect("Incorrect signature length");
		Ok(Self { script: builder.to_bytes() })
	}

	pub fn from_signatures(signatures: &[Secp256r1Signature]) -> Self {
		let mut builder = ScriptBuilder::new();
		for signature in signatures {
			let mut signature_bytes = signature.to_bytes();
			// signature.write_scalars(&mut signature_bytes).unwrap();

			builder.push_data(signature_bytes.to_vec()).expect("Incorrect signature length");
		}
		Self { script: builder.to_bytes() }
	}
}

impl NeoSerializable for InvocationScript {
	type Error = BuilderError;

	fn size(&self) -> usize {
		self.script.len()
	}

	fn encode(&self, writer: &mut Encoder) {
		writer.write_var_bytes(&self.script);
	}

	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error> {
		let script = reader.read_var_bytes()?;
		Ok(Self { script })
	}
	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
