use std::{env, pin::Pin, time::Duration};

use database::{database, Database};
use futures::{
    future::{select_ok, Fuse},
    FutureExt,
};
use json_rpc::RpcClient;
use sequencer_avs::{
    config::Config, error::Error, rpc::cluster::*, state::AppState, task::TraceExt, types::*,
};
use serde::{de::DeserializeOwned, ser::Serialize};
use ssal::avs::{types::*, SsalClient, SsalEventListener};
use tokio::time::sleep;

const SSAL_BLOCK_NUMBER_KEY: &'static str = "0";

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    std::panic::set_hook(Box::new(|panic_info| tracing::error!("{}", panic_info)));

    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path = arguments
        .get(0)
        .expect("Provide the config file path.")
        .to_owned();
    let block_margin: u64 = arguments
        .get(1)
        .expect("Provide the block margin.")
        .parse()
        .expect("Failed to parse the block margin argument to `u64`");
    let block_creation_time: u64 = arguments
        .get(2)
        .expect("Provide the block creation time.")
        .parse()
        .expect("Failed to parse the block creation time argument to `u64`");

    // Load the configuration from the path.
    let config = Config::load(&config_path)?;
    tracing::info!(
        "Successfully loaded the configuration file at {}",
        config_path,
    );

    // Initialize the database.
    Database::new(config.database_path())?.init();
    tracing::info!(
        "Succesfully initialized the database at {:?}",
        config.database_path(),
    );

    // Initialize the SSAL client.
    let ssal_client = SsalClient::new(
        config.ethereum_rpc_url(),
        config.signing_key(),
        config.seeder_rpc_url(),
        config.ssal_contract_address(),
        config.delegation_manager_contract_address(),
        config.stake_registry_contract_address(),
        config.avs_directory_contract_address(),
        config.avs_contract_address(),
    )?;
    tracing::info!("Successfully initialized the SSAL client");

    // Initialize an application-wide state instance.
    let app_state = AppState::new(config, ssal_client, None);

    // Initialize the event manager.
    event_manager(app_state.clone());

    let mut rollup_block_number: u64 = 0;
    loop {
        sleep(Duration::from_secs(block_creation_time)).await;
        match request_build_block(block_margin, rollup_block_number).await {
            Ok((sequencer_status, ssal_block_number)) => {
                tracing::info!(
                    "[{}]: {:?}\nEthereum block number: {}\nRollup block number: {}",
                    BuildBlock::METHOD_NAME,
                    sequencer_status,
                    ssal_block_number,
                    rollup_block_number,
                );
                database()?.put(&rollup_block_number, &ssal_block_number)?;
                rollup_block_number += 1;
            }
            Err(error) => {
                tracing::error!("[{}]: {}", BuildBlock::METHOD_NAME, error);
                continue;
            }
        }
    }
}

fn event_manager(context: AppState) {
    tokio::spawn(async move {
        loop {
            let ssal_event_listener = SsalEventListener::connect(
                context.config().ethereum_websocket_url(),
                context.config().ssal_contract_address(),
                context.config().avs_contract_address(),
            )
            .await
            .ok_or_trace();

            if let Some(ssal_event_listener) = ssal_event_listener {
                ssal_event_listener
                    .init(event_callback, context.clone())
                    .await
                    .ok_or_trace();
            }

            sleep(Duration::from_secs(3)).await;
            tracing::warn!("Reconnecting the event listener..");
        }
    });
}

async fn event_callback(event_type: SsalEventType, context: AppState) {
    match event_type {
        SsalEventType::NewBlock(block) => {
            if let Some(block_number) = block.header.number {
                let sequencer_list = context
                    .ssal_client()
                    .get_sequencer_list(context.config().cluster_id())
                    .await
                    .ok_or_trace();

                if let Some(sequencer_list) = sequencer_list {
                    SequencerList::from(sequencer_list)
                        .put(block_number)
                        .ok_or_trace();

                    SequencerList::delete(block_number.wrapping_sub(100)).ok_or_trace();
                }

                database()
                    .unwrap()
                    .put(&SSAL_BLOCK_NUMBER_KEY, &block_number)
                    .unwrap();
            }
        }
        SsalEventType::BlockCommitment((event, _log)) => {
            if &event.clusterID.to_string() == context.config().cluster_id() {
                match get_block(event.blockNumber).await {
                    Ok(_rollup_block) => {
                        tracing::info!(
                            "[{}]: Fetched the block({})",
                            GetBlock::METHOD_NAME,
                            event.blockNumber,
                        );
                    }
                    Err(error) => tracing::error!("[{}]: {}", GetBlock::METHOD_NAME, error),
                }
            }
        }
        _ => {}
    }
}

