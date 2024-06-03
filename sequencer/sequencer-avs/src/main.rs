use std::env;

use database::Database;
use sequencer_avs::{config::Config, error::Error, task::cluster_manager};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path = arguments
        .get(0)
        .expect("Provide the config file path.")
        .to_owned();

    // Load the configuration from the path.
    let config = Config::load(config_path)?;

    // Initialize the database.
    let database = Database::new(&config.database_path).map_err(Error::Database)?;

    // Initialize the cluster manager.
    cluster_manager::init(&config, database.clone())?;
    Ok(())
}
