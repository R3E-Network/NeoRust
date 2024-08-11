use num_enum::TryFromPrimitive;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
    Display,
    EnumString,
    Debug,
    Clone,
    Hash,
    Copy,
    PartialEq,
    Eq,
    TryFromPrimitive,
    Serialize,
    Deserialize,
)]
#[repr(u8)]
#[serde(rename_all = "PascalCase")]
pub enum ContractParameterType {
    #[strum(serialize = "Any")]
    Any = 0x00,
    #[strum(serialize = "Boolean")]
    Boolean = 0x10,
    #[strum(serialize = "Integer")]
    Integer = 0x11,
    #[strum(serialize = "ByteArray")]
    ByteArray = 0x12,
    #[strum(serialize = "String")]
    String = 0x13,
    #[strum(serialize = "Hash160")]
    #[serde(rename = "Hash160")]
    H160 = 0x14,
    #[strum(serialize = "Hash256")]
    #[serde(rename = "Hash256")]
    H256 = 0x15,
    #[strum(serialize = "PublicKey")]
    PublicKey = 0x16,
    #[strum(serialize = "Signature")]
    Signature = 0x17,
    #[strum(serialize = "Array")]
    Array = 0x20,
    #[strum(serialize = "Map")]
    Map = 0x22,
    #[strum(serialize = "InteropInterface")]
    InteropInterface = 0x30,
    #[strum(serialize = "Void")]
    Void = 0xff,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_parameter_type_deserialization() {
        let json = r#"
        {
            "type": "Boolean"
        }
        "#;

        #[derive(Deserialize)]
        struct Test {
            #[serde(rename = "type")]
            param_type: ContractParameterType,
        }

        let result: Test = serde_json::from_str(json).unwrap();
        assert_eq!(result.param_type, ContractParameterType::Boolean);
    }

    #[test]
    fn test_contract_parameter_type_serialization() {
        assert_eq!(serde_json::to_string(&ContractParameterType::Boolean).unwrap(), "\"Boolean\"");
        assert_eq!(serde_json::to_string(&ContractParameterType::Integer).unwrap(), "\"Integer\"");
        assert_eq!(serde_json::to_string(&ContractParameterType::String).unwrap(), "\"String\"");

        assert_eq!(serde_json::to_string(&ContractParameterType::H160).unwrap(), "\"Hash160\"");
        assert_eq!(serde_json::to_string(&ContractParameterType::H256).unwrap(), "\"Hash256\"");

        #[derive(Serialize)]
        struct Test {
            #[serde(rename = "type")]
            param_type: ContractParameterType,
        }

        let test = Test { param_type: ContractParameterType::Array };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"type":"Array"}"#);
    }
}
