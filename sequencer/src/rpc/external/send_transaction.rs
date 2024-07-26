use crate::{
    models::{ClusterMetadataModel, TransactionModel},
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendTransaction {
    pub rollup_id: RollupId,
    pub transaction: TransactionModel,
}

impl SendTransaction {
    pub const METHOD_NAME: &'static str = stringify!(SendTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        match ClusterMetadataModel::get_mut() {
            Ok(mut cluster_metadata) => match cluster_metadata.is_leader {
                true => {
                    let block_height = cluster_metadata.rollup_block_height.clone();
                    let transaction_order = cluster_metadata.get_transaction_order();
                    cluster_metadata.increment_transaction_order();
                    cluster_metadata.commit()?;

                    // Issue an order commitment.
                    let order_commitment = OrderCommitment {
                        data: OrderCommitmentData {
                            rollup_id: parameter.rollup_id.clone(),
                            block_height,
                            transaction_order,
                            previous_order_hash: OrderHash::default(), // TODO: Implement this.
                        },
                        signature: Signature::default(), // TODO: Implement this.
                    };

                    syncer::sync_user_transaction(
                        context.cluster().await?,
                        parameter.rollup_id,
                        parameter.transaction,
                        order_commitment.clone(),
                    );

                    Ok(order_commitment)
                }
                false => context
                    .cluster()
                    .await?
                    .leader()
                    .request(Self::METHOD_NAME, parameter)
                    .await
                    .map_err(|error| error.into()),
            },
            Err(error) => {
                // Return `Error::Uninitialized` to the user if the sequencer has not
                // received any close block request from the rollup previously.
                match error.kind() {
                    database::ErrorKind::KeyDoesNotExist => Err(Error::Uninitialized.into()),
                    _others => Err(error.into()),
                }
            }
        }
    }
}
