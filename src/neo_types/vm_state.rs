#[cfg(feature = "num_enum")]
use num_enum::TryFromPrimitive;
#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "strum")]
use strum_macros::{Display, EnumString};

/// Represents the state of a virtual machine.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "strum", derive(Display, EnumString))]
#[cfg_attr(feature = "num_enum", derive(TryFromPrimitive))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
#[serde(rename_all = "UPPERCASE")]
pub enum VMState {
	/// The virtual machine is in the "NONE" state.
	#[strum(serialize = "NONE")]
	None = 0,
	/// The virtual machine is in the "HALT" state.
	#[strum(serialize = "HALT")]
	Halt = 1,
	/// The virtual machine is in the "FAULT" state.
	#[strum(serialize = "FAULT")]
	Fault = 2,
	/// The virtual machine is in the "BREAK" state.
	#[strum(serialize = "BREAK")]
	Break = 4,
}

impl Default for VMState {
	fn default() -> Self {
		// Provide a default implementation for VMState
		VMState::None
	}
}
