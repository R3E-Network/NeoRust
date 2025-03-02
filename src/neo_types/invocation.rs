use serde::{Deserialize, Serialize};
use crate::neo_types::stack_item::StackItem;

/// Represents the result of a contract invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationResult {
    /// The script that was invoked.
    pub script: String,
    /// The state of the virtual machine after execution.
    pub state: String,
    /// The amount of gas consumed during execution.
    pub gas_consumed: String,
    /// The stack items returned from the invocation.
    pub stack: Vec<StackItem>,
    /// Notifications produced during execution.
    pub notifications: Vec<Notification>,
}

/// Represents a notification emitted during contract execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// The contract hash.
    pub contract: String,
    /// The event name.
    pub event_name: String,
    /// The state.
    pub state: StackItem,
}

impl InvocationResult {
    /// Checks if the VM state is "HALT", indicating successful execution.
    pub fn is_halt(&self) -> bool {
        self.state.to_uppercase() == "HALT"
    }

    /// Checks if the VM state is "FAULT", indicating failed execution.
    pub fn is_fault(&self) -> bool {
        self.state.to_uppercase() == "FAULT"
    }

    /// Gets the first stack item, if any.
    pub fn first_item(&self) -> Option<&StackItem> {
        self.stack.first()
    }
} 