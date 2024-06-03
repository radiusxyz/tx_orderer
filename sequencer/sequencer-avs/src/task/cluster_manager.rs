use std::time::Duration;

use database::database;
use ssal::ethereum::{SeederClient, SsalClient};
use tokio::time::sleep;
use tracing;

use crate::{config::Config, error::Error};

pub fn init(config: &Config) -> Result<(), Error> {
    // Initialize SSAL client.
    let ssal_client = SsalClient::new(
        &config.ssal_address,
        &config.contract_address,
        config.cluster_id,
    )
    .map_err(Error::Ssal)?;

    // Initialize Seeder client.
    let seeder_client = SeederClient::new(&config.seeder_address).map_err(Error::Seeder)?;

    tokio::spawn(async move {
        loop {
            let cluster_info = match ssal_client.get_sequencer_list().await {
                Ok(cluster_info) => cluster_info,
                Err(error) => {
                    tracing::error!("{}", error);
                    None
                }
            };

            if let Some((block_number, sequencer_list)) = cluster_info {
                match seeder_client.get_address_list(&sequencer_list).await {
                    Ok(address_list) => {
                        let _ = database().put(&block_number, &address_list);
                    }
                    Err(error) => {
                        tracing::error!("{}", error);
                    }
                }
            }

            // Wait for the next request.
            sleep(Duration::from_secs(3)).await;
        }
    });
    Ok(())
}
