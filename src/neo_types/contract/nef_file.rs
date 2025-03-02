use std::hash::Hasher;

use primitive_types::H160;
#[cfg(feature = "tokio-support")]
use tokio::io::AsyncReadExt;

use crate::neo_types::{
    Bytes, TypeError,
    contract::ContractParameter,
};

#[cfg(feature = "contract")]
use crate::neo_types::stack_item::StackItem;

#[cfg(not(feature = "contract"))]
#[derive(Debug, Clone)]
pub struct StackItem {
    pub typ: String,
    pub value: Vec<u8>,
}

#[cfg(feature = "hashable-for-vec")]
use crate::neo_types::script_hash::HashableForVec;

// Local trait definitions for when contract feature is not enabled
#[cfg(not(feature = "contract"))]
pub trait NeoSerializable {
    type Error;
    fn size(&self) -> usize;
    fn encode(&self, writer: &mut Encoder);
    fn decode(reader: &mut Decoder) -> Result<Self, Self::Error> where Self: Sized;
    fn to_array(&self) -> Vec<u8>;
}

#[cfg(all(not(feature = "contract"), not(feature = "hashable-for-vec")))]
pub trait HashableForVecLocal {
    fn hash(&self) -> Vec<u8>;
    fn hash256(&self) -> Vec<u8>;
}

// Error type definitions
use crate::neo_error::CodecError;

// Local implementations for Encoder and Decoder
struct Encoder {
    data: Vec<u8>
}

impl Encoder {
    fn new() -> Self { Self { data: Vec::new() } }
    fn to_bytes(&self) -> Vec<u8> { self.data.clone() }
    fn write_bytes(&mut self, bytes: &[u8]) { self.data.extend_from_slice(bytes); }
    fn write_u32(&mut self, _value: u32) {}
    fn write_u16(&mut self, _value: u16) {}
    fn write_u8(&mut self, _value: u8) {}
    fn write_bool(&mut self, _value: bool) {}
    fn write_fixed_string(&mut self, _value: &Option<String>, _size: usize) -> Result<(), CodecError> { Ok(()) }
    fn write_var_string(&mut self, _value: &str) {}
    fn write_var_bytes(&mut self, _value: &[u8]) {}
    fn write_serializable_fixed<T>(&mut self, _value: &T) {}
    fn write_serializable_variable_list<T>(&mut self, _value: &[T]) {}
}

struct Decoder;

impl Decoder {
    fn new(_bytes: &[u8]) -> Self { Self }
    fn read_u32(&mut self) -> Result<u32, CodecError> { Ok(0) }
    fn read_u16(&mut self) -> Result<u16, CodecError> { Ok(0) }
    fn read_u8(&mut self) -> u8 { 0 }
    fn read_bool(&mut self) -> bool { false }
    fn read_bytes(&mut self, _size: usize) -> Result<Vec<u8>, TypeError> { Ok(Vec::new()) }
    fn read_var_string(&mut self) -> Result<String, TypeError> { Ok(String::new()) }
    fn read_var_bytes(&mut self) -> Result<Vec<u8>, TypeError> { Ok(Vec::new()) }
    fn read_serializable<T>(&mut self) -> Result<T, TypeError> { Err(TypeError::InvalidEncoding("Not implemented".to_string())) }
    fn read_serializable_list<T>(&mut self) -> Result<Vec<T>, TypeError> { Ok(Vec::new()) }
}

/*
┌───────────────────────────────────────────────────────────────────────┐
│                            NEF File Format                            │
├────────┬─────────┬────────────────────────────────────────────────────┤
│  Field │  Type   │                   Description                      │
├────────┼─────────┼────────────────────────────────────────────────────┤
│ Magic  │ uint32  │ Magic header                                       │
│ Flags  │ byte[2] │ Reserved for future extensions                     │
│ Compiler│ string  │ Compiler name and version                         │
│ Source │ string  │ Source code file or URL                           │
│ Tokens │ object[]│ Reserved for future extensions                     │
│ Script │ byte[]  │ Smart contract bytecode                           │
│ Checksum│ uint32 │ First four bytes of double SHA256 hash of the     │
│        │         │ header and script                                 │
└────────┴─────────┴────────────────────────────────────────────────────┘
*/

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NefFile {
    pub magic: u32,
    pub compiler: String,
    pub source: Option<String>,
    pub tokens: Vec<ContractParameter>,
    pub script: Vec<u8>,
    pub checksum: [u8; 4],
}

