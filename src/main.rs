use tx_orderer_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tx_orderer_cli::run().await?;
    Ok(())
}