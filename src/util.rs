use std::time::Duration;

use reqwest::Client;

use crate::error::Error;

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