async fn get_block(rollup_block_number: u64) -> Result<RollupBlock, Error> {
    let database = database()?;
    let ssal_block_number: u64 = database.get(&rollup_block_number)?;
    let sequencer_list = SequencerList::get(ssal_block_number).ok_or_trace();

    if let Some(sequencer_list) = sequencer_list {
        let rpc_method = GetBlock {
            rollup_block_number,
        };

        // Get `RollupBlock` from any sequencer in the list.
        return fetch::<GetBlock, RollupBlock>(
            sequencer_list.into_inner(),
            GetBlock::METHOD_NAME,
            rpc_method,
        )
        .await;
    } else {
        return Err(Error::EmptySequencerList);
    }
}

async fn fetch<P, R>(
    sequencer_list: Vec<(Address, Option<String>)>,
    method: &'static str,
    parameter: P,
) -> Result<R, Error>
where
    P: Clone + Serialize + Send,
    R: DeserializeOwned,
{
    let rpc_client_list: Vec<RpcClient> = sequencer_list
        .into_iter()
        .filter_map(|(_address, rpc_url)| match rpc_url {
            Some(rpc_url) => RpcClient::new(rpc_url).ok(),
            None => None,
        })
        .collect();

    let fused_futures: Vec<Pin<Box<Fuse<_>>>> = rpc_client_list
        .iter()
        .map(|rpc_client| Box::pin(rpc_client.request::<P, R>(method, parameter.clone()).fuse()))
        .collect();

    let (rpc_response, _): (R, Vec<_>) = select_ok(fused_futures)
        .await
        .map_err(|_| Error::FetchResponse)?;

    Ok(rpc_response)
}

async fn request_build_block(
    block_margin: u64,
    rollup_block_number: u64,
) -> Result<(SequencerStatus, u64), Error> {
    let ssal_block_number =
        database()?.get::<&'static str, u64>(&SSAL_BLOCK_NUMBER_KEY)? - block_margin;
    let mut sequencer_list = SequencerList::get(ssal_block_number)?.into_inner();

    let leader_index = rollup_block_number.checked_rem(sequencer_list.len() as u64);
    if let Some(leader_index) = leader_index {
        let rpc_method = BuildBlock {
            ssal_block_number,
            rollup_block_number,
        };

        // Remove leader from the sequencer_list, making the sequencer list a list of followers.
        let (_leader_address, leader_rpc_url) = sequencer_list.remove(leader_index as usize);

        // Try the leader.
        if let Some(rpc_url) = leader_rpc_url {
            let rpc_client = RpcClient::new(rpc_url)?.max_retry(2).retry_interval(1);
            let rpc_response: Option<SequencerStatus> = rpc_client
                .request(BuildBlock::METHOD_NAME, rpc_method.clone())
                .await
                .ok();

            if let Some(sequencer_status) = rpc_response {
                Ok((sequencer_status, ssal_block_number))
            } else {
                // Try the followers.
                tracing::warn!(
                    "[{}] The leader is unresponsive. Trying the followers..",
                    BuildBlock::METHOD_NAME
                );

                for (address, rpc_url) in sequencer_list.into_iter() {
                    if let Some(rpc_url) = rpc_url {
                        let rpc_client = RpcClient::new(rpc_url)?;
                        let rpc_response: Option<SequencerStatus> = rpc_client
                            .request(BuildBlock::METHOD_NAME, rpc_method.clone())
                            .await
                            .ok();

                        if let Some(sequencer_status) = rpc_response {
                            tracing::info!("[{}]: {:?}", BuildBlock::METHOD_NAME, sequencer_status);

                            return Ok((sequencer_status, ssal_block_number));
                        } else {
                            continue;
                        }
                    } else {
                        tracing::warn!(
                            "[{}]: Empty RPC URL (address: {})",
                            BuildBlock::METHOD_NAME,
                            address,
                        );
                    }
                }

                Err(Error::ClusterDown)
            }
        } else {
            // Simply return without trying the followers because followers do not
            // have the leader RPC URL either, therefore, not building any block.
            Err(Error::EmptyLeaderRpcUrl)
        }
    } else {
        Err(Error::EmptySequencerList)
    }
}
