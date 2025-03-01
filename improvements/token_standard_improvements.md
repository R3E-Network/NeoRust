# Enhanced NEP-17 and NEP-11 Token Standards Support

This document outlines improvements to enhance support for Neo N3 token standards NEP-17 (fungible tokens) and NEP-11 (non-fungible tokens) in the NeoRust SDK.

## Current State Assessment

The NeoRust SDK currently provides basic support for NEP-17 tokens through the `FungibleTokenContract` abstraction, but could benefit from more comprehensive implementations for both NEP-17 and NEP-11 standards with additional utility functions, better type safety, and improved developer ergonomics.

## Proposed Improvements for NEP-17 Support

### 1. Enhanced `Nep17Token` Trait and Implementation

```rust
/// Trait defining the standard NEP-17 interface
#[async_trait]
pub trait Nep17Token {
    /// Returns the token symbol
    async fn symbol(&self) -> Result<String, ContractError>;
    
    /// Returns the token decimals
    async fn decimals(&self) -> Result<u8, ContractError>;
    
    /// Returns the token name
    async fn name(&self) -> Result<String, ContractError>;
    
    /// Returns the total token supply
    async fn total_supply(&self) -> Result<u64, ContractError>;
    
    /// Returns the token balance for the specified account
    async fn balance_of(&self, account: &ScriptHash) -> Result<u64, ContractError>;
    
    /// Transfers tokens from the sender to the specified recipient
    async fn transfer(
        &self, 
        from: &Account, 
        to: &ScriptHash, 
        amount: u64, 
        data: Option<&str>
    ) -> Result<String, ContractError>;
    
    /// Returns the formatted token balance with proper decimal formatting
    async fn formatted_balance_of(&self, account: &ScriptHash) -> Result<String, ContractError> {
        let balance = self.balance_of(account).await?;
        let decimals = self.decimals().await?;
        let divisor = 10_u64.pow(decimals as u32);
        let integer_part = balance / divisor;
        let fractional_part = balance % divisor;
        Ok(format!("{}.{:0width$}", integer_part, fractional_part, width = decimals as usize))
    }
    
    /// Creates a transfer transaction without sending it
    async fn create_transfer_tx(
        &self,
        from: &Account,
        to: &ScriptHash,
        amount: u64,
        data: Option<&str>
    ) -> Result<TransactionBuilder, ContractError>;
    
    /// Transfers tokens from multiple senders to multiple recipients in a batch
    async fn batch_transfer(
        &self,
        from: &Account,
        transfers: &[(ScriptHash, u64)],
        data: Option<&str>
    ) -> Result<String, ContractError>;
}
```

### 2. Token Amount Type with Decimal Handling

```rust
/// A type-safe token amount representation with decimal awareness
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenAmount {
    /// The raw token amount as smallest unit
    raw_amount: u64,
    /// The number of decimal places for this token
    decimals: u8,
}

impl TokenAmount {
    /// Creates a new token amount from a raw value and decimals
    pub fn new(raw_amount: u64, decimals: u8) -> Self {
        Self { raw_amount, decimals }
    }
    
    /// Creates a token amount from a human-readable float value
    pub fn from_float(amount: f64, decimals: u8) -> Self {
        let multiplier = 10_u64.pow(decimals as u32) as f64;
        let raw_amount = (amount * multiplier).round() as u64;
        Self { raw_amount, decimals }
    }
    
    /// Creates a token amount from a string representation
    pub fn from_string(amount: &str, decimals: u8) -> Result<Self, ContractError> {
        let parts: Vec<&str> = amount.split('.').collect();
        if parts.len() > 2 {
            return Err(ContractError::InvalidArgError(
                "Invalid token amount format".to_string(),
            ));
        }
        
        let integer_part = parts[0].parse::<u64>().map_err(|_| {
            ContractError::InvalidArgError("Could not parse integer part".to_string())
        })?;
        
        let fractional_part = if parts.len() == 2 {
            let frac = parts[1];
            let frac_len = frac.len().min(decimals as usize);
            let frac_str = format!("{:0<width$}", frac, width = decimals as usize);
            frac_str[0..frac_len].parse::<u64>().map_err(|_| {
                ContractError::InvalidArgError("Could not parse fractional part".to_string())
            })?
        } else {
            0
        };
        
        let raw_amount = integer_part * 10_u64.pow(decimals as u32) + fractional_part;
        Ok(Self { raw_amount, decimals })
    }
    
    /// Returns the raw token amount
    pub fn raw_amount(&self) -> u64 {
        self.raw_amount
    }
    
    /// Returns the formatted token amount as a string
    pub fn formatted(&self) -> String {
        let divisor = 10_u64.pow(self.decimals as u32);
        let integer_part = self.raw_amount / divisor;
        let fractional_part = self.raw_amount % divisor;
        format!("{}.{:0width$}", integer_part, fractional_part, width = self.decimals as usize)
    }
}
```

