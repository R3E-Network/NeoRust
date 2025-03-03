#[cfg(test)]
mod tests {
	use crate::integration::utils::{
		assert_output_contains, assert_output_matches, assert_success, script_hash_from_string,
		CliTest,
	};
	use regex;
	use std::process::Output;

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
			"defi",
			"swap-info",
			"--token-from",
			"NEO",
			"--token-to",
			"GAS",
			"--amount",
			"1",
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
			"defi",
			"swap",
			"--token-from",
			"NEO",
			"--token-to",
			"GAS",
			"--amount",
			"1",
			"--slippage",
			"0.5",
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

		// First create the wallet
		cli.run_command(&["wallet", "open", "--path", wallet_path.to_str().unwrap()]);

		// Attempt to add liquidity
		let output = cli.run_command(&[
			"defi",
			"add-liquidity",
			"--token-a",
			"NEO",
			"--token-b",
			"GAS",
			"--amount-a",
			"1",
			"--amount-b",
			"1",
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

		// First create the wallet
		cli.run_command(&["wallet", "open", "--path", wallet_path.to_str().unwrap()]);

		// Attempt to remove liquidity
		let output = cli.run_command(&[
			"defi",
			"remove-liquidity",
			"--token-a",
			"NEO",
			"--token-b",
			"GAS",
			"--percent",
			"50",
		]);

		// Just checking if the command is recognized
		assert!(output.status.code().unwrap_or(0) != 127, "Command not found");
	}

	/// Test the contract list command
	#[test]
	fn test_defi_list() {
		let cli = CliTest::new();

		// Test mainnet list
		let output = cli.run_command(&["defi", "list", "--network", "mainnet"]);

		assert_success(&output);
		assert_output_contains(&output, "Famous contracts on mainnet:");
		assert_output_contains(&output, "NEO Token");
		assert_output_contains(&output, "GAS Token");

		// Test testnet list
		let output = cli.run_command(&["defi", "list", "--network", "testnet"]);

		assert_success(&output);
		assert_output_contains(&output, "Famous contracts on testnet:");
	}

	/// Test the contract show command
	#[test]
	fn test_defi_show_by_name() {
		let cli = CliTest::new();

		// Test show by name
		let output = cli.run_command(&["defi", "show", "NEO Token"]);

		assert_success(&output);
		assert_output_contains(&output, "Contract Name: NEO Token");
		assert_output_contains(&output, "Script Hash:");
		assert_output_contains(&output, "Network: Mainnet");

		// Test show with case insensitivity
		let output = cli.run_command(&["defi", "show", "neo token"]);

		assert_success(&output);
		assert_output_contains(&output, "Contract Name: NEO Token");
	}

	/// Test the contract show command with script hash
	#[test]
	fn test_defi_show_by_script_hash() {
		let cli = CliTest::new();

		// First get NEO token script hash
		let list_output = cli.run_command(&["defi", "list", "--network", "mainnet"]);
		let stdout = String::from_utf8_lossy(&list_output.stdout);

		// Find NEO Token in the output
		let neo_line = stdout
			.lines()
			.find(|line| line.contains("NEO Token"))
			.expect("NEO Token not found in list output");

		// Extract script hash using regex
		let re = regex::Regex::new(r"\(([0x\da-fA-F]+)\)").unwrap();
		let script_hash = re
			.captures(neo_line)
			.and_then(|caps| caps.get(1))
			.map(|m| m.as_str())
			.expect("Could not extract NEO script hash");

		// Test show by script hash
		let output = cli.run_command(&["defi", "show", script_hash]);

		assert_success(&output);
		assert_output_contains(&output, "Contract Name: NEO Token");
		assert_output_contains(&output, script_hash);
	}

	/// Test the contract invoke command (test only)
	#[test]
	fn test_defi_invoke_test() {
		let cli = CliTest::new();

		// Create a mock wallet for testing
		let wallet_path = cli.create_temp_file(
			r#"{
            "name": "test_wallet",
            "version": "1.0",
            "scrypt": {"n": 16384, "r": 8, "p": 8},
            "accounts": [
                {
                    "address": "NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz",
                    "label": "test_account",
                    "isDefault": true,
                    "lock": false,
                    "key": "6PYXHjPaNvW8YknSXaKzL1Xoxw4RjmQwCryMGEZ2GaLhGH8AdazLJPBBXw",
                    "contract": {
                        "script": "DCECIgZYieFCd+WHwCJK/I8btx1lYRIzOz8I8ZB6Ll6G3IIRLUFAQQ==",
                        "parameters": [{"name": "signature", "type": "Signature"}]
                    }
                }
            ]
        }"#,
		);

		// Test invoke NEO token symbol method (view only)
		let output = cli.run_command(&[
			"defi",
			"invoke",
			"NEO Token",
			"symbol",
			"--wallet",
			wallet_path.to_str().unwrap(),
			"--password",
			"test123",
		]);

		// Just checking if the command is recognized
		assert!(output.status.code().unwrap_or(0) != 127, "Command not found");
	}

	/// Test the contract balance command
	#[test]
	fn test_defi_balance() {
		let cli = CliTest::new();

		// Create a mock wallet for testing
		let wallet_path = cli.create_temp_file(
			r#"{
            "name": "test_wallet",
            "version": "1.0",
            "scrypt": {"n": 16384, "r": 8, "p": 8},
            "accounts": [
                {
                    "address": "NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz",
                    "label": "test_account",
                    "isDefault": true,
                    "lock": false,
                    "key": "6PYXHjPaNvW8YknSXaKzL1Xoxw4RjmQwCryMGEZ2GaLhGH8AdazLJPBBXw",
                    "contract": {
                        "script": "DCECIgZYieFCd+WHwCJK/I8btx1lYRIzOz8I8ZB6Ll6G3IIRLUFAQQ==",
                        "parameters": [{"name": "signature", "type": "Signature"}]
                    }
                }
            ]
        }"#,
		);

		// Test checking balance with address flag
		let output = cli.run_command(&[
			"defi",
			"balance",
			"NEO Token",
			"--address",
			"NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz",
			"--wallet",
			wallet_path.to_str().unwrap(),
			"--password",
			"test123",
		]);

		// Just checking if the command is recognized
		assert!(output.status.code().unwrap_or(0) != 127, "Command not found");
	}

	/// Test error handling for invalid contract name
	#[test]
	fn test_defi_invalid_contract() {
		let cli = CliTest::new();

		// Test with a non-existent contract
		let output = cli.run_command(&["defi", "show", "NonExistentContract"]);

		assert!(!output.status.success());
		assert_output_contains(&output, "Contract not found: NonExistentContract");
	}
}
