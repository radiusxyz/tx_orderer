use std::collections::HashMap;

use radius_sequencer_sdk::liveness::publisher::Publisher;
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
    sequencing_info: &SequencingInfo,
    cluster_id: &ClusterId,
) -> Result<Cluster, Error> {
    info!(
        "Start initializing the liveness cluster - cluster_id: {:?}",
        cluster_id
    );

    let (sequencer_list, sequencer_rpc_client_list) = match sequencing_info.platform {
        PlatForm::Local => {
            // get rpc urls from seeder
            let sequencer_rpc_urls: Vec<(Address, IpAddress)> = seeder_client
                .get_rpc_url_list(
                    sequencing_info.platform(),
                    sequencing_info.sequencing_function_type(),
                    sequencing_info.service_type(),
                    cluster_id,
                )
                .await?;

            info!(
                "Complete to load rpc urls from seeder - sequencer_rpc_urls: {:?}",
                sequencer_rpc_urls,
            );

            sequencer_rpc_urls
                .into_iter()
                .map(|(address, ip_address)| {
                    (
                        address.clone(),
                        (address, SequencerClient::new(ip_address).unwrap()),
                    )
                })
                .unzip()
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

            // get sequencer address list from contract
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

            // get rpc urls from seeder
            let sequencer_rpc_urls: HashMap<Address, IpAddress> = seeder_client
                .get_rpc_url_list(
                    sequencing_info.platform(),
                    sequencing_info.sequencing_function_type(),
                    sequencing_info.service_type(),
                    &cluster_id,
                )
                .await?
                .into_iter()
                .collect();

            info!(
                "Complete to load rpc urls from seeder - sequencer_rpc_urls: {:?}",
                sequencer_rpc_urls,
            );

            let sorted_sequencer_rpc_url_list: Vec<(Address, SequencerClient)> = sequencer_list
                .iter()
                .filter_map(|address| {
                    sequencer_rpc_urls.get(address).map(|ip_address| {
                        (
                            address.clone(),
                            SequencerClient::new(ip_address.clone()).unwrap(),
                        )
                    })
                })
                .collect();

            (sequencer_list, sorted_sequencer_rpc_url_list)
        }
    };

    // Initialize sequencer_rpc_clients
    let node_address = signing_key.get_address();

    // Update liveness_cluster_model
    let mut liveness_cluster_model = LivenessClusterModel::get_mut(
        sequencing_info.platform(),
        sequencing_info.service_type(),
        cluster_id,
    )?;
    liveness_cluster_model.set_sequencer_list(sequencer_list);
    liveness_cluster_model.update()?;

    // Update sequencer_rpc_clients in cluster
    // Todo: check(get cluster first or not)
    let mut cluster = Cluster::new(cluster_id.clone(), node_address);
    cluster.set_sequencer_rpc_client_list(sequencer_rpc_client_list);

    Ok(cluster)
}
