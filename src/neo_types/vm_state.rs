use num_enum::TryFromPrimitive;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
	Display,
	EnumString,
	Debug,
	Clone,
	Copy,
	Hash,
	PartialEq,
	Eq,
	TryFromPrimitive,
	Serialize,
	Deserialize,
)]
#[repr(u8)]
pub enum VMState {
	#[strum(serialize = "NONE")]
	None = 0,
	#[strum(serialize = "HALT")]
	Halt = 1,
	#[strum(serialize = "FAULT")]
	Fault = 2,
	#[strum(serialize = "BREAK")]
	Break = 4,
}
