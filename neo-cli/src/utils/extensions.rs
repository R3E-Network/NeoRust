use neo3::prelude::*;

/// Extension trait for Transaction to get human-readable transaction type name
pub trait TransactionExtensions {
    fn type_name(&self) -> String;
}

impl TransactionExtensions for Transaction {
    fn type_name(&self) -> String {
        match self.version {
            0 => "Invocation".to_string(),
            _ => format!("Unknown (Version {})", self.version),
        }
    }
} 