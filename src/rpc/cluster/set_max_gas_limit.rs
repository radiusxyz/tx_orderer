use crate::rpc::{
    cluster::{SyncMaxGasLimit, SyncMaxGasLimitMessage},
    prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SetMaxGasLimit {
    pub rollup_id: RollupId,
    pub max_gas_limit: u64,
}

impl RpcParameter<AppState> for SetMaxGasLimit {
    type Response = ();

    fn method() -> &'static str {
        "set_max_gas_limit"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        let mut locked_rollup = Rollup::get_mut(&self.rollup_id)?;
        let platform = locked_rollup.platform;
        let service_provider = locked_rollup.liveness_service_provider;

        let rollup_metadata = RollupMetadata::get(&self.rollup_id)?;
        let cluster_metadata =
            ClusterMetadata::get(platform, service_provider, &rollup_metadata.cluster_id)?;

        let cluster = Cluster::get(
            platform,
            service_provider,
            &locked_rollup.cluster_id,
            cluster_metadata.platform_block_height,
        )?;

        locked_rollup.max_gas_limit = self.max_gas_limit;
        locked_rollup.update()?;

        sync_set_max_gas_limit(
            cluster,
            context.clone(),
            platform,
            self.rollup_id.clone(),
            self.max_gas_limit,
        );

        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn sync_set_max_gas_limit(
    cluster: Cluster,
    context: AppState,
    platform: Platform,
    rollup_id: RollupId,
    max_gas_limit: u64,
) {
    tokio::spawn(async move {
        let other_cluster_rpc_url_list: Vec<String> = cluster.get_other_cluster_rpc_url_list();

        if !other_cluster_rpc_url_list.is_empty() {
            let message = SyncMaxGasLimitMessage {
                rollup_id,
                max_gas_limit,
            };
            let signature = context
                .get_signer(platform)
                .await
                .unwrap()
                .sign_message(&message)
                .unwrap();
            let params = SyncMaxGasLimit { message, signature };

            context
                .rpc_client()
                .fire_and_forget_multicast(
                    other_cluster_rpc_url_list,
                    SyncMaxGasLimit::method(),
                    &params,
                    Id::Null,
                )
                .await;
        }
    });
}
