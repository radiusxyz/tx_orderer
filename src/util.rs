use std::time::Duration;

use reqwest::Client;

use crate::{
    error::{self, Error},
    logger::Logger,
    types::Config,
};

pub async fn health_check(sequencer_rpc_url: impl AsRef<str>) -> Result<(), Error> {
    let health_check_url = format!("{}/health", sequencer_rpc_url.as_ref());

    let client = Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(Error::InvalidURL)?;

    client
        .get(health_check_url)
        .send()
        .await
        .map_err(Error::HealthCheck)?;

    Ok(())
}

pub fn initialize_logger(config: &Config) -> Result<(), Error> {
    Logger::new(config.log_path())
        .map_err(error::Error::Logger)?
        .init();
    tracing::info!("Logger initialized.");
    Ok(())
}
