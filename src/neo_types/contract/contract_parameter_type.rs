#[cfg(feature = "num_enum")]
use num_enum::TryFromPrimitive;
#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "strum")]
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Hash, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "strum", derive(Display, EnumString))]
#[cfg_attr(feature = "num_enum", derive(TryFromPrimitive))]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
#[repr(u8)]
#[serde(rename_all = "PascalCase")]
pub enum ContractParameterType {
    Any = 0x00,
    Boolean = 0x10,
    Integer = 0x11,
    ByteArray = 0x12,
    String = 0x13,
    Hash160 = 0x14,
    Hash256 = 0x15,
    PublicKey = 0x16,
    Signature = 0x17,
    Array = 0x20,
    Map = 0x22,
    InteropInterface = 0x30,
    Void = 0xff,
}
