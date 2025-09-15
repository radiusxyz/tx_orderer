pub mod merkle_tree_manager;

pub use merkle_tree_manager::*;

use radius_sdk::json_rpc::client::{Id, RpcClient};
use crate::types::*;
use tx_orderer_primitives::Error;

pub async fn fetch_raw_transaction_info(
    rpc_client: &RpcClient,
    cluster: &Cluster,
    rollup_id: &RollupId,
    batch_number: u64,
    transaction_order: u64,
) -> Result<(RawTransaction, bool), Error> {
    let others_external_rpc_url_list = cluster.get_others_external_rpc_url_list();

    if others_external_rpc_url_list.is_empty() {
        tracing::warn!(
            "fetch_raw_transaction_info: others_external_rpc_url_list is empty - rollup_id: {:?} / batch_number: {:?} / transaction_order: {:?}",
            rollup_id,
            batch_number,
            transaction_order
        );
        return Err(Error::NoEndpointsAvailable);
    }

    for external_rpc_url in others_external_rpc_url_list {
        let result = rpc_client
            .request(
                &external_rpc_url,
                "get_raw_transaction_with_order_commitment",
                serde_json::json!({
                    "rollup_id": rollup_id,
                    "batch_number": batch_number,
                    "transaction_order": transaction_order
                }),
                Id::Null,
            )
            .await;

        match result {
            Ok(response) => {
                return Ok((response, false));
            }
            Err(error) => {
                tracing::warn!(
                    "fetch_raw_transaction_info: Failed to get raw transaction - rollup_id: {:?} / batch_number: {:?} / transaction_order: {:?} / external_rpc_url: {:?} / error: {:?}",
                    rollup_id,
                    batch_number,
                    transaction_order,
                    external_rpc_url,
                    error
                );
                continue;
            }
        }
    }

    Err(Error::NoEndpointsAvailable)
}

pub async fn fetch_encrypted_transaction(
    rpc_client: &RpcClient,
    cluster: &Cluster,
    rollup_id: &RollupId,
    batch_number: u64,
    transaction_order: u64,
) -> Result<EncryptedTransaction, Error> {
    let others_external_rpc_url_list = cluster.get_others_external_rpc_url_list();

    if others_external_rpc_url_list.is_empty() {
        tracing::warn!(
            "fetch_encrypted_transaction: others_external_rpc_url_list is empty - rollup_id: {:?} / batch_number: {:?} / transaction_order: {:?}",
            rollup_id,
            batch_number,
            transaction_order
        );
        return Err(Error::NoEndpointsAvailable);
    }

    for external_rpc_url in others_external_rpc_url_list {
        let result = rpc_client
            .request(
                &external_rpc_url,
                "get_encrypted_transaction_with_order_commitment", 
                serde_json::json!({
                    "rollup_id": rollup_id,
                    "batch_number": batch_number,
                    "transaction_order": transaction_order
                }),
                Id::Null,
            )
            .await;

        match result {
            Ok(response) => {
                return Ok(response);
            }
            Err(error) => {
                tracing::warn!(
                    "fetch_encrypted_transaction: Failed to get encrypted transaction - rollup_id: {:?} / batch_number: {:?} / transaction_order: {:?} / external_rpc_url: {:?} / error: {:?}",
                    rollup_id,
                    batch_number,
                    transaction_order,
                    external_rpc_url,
                    error
                );
                continue;
            }
        }
    }

    Err(Error::NoEndpointsAvailable)
}