# Example Code

This section contains examples demonstrating how to use the NeoRust SDK.

## Wallet Management

```rust
use neo::prelude::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new wallet
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

For more examples, see the `examples` directory in the NeoRust repository.
