use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex as TokioMutex,
    },
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::types::{MevSearcherInfos, RollupId, IP};

pub type SharedChannelInfos = Arc<
    Mutex<
        HashMap<
            IP,
            (
                UnboundedSender<MevSourceTransaction>,
                Arc<tokio::sync::Mutex<UnboundedReceiver<MevTargetTransaction>>>,
            ),
        >,
    >,
>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MevSourceTransaction {
    pub rollup_id: RollupId,
    pub raw_transaction_list: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MevTargetTransaction {
    pub rollup_id: RollupId,
    pub backrunning_transaction_list: Vec<String>,
}

pub async fn run_backrunning_server(shared_channel_infos: SharedChannelInfos) {
    tokio::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:9001").await.unwrap();

        while let Ok((tcp_stream, socket_addr)) = listener.accept().await {
            let mev_searcher_infos = MevSearcherInfos::get_or(MevSearcherInfos::default).unwrap();
            let peer_ip = socket_addr.ip().to_string();

            if !mev_searcher_infos.contains_ip(&peer_ip) {
                tracing::info!("Unauthorized IPs blocked: {}", peer_ip);
                continue;
            }

            let cloned_shared_channel_infos = shared_channel_infos.clone();

            tokio::spawn(async move {
                let ws_stream = accept_async(tcp_stream)
                    .await
                    .expect("WebSocket handshake failure");

                // interaction with MEV searcher
                let (write, mut read) = ws_stream.split();
                let write = Arc::new(TokioMutex::new(write));
                let cloned_write = write.clone();
                // for sending transaction list to MEV searcher (internal)
                let (mev_source_transaction_sender, mut mev_source_transaction_receiver) =
                    unbounded_channel::<MevSourceTransaction>();

                let (backrunning_transaction_sender, backrunning_transaction_receiver) =
                    unbounded_channel::<MevTargetTransaction>();

                {
                    let mut mev_searcher_channel_infos =
                        cloned_shared_channel_infos.lock().unwrap();

                    mev_searcher_channel_infos.insert(
                        peer_ip.clone(),
                        (
                            mev_source_transaction_sender,
                            Arc::new(TokioMutex::new(backrunning_transaction_receiver)),
                        ),
                    );
                }

                let send_raw_transaction_task = tokio::spawn(async move {
                    // receive messages (transaction list) from internal channel and send them to
                    // MEV searcher
                    while let Some(mev_source_transaction) =
                        mev_source_transaction_receiver.recv().await
                    {
                        let mev_source_transaction_str =
                            serde_json::to_string(&mev_source_transaction).unwrap_or_default();
                        let mut locked_write = cloned_write.lock().await;

                        if locked_write
                            .send(Message::Text(mev_source_transaction_str))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                });

                // receive messages (backrunning transactions) from MEV searcher
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(msg) => {
                            let mev_searcher_infos =
                                MevSearcherInfos::get_or(MevSearcherInfos::default).unwrap();
                            let peer_ip = socket_addr.ip().to_string();

                            if !mev_searcher_infos.contains_ip(&peer_ip) {
                                tracing::info!("Unauthorized IPs blocked: {}", peer_ip);
                                write.lock().await.close().await.unwrap();
                                break;
                            }

                            if msg.is_text() {
                                let mev_target_transaction =
                                    serde_json::from_str(&msg.to_string()).unwrap();

                                backrunning_transaction_sender
                                    .send(mev_target_transaction)
                                    .unwrap_or_default();
                            }
                        }
                        Err(e) => {
                            tracing::error!("⚠️ WebSocket error: {}", e);
                            break;
                        }
                    }
                }

                send_raw_transaction_task.abort();

                tracing::info!("Disconnected: {}", peer_ip);
            });
        }
    });
}

pub fn send_transaction_list_to_mev_searcher(
    rollup_id: &RollupId,
    raw_transaction_list: Vec<String>,
    shared_channel_infos: &SharedChannelInfos,
    mev_searcher_infos: &MevSearcherInfos,
) {
    let ip_list = mev_searcher_infos.get_ip_list_by_rollup_id(rollup_id);
    let locked_shared_channel_infos = shared_channel_infos.lock().unwrap();

    for ip in ip_list {
        if let Some((raw_transaction_list_sender, _)) = locked_shared_channel_infos.get(&ip) {
            let mev_source_transaction = MevSourceTransaction {
                rollup_id: rollup_id.to_string(),
                raw_transaction_list: raw_transaction_list.clone(),
            };
            let _ = raw_transaction_list_sender.send(mev_source_transaction);
        }
    }
}
