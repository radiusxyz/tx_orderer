use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RemoveMevSearcherInfo {
    pub remove_mev_searcher_info_message: RemoveMevSearcherInfoMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RemoveMevSearcherInfoMessage {
    pub rollup_id: RollupId,
    pub mev_searcher_ip: String,
}

impl RpcParameter<AppState> for RemoveMevSearcherInfo {
    type Response = ();

    fn method() -> &'static str {
        "remove_mev_searcher_info"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        let msg = &self.remove_mev_searcher_info_message;

        let rollup = Rollup::get(&msg.rollup_id)?;

        let mut mut_mev_searcher_infos = MevSearcherInfos::get_mut_or(MevSearcherInfos::default)?;

        if !mut_mev_searcher_infos.contains_rollup_id(&msg.mev_searcher_ip, &msg.rollup_id) {
            return Ok(());
        }

        mut_mev_searcher_infos.remove_rollup_id(&msg.mev_searcher_ip.clone(), &msg.rollup_id);
        mut_mev_searcher_infos.update()?;

        self.sync_remove_mev_searcher_info(context.clone(), &rollup)
            .await?;

        Ok(())
    }
}

impl RemoveMevSearcherInfo {
    pub async fn sync_remove_mev_searcher_info(
        &self,
        context: AppState,
        rollup: &Rollup,
    ) -> Result<(), RpcError> {
        let cluster_metadata = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;

        let cluster = Cluster::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
            cluster_metadata.platform_block_height,
        )?;

        let other_urls = cluster.get_other_cluster_rpc_url_list();

        if !other_urls.is_empty() {
            context
                .rpc_client()
                .fire_and_forget_multicast(
                    other_urls,
                    RemoveMevSearcherInfo::method(),
                    self,
                    Id::Null,
                )
                .await;
        }

        Ok(())
    }
}
