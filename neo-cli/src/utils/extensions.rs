use neo::prelude::*;
use primitive_types::H160;

/// Extension trait for transactions with additional helpful methods
pub trait TransactionExtensions {
    /// Get a string representation of the transaction type
    fn get_type_name(&self) -> &'static str;
    
    /// Format the transaction for CLI display
    fn format_for_cli(&self) -> String;
}

/// Implementation of the extension trait for Neo transactions
impl TransactionExtensions for neo::neo_types::Transaction {
    fn get_type_name(&self) -> &'static str {
        match self.version {
            0 => "Invocation Transaction",
            _ => "Unknown Transaction Type",
        }
    }
    
    fn format_for_cli(&self) -> String {
        let mut result = String::new();
        
        result.push_str(&format!("Hash: {}\n", self.hash));
        result.push_str(&format!("Type: {}\n", self.get_type_name()));
        result.push_str(&format!("Version: {}\n", self.version));
        result.push_str(&format!("Nonce: {}\n", self.nonce));
        result.push_str(&format!("Sender: {}\n", self.sender));
        result.push_str(&format!("System Fee: {}\n", self.sys_fee));
        result.push_str(&format!("Network Fee: {}\n", self.net_fee));
        result.push_str(&format!("Valid Until Block: {}\n", self.valid_until_block));
        
        if !self.signers.is_empty() {
            result.push_str("Signers:\n");
            for (i, signer) in self.signers.iter().enumerate() {
                result.push_str(&format!("  {}. {}\n", i+1, signer.account));
            }
        }
        
        result
    }
}

/// Helper function to calculate contract hash based on owner, checksum and name
pub fn calculate_contract_hash(owner: &H160, checksum: u32, name_bytes: &[u8]) -> H160 {
    use neo::neo_types::script_hash::HashableForVec;
    
    let mut data = Vec::new();
    data.extend_from_slice(owner.as_bytes());
    data.extend_from_slice(&checksum.to_be_bytes());
    data.extend_from_slice(name_bytes);
    
    data.hash160()
} 