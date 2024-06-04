use super::SyncCloseBlock;
use crate::rpc::{prelude::*, util::update_cluster_metadata};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CloseBlock {
    pub ssal_block_number: SsalBlockNumber,
    pub rollup_block_number: RollupBlockNumber,
}

#[async_trait]
impl RpcMethod for CloseBlock {
    type Response = SequencerStatus;

    fn method_name() -> &'static str {
        stringify!(CloseBlock)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        match ClusterMetadata::get() {
            Ok(_cluster_metadata) => {
                self.sync_close_block()?;
                update_cluster_metadata(self.ssal_block_number, self.rollup_block_number)?;
                Ok(SequencerStatus::Running)
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    self.sync_close_block()?;
                    update_cluster_metadata(self.ssal_block_number, self.rollup_block_number)?;
                    Ok(SequencerStatus::Uninitialized)
                } else {
                    Err(error.into())
                }
            }
        }
    }
}

impl CloseBlock {
    pub fn sync_close_block(&self) -> Result<(), RpcError> {
        let me = Me::get()?;
        let sequencer_list = SequencerList::get(self.ssal_block_number)?;
        for (public_key, rpc_address) in sequencer_list.iter() {
            // Always skip forwarding to myself to avoid redundant handling.
            if me.as_public_key() == public_key {
                continue;
            }

            if let Some(rpc_address) = rpc_address {
                let rpc_client = RpcClient::new(rpc_address, 3)?;
                let rpc_method = SyncCloseBlock {
                    ssal_block_number: self.ssal_block_number,
                    rollup_block_number: self.rollup_block_number,
                };

                // Fire and forget.
                tokio::spawn(async move {
                    let _ = rpc_client.request(rpc_method);
                });
            }
        }
        Ok(())
    }
}
