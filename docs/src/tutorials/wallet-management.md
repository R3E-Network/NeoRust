# Wallet Management

This tutorial covers wallet management with the NeoRust SDK, including creating, loading, and using wallets.

## Creating a New Wallet

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new wallet with a password
    let password = "my-secure-password";
    let wallet = Wallet::new(password)?;
    
    // Generate a new account
    let account = wallet.create_account()?;
    println!("New account address: {}", account.address());
    
    // Save the wallet to a file
    wallet.save("my-wallet.json")?;
    
    Ok(())
}
```

## Loading an Existing Wallet

```rust
use neo::prelude::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a wallet from a file
    let wallet_path = Path::new("my-wallet.json");
    let password = "my-secure-password";
    let wallet = Wallet::load(wallet_path, password)?;
    
    // Get the default account
    let account = wallet.default_account()?;
    println!("Default account address: {}", account.address());
    
    Ok(())
}
```

## Importing a Private Key

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new wallet
    let password = "my-secure-password";
    let mut wallet = Wallet::new(password)?;
    
    // Import a private key
    let private_key = "your-private-key-here";
    let account = wallet.import_private_key(private_key, password)?;
    println!("Imported account address: {}", account.address());
    
    // Save the wallet
    wallet.save("my-wallet.json")?;
    
    Ok(())
}
```

## Signing a Message

```rust
use neo::prelude::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a wallet
    let wallet_path = Path::new("my-wallet.json");
    let password = "my-secure-password";
    let wallet = Wallet::load(wallet_path, password)?;
    
    // Get the default account
    let account = wallet.default_account()?;
    
    // Sign a message
    let message = b"Hello, Neo!";
    let signature = account.sign_message(message)?;
    println!("Signature: {:?}", signature);
    
    // Verify the signature
    let is_valid = account.verify_signature(message, &signature)?;
    println!("Signature valid: {}", is_valid);
    
    Ok(())
}
```

## Wallet Backup and Recovery

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new wallet
    let password = "my-secure-password";
    let wallet = Wallet::new(password)?;
    
    // Generate a new account
    let account = wallet.create_account()?;
    
    // Get the mnemonic phrase for backup
    let mnemonic = wallet.export_mnemonic(password)?;
    println!("Backup phrase: {}", mnemonic);
    
    // Later, recover the wallet from the mnemonic
    let recovered_wallet = Wallet::from_mnemonic(&mnemonic, password)?;
    
    // Verify the recovered wallet has the same account
    let recovered_account = recovered_wallet.default_account()?;
    println!("Recovered account address: {}", recovered_account.address());
    
    assert_eq!(account.address(), recovered_account.address());
    
    Ok(())
}
```

## Using SGX-Protected Wallets

If you have enabled the SGX feature, you can use SGX-protected wallets for enhanced security:

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the SGX enclave
    let enclave_path = "path/to/enclave.so";
    let enclave_manager = SgxEnclaveManager::new(enclave_path)?;
    
    // Create a wallet with a password
    let password = "my-secure-password";
    let wallet = enclave_manager.create_wallet(password)?;
    
    // Get the wallet's public key
    let public_key = wallet.get_public_key();
    println!("Wallet public key: {:?}", public_key);
    
    // Sign a transaction using the SGX-protected private key
    let transaction_data = b"Sample transaction data";
    let signature = wallet.sign_transaction(transaction_data)?;
    
    Ok(())
}
```

## Best Practices

1. **Secure Password Storage**: Never hardcode passwords in your application.
2. **Regular Backups**: Always backup your wallet's mnemonic phrase or private keys.
3. **Verify Addresses**: Always verify addresses before sending transactions.
4. **Use Hardware Wallets**: For production applications, consider using hardware wallets via the Ledger feature.
5. **SGX Protection**: For high-security applications, use SGX-protected wallets.

<!-- toc -->
