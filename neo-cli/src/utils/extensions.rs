use neo3::prelude::*;
use neo3::neo_builder::Transaction;
use neo3::neo_providers::JsonRpcProvider;

/// Extension trait for Transaction to get human-readable transaction type name
pub trait TransactionExtensions {
    fn type_name(&self) -> String;
}

impl<'a, P> TransactionExtensions for Transaction<'a, P> 
where 
    P: JsonRpcProvider + 'static
{
    fn type_name(&self) -> String {
        match self.version {
            0 => "Invocation".to_string(),
            _ => format!("Unknown (Version {})", self.version),
        }
    }
}

/// Also implement for RTransaction (RPC response transaction)
impl TransactionExtensions for RTransaction {
    fn type_name(&self) -> String {
        match self.version {
            0 => "Invocation".to_string(),
            _ => format!("Unknown (Version {})", self.version),
        }
    }
} 