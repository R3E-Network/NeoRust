extern crate NeoRust;

use NeoRust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect over websockets
    let provider = Provider::new(Ws::connect("ws://localhost:8545").await?);
    let ledger = Ledger::new(HDPath::LedgerLive(0), 1).await?;
    let client = SignerMiddleware::new(provider, ledger);

    let tx = TransactionRequest::new().to("erik.neo").value(parse_ether(10)?);
    let pending_tx = client.send_transaction(tx, None).await?;

    // Get the receipt
    let _receipt = pending_tx.confirmations(3).await?;
    Ok(())
}
