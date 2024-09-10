use radius_sequencer_sdk::signature::PrivateKeySigner;

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinCluster {
    platform: Platform,
    service_provider: ServiceProvider,
    cluster_id: String,
}

impl JoinCluster {
    pub const METHOD_NAME: &'static str = "join_cluster";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let seeder_client = context.seeder_client();
        match parameter.platform {
            Platform::Ethereum => {
                let signing_key = context.config().signing_key();
                let signer = ChainType::Ethereum.create_signer_from_str(&signing_key)?;
                let address = signer.address();

                seeder_client
                    .register(
                        parameter.platform,
                        parameter.service_provider,
                        &parameter.cluster_id,
                        ChainType::Ethereum,
                        address.as_ref(),
                        context.config().sequencer_rpc_url(),
                    )
                    .await?;

                let mut cluster_id_list =
                    ClusterIdListModel::get_mut(parameter.platform, parameter.service_provider)?;
                cluster_id_list.insert(&parameter.cluster_id);
                cluster_id_list.update()?;
            }
            Platform::Local => unimplemented!("Local client needs to be implemented."),
        }

        Ok(())
    }
}
