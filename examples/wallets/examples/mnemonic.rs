use eyre::Result;
use NeoRust::prelude::*;

fn main() -> Result<()> {
    // Create a new BIP39 account with password
    let password = "my secure password";
    let account = Bip39Account::create(password)?;
    
    println!("New Account:");
    println!("Address: {}", account.account.get_address());
    println!("Mnemonic: {}", account.mnemonic);

    // Recover an existing account from mnemonic
    let mnemonic = "work man father plunge mystery proud hollow address reunion sauce theory bonus";
    let recovered = Bip39Account::from_bip39_mnemonic(password, mnemonic)?;

    println!("\nRecovered Account:");
    println!("Address: {}", recovered.account.get_address());
    println!("Mnemonic: {}", recovered.mnemonic);

    Ok(())
}
