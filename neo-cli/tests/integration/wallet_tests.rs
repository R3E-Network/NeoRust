use super::utils::{assert_output_contains, assert_success, CliTest};

const TEST_WALLET_PASSWORD: &str = "password123";

#[test]
fn test_wallet_create() {
	let cli = CliTest::new();

	// Create wallet
	let wallet_path = cli.temp_dir.path().join("test-wallet.json").to_string_lossy().to_string();
	let output = cli.run_with_input(
		&["wallet", "create", "--path", &wallet_path],
		&format!("{0}\n{0}\n", TEST_WALLET_PASSWORD),
	);

	assert_success(&output);
	assert_output_contains(&output, "Creating new wallet");

	// Verify file exists
	assert!(std::path::Path::new(&wallet_path).exists());
}

#[test]
fn test_wallet_open_and_close() {
	let cli = CliTest::new();

	// Create wallet first
	let wallet_path = cli.temp_dir.path().join("test-wallet.json").to_string_lossy().to_string();
	let create_output = cli.run_with_input(
		&["wallet", "create", "--path", &wallet_path],
		&format!("{0}\n{0}\n", TEST_WALLET_PASSWORD),
	);
	assert_success(&create_output);

	// Test open wallet
	let open_output = cli.run_with_input(
		&["wallet", "open", "--path", &wallet_path],
		&format!("{}\n", TEST_WALLET_PASSWORD),
	);
	assert_success(&open_output);
	assert_output_contains(&open_output, "Wallet opened successfully");

	// Test close wallet
	let close_output = cli.run(&["wallet", "close"]);
	assert_success(&close_output);
	assert_output_contains(&close_output, "Wallet closed");
}

#[test]
fn test_wallet_create_address() {
	let cli = CliTest::new();

	// Create and open wallet
	let wallet_path = cli.temp_dir.path().join("test-wallet.json").to_string_lossy().to_string();
	cli.run_with_input(
		&["wallet", "create", "--path", &wallet_path],
		&format!("{0}\n{0}\n", TEST_WALLET_PASSWORD),
	);
	cli.run_with_input(
		&["wallet", "open", "--path", &wallet_path],
		&format!("{}\n", TEST_WALLET_PASSWORD),
	);

	// Create a new address
	let output = cli.run_with_input(
		&["wallet", "create-address", "--count", "1"],
		&format!("{}\n", TEST_WALLET_PASSWORD),
	);

	assert_success(&output);
	assert_output_contains(&output, "New address created");
}

#[test]
fn test_wallet_list_address() {
	let cli = CliTest::new();

	// Create and open wallet
	let wallet_path = cli.temp_dir.path().join("test-wallet.json").to_string_lossy().to_string();
	cli.run_with_input(
		&["wallet", "create", "--path", &wallet_path],
		&format!("{0}\n{0}\n", TEST_WALLET_PASSWORD),
	);
	cli.run_with_input(
		&["wallet", "open", "--path", &wallet_path],
		&format!("{}\n", TEST_WALLET_PASSWORD),
	);

	// List addresses
	let output = cli.run(&["wallet", "list-address"]);

	assert_success(&output);
	// Should contain address details (even if just showing there are no addresses)
	assert_output_contains(&output, "Address");
}

#[test]
fn test_wallet_balance() {
	let cli = CliTest::new();

	// Create and open wallet
	let wallet_path = cli.temp_dir.path().join("test-wallet.json").to_string_lossy().to_string();
	cli.run_with_input(
		&["wallet", "create", "--path", &wallet_path],
		&format!("{0}\n{0}\n", TEST_WALLET_PASSWORD),
	);
	cli.run_with_input(
		&["wallet", "open", "--path", &wallet_path],
		&format!("{}\n", TEST_WALLET_PASSWORD),
	);

	// Create an address
	cli.run_with_input(
		&["wallet", "create-address", "--count", "1"],
		&format!("{}\n", TEST_WALLET_PASSWORD),
	);

	// Check balance (will be zero, but should run successfully)
	let output = cli.run(&["wallet", "balance"]);

	assert_success(&output);
}
