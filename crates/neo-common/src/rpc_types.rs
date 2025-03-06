//! RPC types
//!
//! This module defines marker traits for RPC-related types
//! to help break circular dependencies between crates.

/// Marker trait for JSON-RPC providers
pub trait JsonRpcProvider: Send + Sync {}

/// Re-export RpcClient from rpc_client_trait
pub use crate::rpc_client_trait::RpcClient;

/// Marker trait for HTTP providers
pub trait HttpProvider: Send + Sync {}

/// Marker trait for WebSocket providers
pub trait WebSocketProvider: Send + Sync {}

/// Marker trait for IPC providers
pub trait IpcProvider: Send + Sync {}
