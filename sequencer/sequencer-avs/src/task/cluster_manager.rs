use std::time::Duration;

use ssal::ethereum::{SeederClient, SsalClient};
use tokio::time::sleep;

use crate::{
    config::Config,
    error::Error,
    rpc::prelude::{SequencerList, SsalBlockNumber},
};

pub fn init(config: &Config) -> Result<(), Error> {
    // Initialize SSAL client.
    let ssal_client = SsalClient::new(
        &config.ssal_rpc_address,
        &config.contract_address,
        config.cluster_id,
    )
    .map_err(Error::Ssal)?;

    // Initialize Seeder client.
    let seeder_client = SeederClient::new(&config.seeder_rpc_address).map_err(Error::Seeder)?;

    tokio::spawn(async move {
        loop {
            let cluster_info = ssal_client
                .get_sequencer_list()
                .await
                .unwrap_or_else(|error| {
                    tracing::error!("{}", error);
                    None
                });

            if let Some((ssal_block_number, sequencer_list)) = cluster_info {
                match seeder_client.get_address_list(&sequencer_list).await {
                    Ok(address_list) => SequencerList::new(sequencer_list, address_list)
                        .put(SsalBlockNumber::from(ssal_block_number))
                        .unwrap_or_else(|error| tracing::error!("{}", error)),
                    Err(error) => tracing::error!("{}", error),
                }
            }

            // Wait for the next request.
            sleep(Duration::from_secs(3)).await;
        }
    });
    Ok(())
}
