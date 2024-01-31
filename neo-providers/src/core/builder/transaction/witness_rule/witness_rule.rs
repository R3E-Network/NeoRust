use crate::core::transaction::{
	transaction_error::TransactionError,
	witness_rule::{witness_action::WitnessAction, witness_condition::WitnessCondition},
	witness_scope::WitnessScope::WitnessRules,
};
use neo_codec::{encode::NeoSerializable, Decoder, Encoder};
use serde::{ser::SerializeTuple, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Clone)]
pub struct WitnessRule {
	pub action: WitnessAction,
	pub condition: WitnessCondition,
}

impl WitnessRule {
	pub fn new(action: WitnessAction, condition: WitnessCondition) -> Self {
		Self { action, condition }
	}
}

impl NeoSerializable for WitnessRule {
	type Error = TransactionError;

	fn size(&self) -> usize {
		1 + self.condition.size()
	}

	fn encode(&self, writer: &mut Encoder) {
		writer.write_u8(self.action as u8);
		writer.write_serializable_fixed(&self.condition);
	}

	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error> {
		let action = reader.read_u8();
		let condition = WitnessCondition::decode(reader)?;
		Ok(Self { action: WitnessAction::try_from(action).unwrap(), condition })
	}
	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
