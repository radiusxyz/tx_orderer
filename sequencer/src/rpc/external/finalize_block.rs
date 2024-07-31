use crate::{models::ClusterMetadataModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildBlock {
    pub rollup_id: RollupId,
    pub liveness_block_height: BlockHeight,
    pub rollup_block_height: BlockHeight,
}

impl BuildBlock {
    pub const METHOD_NAME: &'static str = stringify!(BuildBlock);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<SequencerStatus, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let cluster = context.get_rollup_cluster(&parameter.rollup_id);

        match cluster {
            None => Ok(SequencerStatus::Uninitialized), // TODO: check
            Some(cluster) => {
                let client = cluster.get_liveness_client().unwrap();

                match ClusterMetadataModel::get_mut(&parameter.rollup_id) {
                    Ok(cluster_metadata) => {
                        let finalized_block_height = cluster_metadata.rollup_block_height.clone();
                        let transaction_count = cluster_metadata.transaction_order.clone();

                        syncer::sync_block(
                            cluster.clone(),
                            parameter.rollup_id.clone(),
                            parameter.liveness_block_height,
                            parameter.rollup_block_height,
                            transaction_count.clone(),
                        );

                        builder::build_block(
                            client.clone(),
                            cluster,
                            parameter.rollup_id,
                            finalized_block_height,
                            transaction_count,
                            true,
                        );

                        Ok(SequencerStatus::Running)
                    }
                    Err(error) => {
                        if error.kind() == database::ErrorKind::KeyDoesNotExist {
                            let transaction_count = TransactionOrder::new(0);
                            let cluster_metadata = ClusterMetadataModel::new(
                                parameter.rollup_id.clone(),
                                parameter.liveness_block_height.clone(),
                                parameter.rollup_block_height.clone(),
                                transaction_count.clone(),
                                false, // TODO: check
                            );

                            cluster_metadata.put()?;

                            syncer::sync_block(
                                cluster,
                                parameter.rollup_id,
                                parameter.liveness_block_height,
                                parameter.rollup_block_height,
                                transaction_count,
                            );

                            Ok(SequencerStatus::Uninitialized)
                        } else {
                            Err(error.into())
                        }
                    }
                }
            }
        }
    }
}
