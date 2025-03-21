use std::{fmt, sync::Arc};

use crate::{
	neo_clients::{APITrait, JsonRpcProvider, RpcClient},
	neo_contract::ContractError,
};
use neo3::prelude::*;

pub struct NeoIterator<'a, T, P: JsonRpcProvider> {
	session_id: String,
	iterator_id: String,
	mapper: Arc<dyn Fn(StackItem) -> T + Send + Sync>,
	provider: Option<&'a RpcClient<P>>,
}

impl<'a, T, P: JsonRpcProvider> fmt::Debug for NeoIterator<'a, T, P> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("NeoIterator")
			.field("session_id", &self.session_id)
			.field("iterator_id", &self.iterator_id)
			// For the mapper, you can decide what to print. Here, we just print a static string.
			.field("mapper", &"<function>")
			.finish()
	}
}

impl<'a, T, P: JsonRpcProvider> NeoIterator<'a, T, P> {
	pub fn new(
		session_id: String,
		iterator_id: String,
		mapper: Arc<dyn Fn(StackItem) -> T + Send + Sync>,
		provider: Option<&'a RpcClient<P>>,
	) -> Self {
		Self { session_id, iterator_id, mapper, provider }
	}

	pub async fn traverse(&self, count: i32) -> Result<Vec<T>, ContractError> {
		let result = self
			.provider
			.unwrap()
			.traverse_iterator(self.session_id.clone(), self.iterator_id.clone(), count as u32)
			.await?;
		let mapped = result.iter().map(|item| (self.mapper)(item.clone())).collect();
		Ok(mapped)
	}

	pub async fn terminate_session(&self) -> Result<(), ContractError> {
		self.provider
			.unwrap()
			.terminate_session(&self.session_id)
			.await
			.expect("Could not terminate session");
		Ok(())
	}
}