### 3. Enhanced NEP-17 Contract Implementation

```rust
/// A Concrete implementation of NEP-17 token standard
pub struct Nep17Contract<'a, P: JsonRpcProvider> {
    script_hash: ScriptHash,
    provider: Option<&'a RpcClient<P>>,
    symbol_cache: Option<String>,
    decimals_cache: Option<u8>,
    name_cache: Option<String>,
}

#[async_trait]
impl<'a, P: JsonRpcProvider + 'static> Nep17Token for Nep17Contract<'a, P> {
    async fn symbol(&self) -> Result<String, ContractError> {
        if let Some(symbol) = &self.symbol_cache {
            return Ok(symbol.clone());
        }
        
        let provider = self.provider.ok_or(ContractError::ProviderNotSet("No provider".to_string()))?;
        let result = provider.invoke_function(&self.script_hash, "symbol".to_string(), vec![], None).await?;
        let symbol = result.stack[0].as_string().ok_or_else(|| {
            ContractError::UnexpectedReturnType("Expected string for symbol".to_string())
        })?;
        
        Ok(symbol)
    }
    
    async fn decimals(&self) -> Result<u8, ContractError> {
        if let Some(decimals) = self.decimals_cache {
            return Ok(decimals);
        }
        
        let provider = self.provider.ok_or(ContractError::ProviderNotSet("No provider".to_string()))?;
        let result = provider.invoke_function(&self.script_hash, "decimals".to_string(), vec![], None).await?;
        let decimals = result.stack[0].as_int().ok_or_else(|| {
            ContractError::UnexpectedReturnType("Expected integer for decimals".to_string())
        })? as u8;
        
        Ok(decimals)
    }
    
    // ... other method implementations ...
    
    async fn create_transfer_tx(
        &self,
        from: &Account,
        to: &ScriptHash,
        amount: u64,
        data: Option<&str>
    ) -> Result<TransactionBuilder, ContractError> {
        let mut params = vec![
            ContractParameter::from(from.get_script_hash()),
            ContractParameter::from(to.clone()),
            ContractParameter::from(amount),
        ];
        
        if let Some(data_str) = data {
            params.push(ContractParameter::from(data_str));
        } else {
            params.push(ContractParameter::any(None));
        }
        
        let provider = self.provider.ok_or(ContractError::ProviderNotSet("No provider".to_string()))?;
        let mut builder = TransactionBuilder::new();
        
        let script = ScriptBuilder::new()
            .contract_call(&self.script_hash, "transfer", &params, Some(CallFlags::All))
            .map_err(|e| ContractError::InvalidArgError(e.to_string()))?
            .to_bytes();
        
        builder.set_script(script);
        builder.add_signer(Signer::account_with_scope(from, WitnessScope::CalledByEntry));
        
        Ok(builder)
    }
}
```

## Proposed Improvements for NEP-11 Support

### 1. Comprehensive `Nep11Token` Trait

