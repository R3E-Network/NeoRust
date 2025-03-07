use neo_builder::{AccountSigner, TransactionBuilder};
use neo_clients::JsonRpcProvider;
use crate::{ContractError, FungibleTokenContract, TokenTrait};
use neo_protocol::{Account, AccountTrait};
use neo_wallets::Wallet;
use neo_types::{Bytes, ContractParameter, NNSName, ScriptHash};
use async_trait::async_trait;
use primitive_types::H160;
use std::fmt::Debug;

#[async_trait]
pub trait FungibleTokenTrait<'a, P: JsonRpcProvider>: TokenTrait<'a, P> {
	const BALANCE_OF: &'static str = "balanceOf";
	const TRANSFER: &'static str = "transfer";

	async fn get_balance_of(&self, script_hash: &ScriptHash) -> Result<i32, ContractError> {
		self.get_balance_of_hash160(script_hash).await
	}

	async fn get_balance_of_hash160(&self, script_hash: &H160) -> Result<i32, ContractError> {
		self.call_function_returning_int(Self::BALANCE_OF, vec![script_hash.into()])
			.await
	}

	async fn get_total_balance(&self, wallet: &Wallet) -> Result<i32, ContractError> {
		let mut sum = 0;
		for (_, account) in &wallet.accounts {
			sum += self
				.get_balance_of(&account.address_or_scripthash().script_hash())
				.await
				.unwrap();
		}
		Ok(sum)
	}

	async fn transfer_from_account<W: Clone + Debug + Send + Sync>(
		&self,
		from: &Account<W>,
		to: &H160,
		amount: i64,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let mut builder = self
			.transfer_from_hash160(&from.address_or_scripthash().script_hash(), to, amount, None)
			.await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(from.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	async fn transfer_from_hash160(
		&self,
		from: &ScriptHash,
		to: &ScriptHash,
		amount: i32,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		if amount < 0 {
			return Err(ContractError::InvalidArgError(
				"The amount must be greater than or equal to 0.".to_string(),
			));
		}

		let transfer_script = self.build_transfer_script(from, to, amount, data).await.unwrap();
		let mut builder = TransactionBuilder::new();
		builder.set_script(Some(transfer_script));
		Ok(builder)
	}

	async fn build_transfer_script(
		&self,
		from: &ScriptHash,
		to: &ScriptHash,
		amount: i32,
		data: Option<ContractParameter>,
	) -> Result<Bytes, ContractError> {
		self.build_invoke_function_script(
			<FungibleTokenContract<P> as FungibleTokenTrait<P>>::TRANSFER,
			vec![from.into(), to.into(), amount.into(), data.unwrap()],
		)
		.await
	}

	// MARK: Transfer using NNS

	async fn transfer_from_account_to_nns<W>(
		&self,
		from: &Account<W>,
		to: &NNSName,
		amount: i32,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let mut builder = self
			.transfer_from_hash160_to_nns(&from.address_or_scripthash().script_hash(), to, amount, data)
			.await
			.unwrap();
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(from.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	async fn transfer_from_hash160_to_nns(
		&self,
		from: &ScriptHash,
		to: &NNSName,
		amount: i32,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let script_hash = self.resolve_nns_text_record(to).await.unwrap();
		self.transfer_from_hash160(from, &script_hash, amount, data).await
	}

	async fn transfer<W: Clone + Debug + Send + Sync>(
		&self,
		from: &H160,
		to: &H160,
		amount: i64,
		data: Option<&[u8]>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		// ... existing code ...
		Ok(TransactionBuilder::new())
	}

	async fn transfer_divisible<W: Clone + Debug + Send + Sync>(
		&self,
		from: &Account<W>,
		to: &H160,
		amount: i64,
		divisible: bool,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		// ... existing code ...
		Ok(TransactionBuilder::new())
	}
}
