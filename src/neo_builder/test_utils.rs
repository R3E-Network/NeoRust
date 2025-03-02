use lazy_static::lazy_static;
use primitive_types::H160;
use rustc_serialize::hex::FromHex;

use neo::prelude::{Account, KeyPair, ScriptHash, Secp256r1PrivateKey, Secp256r1PublicKey};

/// Test constants for neo_builder module tests
#[allow(unused)]
pub struct TestConstants;

lazy_static! {
	// Script hash constants for tests
	pub static ref SCRIPT_HASH: ScriptHash = {
		Account::from_wif("Kzt94tAAiZSgH7Yt4i25DW6jJFprZFPSqTgLr5dWmWgKDKCjXMfZ")
			.unwrap()
			.get_script_hash()
	};
	pub static ref SCRIPT_HASH1: H160 = H160::from_script(&"d802a401".from_hex().unwrap());
	pub static ref SCRIPT_HASH2: H160 = H160::from_script(&"c503b112".from_hex().unwrap());

	// Public key constants for group tests
	pub static ref GROUP_PUB_KEY1: Secp256r1PublicKey = Secp256r1PublicKey::from_encoded(
		"0306d3e7f18e6dd477d34ce3cfeca172a877f3c907cc6c2b66c295d1fcc76ff8f7",
	)
	.unwrap();
	pub static ref GROUP_PUB_KEY2: Secp256r1PublicKey = Secp256r1PublicKey::from_encoded(
		"02958ab88e4cea7ae1848047daeb8883daf5fdf5c1301dbbfe973f0a29fe75de60",
	)
	.unwrap();

	// Account constants for tests
	pub static ref ACCOUNT1: Account = Account::from_key_pair(
		KeyPair::from_secret_key(
			&Secp256r1PrivateKey::from_bytes(
				&hex::decode(
					"e6e919577dd7b8e97805151c05ae07ff4f752654d6d8797597aca989c02c4cb3"
				)
				.expect("Test private key 1 should be valid hex")
			)
			.expect("Test private key 1 should be valid bytes for Secp256r1PrivateKey")
		),
		None,
		None
	)
	.expect("Failed to create ACCOUNT1 from valid key pair");

	pub static ref ACCOUNT2: Account = Account::from_key_pair(
		KeyPair::from_secret_key(
			&Secp256r1PrivateKey::from_bytes(
				&hex::decode(
					"b4b2b579cac270125259f08a5f414e9235817e7637b9a66cfeb3b77d90c8e7f9"
				)
				.expect("Test private key 2 should be valid hex")
			)
			.expect("Test private key 2 should be valid bytes for Secp256r1PrivateKey")
		),
		None,
		None
	)
	.expect("Failed to create ACCOUNT2 from valid key pair");
}

/// NEO blockchain-specific constants
pub struct NeoConstants;
impl NeoConstants {
	// Network Magic Numbers
	pub const MAGIC_NUMBER_MAINNET: u32 = 860833102;
	pub const MAGIC_NUMBER_TESTNET: u32 = 894710606;

	// Transaction & Signer constraints
	pub const MAX_TRANSACTION_SIZE: u32 = 102400;
	pub const MAX_TRANSACTION_ATTRIBUTES: u32 = 16;
	pub const MAX_SIGNER_SUBITEMS: u32 = 16;
	pub const MAX_MANIFEST_SIZE: u32 = 0xFFFF;
}

/// Contract parameter types for Neo smart contracts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractParameterType {
	Any,
	Boolean,
	Integer,
	ByteArray,
	String,
	Hash160,
	Hash256,
	PublicKey,
	Signature,
	Array,
	Map,
	InteropInterface,
	Void,
}

/// Value representation for contract parameters
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValue {
	Any,
	Boolean(bool),
	Integer(i64),
	ByteArray(String),
	String(String),
	H160(String),
	H256(String),
	PublicKey(String),
	Signature(String),
	Array(Vec<neo::prelude::ContractParameter>),
	Map(Vec<(neo::prelude::ContractParameter, neo::prelude::ContractParameter)>),
}

/// A type to represent a contract parameter for testing
pub type ContractParameter = neo::prelude::ContractParameter;

/// A mapping type for contract parameters
pub type ContractParameterMap = std::collections::HashMap<String, ContractParameter>;

use crate::neo_clients::{JsonRpcProvider, MockClient, OfflineMockClient, RpcClient};
use std::{env, sync::Arc};
use tokio::sync::Mutex;

/// Create the appropriate mock client based on environment configuration
/// Uses OfflineMockClient if NEORUSE_OFFLINE_TESTS=1 or if a VPN is detected
/// Otherwise uses the regular MockClient
pub async fn create_test_mock_client() -> Arc<Mutex<MockClient>> {
	let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
	mock_provider
}

/// Create a fully offline mock client that doesn't use any network connections
/// Use this when running tests with a VPN or without network access
pub fn create_offline_test_mock_client() -> OfflineMockClient {
	let mut mock_client = OfflineMockClient::new();
	mock_client.mock_default_responses();
	mock_client
}

