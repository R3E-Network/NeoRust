use crate::utils::error::{CliError, CliResult};
use neo::prelude::*;
use std::time::Duration;

/// Connect to a Ledger device and retrieve accounts
pub async fn connect_ledger() -> CliResult<Vec<neo::neo_protocol::account::Account>> {
    #[cfg(feature = "ledger")]
    {
        // Connect to ledger
        use crate::utils::{print_info, print_error};
        
        print_info("Connecting to Ledger device...");
        print_info("Please make sure your Ledger is connected and the Neo app is open.");
        
        // Try to connect with a timeout
        let timeout = Duration::from_secs(60); // 60 seconds timeout
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout {
            match neo::neo_protocol::ledger::LedgerWallet::connect() {
                Ok(ledger) => {
                    print_info("Ledger device connected successfully!");
                    
                    // Get accounts from ledger
                    let mut accounts = Vec::new();
                    
                    // Try to get the first 5 accounts
                    for i in 0..5 {
                        match ledger.get_account(i) {
                            Ok(account) => {
                                accounts.push(account);
                            },
                            Err(e) => {
                                if i == 0 {
                                    return Err(CliError::Wallet(format!("Failed to get account from Ledger: {}", e)));
                                }
                                // Stop if we can't get more accounts
                                break;
                            }
                        }
                    }
                    
                    if accounts.is_empty() {
                        return Err(CliError::Wallet("No accounts found on Ledger device".to_string()));
                    }
                    
                    return Ok(accounts);
                },
                Err(_) => {
                    // Wait a bit before retrying
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
        
        print_error("Timeout connecting to Ledger device.");
        Err(CliError::Wallet("Timeout connecting to Ledger device".to_string()))
    }
    
    #[cfg(not(feature = "ledger"))]
    {
        Err(CliError::Wallet("Ledger support is not enabled in this build".to_string()))
    }
}

/// Sign a transaction using a Ledger device
pub async fn sign_transaction_with_ledger(
    tx: &mut neo::neo_types::Transaction,
    account_index: u32
) -> CliResult<()> {
    #[cfg(feature = "ledger")]
    {
        // Connect to ledger
        let ledger = neo::neo_protocol::ledger::LedgerWallet::connect()
            .map_err(|e| CliError::Wallet(format!("Failed to connect to Ledger: {}", e)))?;
        
        // Sign the transaction
        ledger.sign_transaction(tx, account_index)
            .map_err(|e| CliError::Wallet(format!("Failed to sign transaction with Ledger: {}", e)))?;
        
        Ok(())
    }
    
    #[cfg(not(feature = "ledger"))]
    {
        Err(CliError::Wallet("Ledger support is not enabled in this build".to_string()))
    }
} 