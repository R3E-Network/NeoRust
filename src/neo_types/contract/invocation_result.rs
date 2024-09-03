use std::{
	collections::HashMap,
	hash::{Hash, Hasher},
};

use primitive_types::H160;
use serde::{Deserialize, Serialize};
use strum;
use strum_macros::{AsRefStr, Display, EnumString};

use neo::prelude::{deserialize_script_hash, serialize_script_hash, ContractParameter, StackItem};

use crate::prelude::TypeError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct InvocationResult {
	#[serde(default)]
	pub script: String,
	#[serde(default)]
	pub state: NeoVMStateType,
	#[serde(rename = "gasconsumed", default)]
	pub gas_consumed: String,
	#[serde(default)]
	pub exception: Option<String>,
	#[serde(default)]
	pub notifications: Option<Vec<Notification>>,
	#[serde(default)]
	pub diagnostics: Option<Diagnostics>,
	#[serde(default)]
	pub stack: Vec<StackItem>,
	#[serde(default)]
	pub tx: Option<String>,
	#[serde(rename = "pendingsignature", default)]
	pub pending_signature: Option<PendingSignature>,
	#[serde(rename = "session", default)]
	pub session_id: Option<String>,
}

#[derive(Serialize, Deserialize, EnumString, AsRefStr, Debug, PartialEq, Eq, Clone, Hash)]
#[strum(serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum NeoVMStateType {
	Halt,
	Fault,
	Break,
	StepInto,
	StepOut,
	StepOver,
	Exception,
}

impl Default for NeoVMStateType {
	fn default() -> Self {
		NeoVMStateType::Halt
	}
}

impl InvocationResult {
	// constructor and helper methods
	pub fn new(
		script: String,
		state: NeoVMStateType,
		gas_consumed: String,
		exception: Option<String>,
		notifications: Option<Vec<Notification>>,
		diagnostics: Option<Diagnostics>,
		stack: Vec<StackItem>,
		tx: Option<String>,
		pending_signature: Option<PendingSignature>,
		session_id: Option<String>,
	) -> Self {
		Self {
			script,
			state,
			gas_consumed,
			exception,
			notifications,
			diagnostics,
			stack,
			tx,
			pending_signature,
			session_id,
		}
	}

	pub fn has_state_fault(&self) -> bool {
		matches!(self.state, NeoVMStateType::Fault)
	}

	pub fn get_first_stack_item(&self) -> Result<&StackItem, TypeError> {
        if self.stack.is_empty() {
            return Err(TypeError::IndexOutOfBounds(
                "The stack is empty. This means that no items were left on the NeoVM stack after this invocation."
                    .to_string(),
            ));
        }
        self.get_stack_item(0)
    }

    pub fn get_stack_item(&self, index: usize) -> Result<&StackItem, TypeError> {
        if index >= self.stack.len() {
            return Err(TypeError::IndexOutOfBounds(format!(
                "There were only {} items left on the NeoVM stack after this invocation",
                self.stack.len()
            )));
        }
        Ok(&self.stack[index])
    }

	pub fn get_first_notification(&self) -> Result<&Notification, TypeError> {
        if self.notifications.as_ref().unwrap().is_empty() {
            return Err(TypeError::IndexOutOfBounds(
                "No notifications have been sent in this invocation.".to_string(),
            ));
        }
        self.get_notification(0)
    }

    pub fn get_notification(&self, index: usize) -> Result<&Notification, TypeError> {
        if index >= self.notifications.as_ref().unwrap().len() {
            return Err(TypeError::IndexOutOfBounds(format!(
                "Only {} notifications have been sent in this invocation. Tried to access index {} in the invocation result.",
                self.notifications.as_ref().unwrap().len(),
                index
            )));
        }
        Ok(&self.notifications.as_ref().unwrap()[index])
    }
}

impl Default for InvocationResult {
	fn default() -> Self {
		Self {
			script: "0001020304".to_string(),
			state: NeoVMStateType::Halt,
			gas_consumed: "1234567".to_string(),
			exception: None,
			notifications: None,
			diagnostics: None,
			stack: vec![],
			tx: None,
			pending_signature: None,
			session_id: None,
		}
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PendingSignature {
	#[serde(rename = "type")]
	pub typ: String,
	pub data: String,
	pub items: HashMap<String, Item>,
	pub network: u32,
}

impl Hash for PendingSignature {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.typ.hash(state);
		self.data.hash(state);
		// self.items.hash(state);
		self.network.hash(state);
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Item {
	pub script: String,
	#[serde(default)]
	pub parameters: Vec<ContractParameter>,
	pub signatures: HashMap<String, String>,
}

impl Hash for Item {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.script.hash(state);
		self.parameters.hash(state);
		// self.signatures.hash(state);
	}
}

// Other structs like Diagnostics, Notification

// Diagnostics
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Diagnostics {
	#[serde(rename = "invokedcontracts")]
	pub invoked_contracts: InvokedContract,
	#[serde(rename = "storagechanges", default)]
	pub storage_changes: Vec<StorageChange>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct InvokedContract {
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	pub hash: H160,
	#[serde(rename = "call", default)]
	pub invoked_contracts: Vec<InvokedContract>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StorageChange {
	pub state: String,
	pub key: String,
	pub value: String,
}

// Notification
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Notification {
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	pub contract: H160,
	#[serde(rename = "eventname")]
	pub event_name: String,
	pub state: StackItem,
	// pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
pub enum NotificationState {
	Failure,
	Success,
	Halt,
	Fault,
	StepInto,
	StepOut,
	StepOver,
	Break,
}
