//! OpCode definitions for Neo VM
//!
//! This module defines the opcodes used in the Neo Virtual Machine.

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::convert::From;

/// Neo VM OpCodes
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Serialize, Deserialize
)]
#[repr(u8)]
pub enum OpCode {
    // Constants
    /// Pushes a constant 0 onto the stack.
    Push0 = 0x00,
    /// Pushes a constant -1 onto the stack.
    PushM1 = 0x01,
    /// Pushes a constant 1 onto the stack.
    Push1 = 0x11,
    /// Pushes a constant 2 onto the stack.
    Push2 = 0x12,
    /// Pushes a constant 3 onto the stack.
    Push3 = 0x13,
    /// Pushes a constant 4 onto the stack.
    Push4 = 0x14,
    /// Pushes a constant 5 onto the stack.
    Push5 = 0x15,
    /// Pushes a constant 6 onto the stack.
    Push6 = 0x16,
    /// Pushes a constant 7 onto the stack.
    Push7 = 0x17,
    /// Pushes a constant 8 onto the stack.
    Push8 = 0x18,
    /// Pushes a constant 9 onto the stack.
    Push9 = 0x19,
    /// Pushes a constant 10 onto the stack.
    Push10 = 0x1a,
    /// Pushes a constant 11 onto the stack.
    Push11 = 0x1b,
    /// Pushes a constant 12 onto the stack.
    Push12 = 0x1c,
    /// Pushes a constant 13 onto the stack.
    Push13 = 0x1d,
    /// Pushes a constant 14 onto the stack.
    Push14 = 0x1e,
    /// Pushes a constant 15 onto the stack.
    Push15 = 0x1f,
    /// Pushes a constant 16 onto the stack.
    Push16 = 0x20,

    // Flow control
    /// No operation.
    Nop = 0x21,
    /// Jumps to a target instruction.
    Jmp = 0x22,
    /// Jumps to a target instruction if the top stack item is true.
    JmpIf = 0x23,
    /// Jumps to a target instruction if the top stack item is false.
    JmpIfNot = 0x24,
    /// Jumps to a target instruction if the top two stack items are equal.
    JmpEq = 0x25,
    /// Jumps to a target instruction if the top two stack items are not equal.
    JmpNe = 0x26,
    /// Jumps to a target instruction if the top stack item is greater than the second top stack item.
    JmpGt = 0x27,
    /// Jumps to a target instruction if the top stack item is greater than or equal to the second top stack item.
    JmpGe = 0x28,
    /// Jumps to a target instruction if the top stack item is less than the second top stack item.
    JmpLt = 0x29,
    /// Jumps to a target instruction if the top stack item is less than or equal to the second top stack item.
    JmpLe = 0x2a,
    /// Calls a target instruction.
    Call = 0x2b,
    /// Calls a target instruction if the top stack item is true.
    CallIf = 0x2c,
    /// Calls a target instruction if the top stack item is false.
    CallIfNot = 0x2d,
    /// Calls a target instruction if the top two stack items are equal.
    CallEq = 0x2e,
    /// Calls a target instruction if the top two stack items are not equal.
    CallNe = 0x2f,
    /// Calls a target instruction if the top stack item is greater than the second top stack item.
    CallGt = 0x30,
    /// Calls a target instruction if the top stack item is greater than or equal to the second top stack item.
    CallGe = 0x31,
    /// Calls a target instruction if the top stack item is less than the second top stack item.
    CallLt = 0x32,
    /// Calls a target instruction if the top stack item is less than or equal to the second top stack item.
    CallLe = 0x33,
    /// Returns from the current method.
    Ret = 0x34,
    /// Calls a method of the current contract.
    CallToken = 0x35,
    /// Calls a method of a specific contract.
    CallContract = 0x36,
    /// Aborts the execution.
    Abort = 0x37,
    /// Asserts that the top stack item is true.
    Assert = 0x38,
    /// Throws an exception.
    Throw = 0x39,
    /// Catches an exception.
    Try = 0x3a,
    /// Catches an exception.
    TryCatch = 0x3b,
    /// Finalizes a try-catch block.
    Finally = 0x3c,
    /// Finalizes a try-catch block.
    EndTry = 0x3d,
    /// Finalizes a try-catch block.
    EndFinally = 0x3e,
    /// Returns from the current method with a value.
    EndTryCatch = 0x3f,

