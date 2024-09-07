//! This module contains implementations for witness rules in the NEO blockchain.
//!
//! It includes:
//! - `WitnessAction`: Represents the action to be taken (Allow or Deny).
//! - `WitnessCondition`: Represents various conditions for witness rules.
//! - `WitnessRule`: Combines an action and a condition to form a complete rule.
//!
//! This module provides structures and implementations for creating, serializing,
//! and deserializing witness rules used in NEO smart contracts.
//!
//! # Usage
//!
//! To use witness rules in your NEO blockchain transactions:
//!
//! 1. Import the necessary types:
//!    ```rust
//!    use neo_builder::transaction::witness_rule::{WitnessAction, WitnessCondition, WitnessRule};
//!    ```
//!
//! 2. Create a witness rule:
//!    ```rust
//!    let condition = WitnessCondition::CalledByEntry;
//!    let rule = WitnessRule::new(WitnessAction::Allow, condition);
//!    ```
//!
//! 3. Use the rule in your transaction or smart contract:
//!    ```
//!    let mut tx_builder = TransactionBuilder::new();
//!    tx_builder.add_witness_rule(rule);
//!    // ... add other transaction details ...
//!    let tx = tx_builder.build().unwrap();
//!    ```
//!
//! 4. Serialize or deserialize witness rules as needed:
//!    ```rust
//!    let serialized = rule.to_array();
//!    let deserialized = WitnessRule::from_bytes(&serialized).unwrap();
//!    ```
//!
//! Remember to handle errors and consider the implications of different witness conditions
//! and actions for your specific use case.

pub use witness_action::*;
pub use witness_condition::*;
pub use witness_rule::*;

mod witness_action;
mod witness_condition;
mod witness_rule;
