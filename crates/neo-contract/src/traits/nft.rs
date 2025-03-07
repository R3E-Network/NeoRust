use std::{collections::HashMap, sync::Arc};

use neo_builder::{AccountSigner, TransactionBuilder};
use neo_clients::JsonRpcProvider;
use crate::{ContractError, NeoIterator, NftContract, TokenTrait};
use neo_protocol::Account;
use neo_types::{Address, Bytes, ContractParameter, NNSName, ScriptHash, ScriptHashExtension, StackItem};
use async_trait::async_trait;
use primitive_types::H160;
use std::fmt::Debug;

#[async_trait]
pub trait NonFungibleTokenTrait<'a, P: JsonRpcProvider>: TokenTrait<'a, P> + Send {
	const OWNER_OF: &'static str = "ownerOf";
	const TOKENS_OF: &'static str = "tokensOf";
	const BALANCE_OF: &'static str = "balanceOf";
	const TRANSFER: &'static str = "transfer";
	const TOKENS: &'static str = "tokens";
	const PROPERTIES: &'static str = "properties";

	// Token methods

	async fn balance_of(&mut self, owner: H160) -> Result<i32, ContractError> {
		self.call_function_returning_int(
			<NftContract<P> as NonFungibleTokenTrait<P>>::BALANCE_OF,
			vec![owner.into()],
		)
		.await
	}

	// NFT methods

	async fn tokens_of(&self, owner: &H160) -> Result<NeoIterator<'_, Vec<u8>, P>, ContractError> {
		let mapper_fn = Arc::new(|item: StackItem| item.as_bytes().unwrap_or_default());

