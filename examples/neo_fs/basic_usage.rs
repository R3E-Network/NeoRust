use anyhow::Result;
use neo3::{
	fs::{AccessPermission, Container, NeoFSAuth, NeoFSClient, NeoFSConfig, Object},
	prelude::*,
};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
	// Configure NeoFS client for the testnet (or mainnet)
	let config = NeoFSConfig {
		endpoint: "grpc.testnet.fs.neo.org:8082".to_string(),
		auth: None, // Will be set with wallet
		timeout_sec: 60,
		insecure: false,
	};

	// Load wallet
	let wallet = neo3::wallets::Wallet::load_from_file("wallet.json")?;
	let account = wallet.default_account().unwrap().decrypt("password")?;

	// Create NeoFS client with account
	let client = NeoFSClient::new(config).with_account(account);

	// Create a container
	let mut container = Container::new();
	container.set_name("My Test Container");
	container.set_basic_acl(AccessPermission::AllowAllOperations);
	container.attributes.add("Type", "Documents");

	// Note: These operations will return NotImplemented error until the gRPC client implementation is complete

	// Create the container
	println!("Creating container...");
	match client.create_container(&container).await {
		Ok(container_id) => {
			println!("Container created: {}", container_id);

			// Upload a file to the container
			let file_path = Path::new("example.txt");

			if file_path.exists() {
				println!("Uploading file...");

				// Read file content
				let content = std::fs::read(file_path)?;

				// Create object with file content
				let mut object = Object::new(container_id.clone(), client.get_owner_id()?);
				object.set_payload(content);
				object.set_filename("example.txt");
				object.set_content_type("text/plain");

				// Upload object
				match client.put_object(&container_id, &object).await {
					Ok(object_id) => {
						println!("Object uploaded: {}", object_id);

						// List objects in container
						println!("Listing objects...");
						match client.list_objects(&container_id).await {
							Ok(object_ids) => {
								println!("Objects in container:");
								for (i, id) in object_ids.iter().enumerate() {
									println!("  {}. {}", i + 1, id);
								}

								// Download object
								println!("Downloading object...");
								match client.get_object(&container_id, &object_id).await {
									Ok(retrieved_object) => {
										println!(
											"Object downloaded (size: {} bytes)",
											retrieved_object.size()
										);

										// Create bearer token for container access
										println!("Creating bearer token...");
										let permissions = vec![
											AccessPermission::GetObject,
											AccessPermission::PutObject,
										];

										match client
											.create_bearer_token(&container_id, permissions, 3600)
											.await
										{
											Ok(token) => {
												println!(
													"Bearer token created: {}",
													token.token_id
												);

												// Delete object
												println!("Deleting object...");
												match client
													.delete_object(&container_id, &object_id)
													.await
												{
													Ok(true) =>
														println!("Object deleted successfully"),
													Ok(false) =>
														println!("Failed to delete object"),
													Err(e) =>
														println!("Error deleting object: {}", e),
												}

												// Delete container
												println!("Deleting container...");
												match client.delete_container(&container_id).await {
													Ok(true) =>
														println!("Container deleted successfully"),
													Ok(false) =>
														println!("Failed to delete container"),
													Err(e) =>
														println!("Error deleting container: {}", e),
												}
											},
											Err(e) =>
												println!("Error creating bearer token: {}", e),
										}
									},
									Err(e) => println!("Error downloading object: {}", e),
								}
							},
							Err(e) => println!("Error listing objects: {}", e),
						}
					},
					Err(e) => println!("Error uploading object: {}", e),
				}
			} else {
				println!("Example file not found: {}", file_path.display());
			}
		},
		Err(e) => println!("Error creating container: {}", e),
	}

	Ok(())
}
