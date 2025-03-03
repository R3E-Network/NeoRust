// Copyright (c) 2023-2025 R3E Network
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Neo File Storage (NeoFS) Module (v0.1.4)
//!
//! NeoFS is a decentralized distributed object storage network integrated with
//! the Neo Blockchain. It provides a robust platform for storing, retrieving,
//! and managing digital assets with blockchain-level security.
//!
//! ## Overview
//!
//! This module provides Rust bindings to interact with NeoFS services, including:
//!
//! - **Container Management**: Create, retrieve, list, and delete NeoFS containers
//! - **Object Operations**: Upload, download, and manage objects in containers
//! - **Access Control**: Manage permissions and generate access tokens
//! - **Extended Features**: Support for multipart uploads and specialized storage operations
//!
//! ## Example
//!
//! ```no_run
//! use neo_rust::neo_fs::{NeoFSClient, NeoFSConfig, NeoFSService};
//! use neo_rust::neo_fs::container::{Container, ContainerId};
//! use neo_rust::neo_fs::object::{Object, ObjectId};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a NeoFS client with default configuration
//!     let client = NeoFSClient::new(NeoFSConfig::default()).await?;
//!     
//!     // List available containers
//!     let containers = client.list_containers().await?;
//!     
//!     // Process container IDs
//!     for container_id in containers {
//!         println!("Found container: {}", container_id);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod acl;
pub mod client;
pub mod container;
pub mod error;
pub mod object;
pub mod types;

pub use client::NeoFSClient;
pub use error::{NeoFSError, NeoFSResult};

// Re-export types directly from types module
pub use acl::{BearerToken, SessionToken};
pub use container::Container;
pub use object::{Object, MultipartUpload, Part, MultipartUploadResult};
pub use types::{OwnerId, Attributes, ObjectType, PlacementPolicy, ContainerId, ObjectId, AccessPermission};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The default mainnet NeoFS endpoint
pub const DEFAULT_MAINNET_ENDPOINT: &str = "grpc+tls://st01.testnet.fs.neo.org:8082";

/// The default testnet NeoFS endpoint
pub const DEFAULT_TESTNET_ENDPOINT: &str = "grpc+tls://st01.testnet.fs.neo.org:8082";

/// Default NeoFS endpoint
pub const DEFAULT_ENDPOINT: &str = "grpc.main.fs.neo.org:8082";

/// Represents a NeoFS service provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoFSConfig {
    /// The NeoFS service endpoint URL
    pub endpoint: String,
    /// Authentication information, typically from a Neo wallet
    pub auth: Option<NeoFSAuth>,
    /// Timeout for NeoFS operations in seconds
    pub timeout_sec: u64,
    /// Specifies whether to use insecure connections
    pub insecure: bool,
}

impl Default for NeoFSConfig {
    fn default() -> Self {
        Self {
            endpoint: DEFAULT_TESTNET_ENDPOINT.to_string(),
            auth: None,
            timeout_sec: 60,
            insecure: false,
        }
    }
}

/// Authentication information for NeoFS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoFSAuth {
    /// The wallet account used for authentication
    pub wallet_address: String,
    /// The private key to sign NeoFS requests
    pub private_key: Option<String>,
}

/// Service trait for interacting with NeoFS
#[async_trait]
pub trait NeoFSService {
    /// Creates a new container in NeoFS
    async fn create_container(&self, container: &Container) -> NeoFSResult<ContainerId>;
    
    /// Gets a container by its ID
    async fn get_container(&self, id: &ContainerId) -> NeoFSResult<Container>;
    
    /// Lists all containers owned by the current account
    async fn list_containers(&self) -> NeoFSResult<Vec<ContainerId>>;
    
    /// Deletes a container by its ID
    async fn delete_container(&self, id: &ContainerId) -> NeoFSResult<bool>;
    
    /// Uploads an object to a container
    async fn put_object(&self, container_id: &ContainerId, object: &Object) -> NeoFSResult<ObjectId>;
    
    /// Gets an object by its ID from a container
    async fn get_object(&self, container_id: &ContainerId, object_id: &ObjectId) -> NeoFSResult<Object>;
    
    /// Lists all objects in a container
    async fn list_objects(&self, container_id: &ContainerId) -> NeoFSResult<Vec<ObjectId>>;
    
    /// Deletes an object by its ID from a container
    async fn delete_object(&self, container_id: &ContainerId, object_id: &ObjectId) -> NeoFSResult<bool>;
    
    /// Creates a bearer token for accessing objects in a container
    async fn create_bearer_token(&self, container_id: &ContainerId, permissions: Vec<AccessPermission>, expires_sec: u64) -> NeoFSResult<BearerToken>;
    
    /// Gets a session token for the current account
    async fn get_session_token(&self) -> NeoFSResult<SessionToken>;
    
    /// Initiates a multipart upload for a large object
    async fn initiate_multipart_upload(&self, container_id: &ContainerId, object: &Object) -> NeoFSResult<MultipartUpload>;
    
    /// Uploads a part of a multipart upload
    async fn upload_part(&self, upload: &MultipartUpload, part_number: u32, data: Vec<u8>) -> NeoFSResult<Part>;
    
    /// Completes a multipart upload
    async fn complete_multipart_upload(&self, upload: &MultipartUpload, parts: Vec<Part>) -> NeoFSResult<MultipartUploadResult>;
    
    /// Aborts a multipart upload
    async fn abort_multipart_upload(&self, upload: &MultipartUpload) -> NeoFSResult<bool>;
}
