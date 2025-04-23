use radius_sdk::signature::PrivateKeySigner;

use crate::{client::liveness_service_manager::radius::initialize_new_cluster, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddCluster {
    pub platform: Platform,
    pub liveness_service_provider: LivenessServiceProvider,
    pub cluster_id: ClusterId,
}

impl RpcParameter<AppState> for AddCluster {
    type Response = ();

    fn method() -> &'static str {
        "add_cluster"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        tracing::info!(
            "Add cluster - platform: {:?}, service provider: {:?}, cluster id: {:?}",
            self.platform,
            self.liveness_service_provider,
            self.cluster_id
        );

        let seeder_client = context.seeder_client();
        match self.platform {
            Platform::Ethereum => {
                let signing_key = &context.config().signing_key;
                let signer = PrivateKeySigner::from_str(self.platform.into(), signing_key)?;

                seeder_client
                    .register_tx_orderer(
                        self.platform,
                        self.liveness_service_provider,
                        &self.cluster_id,
                        &context.config().external_rpc_url,
                        &context.config().cluster_rpc_url,
                        &signer,
                    )
                    .await?;

                let liveness_service_manager_client: liveness_service_manager::radius::LivenessServiceManagerClient = context
                .get_liveness_service_manager_client::<liveness_service_manager::radius::LivenessServiceManagerClient>(
                    self.platform,
                    self.liveness_service_provider,
                )
                .await?;

                let platform_block_height = liveness_service_manager_client
                    .publisher()
                    .get_block_number()
                    .await
                    .expect("Failed to get block number");

                let block_margin = liveness_service_manager_client
                    .publisher()
                    .get_block_margin()
                    .await
                    .expect("Failed to get block margin")
                    .try_into()
                    .expect("Failed to convert block margin");

                let cluster_metadata =
                    ClusterMetadata::new(self.cluster_id.clone(), platform_block_height);

                cluster_metadata.put(
                    self.platform,
                    self.liveness_service_provider,
                    &self.cluster_id,
                )?;

                let _ = initialize_new_cluster(
                    context,
                    &liveness_service_manager_client,
                    &self.cluster_id,
                    platform_block_height,
                    block_margin,
                )
                .await
                .expect("Failed to initialize new cluster");

                let mut cluster_id_list = ClusterIdList::get_mut_or(
                    self.platform,
                    self.liveness_service_provider,
                    ClusterIdList::default,
                )?;
                cluster_id_list.insert(&self.cluster_id);
                cluster_id_list.update()?;
            }
            Platform::Holesky => unimplemented!("Holesky client needs to be implemented."),
            Platform::Local => unimplemented!("Local client needs to be implemented."),
        }

        Ok(())
    }
}
