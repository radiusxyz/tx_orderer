use std::collections::HashMap;

use radius_sequencer_sdk::{json_rpc::RpcClient, liveness::publisher::Publisher};
use tracing::info;

use crate::{client::SeederClient, error::Error, models::LivenessClusterModel, types::*};

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

    // Initialize sequencer_rpc_clients
    // TODO: Implement RpcClient
    let node_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    let mut cluster = Cluster::new(cluster_id.clone(), Address::from(node_address));
    let mut sequencer_rpc_clients = HashMap::new();
    for (index, sequencer_address) in sequencer_list.iter().enumerate() {
        let rpc_url = sequencer_rpc_urls.get(&sequencer_address).unwrap();
        let rpc_client = RpcClient::new(rpc_url.clone()).unwrap();

        sequencer_rpc_clients.insert((index, sequencer_address.clone()), rpc_client);
    }

    // Update liveness_cluster_model
    liveness_cluster_model.set_sequencer_list(sequencer_list);
    let _ = liveness_cluster_model.update()?;

    // Update sequencer_rpc_clients in cluster
    cluster
        .set_sequencer_rpc_clients(sequencer_rpc_clients)
        .await;

    Ok(cluster)
}