    // Stack
    /// Duplicates the top stack item.
    Dup = 0x40,
    /// Swaps the top two stack items.
    Swap = 0x41,
    /// Removes the top stack item.
    Pop = 0x42,
    /// Removes the second item on the stack.
    Nip = 0x43,
    /// Converts the top stack item to a boolean.
    ConvertTo = 0x44,
    /// Converts the top stack item to a different type.
    ConvertTo2 = 0x45,
    /// Packs the top n stack items into an array.
    Pack = 0x46,
    /// Unpacks an array into multiple stack items.
    Unpack = 0x47,
    /// Duplicates the top n stack items.
    DupN = 0x48,
    /// Reverses the top n stack items.
    RevN = 0x49,
    /// Removes the top n stack items.
    PopN = 0x4a,
    /// Pushes a null value onto the stack.
    PushNull = 0x4b,
    /// Pushes a data value onto the stack.
    PushData1 = 0x4c,
    /// Pushes a data value onto the stack.
    PushData2 = 0x4d,
    /// Pushes a data value onto the stack.
    PushData4 = 0x4e,
    /// Pushes an 8-bit integer onto the stack.
    PushInt8 = 0x50,
    /// Pushes a 16-bit integer onto the stack.
    PushInt16 = 0x51,
    /// Pushes a 32-bit integer onto the stack.
    PushInt32 = 0x52,
    /// Pushes a 64-bit integer onto the stack.
    PushInt64 = 0x53,
    /// Pushes a 128-bit integer onto the stack.
    PushInt128 = 0x54,
    /// Pushes a 256-bit integer onto the stack.
    PushInt256 = 0x55,
    /// Pushes a boolean true value onto the stack.
    PushTrue = 0x56,
    /// Pushes a boolean false value onto the stack.
    PushFalse = 0x57,

    // Slots
    /// Initializes a static field.
    InitStaticField = 0x58,
    /// Loads a static field onto the stack.
    LdStaticField = 0x59,
    /// Stores the top stack item to a static field.
    StStaticField = 0x5a,
    /// Loads a constant onto the stack.
    LdConstant = 0x5b,

    // Exceptions
    /// Loads the current exception onto the stack.
    LdException = 0x5c,

    // Context
    /// Loads the current context onto the stack.
    LdContext = 0x5d,

    // Type
    /// Checks if the top stack item is of a specific type.
    IsType = 0x5e,
    /// Converts the top stack item to a specific type.
    Convert2 = 0x5f,

    // Bitwise operations
    /// Performs a bitwise AND operation on the top two stack items.
    And = 0x60,
    /// Performs a bitwise OR operation on the top two stack items.
    Or = 0x61,
    /// Performs a bitwise XOR operation on the top two stack items.
    Xor = 0x62,
    /// Performs a bitwise NOT operation on the top stack item.
    Not = 0x63,
    /// Performs a bitwise left shift operation on the top two stack items.
    Shl = 0x64,
    /// Performs a bitwise right shift operation on the top two stack items.
    Shr = 0x65,

    // Arithmetic operations
    /// Adds the top two stack items.
    Add = 0x66,
    /// Subtracts the top stack item from the second top stack item.
    Sub = 0x67,
    /// Multiplies the top two stack items.
    Mul = 0x68,
    /// Divides the second top stack item by the top stack item.
    Div = 0x69,
    /// Calculates the remainder of the division of the second top stack item by the top stack item.
    Mod = 0x6a,
    /// Negates the top stack item.
    Neg = 0x6b,
    /// Increments the top stack item by 1.
    Inc = 0x6c,
    /// Decrements the top stack item by 1.
    Dec = 0x6d,
    /// Calculates the sign of the top stack item.
    Sign = 0x6e,
    /// Calculates the absolute value of the top stack item.
    Abs = 0x6f,
    /// Calculates the minimum of the top two stack items.
    Min = 0x70,
    /// Calculates the maximum of the top two stack items.
    Max = 0x71,
    /// Calculates the top stack item within the range of the second and third top stack items.
    Within = 0x72,

