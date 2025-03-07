use async_trait::async_trait;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt::Debug;

use neo_builder::{AccountSigner, TransactionBuilder, Signer};
use neo_clients::{APITrait, JsonRpcProvider, RpcClient};
use crate::{ContractError, SmartContractTrait};
use neo_protocol::{Account, AccountTrait};
use neo_common::{deserialize_script_hash, serialize_script_hash};
use neo_types::{ContractParameter, ScriptHash};

/// NeoCompound contract interface for Neo N3
///
/// NeoCompound is an automated interest compounding service for Neo ecosystem tokens.
/// This contract interface provides methods to interact with the NeoCompound smart contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoCompoundContract<'a, P: JsonRpcProvider + APITrait> {
    #[serde(deserialize_with = "deserialize_script_hash")]
    #[serde(serialize_with = "serialize_script_hash")]
    script_hash: ScriptHash,
    #[serde(skip)]
    provider: Option<&'a RpcClient<P>>,
}

impl<'a, P: JsonRpcProvider + APITrait + 'static> NeoCompoundContract<'a, P> {
    /// The script hash of the NeoCompound contract on Neo N3 MainNet
    pub const CONTRACT_HASH: &'static str = "f0151f528127558851b39c2cd8aa47da7418ab28";

    // Method constants
    /// Method name for depositing tokens
    pub const DEPOSIT: &'static str = "deposit";
    /// Method name for withdrawing tokens
    pub const WITHDRAW: &'static str = "withdraw";
    /// Method name for compounding interest
    pub const COMPOUND: &'static str = "compound";
    /// Method name for getting the APY
    pub const GET_APY: &'static str = "getAPY";
    /// Method name for claiming gas
    pub const CLAIM_GAS: &'static str = "claimGas";
    /// Method name for claiming Neo
    pub const CLAIM_NEO: &'static str = "claimNeo";
    /// Method name for claiming all
    pub const CLAIM_ALL: &'static str = "claimAll";
    /// Method name for checking if auto compound is enabled
    pub const IS_AUTO_COMPOUND_ENABLED: &'static str = "isAutoCompoundEnabled";
    /// Method name for enabling auto compound
    pub const ENABLE_AUTO_COMPOUND: &'static str = "enableAutoCompound";
    /// Method name for disabling auto compound
    pub const DISABLE_AUTO_COMPOUND: &'static str = "disableAutoCompound";
    /// Method name for getting deposited amount
    pub const GET_DEPOSITED_AMOUNT: &'static str = "getDepositedAmount";
    /// Method name for getting pending rewards
    pub const GET_PENDING_REWARDS: &'static str = "getPendingRewards";

    /// Creates a new NeoCompoundContract instance with the default contract hash
    ///
    /// # Arguments
    ///
    /// * `provider` - An optional reference to an RPC client
    ///
    /// # Returns
    ///
    /// A new NeoCompoundContract instance
    pub fn new(provider: Option<&'a RpcClient<P>>) -> Self {
        Self { script_hash: ScriptHash::from_str(Self::CONTRACT_HASH).unwrap(), provider }
    }

    /// Creates a new NeoCompoundContract instance with a custom script hash
    ///
    /// # Arguments
    ///
    /// * `script_hash` - The script hash of the NeoCompound contract
    /// * `provider` - An optional reference to an RPC client
    ///
    /// # Returns
    ///
    /// A new NeoCompoundContract instance
    pub fn with_script_hash(script_hash: ScriptHash, provider: Option<&'a RpcClient<P>>) -> Self {
        Self { script_hash, provider }
    }

    /// Deposits tokens into NeoCompound
    ///
    /// # Arguments
    ///
    /// * `token` - The script hash of the token to deposit
    /// * `amount` - The amount of tokens to deposit
    /// * `account` - The account that will sign the transaction
    ///
    /// # Returns
    ///
    /// A transaction builder that can be used to build and sign the transaction
    pub async fn deposit<W>(
        &self,
        token: H160,
        amount: u64,
        account: &Account<W>,
    ) -> Result<TransactionBuilder<'_>, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![token.into(), ContractParameter::integer(amount as i64)];
        
        let mut builder = self.invoke_function(Self::DEPOSIT, params).await?;
        builder.set_signers(vec![Signer::AccountSigner(AccountSigner::called_by_entry_hash160(
            account.address_or_scripthash().script_hash(),
        ).unwrap())]);
        
        Ok(builder)
    }

    /// Withdraws tokens from NeoCompound
    ///
    /// # Arguments
    ///
    /// * `token` - The script hash of the token to withdraw
    /// * `amount` - The amount of tokens to withdraw
    /// * `account` - The account that will sign the transaction
    ///
    /// # Returns
    ///
    /// A transaction builder that can be used to build and sign the transaction
    pub async fn withdraw<W>(
        &self,
        token: H160,
        amount: u64,
        account: &Account<W>,
    ) -> Result<TransactionBuilder<'_>, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![token.into(), ContractParameter::integer(amount as i64)];
        
        let mut builder = self.invoke_function(Self::WITHDRAW, params).await?;
        builder.set_signers(vec![Signer::AccountSigner(AccountSigner::called_by_entry_hash160(
            account.address_or_scripthash().script_hash(),
        ).unwrap())]);
        
        Ok(builder)
    }

    /// Compounds interest for a specific token
    ///
    /// # Arguments
    ///
    /// * `token` - The token to compound interest for
    /// * `account` - The account that will sign the transaction
    ///
    /// # Returns
    ///
    /// A transaction builder that can be used to build and sign the transaction
    pub async fn compound<W>(
        &self,
        token: H160,
        account: &Account<W>,
    ) -> Result<TransactionBuilder<'_>, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![token.into()];
        
        let mut builder = self.invoke_function(Self::COMPOUND, params).await?;
        builder.set_signers(vec![Signer::AccountSigner(AccountSigner::called_by_entry_hash160(
            account.address_or_scripthash().script_hash(),
        ).unwrap())]);
        
        Ok(builder)
    }

    /// Gets the current APY for a specific token
    ///
    /// # Arguments
    ///
    /// * `token` - The script hash of the token to get the APY for
    ///
    /// # Returns
    ///
    /// The APY as a floating-point percentage
    pub async fn get_apy(&self, token: H160) -> Result<f64, ContractError> {
        let result = self.call_function_returning_int(Self::GET_APY, vec![token.into()]).await?;
        // Convert the integer result to a floating-point percentage (assuming APY is stored as an integer with a fixed decimal point)
        Ok(result as f64 / 100.0) // Assuming 2 decimal places for percentage
    }

    /// Claims gas for a specific account
    ///
    /// # Arguments
    ///
    /// * `account` - The account to claim gas for
    ///
    /// # Returns
    ///
    /// A transaction builder that can be used to build and sign the transaction
    pub async fn claim_gas<W>(
        &self,
        account: &Account<W>,
    ) -> Result<TransactionBuilder<'_>, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![];

        let mut builder = self.invoke_function(Self::CLAIM_GAS, params).await?;
        builder.set_signers(vec![Signer::AccountSigner(AccountSigner::called_by_entry_hash160(
            account.address_or_scripthash().script_hash(),
        ).unwrap())]);

        Ok(builder)
    }

    /// Claims Neo for a specific account
    ///
    /// # Arguments
    ///
    /// * `account` - The account to claim Neo for
    ///
    /// # Returns
    ///
    /// A transaction builder that can be used to build and sign the transaction
    pub async fn claim_neo<W>(
        &self,
        account: &Account<W>,
    ) -> Result<TransactionBuilder<'_>, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![];

        let mut builder = self.invoke_function(Self::CLAIM_NEO, params).await?;
        builder.set_signers(vec![Signer::AccountSigner(AccountSigner::called_by_entry_hash160(
            account.address_or_scripthash().script_hash(),
        ).unwrap())]);

        Ok(builder)
    }

    /// Claims all tokens for a specific account
    ///
    /// # Arguments
    ///
    /// * `account` - The account to claim all tokens for
    ///
    /// # Returns
    ///
    /// A transaction builder that can be used to build and sign the transaction
    pub async fn claim_all<W>(
        &self,
        account: &Account<W>,
    ) -> Result<TransactionBuilder<'_>, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![];

        let mut builder = self.invoke_function(Self::CLAIM_ALL, params).await?;
        builder.set_signers(vec![Signer::AccountSigner(AccountSigner::called_by_entry_hash160(
            account.address_or_scripthash().script_hash(),
        ).unwrap())]);

        Ok(builder)
    }

    /// Checks if auto-compound is enabled for a specific token and account
    ///
    /// # Arguments
    ///
    /// * `token` - The token to check
    /// * `account` - The account to check
    ///
    /// # Returns
    ///
    /// True if auto-compound is enabled, false otherwise
    pub async fn is_auto_compound_enabled<W>(
        &self,
        token: H160,
        account: &Account<W>,
    ) -> Result<bool, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![token.into(), account.address_or_scripthash().script_hash().into()];
        
        let result = self
            .call_function_returning_bool(Self::IS_AUTO_COMPOUND_ENABLED, params)
            .await?;
        
        Ok(result)
    }

    /// Enables auto-compound for a specific token
    ///
    /// # Arguments
    ///
    /// * `token` - The token to enable auto-compound for
    /// * `account` - The account that will sign the transaction
    ///
    /// # Returns
    ///
    /// A transaction builder that can be used to build and sign the transaction
    pub async fn enable_auto_compound<W>(
        &self,
        token: H160,
        account: &Account<W>,
    ) -> Result<TransactionBuilder<'_>, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![token.into()];
        
        let mut builder = self.invoke_function(Self::ENABLE_AUTO_COMPOUND, params).await?;
        builder.set_signers(vec![Signer::AccountSigner(AccountSigner::called_by_entry_hash160(
            account.address_or_scripthash().script_hash(),
        ).unwrap())]);
        
        Ok(builder)
    }

    /// Disables auto-compound for a specific token
    ///
    /// # Arguments
    ///
    /// * `token` - The token to disable auto-compound for
    /// * `account` - The account that will sign the transaction
    ///
    /// # Returns
    ///
    /// A transaction builder that can be used to build and sign the transaction
    pub async fn disable_auto_compound<W>(
        &self,
        token: H160,
        account: &Account<W>,
    ) -> Result<TransactionBuilder<'_>, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![token.into()];
        
        let mut builder = self.invoke_function(Self::DISABLE_AUTO_COMPOUND, params).await?;
        builder.set_signers(vec![Signer::AccountSigner(AccountSigner::called_by_entry_hash160(
            account.address_or_scripthash().script_hash(),
        ).unwrap())]);
        
        Ok(builder)
    }

    /// Gets the deposited amount for a specific token and account
    ///
    /// # Arguments
    ///
    /// * `token` - The token to get the deposited amount for
    /// * `account` - The account to get the deposited amount for
    ///
    /// # Returns
    ///
    /// The deposited amount
    pub async fn get_deposited_amount<W>(
        &self,
        token: H160,
        account: &Account<W>,
    ) -> Result<u64, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![token.into(), account.address_or_scripthash().script_hash().into()];
        
        let result = self
            .call_function_returning_int(Self::GET_DEPOSITED_AMOUNT, params)
            .await?;
        
        Ok(result as u64)
    }

    /// Gets the pending rewards for a specific token and account
    ///
    /// # Arguments
    ///
    /// * `token` - The token to get the pending rewards for
    /// * `account` - The account to get the pending rewards for
    ///
    /// # Returns
    ///
    /// The pending rewards
    pub async fn get_pending_rewards<W>(
        &self,
        token: H160,
        account: &Account<W>,
    ) -> Result<u64, ContractError>
    where
        W: Clone + Debug + Send + Sync,
    {
        let params = vec![token.into(), account.address_or_scripthash().script_hash().into()];
        
        let result = self
            .call_function_returning_int(Self::GET_PENDING_REWARDS, params)
            .await?;
        
        Ok(result as u64)
    }
}

#[async_trait]
impl<'a, P: JsonRpcProvider + APITrait + 'static> SmartContractTrait<'a> for NeoCompoundContract<'a, P> {
    type P = P;

    fn script_hash(&self) -> H160 {
        self.script_hash
    }

    fn provider(&self) -> Option<&RpcClient<Self::P>> {
        self.provider
    }
}
