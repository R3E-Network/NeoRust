//! Role management types for Neo blockchain
//!
//! This module provides types for role management in the Neo blockchain.

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// Roles in the Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// State validator role
    #[strum(serialize = "state_validator")]
    StateValidator,
    
    /// Oracle role
    Oracle,
    
    /// Neo FS Alphabet node role
    #[strum(serialize = "neo_fs_alphabet_node")]
    NeoFSAlphabetNode,
    
    /// Committee role
    Committee,
    
    /// P2P notary role
    #[strum(serialize = "p2p_notary")]
    P2PNotary,
}

impl Role {
    /// Get the role as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::StateValidator => "state_validator",
            Role::Oracle => "oracle",
            Role::NeoFSAlphabetNode => "neo_fs_alphabet_node",
            Role::Committee => "committee",
            Role::P2PNotary => "p2p_notary",
        }
    }
}
