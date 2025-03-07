use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSigner {
    pub account: H160,
    pub scopes: Vec<String>,
    pub allowed_contracts: Option<Vec<H160>>,
    pub allowed_groups: Option<Vec<String>>,
    pub rules: Option<Vec<WitnessRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessRule {
    pub action: String,
    pub condition: WitnessCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WitnessCondition {
    #[serde(rename = "Boolean")]
    Boolean { expression: bool },
    #[serde(rename = "Not")]
    Not { expression: Box<WitnessCondition> },
    #[serde(rename = "And")]
    And { expressions: Vec<WitnessCondition> },
    #[serde(rename = "Or")]
    Or { expressions: Vec<WitnessCondition> },
    #[serde(rename = "ScriptHash")]
    ScriptHash { hash: H160 },
    #[serde(rename = "Group")]
    Group { group: String },
    #[serde(rename = "CalledByEntry")]
    CalledByEntry,
    #[serde(rename = "CalledByContract")]
    CalledByContract { hash: H160 },
    #[serde(rename = "CalledByGroup")]
    CalledByGroup { group: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoBlock {
    pub hash: H256,
    pub size: u32,
    pub version: u32,
    pub previousblockhash: Option<String>,
    pub merkleroot: String,
    pub time: u64,
    pub index: u32,
    pub nonce: String,
    pub nextconsensus: String,
    pub witnesses: Vec<Witness>,
    pub tx: Option<Vec<Transaction>>,
    pub confirmations: Option<u32>,
    pub nextblockhash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Witness {
    pub invocation: String,
    pub verification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: H256,
    pub size: u32,
    pub version: u32,
    pub nonce: u64,
    pub sender: String,
    pub sys_fee: String,
    pub net_fee: String,
    pub valid_until_block: u32,
    pub signers: Vec<TransactionSigner>,
    pub attributes: Vec<Attribute>,
    pub script: String,
    pub witnesses: Vec<Witness>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub usage: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoAddress {
    pub address: String,
    pub script_hash: H160,
    pub public_key: Option<String>,
    pub label: Option<String>,
    pub has_key: bool,
    pub watch_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemPoolDetails {
    pub verified: Vec<H256>,
    pub unverified: Vec<H256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoVersion {
    pub tcp_port: u16,
    pub ws_port: u16,
    pub nonce: u64,
    pub user_agent: String,
    pub protocol: Option<Protocol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    pub network: u32,
    pub validatorscount: u32,
    pub millisecondsperblock: u32,
    pub maxtraceableblocks: u32,
    pub maxvaliduntilblockincrement: u32,
    pub addressversion: u8,
    pub standbyvalidators: Vec<String>,
    pub seedlist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VMState {
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "HALT")]
    Halt,
    #[serde(rename = "FAULT")]
    Fault,
    #[serde(rename = "BREAK")]
    Break,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RTransaction {
    pub hash: H256,
    pub size: u32,
    pub version: u32,
    pub nonce: u64,
    pub sender: String,
    pub sys_fee: String,
    pub net_fee: String,
    pub valid_until_block: u32,
    pub signers: Vec<TransactionSigner>,
    pub attributes: Vec<Attribute>,
    pub script: String,
    pub witnesses: Vec<Witness>,
    pub blockhash: Option<String>,
    pub confirmations: Option<u32>,
    pub blocktime: Option<u64>,
    pub vmstate: VMState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransaction {
    pub hash: String,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationLog {
    pub txid: String,
    pub executions: Vec<Execution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub trigger: String,
    pub vmstate: String,
    pub gas_consumed: String,
    pub stack: Vec<neo_types::StackItem>,
    pub notifications: Vec<Notification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub contract: String,
    pub event_name: String,
    pub state: neo_types::StackItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nep17Balances {
    pub address: String,
    pub balance: Vec<Nep17Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nep17Balance {
    pub asset_hash: String,
    pub amount: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nep17Transfers {
    pub address: String,
    pub sent: Vec<Nep17Transfer>,
    pub received: Vec<Nep17Transfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nep17Transfer {
    pub timestamp: u64,
    pub asset_hash: String,
    pub transfer_address: Option<String>,
    pub amount: String,
    pub block_index: u32,
    pub transfer_notify_index: u32,
    pub tx_hash: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nep11Balances {
    pub address: String,
    pub balance: Vec<Nep11Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nep11Balance {
    pub asset_hash: String,
    pub tokens: HashMap<String, String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nep11Transfers {
    pub address: String,
    pub sent: Vec<Nep11Transfer>,
    pub received: Vec<Nep11Transfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nep11Transfer {
    pub timestamp: u64,
    pub asset_hash: String,
    pub transfer_address: Option<String>,
    pub amount: String,
    pub token_id: String,
    pub block_index: u32,
    pub transfer_notify_index: u32,
    pub tx_hash: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRoot {
    pub version: u8,
    pub index: u32,
    pub root_hash: H256,
    pub witnesses: Vec<Witness>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateHeight {
    pub local: u32,
    pub validated: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct States {
    pub first_proven_height: u32,
    pub last_proven_height: u32,
    pub results: Vec<StateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peers {
    pub unconnected: Vec<Peer>,
    pub connected: Vec<Peer>,
    pub bad: Vec<Peer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub public_key: String,
    pub votes: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub interfaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnclaimedGas {
    pub unclaimed: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateAddress {
    pub address: String,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoNetworkFee {
    pub network_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitBlock {
    pub hash: H256,
} 