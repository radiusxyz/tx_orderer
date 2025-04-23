use crate::{rpc::prelude::*, task::get_raw_transaction_info_list};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetPostMerklePath {
    pub rollup_id: RollupId,
    pub batch_number: u64,
    pub transaction_order: usize,
}

impl RpcParameter<AppState> for GetPostMerklePath {
    type Response = Vec<String>;

    fn method() -> &'static str {
        "get_post_merkle_path"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        let merkle_tree = MerkleTree::new();
        let rpc_client = context.rpc_client();

        let rollup = Rollup::get(&self.rollup_id)?;
        let max_transaction_count_per_batch = rollup.max_transaction_count_per_batch;
        let cluster_meta = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;
        let cluster = Cluster::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
            cluster_meta.platform_block_height,
        )?;

        let raw_transaction_info_list = get_raw_transaction_info_list(
            &self.rollup_id,
            rpc_client,
            &cluster,
            self.batch_number,
            max_transaction_count_per_batch,
        )
        .await?;

        for (raw_transaction, _) in &raw_transaction_info_list {
            merkle_tree
                .add_data(raw_transaction.raw_transaction_hash().as_ref())
                .await;
        }

        merkle_tree.finalize_tree().await;

        let post_merkle_path = merkle_tree
            .get_post_merkle_path(self.transaction_order)
            .await;

        let merkle_path = post_merkle_path
            .iter()
            .map(|node| const_hex::encode_prefixed(node))
            .collect();

        Ok(merkle_path)
    }
}
