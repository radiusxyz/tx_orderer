use database::Database;
use ssal::ethereum::{SeederClient, SsalClient};

use crate::{config::Config, error::Error};

pub fn init(config: &Config, database: Database) -> Result<(), Error> {
    let ssal_client = SsalClient::new(
        &config.ssal_address,
        &config.contract_address,
        config.cluster_id,
    )
    .map_err(Error::Ssal)?;

    let seeder_client = SeederClient::new(&config.seeder_address).map_err(Error::Seeder)?;
    tokio::spawn(async move {
        loop {
            let sequencer_list = ssal_client.get_sequencer_list().await;
            // let address_list = ssal
        }
    });
    Ok(())
}
