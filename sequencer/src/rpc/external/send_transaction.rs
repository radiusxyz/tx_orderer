use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendTransaction {
    pub rollup_id: u32,
    pub transaction: UserTransaction,
}

impl SendTransaction {
    pub const METHOD_NAME: &'static str = stringify!(SendTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        match ClusterMetadata::get_mut() {
            Ok(mut cluster_metadata) => match cluster_metadata.is_leader {
                true => {
                    // Issue an order commitment.
                    let order_commitment = cluster_metadata
                        .issue_order_commitment(parameter.rollup_id, &parameter.transaction)?;

                    cluster_metadata.commit()?;

                    syncer::sync_user_transaction(
                        context.cluster().await?,
                        parameter.rollup_id,
                        parameter.transaction,
                        order_commitment,
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
