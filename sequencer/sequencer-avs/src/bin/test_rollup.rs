use std::{env, time::Duration};

use database::Database;
use json_rpc::{RpcClient, RpcMethod};
use sequencer_avs::{config::Config, error::Error, rpc::external::*, types::*};
use ssal::ethereum::{types::*, SsalClient};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path: String = arguments
        .get(0)
        .expect("Provide a configuration file path")
        .to_owned();
    let block_margin: u64 = arguments
        .get(1)
        .expect("Provide the block margin.")
        .parse()
        .expect("Failed to parse the block margin argument to `u64`");
    let block_creation_time: u64 = arguments
        .get(2)
        .expect("Provide the block creation time.")
        .parse()
        .expect("Failed to parse the block creation time argument to `u64`");

    // Load the configuration from the path.
    let config = Config::load(config_path)?;

    // Initialize the database as a global singleton called by `database::database()`.
    Database::new(&config.database_path)?.init();

    // Initialize the SSAL client.
    let ssal_client = SsalClient::new(
        &config.ssal_rpc_address,
        &config.ssal_private_key,
        &config.contract_address,
        config.cluster_id,
        &config.seeder_rpc_address,
    )
    .await?;

    // Initialize the cluster manager.
    cluster_manager(&ssal_client);

    // Initialize the rollup block number.
    let mut rollup_block_number = RollupBlockNumber::from(0);
    loop {
        let ssal_block_number = SsalBlockNumber::get()? - block_margin;
        let sequencer_list = SequencerList::get(ssal_block_number)?;
        let leader_index = rollup_block_number % sequencer_list.len();

        match send_build_block().await {
            Ok(sequencer_status) => match sequencer_status {
                SequencerStatus::Running => {
                    send_get_block().await;
                    rollup_block_number += 1
                }
                SequencerStatus::Uninitialized => rollup_block_number += 1,
            },
            Err(error) => tracing::error!("{}", error),
        }

        sleep(Duration::from_secs(block_creation_time)).await;
    }
}

fn cluster_manager(ssal_client: &SsalClient) {
    let ssal_client = ssal_client.clone();
    tokio::spawn(async move {
        ssal_client
            .sequencer_list_subscriber(handler)
            .await
            .unwrap();
    });
}

async fn handler(
    ssal_block_number: u64,
    sequencer_list: (Vec<PublicKey>, Vec<Option<RpcAddress>>),
) {
    // Store the current SSAL block number.
    SsalBlockNumber::from(ssal_block_number).put().unwrap();

    // Store the sequencer list corresponding to the current block number.
    SequencerList::from(sequencer_list)
        .put(ssal_block_number.into())
        .unwrap();
}

async fn send_to_leader(rpc_address: &Option<RpcAddress>, rpc_method: impl RpcMethod) -> Result<(), Error> {
    if let Some()
    let rpc_client = RpcClient::new(rpc_, 2)?;
    Ok(())
}

async fn send_to_followers(rpc_method: impl RpcMethod) -> Result<(), Error> {
    Ok(())
}
