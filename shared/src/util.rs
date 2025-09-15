use std::{fs, io, path::Path, time::Duration};

use radius_sdk::json_rpc::client::{Id, RpcClient, RpcClientError};
use reqwest::Client;

use tx_orderer_primitives::{error::{self, Error}};
use crate::logger::Logger;

pub async fn health_check(tx_orderer_external_rpc_url: impl AsRef<str>) -> Result<(), Error> {
    let health_check_url = format!("{}/health", tx_orderer_external_rpc_url.as_ref());

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

// initialize_logger moved to tx_orderer main crate to avoid circular dependencies

// fetch_raw_transaction_info function commented out to avoid circular dependencies
// These functions need to be moved to node crate or made generic
/*
pub async fn fetch_raw_transaction_info(
    rpc_client: &RpcClient,
    cluster: &Cluster,
    rollup_id: &RollupId,
    batch_number: u64,
    transaction_order: u64,
) -> Result<(RawTransaction, bool), RpcClientError> {
    // Function body commented out to avoid circular dependencies
    unimplemented!("This function needs to be moved to node crate")
}
*/

/*
// This function also commented out due to circular dependencies
pub async fn fetch_encrypted_transaction(
    rpc_client: &RpcClient,
    cluster: &Cluster,
    rollup_id: &RollupId,
    batch_number: u64,
    transaction_order: u64,
) -> Result<EncryptedTransaction, RpcClientError> {
    unimplemented!("This function needs to be moved to node crate")
}
*/

pub fn clear_dir<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    if path.as_ref().exists() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}
