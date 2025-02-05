use crate::rpc::{
    cluster::{SyncMaxGasLimit, SyncMaxGasLimitMessage},
    prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SetMaxGasLimit {
    pub rollup_id: String,
    pub max_gas_limit: u64,
}

impl RpcParameter<AppState> for SetMaxGasLimit {
    type Response = ();

    fn method() -> &'static str {
        "set_max_gas_limit"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        let rollup_metadata = RollupMetadata::get(&self.rollup_id)?;
        let mut locked_rollup = Rollup::get_mut(&self.rollup_id)?;
        let platform = locked_rollup.platform;

        let cluster = Cluster::get(
            locked_rollup.platform,
            locked_rollup.service_provider,
            &locked_rollup.cluster_id,
            rollup_metadata.platform_block_height,
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
    rollup_id: String,
    max_gas_limit: u64,
) {
    tokio::spawn(async move {
        let other_cluster_rpc_url_list: Vec<String> = cluster.get_others_cluster_rpc_url_list();

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
                .multicast(
                    other_cluster_rpc_url_list,
                    SyncMaxGasLimit::method(),
                    &params,
                    Id::Null,
                )
                .await
                .unwrap();
        }
    });
}