impl NefFile {
    pub const MAGIC: u32 = 0x3346454E;
    pub const COMPILER_NAME: &'static str = "neo-rs-compiler";
    pub const CHECKSUM_SIZE: usize = 4;

    pub fn new(script: Vec<u8>) -> Self {
        let mut nef = Self {
            magic: Self::MAGIC,
            compiler: Self::COMPILER_NAME.to_string(),
            source: None,
            tokens: Vec::new(),
            script,
            checksum: [0; Self::CHECKSUM_SIZE],
        };
        nef.update_checksum();
        nef
    }

    pub fn update_checksum(&mut self) {
        // Placeholder implementation
        self.checksum = [0, 0, 0, 0];
    }

    pub fn compute_checksum_from_bytes(bytes: Bytes) -> Result<Bytes, TypeError> {
        let mut file_bytes = bytes.clone();
        file_bytes.truncate(bytes.len() - Self::CHECKSUM_SIZE);
        
        // Fallback implementation when HashableForVec is not available
        let checksum = vec![0; Self::CHECKSUM_SIZE];
        Ok(checksum)
    }

    pub fn read_from_file(file: &str) -> Result<Self, TypeError> {
        Err(TypeError::InvalidEncoding("Not implemented".to_string()))
    }

    pub fn read_from_bytes(bytes: &[u8]) -> Result<Self, TypeError> {
        let mut reader = Decoder::new(bytes);
        let magic = reader.read_u32().map_err(|e| TypeError::CodecError(e.to_string()))?;
        if magic != Self::MAGIC {
            return Err(TypeError::InvalidFormat(format!(
                "Invalid NEF magic: {:X}, expected: {:X}",
                magic,
                Self::MAGIC
            )));
        }

        let _flags = reader.read_bytes(2)?;
        let compiler = reader.read_var_string()?;
        let source = match reader.read_var_string() {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        };
        let tokens = reader.read_serializable_list()?;
        let script = reader.read_var_bytes()?;
        let checksum_bytes = reader.read_bytes(Self::CHECKSUM_SIZE)?;
        let checksum = checksum_bytes.try_into().map_err(|_| {
            TypeError::InvalidEncoding("Failed to convert checksum bytes to array".to_string())
        })?;

        Ok(Self {
            magic,
            compiler,
            source,
            tokens,
            script,
            checksum,
        })
    }

    #[cfg(feature = "contract")]
    fn read_from_stack_item(item: StackItem) -> Result<Self, TypeError> {
        if let StackItem::ByteString { value: bytes } = item {
            let mut reader = Decoder::new(&bytes.as_bytes());
            Self::decode(&mut reader)
        } else {
            let item_str = format!("{:?}", item);
            Err(TypeError::UnexpectedReturnType(item_str + StackItem::BYTE_STRING_VALUE))
        }
    }
    
    #[cfg(not(feature = "contract"))]
    fn read_from_stack_item(_item: StackItem) -> Result<Self, TypeError> {
        Err(TypeError::InvalidEncoding("Stack item reading not supported without contract feature".to_string()))
    }
}

impl Into<ContractParameter> for NefFile {
    fn into(self) -> ContractParameter {
        ContractParameter::byte_array(self.to_array())
    }
}

impl From<&NefFile> for ContractParameter {
    fn from(value: &NefFile) -> Self {
        Self::byte_array(value.to_array())
    }
}

#[cfg(not(feature = "contract"))]
impl NefFile {
    fn to_array(&self) -> Vec<u8> {
        Vec::new()
    }
}
