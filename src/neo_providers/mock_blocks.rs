use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};
use primitive_types::{H160, H256};
use rand::Rng;
use neo::builder::{AccountSigner, InvocationScript};
use neo::config::{NEOCONFIG, NeoNetwork};
use crate::neo_builder::Transaction;
use crate::prelude::{Account,  ContractParametersContext, ContractState, NeoBlock, NEP6Wallet, OpCode, WitnessScope};
use crate::prelude::Signer::Contract;

pub struct MockBlocks;

impl MockBlocks {
    pub fn create_block_with_valid_transactions(
        wallet: &NEP6Wallet,
        account: &Account,
        number_of_transactions: usize,
    ) -> NeoBlock<Transaction> {
        let transactions: Vec<Transaction> = (0..number_of_transactions)
            .map(|_| Self::create_valid_tx( wallet, account))
            .collect();

        Self::create_block_with_valid_transactions_from_array(account, &transactions)
    }

    pub fn create_block_with_valid_transactions_from_array(
        account: &Account,
        transactions: &[Transaction],
    ) -> NeoBlock<Transaction> {
        let mut block = NeoBlock::new();

        block.transactions = transactions.to_vec();

        block.header.merkle_root = Self::compute_merkle_root(&block.transactions);

        let contract = Contract::create_multi_sig_contract(1, NEOCONFIG.lock().as_ref().unwrap().standby_committee.clone());
        let mut sc = ContractParametersContext::new(&block, format!("{}", ), None, NEOCONFIG.lock().as_ref().unwrap().network);
        let signature = block.sign(&account.get_key(), NEOCONFIG.lock().as_ref().unwrap().network);
        sc.add_signature(contract, NEOCONFIG.lock().as_ref().unwrap().standby_committee[0], &signature);
        block.header.witness = sc.get_witnesses()[0].clone();

        block
    }

    pub fn create_valid_tx(wallet: &NEP6Wallet, account: &Account) -> Transaction {
        let rand = rand::thread_rng().gen::<u32>();
        let sender = account.script_hash();

        let mut tx = Transaction {
            network: NeoNetwork::TestNet,
            version: 0,
            nonce: rand,
            valid_until_block: NEOCONFIG.lock().as_ref().unwrap().get_max_valid_until_block_increment(),
            signers: vec![AccountSigner::new(account, WitnessScope::CalledByEntry).into()],
            size: 0,
            sys_fee: 0,
            net_fee: 0,
            attributes: vec![],
            script: vec![OpCode::Ret as u8],
            witnesses: vec![],
            block_time: None,
            block_count_when_sent: None,
        };

        let data = tx.send();
        assert!(wallet.sign(&mut data));
        assert!(data.completed());
        tx.witnesses = data.get_witnesses();

        tx
    }

    fn unix_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
    }
pub fn random_uint256() -> H256 {
    let mut rng = rand::thread_rng();
    let mut data = [0u8; 32];
    rng.fill(&mut data);
    H256::new(data)
}

pub fn random_uint160() -> H160 {
    let mut rng = rand::thread_rng();
    let mut data = [0u8; 20];
    rng.fill(&mut data);
    H160::new(data)
}

pub fn create_block(index: u32, prev_hash: H256, timestamp: Option<u64>) -> NeoBlock<Transaction> {
    // let header = Header {
    //     version: 0,
    //     prev_hash,
    //     merkle_root: Self::random_uint256(),
    //     timestamp: timestamp.unwrap_or_else(Self::unix_timestamp_ms),
    //     index,
    //     next_consensus: Self::random_uint160(),
    //     witness: Self::create_witness(),
    // };
    // H256::parse("0x6226416a0e5aca42b5566f5a19ab467692688ba9d47986f6981a7f747bba2772").unwrap()
    NeoBlock<Transaction> {
            version: 0,
            prev_hash,
            merkle_root: Self::random_uint256(),
            timestamp: timestamp.unwrap_or_else(Self::unix_timestamp_ms),
            index,
            next_consensus: Self::random_uint160(),
            witness: Self::create_witness(),
        transactions: vec![],
    }
}

pub fn create_contract_state() -> ContractState {
    ContractState {
        id: 1,
        nef: Self::create_nef_file(),
        update_counter: 0,
        hash: Self::random_uint160(),
        manifest: Self::create_contract_manifest(),
    }
}

pub fn create_invalid_transaction(transaction_type: InvalidTransactionType) -> Transaction {
    let mut tx = Self::create_transaction(Self::random_uint160());

    match transaction_type {
        InvalidTransactionType::InsufficientBalance => {
            tx.sys_fee = i64::MAX;
        },
        InvalidTransactionType::InvalidSignature => {
            tx.witnesses[0].invocation = InvocationScript::from_serialized_script( vec![0xFF; 64]);
        },
        InvalidTransactionType::InvalidScript => {
            tx.script = vec![0xFF];
        },
        InvalidTransactionType::Oversized => {
            tx.script = vec![0; 65536]; // Assuming max size is less than this
        },
        InvalidTransactionType::Expired => {
            tx.valid_until_block = 0;
        },
        // InvalidTransactionType::Conflicting => {
        //     // This would depend on your conflict rules
        //     tx.attributes.push(TransactionAttribute::Conflicts(Self::random_uint256()));
        // },
    }

    tx
}
}

pub enum InvalidTransactionType {
    InsufficientBalance,
    InvalidSignature,
    InvalidScript,
    Oversized,
    Expired,
    // Conflicting,
}