use radius_sequencer_sdk::signature::PrivateKeySigner;

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddCluster {
    platform: Platform,
    service_provider: ServiceProvider,
    cluster_id: String,
}

impl AddCluster {
    pub const METHOD_NAME: &'static str = "add_cluster";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let seeder_client = context.seeder_client();
        match parameter.platform {
            Platform::Ethereum => {
                let signing_key = context.config().signing_key();
                let signer = ChainType::Ethereum.create_signer_from_str(&signing_key)?;
                let address = signer.address();

                seeder_client
                    .register_sequencer(
                        parameter.platform,
                        parameter.service_provider,
                        &parameter.cluster_id,
                        ChainType::Ethereum,
                        address,
                        context.config().cluster_rpc_url(),
                    )
                    .await?;

                let mut cluster_id_list = ClusterIdListModel::get_mut_or_default(
                    parameter.platform,
                    parameter.service_provider,
                )?;
                cluster_id_list.insert(&parameter.cluster_id);
                cluster_id_list.update()?;
            }
            Platform::Local => unimplemented!("Local client needs to be implemented."),
        }

        Ok(())
    }
}
