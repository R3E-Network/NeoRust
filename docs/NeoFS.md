# NeoFS Support in NeoRust

This document describes the NeoFS functionality integrated into the NeoRust SDK and CLI.

## Overview

NeoFS is Neo's decentralized storage system that allows for the storage and retrieval of objects through a decentralized network of storage nodes. NeoRust provides comprehensive support for NeoFS operations through both the SDK and CLI.

## SDK Usage

The NeoFS SDK functionality is accessible through the `neo3::neo_fs` module.

### Client Setup

```rust
use neo3::prelude::*;

async fn example() -> anyhow::Result<()> {
    // Create a NeoFS client configuration
    let config = NeoFsConfig::new()
        .with_endpoint("https://neofs.example.com:8080")
        .with_wallet(wallet);
        
    // Initialize a NeoFS client
    let client = NeoFsClient::new(config).await?;
    
    Ok(())
}
```

### Container Operations

```rust
// Create a new container
let container = ContainerBuilder::new()
    .with_basic_acl(0o644)
    .with_name("my-container")
    .with_owner(account.address())
    .build()?;
    
// Put container to NeoFS
let container_id = client.put_container(&container).await?;

// Get container info
let container = client.get_container(&container_id).await?;

// List all containers
let containers = client.list_containers().await?;

// Delete container
client.delete_container(&container_id).await?;
```

### Object Operations

```rust
// Upload an object
let data = b"Hello, NeoFS!".to_vec();
let object = ObjectBuilder::new()
    .with_container_id(&container_id)
    .with_data(data)
    .build()?;
    
let object_id = client.put_object(&object).await?;

// Get object data
let object = client.get_object(&container_id, &object_id).await?;
let data = object.data();

// List objects in container
let objects = client.list_objects(&container_id).await?;

// Delete object
client.delete_object(&container_id, &object_id).await?;
```

### Access Control

NeoFS supports fine-grained access control through policies:

```rust
use neo3::neo_fs::policy::{ExtendedAclBuilder, EaclRecordBuilder, Operation, Action, TargetType, Role};

// Create an eACL policy
let eacl = ExtendedAclBuilder::new()
    .with_container_id(&container_id)
    .add_record(
        EaclRecordBuilder::new()
            .with_operation(Operation::Put)
            .with_action(Action::Deny)
            .with_target(TargetType::Role(Role::Others))
            .build()?
    )
    .build()?;

// Set the policy on the container
client.set_container_policy(&container_id, &eacl).await?;
```

## CLI Usage

The NeoRust CLI provides commands for interacting with NeoFS:

### General Syntax

```
neo storage [COMMAND] [OPTIONS]
```

### Network Information

```
# Get information about the NeoFS network
neo storage info
```

### Container Commands

```
# List containers
neo storage container list

# Create a new container
neo storage container create --name "my-container" --acl 0644

# Get container info
neo storage container get <container-id>

# Delete a container
neo storage container delete <container-id>
```

### Object Commands

```
# List objects in a container
neo storage object list <container-id>

# Upload a file as an object
neo storage object put <container-id> --file path/to/file

# Download an object
neo storage object get <container-id> <object-id> --output path/to/save

# Delete an object
neo storage object delete <container-id> <object-id>
```

## Permissions and Access Control

NeoFS uses an Extended Access Control List (eACL) to manage permissions. The policy module in the SDK lets you define these access rules:

* Operations include Get, Put, Head, Delete, Search, Range, and more
* Actions can be Allow or Deny
* Targets can be specific roles (e.g., Owner, Others) or specific public keys

See the SDK example above for how to create and apply access policies.

## Error Handling

The SDK provides detailed error information for NeoFS operations. All operations return `Result` types that include context about what went wrong when errors occur.

## Advanced Topics

For more advanced usage, including:

* Streaming large objects
* Working with object attributes
* Creating complex access policies
* Homomorphic storage groups

Please refer to the API documentation for the `neo_fs` module. 