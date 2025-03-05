use num_enum::TryFromPrimitive;
use strum_macros::{Display, EnumString};

#[derive(Display, EnumString, Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub(crate) enum Role {
	#[strum_macros::strum(serialize = "StateValidator")]
	StateValidator = 0x04,
	#[strum_macros::strum(serialize = "Oracle")]
	Oracle = 0x08,
	#[strum_macros::strum(serialize = "NeoFSAlphabetNode")]
	NeoFsAlphabetNode = 0x10,
}

impl Role {
	pub(crate) fn byte_repr(self) -> u8 {
		self as u8
	}
}
