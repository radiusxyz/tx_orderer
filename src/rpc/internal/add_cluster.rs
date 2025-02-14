use radius_sdk::signature::PrivateKeySigner;

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddCluster {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,
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
            self.service_provider,
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
                        self.service_provider,
                        &self.cluster_id,
                        &context.config().external_rpc_url,
                        &context.config().cluster_rpc_url,
                        &signer,
                    )
                    .await?;

                let mut cluster_id_list = ClusterIdList::get_mut_or(
                    self.platform,
                    self.service_provider,
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