```rust
/// Trait defining the standard NEP-11 interface for non-divisible NFTs
#[async_trait]
pub trait Nep11Token {
    /// Returns the token symbol
    async fn symbol(&self) -> Result<String, ContractError>;
    
    /// Returns the token decimals (always 0 for non-divisible NFTs)
    async fn decimals(&self) -> Result<u8, ContractError> {
        Ok(0)
    }
    
    /// Returns the token name
    async fn name(&self) -> Result<String, ContractError>;
    
    /// Returns the total token supply (number of NFTs)
    async fn total_supply(&self) -> Result<u64, ContractError>;
    
    /// Returns token balances for the specified account (number of NFTs owned)
    async fn balance_of(&self, owner: &ScriptHash) -> Result<u64, ContractError>;
    
    /// Returns all token IDs owned by the specified account
    async fn tokens_of(&self, owner: &ScriptHash) -> Result<Vec<String>, ContractError>;
    
    /// Returns the owner of the specified token
    async fn owner_of(&self, token_id: &[u8]) -> Result<ScriptHash, ContractError>;
    
    /// Returns the properties/metadata of the specified token
    async fn properties(&self, token_id: &[u8]) -> Result<HashMap<String, StackItem>, ContractError>;
    
    /// Transfers a token from the sender to the specified recipient
    async fn transfer(
        &self, 
        from: &Account, 
        to: &ScriptHash, 
        token_id: &[u8], 
        data: Option<&str>
    ) -> Result<String, ContractError>;
    
    /// Creates a transfer transaction without sending it
    async fn create_transfer_tx(
        &self,
        from: &Account,
        to: &ScriptHash,
        token_id: &[u8],
        data: Option<&str>
    ) -> Result<TransactionBuilder, ContractError>;
}

/// Extension trait for divisible NFTs
#[async_trait]
pub trait DivisibleNep11Token: Nep11Token {
    /// Returns the token balance for the specified owner and token ID
    async fn balance_of_token(&self, owner: &ScriptHash, token_id: &[u8]) -> Result<u64, ContractError>;
    
    /// Transfers a specified amount of a divisible token
    async fn transfer_from(
        &self,
        from: &Account,
        to: &ScriptHash,
        token_id: &[u8],
        amount: u64,
        data: Option<&str>
    ) -> Result<String, ContractError>;
}
```

### 2. NFT Metadata Type

```rust
/// NFT property keys
pub enum NftPropertyKey {
    Name,
    Description,
    Image,
    TokenUri,
    Attributes,
    ExternalUrl,
    Background,
    Custom(String),
}

impl NftPropertyKey {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Name => "name",
            Self::Description => "description",
            Self::Image => "image",
            Self::TokenUri => "tokenUri",
            Self::Attributes => "attributes",
            Self::ExternalUrl => "externalUrl",
            Self::Background => "background",
            Self::Custom(s) => s,
        }
    }
}

/// A structured representation of NFT metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub token_uri: Option<String>,
    pub external_url: Option<String>,
    pub attributes: Option<Vec<NftAttribute>>,
    #[serde(flatten)]
    pub additional_properties: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftAttribute {
    pub trait_type: String,
    pub value: Value,
    pub display_type: Option<String>,
}

impl NftMetadata {
    /// Creates an NftMetadata from a map of contract properties
    pub fn from_properties(properties: &HashMap<String, StackItem>) -> Result<Self, ContractError> {
        let mut metadata = NftMetadata {
            name: None,
            description: None,
            image: None,
            token_uri: None,
            external_url: None,
            attributes: None,
            additional_properties: HashMap::new(),
        };
        
        // Extract standard properties
        if let Some(StackItem::ByteString { value }) = 
            properties.get(NftPropertyKey::Name.as_str()) {
            metadata.name = Some(value.clone());
        }
        
        if let Some(StackItem::ByteString { value }) = 
            properties.get(NftPropertyKey::Description.as_str()) {
            metadata.description = Some(value.clone());
        }
        
        if let Some(StackItem::ByteString { value }) = 
            properties.get(NftPropertyKey::Image.as_str()) {
            metadata.image = Some(value.clone());
        }
        
        // ... process other properties ...
        
        // Process attributes if they exist
        if let Some(attrs) = properties.get(NftPropertyKey::Attributes.as_str()) {
            if let Some(attrs_array) = attrs.as_array() {
                let mut attributes = Vec::new();
                for attr in attrs_array {
                    if let Some(attr_map) = attr.as_map() {
                        // ... parse attribute ...
                    }
                }
                metadata.attributes = Some(attributes);
            }
        }
        
        Ok(metadata)
    }
}
```