/// Check if tests should use offline mode
/// Returns true if NEORUST_OFFLINE_TESTS=1 or if a VPN connection is detected
pub fn should_use_offline_tests() -> bool {
	// Check environment variable
	if let Ok(value) = env::var("NEORUST_OFFLINE_TESTS") {
		return value == "1" || value.to_lowercase() == "true";
	}

	// Auto-detect VPN connections
	if detect_vpn() {
		println!(">>> VPN connection detected automatically, using offline test mode");
		return true;
	}

	// Default to false (use regular MockClient)
	false
}

/// Attempt to detect if a VPN connection is active
fn detect_vpn() -> bool {
	// Get OS information
	let os = std::env::consts::OS;

	match os {
		"macos" => detect_vpn_macos(),
		"windows" => detect_vpn_windows(),
		"linux" => detect_vpn_linux(),
		_ => {
			println!(">>> VPN detection not supported on {}, set NEORUST_OFFLINE_TESTS=1 manually if needed", os);
			false
		},
	}
}

/// Detect VPN on macOS by checking for VPN-related network interfaces
fn detect_vpn_macos() -> bool {
	use std::process::Command;

	// Check for common VPN interfaces using networksetup
	let output = Command::new("networksetup").args(["-listallnetworkservices"]).output();

	if let Ok(output) = output {
		let services = String::from_utf8_lossy(&output.stdout);

		// Check for common VPN service names
		if services.contains("VPN")
			|| services.contains("Cisco")
			|| services.contains("OpenVPN")
			|| services.contains("Pulse")
			|| services.contains("Tunnelblick")
			|| services.contains("WireGuard")
		{
			return true;
		}
	}

	// Alternative detection using routing table
	let output = Command::new("netstat").args(["-nr"]).output();

	if let Ok(output) = output {
		let routes = String::from_utf8_lossy(&output.stdout);

		// Look for common VPN-related entries in routing table
		// Many macOS VPNs use utun interfaces
		if routes.contains(" tun")
			|| routes.contains("utun")
			|| routes.contains(" ppp")
			|| routes.contains("ipsec")
			|| routes.contains("Cisco")
		{
			return true;
		}
	}

	// Check for VPN-related processes
	let output = Command::new("ps").args(["-ax"]).output();

	if let Ok(output) = output {
		let processes = String::from_utf8_lossy(&output.stdout);

		// Check for VPN processes
		if processes.contains("openvpn")
			|| processes.contains("vpnc")
			|| processes.contains("openconnect")
			|| processes.contains("wireguard")
			|| processes.contains("tunnelblick")
		{
			return true;
		}
	}

	false
}

/// Detect VPN on Windows by checking for VPN-related network interfaces
fn detect_vpn_windows() -> bool {
	use std::process::Command;

	// Check network interfaces using ipconfig
	let output = Command::new("ipconfig").args(["/all"]).output();

	if let Ok(output) = output {
		let interfaces = String::from_utf8_lossy(&output.stdout);

		// Check for common VPN adapter descriptions
		if interfaces.contains("VPN")
			|| interfaces.contains("Virtual")
			|| interfaces.contains("Cisco")
			|| interfaces.contains("OpenVPN")
			|| interfaces.contains("Pulse")
			|| interfaces.contains("WireGuard")
			|| interfaces.contains("TAP-Windows")
			|| interfaces.contains("SonicWALL")
			|| interfaces.contains("Juniper")
			|| interfaces.contains("Fortinet")
			|| interfaces.contains("Check Point")
		{
			return true;
		}
	}

	// Check network connections
	let output = Command::new("netstat").args(["-an"]).output();

	if let Ok(output) = output {
		let connections = String::from_utf8_lossy(&output.stdout);

		// Common VPN ports
		if connections.contains(":1194 ") || // OpenVPN
           connections.contains(":443 ") ||  // SSL VPN
           connections.contains(":1723 ") || // PPTP
           connections.contains(":500 ") ||  // IKE
           connections.contains(":4500 ")
		{
			// NAT-T
			return true;
		}
	}

	// Check for VPN services
	let output = Command::new("tasklist").output();

	if let Ok(output) = output {
		let processes = String::from_utf8_lossy(&output.stdout);

		if processes.contains("openvpn")
			|| processes.contains("vpncli")
			|| processes.contains("vpnagent")
			|| processes.contains("vpnui")
			|| processes.contains("wireguard")
			|| processes.contains("nordvpn")
			|| processes.contains("expressvpn")
		{
			return true;
		}
	}

	false
}

