use std::{str::FromStr, time::Duration};

use skde::{
    key_aggregation::aggregate_key,
    key_generation::{
        generate_partial_key, prove_partial_key_validity, PartialKey, PartialKeyProof,
    },
    setup, BigUint,
};
use tokio::time::sleep;
use tracing::info;

use crate::{
    rpc::cluster::SyncPartialKey,
    types::{Cluster, RollupId},
};

pub const PRIME_P: &str = "8155133734070055735139271277173718200941522166153710213522626777763679009805792017274916613411023848268056376687809186180768200590914945958831360737612803";
pub const PRIME_Q: &str = "13379153270147861840625872456862185586039997603014979833900847304743997773803109864546170215161716700184487787472783869920830925415022501258643369350348243";
pub const GENERATOR: &str = "4";
pub const TIME_PARAM_T: u32 = 2;
pub const MAX_SEQUENCER_NUMBER: u32 = 2;

pub fn init_single_key_generator(rollup_id: RollupId, cluster: Cluster) {
    let time = 2_u32.pow(TIME_PARAM_T);
    let p = BigUint::from_str(PRIME_P).expect("Invalid PRIME_P");
    let q = BigUint::from_str(PRIME_Q).expect("Invalid PRIME_Q");
    let g = BigUint::from_str(GENERATOR).expect("Invalid GENERATOR");
    let max_sequencer_number = BigUint::from(MAX_SEQUENCER_NUMBER);

    let skde_params = setup(time, p, q, g, max_sequencer_number);

    tokio::spawn(async move {
        loop {
            let (secret_value, partial_key) = generate_partial_key(&skde_params);
            let partial_key_proof = prove_partial_key_validity(&skde_params, &secret_value);

            // TODO
            // cluster
            //     .add_partial_key(cluster.node_address().clone(), partial_key.clone())
            //     .await;

            sync_partial_key(
                rollup_id.clone(),
                cluster.clone(),
                partial_key,
                partial_key_proof,
            );

            // TODO
            sleep(Duration::from_secs(3)).await;

            info!("Aggregate key!");
            let partial_key_list = cluster.get_partial_key_list().await;
            println!("stompesi - partial_key_list: {:?}", partial_key_list);
            let aggregated_key = aggregate_key(&skde_params, &partial_key_list);
            println!("aggregated_key: {:?}", aggregated_key);
        }
    });
}

pub fn sync_partial_key(
    rollup_id: RollupId,
    cluster: Cluster,
    partial_key: PartialKey,
    partial_key_proof: PartialKeyProof,
) {
    tokio::spawn(async move {
        let parameter = SyncPartialKey {
            rollup_id,

            node_address: cluster.node_address().clone(),
            cluster_id: cluster.cluster_id().clone(),

            partial_key,
            partial_key_proof,
        };

        // TODO
        // let sequencer_rpc_clients = cluster.get_other_sequencer_rpc_clients().await;
        let sequencer_rpc_clients = cluster.sequencer_rpc_clients().await;

        info!(
            "sync_partial_key - rpc_client_count: {:?}",
            sequencer_rpc_clients.len()
        );

        for sequencer_rpc_client in sequencer_rpc_clients {
            let sequencer_rpc_client = sequencer_rpc_client.clone();
            let parameter = parameter.clone();

            tokio::spawn(async move {
                match sequencer_rpc_client.sync_partial_key(parameter).await {
                    Ok(_) => {
                        info!("Complete to sync partial key");
                    }
                    Err(err) => {
                        info!("Failed to sync partial key - error: {:?}", err);
                    }
                }
            });
        }
    });
}