### 3. NEP-11 Contract Implementation

```rust
/// An implementation of the NEP-11 token standard
pub struct Nep11Contract<'a, P: JsonRpcProvider> {
    script_hash: ScriptHash,
    provider: Option<&'a RpcClient<P>>,
    symbol_cache: Option<String>,
    name_cache: Option<String>,
    is_divisible: Option<bool>,
}

impl<'a, P: JsonRpcProvider> Nep11Contract<'a, P> {
    /// Creates a new NEP-11 contract instance
    pub fn new(script_hash: ScriptHash, provider: Option<&'a RpcClient<P>>) -> Self {
        Self {
            script_hash,
            provider,
            symbol_cache: None,
            name_cache: None,
            is_divisible: None,
        }
    }
    
    /// Checks if this NFT contract supports divisible tokens
    pub async fn is_divisible(&self) -> Result<bool, ContractError> {
        if let Some(is_divisible) = self.is_divisible {
            return Ok(is_divisible);
        }
        
        let provider = self.provider.ok_or(ContractError::ProviderNotSet("No provider".to_string()))?;
        
        // Try to call the divisible method
        let result = provider.invoke_function(&self.script_hash, "decimals".to_string(), vec![], None).await;
        
        match result {
            Ok(invoke_result) => {
                if let Some(value) = invoke_result.stack.get(0) {
                    if let Some(decimals) = value.as_int() {
                        return Ok(decimals > 0);
                    }
                }
                Ok(false)
            }
            Err(_) => Ok(false),
        }
    }
    
    /// Gets the metadata for a specific token
    pub async fn get_token_metadata(&self, token_id: &[u8]) -> Result<NftMetadata, ContractError> {
        let properties = self.properties(token_id).await?;
        NftMetadata::from_properties(&properties)
    }
    
    /// Gets all tokens owned by an account with their metadata
    pub async fn get_all_tokens_with_metadata(
        &self, 
        owner: &ScriptHash
    ) -> Result<HashMap<String, NftMetadata>, ContractError> {
        let token_ids = self.tokens_of(owner).await?;
        let mut result = HashMap::new();
        
        for token_id_str in token_ids {
            let token_id = token_id_str.as_bytes();
            let metadata = self.get_token_metadata(token_id).await?;
            result.insert(token_id_str, metadata);
        }
        
        Ok(result)
    }
}

#[async_trait]
impl<'a, P: JsonRpcProvider + 'static> Nep11Token for Nep11Contract<'a, P> {
    // ... implementations of NEP-11 trait methods ...
}

#[async_trait]
impl<'a, P: JsonRpcProvider + 'static> DivisibleNep11Token for Nep11Contract<'a, P> {
    // ... implementations of divisible NEP-11 trait methods ...
}
```

## Example Usage

### NEP-17 Token Example
```rust
use neo::prelude::*;

async fn work_with_nep17_token() -> Result<(), Box<dyn std::error::Error>> {
    // Setup client
    let provider = HttpProvider::new("https://mainnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Create a reference to a NEP-17 token (e.g., GAS)
    let gas_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
    let gas_token = Nep17Contract::new(gas_hash, Some(&client));
    
    // Get basic token info
    let symbol = gas_token.symbol().await?;
    let decimals = gas_token.decimals().await?;
    println!("Token: {} with {} decimals", symbol, decimals);
    
    // Check balance
    let address = ScriptHash::from_address("NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc")?;
    let balance = gas_token.balance_of(&address).await?;
    let formatted_balance = gas_token.formatted_balance_of(&address).await?;
    println!("Balance: {} ({} raw units)", formatted_balance, balance);
    
    // Create a token amount from a human-readable value
    let amount = TokenAmount::from_string("1.5", decimals)?;
    println!("Amount: {} (raw: {})", amount.formatted(), amount.raw_amount());
    
    // Create a transfer transaction
    let account = Account::from_wif("YOUR_WIF_HERE")?;
    let recipient = ScriptHash::from_address("NYxb4fSZVKAz8YrgvZJX1Vbf9VgXvED2W2")?;
    
    let tx_builder = gas_token.create_transfer_tx(
        &account,
        &recipient,
        amount.raw_amount(),
        Some("Payment")
    ).await?;
    
    // Sign and send the transaction
    let tx = tx_builder.build();
    let signed_tx = tx.sign(&account).await?;
    let tx_hash = client.send_raw_transaction(signed_tx).await?;
    
    println!("Transaction sent: {}", tx_hash);
    
    Ok(())
}
```

