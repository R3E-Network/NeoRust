// Test file to check if the NeoFS module compiles correctly

use neo::neo_fs::{
	container::{Container, ContainerBuilder},
	object::{Object, ObjectBuilder},
	types::{AccessRule, ContainerId, ObjectId, StoragePolicy},
	NeoFsClient, NeoFsConfig,
};

#[test]
fn test_neofs_client_creation() {
	// Create a NeoFS config
	let endpoint = "https://rest.fs.neo.org";

	// Using default constructor and then updating endpoint
	let mut config = NeoFsConfig::default();
	config.endpoint = endpoint.to_string();

	// This test doesn't actually create a client, we're just checking if the code compiles
	println!("NeoFS config created with endpoint: {}", endpoint);

	// Test ContainerId creation
	let id_result =
		ContainerId::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
	match id_result {
		Ok(_) => println!("ContainerId creation successful"),
		Err(e) => println!("Error creating ContainerId: {:?}", e),
	}
}