    // String operations
    /// Concatenates the top two stack items.
    Cat = 0x73,
    /// Extracts a substring from the top stack item.
    SubStr = 0x74,
    /// Extracts a substring from the top stack item starting from a specific index.
    Left = 0x75,
    /// Extracts a substring from the top stack item ending at a specific index.
    Right = 0x76,
    /// Calculates the size of the top stack item.
    Size = 0x77,

    // Array operations
    /// Checks if the top stack item contains a specific element.
    HasKey = 0x78,
    /// Gets the keys of the top stack item.
    Keys = 0x79,
    /// Gets the values of the top stack item.
    Values = 0x7a,
    /// Picks a specific item from the stack.
    PickItem = 0x7b,
    /// Appends an item to the top stack item.
    Append = 0x7c,
    /// Reverses the top stack item.
    Reverse = 0x7d,
    /// Removes a specific item from the top stack item.
    Remove = 0x7e,
    /// Clears the top stack item.
    Clear = 0x7f,

    // Comparison operations
    /// Checks if the top two stack items are equal.
    Eq = 0x80,
    /// Checks if the top two stack items are not equal.
    Ne = 0x81,
    /// Checks if the top stack item is greater than the second top stack item.
    Gt = 0x82,
    /// Checks if the top stack item is greater than or equal to the second top stack item.
    Ge = 0x83,
    /// Checks if the top stack item is less than the second top stack item.
    Lt = 0x84,
    /// Checks if the top stack item is less than or equal to the second top stack item.
    Le = 0x85,

    // Numeric operations
    /// Calculates the square root of the top stack item.
    Sqrt = 0x86,

    // Iterator operations
    /// Creates an iterator for the top stack item.
    NewIterator = 0x87,
    /// Checks if the iterator has more elements.
    IteratorNext = 0x88,
    /// Gets the current element of the iterator.
    IteratorKey = 0x89,
    /// Gets the current element of the iterator.
    IteratorValue = 0x8a,

    // System calls
    /// Calls a system function.
    SysCall = 0xfe,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// The TryFrom<u8> implementation is already provided by the TryFromPrimitive derive macro

impl OpCode {
    /// Returns the opcode as a u8 value
    pub fn opcode(&self) -> u8 {
        *self as u8
    }
    
    /// Returns the opcode as a hex string
    pub fn to_hex_string(&self) -> String {
        format!("{:02x}", *self as u8)
    }
    
    /// Returns the operand size for the opcode, if applicable
    pub fn operand_size(&self) -> Option<usize> {
        match self {
            OpCode::PushData1 => Some(1),
            OpCode::PushData2 => Some(2),
            OpCode::PushData4 => Some(4),
            OpCode::PushInt8 => Some(1),
            OpCode::PushInt16 => Some(2),
            OpCode::PushInt32 => Some(4),
            OpCode::PushInt64 => Some(8),
            OpCode::PushInt128 => Some(16),
            OpCode::PushInt256 => Some(32),
            OpCode::Jmp | OpCode::JmpIf | OpCode::JmpIfNot | OpCode::JmpEq | OpCode::JmpNe |
            OpCode::JmpGt | OpCode::JmpGe | OpCode::JmpLt | OpCode::JmpLe |
            OpCode::Call | OpCode::CallIf | OpCode::CallIfNot | OpCode::CallEq | OpCode::CallNe |
            OpCode::CallGt | OpCode::CallGe | OpCode::CallLt | OpCode::CallLe => Some(2),
            OpCode::SysCall => Some(4),
            _ => None,
        }
    }
}
