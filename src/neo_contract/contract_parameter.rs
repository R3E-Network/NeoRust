use serde::{Serialize, Deserialize};
use crate::neo_types::ScriptHash;

/// The type of a contract parameter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractParameterType {
    /// A null parameter
    Null,
    /// A boolean parameter
    Boolean,
    /// An integer parameter
    Integer,
    /// A byte array parameter
    ByteArray,
    /// A string parameter
    String,
    /// A hash160 parameter (usually a script hash)
    Hash160,
    /// A hash256 parameter (usually a transaction hash)
    Hash256,
    /// A public key parameter
    PublicKey,
    /// A signature parameter
    Signature,
    /// An array parameter
    Array,
    /// A map parameter
    Map,
    /// An InteropInterface parameter
    InteropInterface,
    /// A parameter of any type
    Any,
}

/// A parameter to a smart contract method
#[derive(Debug, Clone)]
pub struct ContractParameter {
    /// The type of the parameter
    parameter_type: ContractParameterType,
    /// The value of the parameter
    value: Option<ContractParameterValue>,
}

/// The value of a contract parameter
#[derive(Debug, Clone)]
pub enum ContractParameterValue {
    /// A boolean value
    Boolean(bool),
    /// An integer value
    Integer(i64),
    /// A byte array value
    ByteArray(Vec<u8>),
    /// A string value
    String(String),
    /// A hash160 value
    Hash160(ScriptHash),
    /// An array of parameters
    Array(Vec<ContractParameter>),
    /// A map of parameter keys to parameter values
    Map(Vec<(ContractParameter, ContractParameter)>),
}

impl ContractParameter {
    /// Create a null parameter
    pub fn null() -> Self {
        Self {
            parameter_type: ContractParameterType::Null,
            value: None,
        }
    }
    
    /// Create a boolean parameter
    pub fn boolean(value: bool) -> Self {
        Self {
            parameter_type: ContractParameterType::Boolean,
            value: Some(ContractParameterValue::Boolean(value)),
        }
    }
    
    /// Create an integer parameter
    pub fn integer<T: Into<i64>>(value: T) -> Self {
        Self {
            parameter_type: ContractParameterType::Integer,
            value: Some(ContractParameterValue::Integer(value.into())),
        }
    }
    
    /// Create a byte array parameter
    pub fn byte_array(value: Vec<u8>) -> Self {
        Self {
            parameter_type: ContractParameterType::ByteArray,
            value: Some(ContractParameterValue::ByteArray(value)),
        }
    }
    
    /// Create a string parameter
    pub fn string<T: Into<String>>(value: T) -> Self {
        Self {
            parameter_type: ContractParameterType::String,
            value: Some(ContractParameterValue::String(value.into())),
        }
    }
    
    /// Create a hash160 parameter
    pub fn hash160(value: &ScriptHash) -> Self {
        Self {
            parameter_type: ContractParameterType::Hash160,
            value: Some(ContractParameterValue::Hash160(value.clone())),
        }
    }
    
    /// Create an array parameter
    pub fn array(value: Vec<ContractParameter>) -> Self {
        Self {
            parameter_type: ContractParameterType::Array,
            value: Some(ContractParameterValue::Array(value)),
        }
    }
    
    /// Create a map parameter
    pub fn map(value: Vec<(ContractParameter, ContractParameter)>) -> Self {
        Self {
            parameter_type: ContractParameterType::Map,
            value: Some(ContractParameterValue::Map(value)),
        }
    }
    
    /// Create a parameter of any type
    pub fn any(value: Option<ContractParameterValue>) -> Self {
        Self {
            parameter_type: ContractParameterType::Any,
            value,
        }
    }
    
    /// Get the type of the parameter
    pub fn get_type(&self) -> &ContractParameterType {
        &self.parameter_type
    }
    
    /// Get the value of the parameter
    pub fn get_value(&self) -> Option<&ContractParameterValue> {
        self.value.as_ref()
    }
}

impl ContractParameterValue {
    /// Get the boolean value, if this is a Boolean parameter
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ContractParameterValue::Boolean(value) => Some(*value),
            _ => None,
        }
    }
    
    /// Get the integer value, if this is an Integer parameter
    pub fn as_int(&self) -> Option<i64> {
        match self {
            ContractParameterValue::Integer(value) => Some(*value),
            _ => None,
        }
    }
    
    /// Get the byte array value, if this is a ByteArray parameter
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            ContractParameterValue::ByteArray(value) => Some(value),
            _ => None,
        }
    }
    
    /// Get the string value, if this is a String parameter
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ContractParameterValue::String(value) => Some(value),
            _ => None,
        }
    }
    
    /// Get the hash160 value, if this is a Hash160 parameter
    pub fn as_hash160(&self) -> Option<&ScriptHash> {
        match self {
            ContractParameterValue::Hash160(value) => Some(value),
            _ => None,
        }
    }
    
    /// Get the array value, if this is an Array parameter
    pub fn as_array(&self) -> Option<&[ContractParameter]> {
        match self {
            ContractParameterValue::Array(value) => Some(value),
            _ => None,
        }
    }
    
    /// Get the map value, if this is a Map parameter
    pub fn as_map(&self) -> Option<&[(ContractParameter, ContractParameter)]> {
        match self {
            ContractParameterValue::Map(value) => Some(value),
            _ => None,
        }
    }
} 