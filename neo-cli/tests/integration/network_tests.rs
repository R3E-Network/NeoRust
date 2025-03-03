use super::utils::{assert_output_contains, assert_success, CliTest};

#[test]
fn test_network_status() {
	let cli = CliTest::new();

	// Get network status
	let output = cli.run(&["network", "status"]);

	assert_success(&output);
	assert_output_contains(&output, "Network");
	assert_output_contains(&output, "Status");
}

#[test]
fn test_network_nodes() {
	let cli = CliTest::new();

	// List connected nodes
	let output = cli.run(&["network", "nodes"]);

	assert_success(&output);
	assert_output_contains(&output, "Connected Nodes");
}

#[test]
fn test_network_switch() {
	let cli = CliTest::new();

	// Switch to TestNet
	let testnet_output = cli.run(&["network", "switch", "--network", "testnet"]);
	assert_success(&testnet_output);
	assert_output_contains(&testnet_output, "Switched to TestNet");

	// Check network status to verify
	let status_output = cli.run(&["network", "status"]);
	assert_success(&status_output);
	assert_output_contains(&status_output, "TestNet");

	// Switch to MainNet
	let mainnet_output = cli.run(&["network", "switch", "--network", "mainnet"]);
	assert_success(&mainnet_output);
	assert_output_contains(&mainnet_output, "Switched to MainNet");
}

#[test]
fn test_network_add_node() {
	let cli = CliTest::new();

	// Add a node
	let output = cli.run(&[
		"network",
		"add-node",
		"--url",
		"http://seed1.ngd.network:10332",
		"--name",
		"test-node",
	]);

	assert_success(&output);
	assert_output_contains(&output, "Node added");

	// Verify node is in the list
	let nodes_output = cli.run(&["network", "nodes"]);
	assert_success(&nodes_output);
	assert_output_contains(&nodes_output, "test-node");
}

#[test]
fn test_network_set_default() {
	let cli = CliTest::new();

	// First add a node
	cli.run(&[
		"network",
		"add-node",
		"--url",
		"http://seed2.ngd.network:10332",
		"--name",
		"default-node",
	]);

	// Set as default
	let output = cli.run(&["network", "set-default", "--name", "default-node"]);

	assert_success(&output);
	assert_output_contains(&output, "Default node set");
}

#[test]
fn test_network_ping() {
	let cli = CliTest::new();

	// Ping a node
	let output = cli.run(&["network", "ping", "--url", "http://seed1.ngd.network:10332"]);

	assert_success(&output);
	// Either ping succeeds or times out but command should complete
	assert_output_contains(&output, "Ping");
}
