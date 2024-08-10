use crate::{
	neo_builder::Transaction,
	prelude::{
		Account, ContractParametersContext, ContractState, NEP6Wallet, NeoBlock, OpCode,
		Signer::Contract, WitnessScope,
	},
};
use neo::{
	builder::{AccountSigner, InvocationScript},
	config::{NeoNetwork, NEOCONFIG},
};
use primitive_types::{H160, H256};
use rand::Rng;
use std::{
	hash::Hash,
	time::{SystemTime, UNIX_EPOCH},
};

pub struct MockBlocks;

impl MockBlocks {
	// pub fn create_block_with_valid_transactions(
	//     wallet: &NEP6Wallet,
	//     account: &Account,
	//     number_of_transactions: usize,
	// ) -> NeoBlock {
	//     let transactions: Vec<Transaction> = (0..number_of_transactions)
	//         .map(|_| Self::create_valid_tx( wallet, account))
	//         .collect();
	//
	//     Self::create_block_with_valid_transactions_from_array(account, &transactions)
	// }

	pub fn create_valid_tx(wallet: &NEP6Wallet, account: &Account) -> Transaction {
		let rand = rand::thread_rng().gen::<u32>();
		let sender = account.script_hash();

		let mut tx = Transaction {
			network: NeoNetwork::TestNet,
			version: 0,
			nonce: rand,
			valid_until_block: NEOCONFIG
				.lock()
				.as_ref()
				.unwrap()
				.get_max_valid_until_block_increment(),
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
				tx.witnesses[0].invocation =
					InvocationScript::from_serialized_script(vec![0xFF; 64]);
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
