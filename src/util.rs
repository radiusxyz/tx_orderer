use std::{fs, io, path::Path, time::Duration};

use radius_sdk::json_rpc::client::{Id, RpcClient, RpcClientError};
use reqwest::Client;

use crate::{
    error::{self, Error},
    logger::Logger,
    rpc::{
        external::{
            GetEncryptedTransactionWithOrderCommitment, GetRawTransactionWithOrderCommitment,
            GetRawTransactionWithOrderCommitmentResponse,
        },
        prelude::*,
    },
    types::{Cluster, Config, RawTransaction},
};

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

pub fn initialize_logger(config: &Config) -> Result<(), Error> {
    Logger::new(config.log_path())
        .map_err(error::Error::Logger)?
        .init();
    tracing::info!("Logger initialized.");
    Ok(())
}

pub async fn fetch_raw_transaction_info(
    rpc_client: &RpcClient,
    cluster: &Cluster,
    rollup_id: &RollupId,
    batch_number: u64,
    transaction_order: u64,
) -> Result<(RawTransaction, bool), RpcClientError> {
    let others_external_rpc_url_list = cluster.get_others_external_rpc_url_list();

    if others_external_rpc_url_list.is_empty() {
        tracing::warn!(
            "No external RPC URLs available for fetching raw transactions. Rollup ID: {}, Batch number: {}, Order: {}",
            rollup_id, batch_number, transaction_order
        );

        return Err(RpcClientError::Response("NoEndpointsAvailable".to_string()));
    }

    let parameter = GetRawTransactionWithOrderCommitment {
        rollup_id: rollup_id.to_owned(),
        batch_number,
        transaction_order,
    };

    match rpc_client
        .fetch::<GetRawTransactionWithOrderCommitment, GetRawTransactionWithOrderCommitmentResponse>(
            others_external_rpc_url_list,
            GetRawTransactionWithOrderCommitment::method(),
            &parameter,
            Id::Null,
        )
        .await
    {
        Ok(rpc_response) => {
            tracing::debug!(
                "Successfully fetched raw transaction for Rollup ID: {}, Block Height: {}, Order: {}",
                parameter.rollup_id, parameter.batch_number, parameter.transaction_order
            );
            Ok((rpc_response.raw_transaction, rpc_response.is_direct_sent))
        }
        Err(error) => {
            tracing::warn!(
                "Failed to fetch raw transaction for Rollup ID: {}, Block Height: {}, Order: {}. Error: {:?}",
                parameter.rollup_id, parameter.batch_number, parameter.transaction_order, error
            );
            Err(error)
        }
    }
}

pub async fn fetch_encrypted_transaction(
    rpc_client: &RpcClient,
    cluster: &Cluster,
    rollup_id: &RollupId,
    batch_number: u64,
    transaction_order: u64,
) -> Result<EncryptedTransaction, RpcClientError> {
    let others_external_rpc_url_list = cluster.get_others_external_rpc_url_list();

    if others_external_rpc_url_list.is_empty() {
        tracing::warn!(
            rollup_id = %rollup_id,
            batch_number = batch_number,
            transaction_order = transaction_order,
            "No external RPC URLs available for fetching encrypted transactions."
        );
        return Err(RpcClientError::Response("NoEndpointsAvailable".to_string()));
    }

    let parameter = GetEncryptedTransactionWithOrderCommitment {
        rollup_id: rollup_id.to_owned(),
        batch_number,
        transaction_order,
    };

    tracing::info!(
        rollup_id = %parameter.rollup_id,
        batch_number = parameter.batch_number,
        transaction_order = parameter.transaction_order,
        url_list = ?others_external_rpc_url_list,
        "Initiating fetch for encrypted transaction."
    );

    rpc_client
        .fetch::<GetEncryptedTransactionWithOrderCommitment, EncryptedTransaction>(
            others_external_rpc_url_list,
            GetEncryptedTransactionWithOrderCommitment::method(),
            &parameter,
            Id::Null,
        )
        .await
        .map(|rpc_response| {
            tracing::info!(
                rollup_id = %parameter.rollup_id,
                batch_number = parameter.batch_number,
                transaction_order = parameter.transaction_order,
                "Successfully fetched encrypted transaction."
            );
            rpc_response
        })
        .map_err(|error| {
            tracing::debug!(
                rollup_id = %parameter.rollup_id,
                batch_number = parameter.batch_number,
                transaction_order = parameter.transaction_order,
                error = ?error,
                "Failed to fetch encrypted transaction."
            );
            error
        })
}

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
