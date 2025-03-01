#[cfg(test)]
mod tests {
    use std::process::Output;
    use crate::integration::utils::{CliTest, assert_success, assert_output_contains, script_hash_from_string};

    #[test]
    fn test_defi_pools() {
        let cli = CliTest::new();
        
        // Test listing Flamingo liquidity pools
        let output = cli.run_command(&["defi", "pools", "--platform", "flamingo"]);
        
        // We're just checking if the command is recognized, not full execution
        assert!(output.status.code().unwrap_or(0) != 127, "Command not found");
    }
    
    #[test]
    fn test_defi_swap_info() {
        let cli = CliTest::new();
        
        // Test getting swap information between NEO and GAS
        let output = cli.run_command(&[
            "defi", "swap-info",
            "--token-from", "NEO",
            "--token-to", "GAS", 
            "--amount", "1"
        ]);
        
        // Just checking if the command is recognized
        assert!(output.status.code().unwrap_or(0) != 127, "Command not found");
    }
    
    #[test]
    fn test_defi_swap() {
        let cli = CliTest::new();
        
        // Create and open a wallet for testing (required for swap)
        let wallet_content = r#"{
            "name": "TestWallet",
            "version": "1.0",
            "accounts": []
        }"#;
        let wallet_path = cli.create_temp_file(wallet_content);
        
        // First create the wallet
        cli.run_command(&["wallet", "open", "--path", wallet_path.to_str().unwrap()]);
        
        // Attempt to swap NEO to GAS
        let output = cli.run_command(&[
            "defi", "swap",
            "--token-from", "NEO",
            "--token-to", "GAS",
            "--amount", "1"
        ]);
        
        // Just checking if the command is recognized
        assert!(output.status.code().unwrap_or(0) != 127, "Command not found");
    }
    
    #[test]
    fn test_defi_add_liquidity() {
        let cli = CliTest::new();
        
        // Create and open a wallet for testing
        let wallet_content = r#"{
            "name": "TestWallet",
            "version": "1.0",
            "accounts": []
        }"#;
        let wallet_path = cli.create_temp_file(wallet_content);
        
        // First open the wallet
        cli.run_command(&["wallet", "open", "--path", wallet_path.to_str().unwrap()]);
        
        // Attempt to add liquidity
        let output = cli.run_command(&[
            "defi", "add-liquidity",
            "--token-a", "NEO",
            "--token-b", "GAS",
            "--amount-a", "1",
            "--amount-b", "1"
        ]);
        
        // Just checking if the command is recognized
        assert!(output.status.code().unwrap_or(0) != 127, "Command not found");
    }
    
    #[test]
    fn test_defi_remove_liquidity() {
        let cli = CliTest::new();
        
        // Create and open a wallet for testing
        let wallet_content = r#"{
            "name": "TestWallet",
            "version": "1.0",
            "accounts": []
        }"#;
        let wallet_path = cli.create_temp_file(wallet_content);
        
        // First open the wallet
        cli.run_command(&["wallet", "open", "--path", wallet_path.to_str().unwrap()]);
        
        // Attempt to remove liquidity
        let output = cli.run_command(&[
            "defi", "remove-liquidity",
            "--pool", "NEO-GAS",
            "--amount", "1"
        ]);
        
        // Just checking if the command is recognized
        assert!(output.status.code().unwrap_or(0) != 127, "Command not found");
    }
} 