		self.call_function_returning_iterator(
			<NftContract<P> as NonFungibleTokenTrait<P>>::TOKENS_OF,
			vec![owner.into()],
			mapper_fn,
		)
		.await
	}

	// Non-divisible NFT methods

	async fn transfer(
		&self,
		from: &H160,
		to: &H160,
		token_id: &Bytes,
		data: Option<&[u8]>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let mut params = vec![from.into(), to.into()];
		
		params.push(ContractParameter::byte_array(token_id.as_ref()));
		
		if let Some(d) = data {
			params.push(ContractParameter::byte_array(d));
		} else {
			params.push(ContractParameter::array(vec![]));
		}

		let mut builder = self.invoke_function(Self::TRANSFER, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(*from).unwrap().into()]);

		Ok(builder)
	}

	async fn transfer_inner(
		&mut self,
		to: ScriptHash,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<Self::P>, ContractError> {
		self.throw_if_divisible_nft().await.unwrap();
		self.invoke_function(
			<NftContract<P> as NonFungibleTokenTrait<P>>::TRANSFER,
			vec![to.into(), token_id.into(), data.unwrap()],
		)
		.await
	}

	async fn transfer_from_name(
		&mut self,
		from: &Account,
		to: &str,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<P>, ContractError> {
		self.throw_if_sender_is_not_owner(&from.get_script_hash(), &token_id)
			.await
			.unwrap();

		let mut build = self
			.transfer_inner(ScriptHash::from_address(to).unwrap(), token_id, data)
			.await
			.unwrap();
		build.set_signers(vec![AccountSigner::called_by_entry(from).unwrap().into()]);

		Ok(build)
	}

	async fn transfer_to_name(
		&mut self,
		to: &str,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<P>, ContractError> {
		self.throw_if_divisible_nft().await.unwrap();

		self.transfer_inner(
			self.resolve_nns_text_record(&NNSName::new(to).unwrap()).await.unwrap(),
			token_id,
			data,
		)
		.await
	}

	async fn build_non_divisible_transfer_script(
		&mut self,
		to: Address,
		token_id: Bytes,
		data: ContractParameter,
	) -> Result<Bytes, ContractError> {
		self.throw_if_divisible_nft().await.unwrap();

		self.build_invoke_function_script(
			<NftContract<P> as NonFungibleTokenTrait<P>>::TRANSFER,
			vec![to.into(), token_id.into(), data],
		)
		.await
	}

	async fn owner_of(&self, token_id: &Bytes) -> Result<H160, ContractError> {
		let token_id_vec = token_id.to_vec();
		let params = vec![ContractParameter::from(token_id_vec.as_slice())];
		
		Ok(self.call_function_returning_script_hash(Self::OWNER_OF, params).await?)
	}

	async fn throw_if_divisible_nft(&mut self) -> Result<(), ContractError> {
		if self.get_decimals().await.unwrap() != 0 {
			return Err(ContractError::InvalidStateError(
				"This method is only intended for non-divisible NFTs.".to_string(),
			));
		}

		Ok(())
	}

	async fn throw_if_sender_is_not_owner(
		&mut self,
		from: &ScriptHash,
		token_id: &Bytes,
	) -> Result<(), ContractError> {
		let token_owner = &self.owner_of(token_id).await?;
		if token_owner != from {
			return Err(ContractError::InvalidArgError(
				"The provided from account is not the owner of this token.".to_string(),
			));
		}

		Ok(())
	}

	// Divisible NFT methods

	async fn transfer_divisible(
		&mut self,
		from: &Account,
		to: &ScriptHash,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<P>, ContractError> {
		let mut builder = self
			.transfer_divisible_from_hashes(&from.get_script_hash(), to, amount, token_id, data)
			.await
			.unwrap();
		builder.set_signers(vec![AccountSigner::called_by_entry(from).unwrap().into()]);
		Ok(builder)
	}

	async fn transfer_divisible_from_hashes(
		&mut self,
		from: &ScriptHash,
		to: &ScriptHash,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<P>, ContractError> {
		self.throw_if_non_divisible_nft().await.unwrap();

		self.invoke_function(
			<NftContract<P> as NonFungibleTokenTrait<P>>::TRANSFER,
			vec![from.into(), to.into(), amount.into(), token_id.into(), data.unwrap()],
		)
		.await
	}

	async fn transfer_divisible_from_name(
		&mut self,
		from: &Account,
		to: &str,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<P>, ContractError> {
		let mut builder = self
			.transfer_divisible_from_hashes(
				&from.get_script_hash(),
				&self.resolve_nns_text_record(&NNSName::new(to).unwrap()).await.unwrap(),
				amount,
				token_id,
				data,
			)
			.await
			.unwrap();
		builder.set_signers(vec![AccountSigner::called_by_entry(from).unwrap().into()]);
		Ok(builder)
	}

	async fn transfer_divisible_to_name(
		&mut self,
		from: &ScriptHash,
		to: &str,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<P>, ContractError> {
		self.throw_if_non_divisible_nft().await.unwrap();

		self.transfer_divisible_from_hashes(
			from,
			&self.resolve_nns_text_record(&NNSName::new(to).unwrap()).await.unwrap(),
			amount,
			token_id,
			data,
		)
		.await
	}

	async fn build_divisible_transfer_script(
		&self,
		from: Address,
		to: Address,
		amount: i32,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<Bytes, ContractError> {
		self.build_invoke_function_script(
			<NftContract<P> as NonFungibleTokenTrait<P>>::TRANSFER,
			vec![from.into(), to.into(), amount.into(), token_id.into(), data.unwrap()],
		)
		.await
	}

	async fn owners_of(
		&mut self,
		token_id: Bytes,
	) -> Result<NeoIterator<Address, P>, ContractError> {
		self.throw_if_non_divisible_nft().await.unwrap();

		Ok(self
			.call_function_returning_iterator(
				<NftContract<P> as NonFungibleTokenTrait<P>>::OWNER_OF,
				vec![token_id.into()],
				Arc::new(|item: StackItem| item.as_address().unwrap()),
			)
			.await)
	}

	async fn throw_if_non_divisible_nft(&mut self) -> Result<(), ContractError> {
		if self.get_decimals().await.unwrap() == 0 {
			return Err(ContractError::InvalidStateError(
				"This method is only intended for divisible NFTs.".to_string(),
			));
		}

		Ok(())
	}

	async fn balance_of_divisible(
		&mut self,
		owner: H160,
		token_id: Bytes,
	) -> Result<i32, ContractError> {
		self.throw_if_non_divisible_nft().await.unwrap();

		self.call_function_returning_int(
			<NftContract<P> as NonFungibleTokenTrait<P>>::BALANCE_OF,
			vec![owner.into(), token_id.into()],
		)
		.await
	}

	// Optional methods

	async fn tokens(&self) -> Result<NeoIterator<'_, Vec<u8>, P>, ContractError> {
		let mapper_fn = Arc::new(|item: StackItem| item.as_bytes().unwrap_or_default());

		self.call_function_returning_iterator(
			<NftContract<P> as NonFungibleTokenTrait<P>>::TOKENS,
			vec![],
			mapper_fn,
		)
		.await
	}

	async fn properties(
		&mut self,
		token_id: Bytes,
	) -> Result<HashMap<String, String>, ContractError> {
		let invocation_result = self
			.call_invoke_function(
				<NftContract<P> as NonFungibleTokenTrait<P>>::PROPERTIES,
				vec![token_id.into()],
				vec![],
			)
			.await
			.unwrap();

		let stack_item = invocation_result.get_first_stack_item().unwrap();
		let map = stack_item
			.as_map()
			.ok_or(ContractError::UnexpectedReturnType(
				stack_item.to_string() + &StackItem::MAP_VALUE.to_string(),
			))
			.unwrap();

		map.iter()
			.map(|(k, v)| {
				let key = k.as_string().unwrap();
				let value = v.as_string().unwrap();
				Ok((key, value))
			})
			.collect()
	}

	async fn custom_properties(
		&mut self,
		token_id: Bytes,
	) -> Result<HashMap<String, StackItem>, ContractError> {
		let invocation_result = self
			.call_invoke_function(
				<NftContract<P> as NonFungibleTokenTrait<P>>::PROPERTIES,
				vec![token_id.into()],
				vec![],
			)
			.await
			.unwrap();

		let stack_item = invocation_result.get_first_stack_item().unwrap();
		let map = stack_item
			.as_map()
			.ok_or(ContractError::UnexpectedReturnType(
				stack_item.to_string() + &StackItem::MAP_VALUE.to_string(),
			))
			.unwrap();

		map.into_iter()
			.map(|(k, v)| {
				let key = k.as_string().unwrap();
				Ok((key, v.clone()))
			})
			.collect()
	}

	async fn build_transfer_script(
		&self,
		from: &H160,
		to: &H160,
		token_id: &Bytes,
		data: Option<ContractParameter>,
	) -> Result<Bytes, ContractError> {
		let mut params = vec![from.into(), to.into()];
		
		let token_id_vec = token_id.to_vec();
		params.push(ContractParameter::from(token_id_vec.as_slice()));
		
		if let Some(d) = data {
			params.push(d);
		}
		
		self.build_invoke_function_script(Self::TRANSFER, params).await
	}

	async fn transfer_from_account<W>(
		&mut self,
		from: &Account<W>,
		to: &H160,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let mut builder = self
			.transfer_from_hash160(&from.address_or_scripthash().script_hash(), to, token_id, data)
			.await?;
		builder.set_signers(vec![Signer::AccountSigner(AccountSigner::called_by_entry_hash160(from.address_or_scripthash().script_hash()).unwrap())]);

		Ok(builder)
	}

	async fn transfer_from_hash160(
		&mut self,
		from: &H160,
		to: &H160,
		token_id: Bytes,
		data: Option<ContractParameter>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let transfer_script = self.build_transfer_script(from, to, token_id, data).await?;
		let mut builder = TransactionBuilder::new();
		builder.set_script(Some(transfer_script));
		Ok(builder)
	}
}
