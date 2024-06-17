use std::{env, time::Duration};

use ::ssal::ethereum::{types::*, SsalClient};
use database::Database;
use json_rpc::RpcClient;
use sequencer_avs::{
    config::Config, error::Error, rpc::external::*, task::cluster_manager, types::*,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Ok(())
}

// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     tracing_subscriber::fmt().init();

//     let arguments: Vec<String> = env::args().skip(1).collect();
//     let config_path: String = arguments
//         .get(0)
//         .expect("Provide a configuration file path")
//         .to_owned();
//     let block_margin: u64 = arguments
//         .get(1)
//         .expect("Provide the block margin.")
//         .parse()
//         .expect("Failed to parse the block margin argument to `u64`");
//     let block_creation_time: u64 = arguments
//         .get(2)
//         .expect("Provide the block creation time.")
//         .parse()
//         .expect("Failed to parse the block creation time argument to `u64`");

//     // Load the configuration from the path.
//     let config = Config::load(config_path)?;

//     // Initialize the database as a global singleton called by `database::database()`.
//     Database::new(&config.database_path)?.init();

//     // Initialize the SSAL client.
//     let ssal_client = SsalClient::new(
//         &config.ssal_rpc_address,
//         &config.ssal_private_key,
//         &config.contract_address,
//         config.cluster_id,
//         &config.seeder_rpc_address,
//     )
//     .await?;

//     // Initialize the cluster manager.
//     // cluster_manager::init(&ssal_client);

//     // Initialize the rollup block number.
//     let mut rollup_block_number = RollupBlockNumber::from(0);
//     loop {
//         if let Some(current_ssal_block_number) = SsalBlockNumber::get().ok() {
//             let ssal_block_number = current_ssal_block_number - block_margin;
//             if let Some(sequencer_list) = SequencerList::get(ssal_block_number).ok() {
//                 tracing::info!("{:?}\n{:?}", ssal_block_number, rollup_block_number);
//                 // tracing::info!("{:?}", sequencer_list);
//                 let leader_index = rollup_block_number % sequencer_list.len();
//                 let (leader, followers) = sequencer_list.split_leader_from_followers(leader_index);

//                 match send_build_block(
//                     ssal_block_number,
//                     rollup_block_number,
//                     &leader,
//                     &followers,
//                     3,
//                     1,
//                 )
//                 .await
//                 {
//                     Ok(sequencer_status) => match sequencer_status {
//                         SequencerStatus::Running => {
//                             match send_get_block(rollup_block_number - 1, &leader, &followers, 3, 1)
//                                 .await
//                             {
//                                 Ok(block) => tracing::info!("{:?}", block),
//                                 Err(error) => tracing::error!("{}", error),
//                             }
//                             rollup_block_number += 1;
//                         }
//                         SequencerStatus::Uninitialized => rollup_block_number += 1,
//                     },
//                     Err(error) => tracing::error!("{}", error),
//                 }
//             }
//         }
//         sleep(Duration::from_secs(block_creation_time - 2)).await;
//     }
// }

// async fn send_build_block(
//     ssal_block_number: SsalBlockNumber,
//     rollup_block_number: RollupBlockNumber,
//     leader: &(H160, Option<String>),
//     followers: &Vec<(H160, Option<String>)>,
//     retry: usize,
//     retry_interval: u64,
// ) -> Result<(), Error> {
//     let rpc_method = BuildBlock {
//         ssal_block_number,
//         rollup_block_number,
//     };

//     for retry_count in 0..retry {
//         tracing::info!(
//             "[{}] Trying the leader.. retry count: {}",
//             stringify!(BuildBlock),
//             retry_count,
//         );
//         if let Some(rpc_response) = send_to_leader(&leader, rpc_method.clone()).await.ok() {
//             return Ok(rpc_response);
//         }
//         sleep(Duration::from_secs(retry_interval)).await;
//     }

//     for retry_count in 0..retry {
//         tracing::info!(
//             "[{}] Trying the followers.. retry count: {}",
//             stringify!(BuildBlock),
//             retry_count
//         );
//         if let Some(rpc_response) = send_to_followers(followers, rpc_method.clone()).await.ok() {
//             return Ok(rpc_response);
//         }
//     }

//     Err(Error::ClusterDown)
// }

// async fn send_get_block(
//     rollup_block_number: RollupBlockNumber,
//     leader: &(H160, Option<String>),
//     followers: &Vec<(H160, Option<String>)>,
//     retry: usize,
//     retry_interval: u64,
// ) -> Result<<GetBlock as RpcMethod>::Response, Error> {
//     let rpc_method = GetBlock {
//         rollup_block_number,
//     };

//     for retry_count in 0..retry {
//         tracing::info!(
//             "[{}] Trying the leader.. retry count: {}",
//             stringify!(GetBlock),
//             retry_count,
//         );
//         if let Some(rpc_response) = send_to_leader(&leader, rpc_method.clone()).await.ok() {
//             return Ok(rpc_response);
//         }
//         sleep(Duration::from_secs(retry_interval)).await;
//     }

//     for retry_count in 0..retry {
//         tracing::info!(
//             "[{}] Trying the followers.. retry count: {}",
//             stringify!(GetBlock),
//             retry_count
//         );
//         if let Some(rpc_response) = send_to_followers(followers, rpc_method.clone()).await.ok() {
//             return Ok(rpc_response);
//         }
//     }

//     Err(Error::ClusterDown)
// }

// async fn send_to_leader<T>(
//     leader: &(H160, Option<String>),
//     rpc_method: T,
// ) -> Result<<T as RpcMethod>::Response, Error>
// where
//     T: RpcMethod + Send,
// {
//     if let Some(rpc_address) = &leader.1 {
//         let rpc_client = RpcClient::new(rpc_address, 2)?;
//         let rpc_response = rpc_client.request(rpc_method).await?;
//         Ok(rpc_response)
//     } else {
//         Err(Error::EmptyLeaderAddress)
//     }
// }

// async fn send_to_followers<T>(
//     followers: &Vec<(H160, Option<String>)>,
//     rpc_method: T,
// ) -> Result<<T as RpcMethod>::Response, Error>
// where
//     T: RpcMethod + Send,
// {
//     for follower in followers {
//         if let Some(rpc_address) = &follower.1 {
//             let rpc_client = RpcClient::new(rpc_address, 2)?;
//             match rpc_client.request(rpc_method.clone()).await {
//                 Ok(rpc_response) => return Ok(rpc_response),
//                 Err(_) => continue,
//             }
//         } else {
//             continue;
//         }
//     }

//     Err(Error::UnresponsiveFollowers)
// }
