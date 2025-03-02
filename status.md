NeoRust SDK Build Status
====================

## Working Features
- websocket: Successfully builds and can be used for WebSocket connections
- ripemd160: Core hashing functionality works
- rest-client: REST API client works

## Features with Issues
- http-client: Multiple issues including JsonRpcProvider trait bounds, conflicting ProviderError implementations, and module visibility problems

## Fixes Applied
1. Fixed TypeError enum to include the InvalidNeoName variant
2. Fixed HttpProvider usage to properly convert String to &str
3. Added AsRef implementation for RpcClient
4. Made retry and rw modules public
5. Added tokio feature flag for filesystem operations
6. Conditionally compiled tokio-dependent code in neo_fs/client.rs