/// Detect VPN on Linux by checking for VPN-related network interfaces
fn detect_vpn_linux() -> bool {
	use std::{fs, process::Command};

	// Check for common VPN interfaces in /proc/net/dev
	if let Ok(content) = fs::read_to_string("/proc/net/dev") {
		if content.contains("tun")
			|| content.contains("tap")
			|| content.contains("ppp")
			|| content.contains("vpn")
		{
			return true;
		}
	}

	// Check for VPN interfaces using ip command
	let output = Command::new("ip").args(["link", "show"]).output();

	if let Ok(output) = output {
		let interfaces = String::from_utf8_lossy(&output.stdout);

		if interfaces.contains("tun")
			|| interfaces.contains("tap")
			|| interfaces.contains("ppp")
			|| interfaces.contains("vpn")
			|| interfaces.contains("wg")
		{
			// WireGuard
			return true;
		}
	}

	// Check running processes for VPN services
	let output = Command::new("ps").args(["-A"]).output();

	if let Ok(output) = output {
		let processes = String::from_utf8_lossy(&output.stdout);

		// Check for common VPN process names
		if processes.contains("openvpn")
			|| processes.contains("vpnc")
			|| processes.contains("openconnect")
			|| processes.contains("wireguard")
			|| processes.contains("strongswan")
			|| processes.contains("xl2tpd")
			|| processes.contains("nordvpnd")
			|| processes.contains("expressvpn")
		{
			return true;
		}
	}

	// Check for VPN-related services using systemctl
	let output = Command::new("systemctl").args(["status"]).output();

	if let Ok(output) = output {
		let services = String::from_utf8_lossy(&output.stdout);

		if services.contains("openvpn")
			|| services.contains("vpnc")
			|| services.contains("openconnect")
			|| services.contains("wireguard")
			|| services.contains("strongswan")
			|| services.contains("xl2tpd")
		{
			return true;
		}
	}

	false
}

/// Helper function to mount common mocks for transaction tests
pub async fn setup_transaction_test_mocks(mock_provider: &mut MockClient) {
	// Add commonly used mock responses for transaction tests
	mock_provider
		.mock_response_with_file_ignore_param("invokescript", "invokescript_necessary_mock.json")
		.await
		.mock_response_with_file_ignore_param("calculatenetworkfee", "calculatenetworkfee.json")
		.await
		.mount_mocks()
		.await;
}

/// Create an RPC client for testing that works with or without a VPN
///
/// This function handles both scenarios:
/// 1. If NEORUST_OFFLINE_TESTS=1 or a VPN is detected, creates a fully offline mock client with no network
/// 2. Otherwise, creates a regular mock client with a local HTTP server
///
/// # Example
/// ```
/// let client = create_test_client().await;
/// let transaction_builder = TransactionBuilder::with_client(&client);
/// ```
pub async fn create_test_client() -> Arc<dyn JsonRpcProvider + 'static> {
	let offline_mode = should_use_offline_tests();

	if offline_mode {
		// Check if enabled via env var or auto-detected
		if let Ok(value) = env::var("NEORUST_OFFLINE_TESTS") {
			if value == "1" || value.to_lowercase() == "true" {
				println!(">>> Running tests in OFFLINE mode (manual configuration)");
			} else {
				println!(">>> Running tests in OFFLINE mode (VPN auto-detected)");
			}
		} else {
			println!(">>> Running tests in OFFLINE mode (VPN auto-detected)");
		}

		println!(">>> All network calls will be completely mocked in-memory");
		println!(">>> This ensures tests work properly with VPN or limited connectivity");
		println!(">>> To force regular mock client mode, set NEORUST_OFFLINE_TESTS=0");

		// Use completely offline mock client that doesn't use any network
		let mock_client = create_offline_test_mock_client();
		// Need to use type erasure with a Box to create a trait object
		Arc::new(mock_client.into_client())
	} else {
		println!(">>> Running tests with LOCAL HTTP SERVER mock");
		println!(">>> If you experience connection issues due to VPN, set NEORUST_OFFLINE_TESTS=1");

		// Use regular mock client with local HTTP server
		let mock_provider = create_test_mock_client().await;

		// Set up common mocks
		{
			let mut mock_provider_guard = mock_provider.lock().await;
			setup_transaction_test_mocks(&mut mock_provider_guard).await;
		}

		// Create client from mock provider
		let mock_provider = mock_provider.lock().await;
		Arc::new(mock_provider.into_client())
	}
}

// Add test module at the end of the file
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_vpn_detection() {
		// This test is informational only
		let vpn_detected = detect_vpn();
		println!("VPN detection test result: {}", vpn_detected);

		// Check all platform-specific detection methods
		let os = std::env::consts::OS;
		match os {
			"macos" => {
				let result = detect_vpn_macos();
				println!("macOS VPN detection: {}", result);
			},
			"windows" => {
				let result = detect_vpn_windows();
				println!("Windows VPN detection: {}", result);
			},
			"linux" => {
				let result = detect_vpn_linux();
				println!("Linux VPN detection: {}", result);
			},
			_ => {
				println!("VPN detection not tested on {}", os);
			},
		}

		// Check if using offline tests based on both env var and detection
		let using_offline = should_use_offline_tests();
		println!("Using offline tests: {}", using_offline);

		// This test always passes - it's just for information
	}
}
