# Working with NeoFS Decentralized Storage

This guide explains how to use the Neo CLI to interact with NeoFS, Neo's decentralized storage system.

## Overview

NeoFS is a distributed, decentralized object storage network built on the Neo blockchain. The `neo-cli` tool provides direct access to NeoFS through the `fs` command group, allowing you to:

1. Manage containers (storage buckets)
2. Store, retrieve, and manage objects (files)
3. Control access permissions
4. Query information about the NeoFS network

## Connection Management

### Checking Status

To check the status of your NeoFS connection:

```bash
neo-cli fs status
```

This displays information about the connected NeoFS network, including version, available storage, and online nodes.

### Managing Endpoints

To list all available NeoFS endpoints:

```bash
# List mainnet endpoints
neo-cli fs endpoints list --network mainnet

# List testnet endpoints
neo-cli fs endpoints list --network testnet
```

To test a connection to a specific endpoint:

```bash
# Test a gRPC endpoint
neo-cli fs endpoints test --endpoint grpc.mainnet.fs.neo.org:8082 --type grpc

# Test an HTTP gateway
neo-cli fs endpoints test --endpoint https://http.mainnet.fs.neo.org --type http
```

To get detailed information about a specific endpoint:

```bash
neo-cli fs endpoints info --endpoint grpc.mainnet.fs.neo.org:8082
```

## Container Management

Containers are the main storage units in NeoFS, similar to buckets in other object storage systems.

### Creating a Container

To create a new container:

```bash
neo-cli fs container create --config container-config.json
```

Example configuration file (`container-config.json`):
```json
{
  "name": "my-container",
  "basic_acl": 0644,
  "placement_policy": "REP 3",
  "attributes": [
    {"key": "CreatedBy", "value": "NeoRust CLI"},
    {"key": "Description", "value": "Test container for documents"}
  ]
}
```

### Listing Containers

To list all containers owned by your account:

```bash
neo-cli fs container list
```

### Getting Container Info

To get detailed information about a container:

```bash
neo-cli fs container get --id CID
```

Replace `CID` with the actual container ID.

### Deleting a Container

To delete a container (it must be empty):

```bash
neo-cli fs container delete --id CID
```

## Object Management

Objects are the files stored in NeoFS containers.

### Uploading an Object

To upload a file to NeoFS:

```bash
neo-cli fs object put --container CID --file path/to/file
```

You can also specify content type and custom attributes:

```bash
neo-cli fs object put --container CID --file document.pdf --content-type application/pdf --attributes "Author=JohnDoe" "Department=Research"
```

### Downloading an Object

To download a file from NeoFS:

```bash
neo-cli fs object get --container CID --id OID --output path/to/save
```

### Getting Object Information

To get metadata about an object:

```bash
neo-cli fs object info --container CID --id OID
```

### Listing Objects in a Container

To list all objects in a container:

```bash
neo-cli fs object list --container CID
```

### Deleting an Object

To delete an object:

```bash
neo-cli fs object delete --container CID --id OID
```

## Access Control

NeoFS provides flexible access control mechanisms for containers and objects.

### Setting ACL Rules

To set access control rules for a container:

```bash
neo-cli fs acl set --container CID --basic 0644
```

### Creating Extended ACL Rules

For more complex access control, you can create extended ACL tables:

```bash
neo-cli fs acl extend --container CID --config extended-acl.json
```

Example `extended-acl.json`:
```json
{
  "records": [
    {
      "operation": "PUT",
      "action": "ALLOW",
      "filters": [
        {"key": "address", "value": "NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz"}
      ]
    }
  ]
}
```

## Practical Examples

### Creating a Private Photo Storage

```bash
# Create a container for photos
neo-cli fs container create --config photo-container.json

# Upload photos
neo-cli fs object put --container CID --file vacation.jpg --content-type image/jpeg
neo-cli fs object put --container CID --file family.jpg --content-type image/jpeg

# Set private access permissions
neo-cli fs acl set --container CID --basic 0600
```

### Sharing Files with Collaborators

```bash
# Create a container for shared documents
neo-cli fs container create --config shared-docs.json

# Upload documents
neo-cli fs object put --container CID --file presentation.pdf --content-type application/pdf

# Set extended permissions for collaborators
neo-cli fs acl extend --container CID --config collaborators-acl.json
```

## Available Endpoints

### Mainnet

| Service Type | Endpoint URL |
|--------------|--------------|
| gRPC API     | grpc.mainnet.fs.neo.org:8082 |
| HTTP Gateway | https://http.mainnet.fs.neo.org |
| REST API     | https://rest.mainnet.fs.neo.org |

### Testnet

| Service Type | Endpoint URL |
|--------------|--------------|
| gRPC API     | grpc.testnet.fs.neo.org:8082 |
| HTTP Gateway | https://http.testnet.fs.neo.org |
| REST API     | https://rest.testnet.fs.neo.org |

## Tips and Best Practices

1. **Wallet Security**: Always keep your wallet secure - it contains the keys needed to access your NeoFS data.

2. **Container Organization**: Create separate containers for different types of data or different access patterns.

3. **Basic ACL**: Use the following permission values:
   - `0644`: Public read, owner write (like a public website)
   - `0640`: Group read, owner write (like team sharing)
   - `0600`: Owner only (private data)

4. **Metadata**: Use attributes to add searchable metadata to your objects.

5. **HTTP Gateway**: For public content, you can share links via the HTTP gateway:
   ```
   https://http.mainnet.fs.neo.org/container_id/object_id
   ```
