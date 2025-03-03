# NeoFS Module

This module provides functionality for interacting with NeoFS, Neo's decentralized object storage system.

## Features

- Container management: Create, get, delete containers
- Object operations: Upload, download, delete objects
- Access control: Control access to containers and objects with ACLs and bearer tokens
- Multipart uploads: Upload large files in multiple parts

## Usage

### Basic Usage

```rust
use neo3::prelude::*;
use neo3::fs::{NeoFSClient, NeoFSConfig, Container, Object};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Configure NeoFS client
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
    
    // Create the container on NeoFS
    let container_id = client.create_container(&container).await?;
    
    // Create an object
    let content = "Hello, NeoFS!".as_bytes().to_vec();
    let mut object = Object::new(container_id.clone(), client.get_owner_id()?);
    object.set_payload(content);
    object.set_filename("hello.txt");
    object.set_content_type("text/plain");
    
    // Upload the object
    let object_id = client.put_object(&container_id, &object).await?;
    
    // List all objects in the container
    let objects = client.list_objects(&container_id).await?;
    
    // Download an object
    let downloaded = client.get_object(&container_id, &object_id).await?;
    
    // Delete the object
    client.delete_object(&container_id, &object_id).await?;
    
    // Delete the container
    client.delete_container(&container_id).await?;
    
    Ok(())
}
```

### Multipart Uploads

For large files, you can use multipart uploads:

```rust
// Initiate multipart upload
let upload = client.initiate_multipart_upload(&container_id, &object).await?;

// Upload parts
let part1 = client.upload_part(&upload, 1, data1).await?;
let part2 = client.upload_part(&upload, 2, data2).await?;

// Complete multipart upload
let result = client.complete_multipart_upload(&upload, vec![part1, part2]).await?;
```

## Access Control

NeoFS supports fine-grained access control through ACLs:

```rust
// Create a bearer token for temporary access
let permissions = vec![AccessPermission::GetObject, AccessPermission::PutObject];
let token = client.create_bearer_token(&container_id, permissions, 3600).await?;

// Token can be shared with other users for temporary access
```

## Implementation Status

> **Important Note**: This module currently provides a placeholder implementation
> with the full API surface defined. The actual gRPC implementation to communicate
> with NeoFS nodes will be added in a future update.

## Examples

See the [examples directory](../../examples/neo_fs/) for complete usage examples.
