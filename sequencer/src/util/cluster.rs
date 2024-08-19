use std::collections::HashMap;

use radius_sequencer_sdk::{json_rpc::RpcClient, liveness::publisher::Publisher};
use tracing::info;

use crate::{
    client::{SeederClient, SequencerClient},
    error::Error,
    models::LivenessClusterModel,
    types::*,
};

pub async fn initialize_liveness_cluster(
    signing_key: &SigningKey,
    seeder_client: &SeederClient,
    sequencing_info_key: &SequencingInfoKey,
    sequencing_info: &SequencingInfo,
    cluster_id: &ClusterId,
) -> Result<Cluster, Error> {
    info!(
        "Start initializing the liveness cluster - cluster_id: {:?}",
        cluster_id
    );

    let mut liveness_cluster_model = LivenessClusterModel::get_mut(
        sequencing_info_key.platform(),
        sequencing_info_key.service_type(),
        cluster_id,
    )?;

    let (sequencer_list, sequencer_rpc_urls) = match sequencing_info.platform {
        PlatForm::Local => {
            let sequencer_rpc_urls = seeder_client
                .get_rpc_urls(
                    sequencing_info_key.platform(),
                    sequencing_info_key.sequencing_function_type(),
                    sequencing_info_key.service_type(),
                    cluster_id,
                )
                .await?;
            info!(
                "Complete to load rpc urls from seeder - sequencer_rpc_urls: {:?}",
                sequencer_rpc_urls,
            );

            // Change sequencer_rpc_urls(hashmap) to sequencer_list(vec)
            let mut sequencer_list = sequencer_rpc_urls
                .iter()
                .map(|(address, (sequencer_index, _))| (address.clone(), *sequencer_index))
                .collect::<Vec<(Address, SequencerIndex)>>();

            // Sort sequencer_list by sequencer_index
            sequencer_list.sort_by(|(_, sequencer_index1), (_, sequencer_index2)| {
                sequencer_index1.cmp(sequencer_index2)
            });

            let sequencer_list: Vec<Address> = sequencer_list
                .into_iter()
                .map(|(address, _)| address)
                .collect();

            (sequencer_list, sequencer_rpc_urls)
        }
        PlatForm::Ethereum => {
            let provider = Publisher::new(
                sequencing_info.provider_rpc_url.clone(),
                String::from(signing_key.clone()),
                sequencing_info
                    .contract_address
                    .clone()
                    .unwrap()
                    .to_string(),
            )?;
            info!(
                "Complete to load provider - provider_rpc_url: {:?} / contract_address: {:?}",
                sequencing_info.provider_rpc_url,
                sequencing_info.contract_address.clone().unwrap()
            );

            let block_number = provider.get_block_number().await.unwrap();

            let sequencer_list: Vec<Address> = provider
                .get_sequencer_list(cluster_id, block_number)
                .await
                .unwrap()
                .iter()
                .map(|address| address.to_string().into())
                .collect();
            info!(
                "Complete to load sequencer address list from contract - sequencer_address_list: {:?}",
                sequencer_list,
            );

            let sequencer_rpc_urls = seeder_client
                .get_rpc_urls(
                    sequencing_info_key.platform(),
                    sequencing_info_key.sequencing_function_type(),
                    sequencing_info_key.service_type(),
                    &cluster_id,
                )
                .await?;
            info!(
                "Complete to load rpc urls from seeder - sequencer_rpc_urls: {:?}",
                sequencer_rpc_urls,
            );

            (sequencer_list, sequencer_rpc_urls)
        }
    };

    // Initialize sequencer_rpc_clients
    // TODO: Implement RpcClient
    let node_address = signing_key.get_address();
    let mut cluster = Cluster::new(cluster_id.clone(), node_address);

    let mut sequencer_indexes = HashMap::new();

    let mut sequencer_rpc_clients = Vec::new();

    for sequencer_address in sequencer_list.iter() {
        let (sequencer_index, rpc_url) = sequencer_rpc_urls.get(sequencer_address).unwrap();
        let rpc_client = SequencerClient::new(rpc_url.clone()).unwrap();

        sequencer_rpc_clients.push((sequencer_address.clone(), rpc_client));
        sequencer_indexes.insert(*sequencer_index, sequencer_address.clone());
    }

    // Update liveness_cluster_model
    liveness_cluster_model.set_sequencer_list(sequencer_list);
    liveness_cluster_model.update()?;

    // Update sequencer_rpc_clients in cluster
    cluster.set_sequencer_rpc_client_list(sequencer_rpc_clients);

    cluster.set_sequencer_indexes(sequencer_indexes).await;

    Ok(cluster)
}