### NEP-11 NFT Example
```rust
use neo::prelude::*;

async fn work_with_nep11_token() -> Result<(), Box<dyn std::error::Error>> {
    // Setup client
    let provider = HttpProvider::new("https://mainnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Create a reference to a NEP-11 token contract
    let nft_hash = ScriptHash::from_str("0xd7e156f58b3b11881f7b02ac43068c3c07cbc3ec")?;
    let nft_contract = Nep11Contract::new(nft_hash, Some(&client));
    
    // Check if it's a divisible NFT
    let is_divisible = nft_contract.is_divisible().await?;
    println!("NFT contract is{} divisible", if is_divisible { "" } else { " not" });
    
    // Get NFTs owned by an account
    let address = ScriptHash::from_address("NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc")?;
    let token_ids = nft_contract.tokens_of(&address).await?;
    println!("Account owns {} NFTs", token_ids.len());
    
    // Get metadata for each NFT
    for token_id_str in &token_ids {
        let token_id = token_id_str.as_bytes();
        let metadata = nft_contract.get_token_metadata(token_id).await?;
        println!("NFT: {}", metadata.name.unwrap_or_else(|| "Unnamed".to_string()));
        
        if let Some(image) = &metadata.image {
            println!("  Image: {}", image);
        }
        
        if let Some(description) = &metadata.description {
            println!("  Description: {}", description);
        }
        
        if let Some(attributes) = &metadata.attributes {
            println!("  Attributes:");
            for attr in attributes {
                println!("    {}: {}", attr.trait_type, attr.value);
            }
        }
    }
    
    // Transfer an NFT
    if !token_ids.is_empty() {
        let account = Account::from_wif("YOUR_WIF_HERE")?;
        let recipient = ScriptHash::from_address("NYxb4fSZVKAz8YrgvZJX1Vbf9VgXvED2W2")?;
        let token_id = token_ids[0].as_bytes();
        
        let tx_builder = nft_contract.create_transfer_tx(
            &account,
            &recipient,
            token_id,
            Some("Gift")
        ).await?;
        
        let tx = tx_builder.build();
        let signed_tx = tx.sign(&account).await?;
        let tx_hash = client.send_raw_transaction(signed_tx).await?;
        
        println!("NFT transfer transaction sent: {}", tx_hash);
    }
    
    Ok(())
}
```

## Implementation Plan

1. **Define Core Interfaces**: Create comprehensive traits for NEP-17 and NEP-11 tokens
2. **Implement Token Amount Handling**: Add TokenAmount type with proper decimal handling
3. **Create NFT Metadata Support**: Add types for representing NFT metadata
4. **Implement Contract Abstractions**: Create implementations for both token standards
5. **Add Caching Mechanisms**: Optimize by caching commonly accessed values
6. **Create Examples**: Add comprehensive examples demonstrating token usage
7. **Document Standards**: Add thorough documentation explaining the Neo N3 token standards

## Benefits

1. **Improved Developer Experience**: More intuitive and type-safe APIs
2. **Better Error Handling**: Clear error messages for token-specific errors
3. **Decimal Awareness**: Proper handling of token decimals and formatting
4. **Comprehensive NFT Support**: Full support for both divisible and non-divisible NFTs
5. **Metadata Handling**: First-class support for NFT metadata and properties
6. **Performance Optimizations**: Caching of frequently accessed token properties
7. **Stronger Type Safety**: Reduced risk of errors when working with token amounts 