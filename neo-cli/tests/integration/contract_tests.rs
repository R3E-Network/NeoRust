#[cfg(test)]
mod tests {
	use crate::integration::utils::{
		assert_output_contains, assert_success, script_hash_from_string, CliTest,
	};
	use std::process::Output;

	#[test]
	fn test_contract_info() {
		let cli = CliTest::new();

		// For testing, we'll use a known contract hash
		// This should be replaced with a valid hash in a real environment
		let contract_hash = "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"; // Example GAS token hash

		let output = cli.run_command(&["contract", "info", "--script-hash", contract_hash]);

		// We expect the command to be recognized even if the contract doesn't exist
		// The actual validation of contract info would depend on network connectivity
		assert_success(&output);
		assert_output_contains(&output, "Contract Information");
	}

	#[test]
	fn test_contract_manifest() {
		let cli = CliTest::new();

		// For testing purposes
		let contract_hash = "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"; // Example GAS token hash

		let output = cli.run_command(&["contract", "manifest", "--script-hash", contract_hash]);

		// The command structure should be valid even if the contract doesn't exist
		assert_success(&output);
		assert_output_contains(&output, "Manifest");
		assert_output_contains(&output, "name");
	}

	#[test]
	fn test_contract_methods() {
		let cli = CliTest::new();

		// For testing purposes
		let contract_hash = "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"; // Example GAS token hash

		let output = cli.run_command(&["contract", "methods", "--script-hash", contract_hash]);

		// The command structure should be valid
		assert_success(&output);
		assert_output_contains(&output, "Methods");
	}

	#[test]
	fn test_contract_test_invoke() {
		let cli = CliTest::new();

		// Create a parameters file for the test invoke
		let params_json = r#"[
            {"type": "String", "value": "test"}
        ]"#;

		let params_file = cli.create_temp_file(params_json);

		let contract_hash = "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"; // Example GAS token hash
		let method = "transfer";

		let output = cli.run_command(&[
			"contract",
			"invoke",
			"--script-hash",
			contract_hash,
			"--method",
			method,
			"--params",
			params_file.to_str().unwrap(),
			"--test-invoke",
		]);

		// The command structure should be valid even if the invocation can't be completed
		assert_success(&output);
		assert_output_contains(&output, "Invocation result");
	}

	#[test]
	fn test_contract_storage() {
		let cli = CliTest::new();

		let contract_hash = "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"; // Example GAS token hash

		let output = cli.run_command(&["contract", "storage", "--script-hash", contract_hash]);

		// The command structure should be valid even if no storage items exist
		assert_success(&output);
		assert_output_contains(&output, "Storage");
	}

	#[test]
	fn test_contract_deploy() {
		let cli = CliTest::new();

		// Create sample NEF file content (this is just a mock for testing)
		let nef_content = "NEF1FAKECONTENTaa";
		let nef_file = cli.create_temp_file(nef_content);

		// Create sample manifest content
		let manifest_content = r#"{
            "name": "TestContract",
            "groups": [],
            "features": {},
            "abi": {
                "methods": [
                    {
                        "name": "verify",
                        "parameters": [],
                        "returntype": "Boolean",
                        "offset": 0
                    }
                ],
                "events": []
            },
            "permissions": [
                {
                    "contract": "*",
                    "methods": "*"
                }
            ],
            "trusts": [],
            "supportedstandards": [],
            "extra": null
        }"#;
		let manifest_file = cli.create_temp_file(manifest_content);

		let output = cli.run_command(&[
			"contract",
			"deploy",
			"--nef",
			nef_file.to_str().unwrap(),
			"--manifest",
			manifest_file.to_str().unwrap(),
		]);

		// The command structure should be valid regardless of the NEF file validity
		// We don't expect successful deployment but the command should be recognized
		assert_output_contains(&output, "deploy");
	}
}
