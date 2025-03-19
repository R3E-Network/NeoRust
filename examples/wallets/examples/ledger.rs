use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
	println!("Neo N3 Ledger Integration Example");
	println!("================================");
	
	println!("\nLedger hardware wallets provide enhanced security for Neo N3 assets.");
	println!("With a Ledger device, your private keys remain on the secure hardware");
	println!("and never leave the device, protecting them from malware and hackers.");
	
	println!("\nKey features of Ledger integration:");
	println!("- Secure key storage on tamper-resistant hardware");
	println!("- Transaction signing without exposing private keys");
	println!("- Support for multiple Neo addresses");
	println!("- Physical confirmation of transactions");
	
	println!("\nTo use a Ledger with Neo:");
	println!("1. Install the Neo app on your Ledger device");
	println!("2. Connect the device to your computer");
	println!("3. Use compatible wallet software that supports Ledger");
	println!("4. Confirm all transactions on the device screen");
	
	println!("\nNote: This example does not actually connect to a Ledger device.");
	
	Ok(())
}
