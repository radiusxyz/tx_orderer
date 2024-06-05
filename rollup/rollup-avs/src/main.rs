use std::{env, time::Duration};

use database::Database;
use rollup_avs::{cluster_manager, config::Config, error::Error, rpc::*, types::*};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path: String = arguments
        .get(0)
        .expect("Provide a configuration file path")
        .to_owned();
    let config = Config::load(config_path)?;

    // Initialize the database.
    Database::new(&config.database_path)?.init();

    // Initialize the cluster manager.
    cluster_manager::init(&config)?;

    // Initialize the rollup block number.
    let mut rollup_block_number = RollupBlockNumber::from(0);

    loop {
        sleep(Duration::from_secs(config.block_creation_time)).await;

        // Send a `BuildBlock` request.
        match BuildBlock::request(rollup_block_number).await {
            Ok(sequencer_status) => match sequencer_status {
                SequencerStatus::Uninitialized => rollup_block_number += 1,
                SequencerStatus::Running => {
                    GetBlock::request(rollup_block_number - 1).await?;
                    rollup_block_number += 1;
                }
            },
            Err(error) => tracing::error!("{}", error),
        }
    }
}
