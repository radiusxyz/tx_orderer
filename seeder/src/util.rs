use std::time::Duration;

use reqwest::Client;

use crate::error::Error;

pub async fn health_check(sequencer_address: impl AsRef<str>) -> Result<(), Error> {
    let health_check_url = format!("{}/health", sequencer_address.as_ref());

    let client = Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|_| Error::HealthCheck)?;

    client
        .get(health_check_url)
        .send()
        .await
        .map_err(|_| Error::HealthCheck)?;

    Ok(())
}
