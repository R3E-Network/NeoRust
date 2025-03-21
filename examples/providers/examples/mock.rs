//! `MockProvider` is a mock Neo provider that can be used for testing purposes.
//! It allows to simulate Neo state and behavior, by explicitly instructing
//! provider's responses on client requests.
//!
//! This can be useful for testing code that relies on providers without the need to
//! connect to a real network or spend real Gas. It also allows to test code in a
//! deterministic manner, as you can control the state and behavior of the provider.
//!
//! In these examples we use the common Arrange, Act, Assert (AAA) test approach.
//! It is a useful pattern for well-structured, understandable and maintainable tests.

use NeoRust::prelude::*;

#[tokio::main]
async fn main() -> eyre::Result<()> {
	mocked_block_number().await?;
	mocked_provider_dependency().await?;
	Ok(())
}

async fn mocked_block_number() -> eyre::Result<()> {
	// Arrange
	let mock = MockProvider::new();
	let block_num_1 = 1u64;
	let block_num_2 = 2u64;
	let block_num_3 = 3u64;
	// Mock responses are organized in a stack (LIFO)
	mock.push(block_num_1)?;
	mock.push(block_num_2)?;
	mock.push(block_num_3)?;

	// Act
	let ret_block_3: u64 = JsonRpcClient::fetch(&mock, "blockNumber", ()).await?;
	let ret_block_2: u64 = JsonRpcClient::fetch(&mock, "blockNumber", ()).await?;
	let ret_block_1: u64 = JsonRpcClient::fetch(&mock, "blockNumber", ()).await?;

	// Assert
	assert_eq!(block_num_1, ret_block_1);
	assert_eq!(block_num_2, ret_block_2);
	assert_eq!(block_num_3, ret_block_3);

	Ok(())
}

/// Here we test the `OddBlockOracle` struct (defined below) that relies
/// on a Provider to perform some logics.
/// The Provider reference is expressed with trait bounds, enforcing lose coupling,
/// maintainability and testability.
async fn mocked_provider_dependency() -> eyre::Result<()> {
	// Arrange
	let (provider, mock) = crate::Provider::mocked();
	mock.push(2)?;

	// Act
	// Let's mock the provider dependency (we ❤️ DI!) then ask for the answer
	let oracle = OddBlockOracle::new(provider);
	let answer: bool = oracle.is_odd_block().await?;

	// Assert
	assert!(answer);
	Ok(())
}

struct OddBlockOracle<P> {
	provider: Provider<P>,
}

impl<P> OddBlockOracle<P>
where
	P: JsonRpcClient,
{
	fn new(provider: Provider<P>) -> Self {
		Self { provider }
	}

	/// We want to test this!
	async fn is_odd_block(&self) -> eyre::Result<bool> {
		let block: u32 = self.provider.get_block_count().await?;
		Ok(block % 2 == 0)
	}
}
