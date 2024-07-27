extern crate NeoRust;

use NeoRust::prelude::*;
// use the eyre crate for easy idiomatic error handling
use eyre::Result;

// Use the `tokio::main` macro for using async on the main function
#[tokio::main]
async fn main() -> Result<()> {
    // Generate a random wallet
    let wallet = LocalWallet::new(&mut thread_rng());

    // Declare the message you want to sign.
    let message = "Some data";

    // sign message from your wallet and print out signature produced.
    let signature = wallet.sign_message(message).await?;
    println!("Produced signature {signature}");

    // verify the signature produced from your wallet.
    signature.verify(message, wallet.address()).unwrap();
    println!("Verified signature produced by {:?}!", wallet.address());

    Ok(())
}
