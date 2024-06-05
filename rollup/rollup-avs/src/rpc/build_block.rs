use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildBlock {
    ssal_block_number: SsalBlockNumber,
    rollup_block_number: RollupBlockNumber,
}

impl RpcMethod for BuildBlock {
    type Response = SequencerStatus;

    fn method_name() -> &'static str {
        stringify!(BuildBlock)
    }
}

impl BuildBlock {
    const BLOCK_MARGIN: u64 = 3;

    pub async fn request(
        rollup_block_number: RollupBlockNumber,
    ) -> Result<<Self as RpcMethod>::Response, Error> {
        // Get the current SSAL block number and subtract the block margin.
        let ssal_block_number = SsalBlockNumber::get()? - Self::BLOCK_MARGIN;

        // Get the sequencer list corresponding to the SSAL block number.
        let sequencer_list = SequencerList::get(ssal_block_number)?;
        let leader_index = rollup_block_number % sequencer_list.len();
        let (leader, followers) = sequencer_list.split_leader_from_followers(leader_index);

        // Build the cluster for the rollup block number.
        let cluster = Cluster::new(leader, followers);
        cluster.put(rollup_block_number)?;

        let rpc_method = Self {
            ssal_block_number,
            rollup_block_number,
        };

        match rpc_method.send(cluster.leader()).await {
            Ok(sequencer_status) => Ok(sequencer_status),
            Err(error) => {
                tracing::warn!("Leader returned {}. Trying the followers instead.", error);

                for follower in cluster.followers() {
                    match rpc_method.send(follower).await {
                        Ok(sequencer_status) => return Ok(sequencer_status),
                        Err(_) => continue,
                    }
                }

                Err(ErrorKind::ClusterDown.into())
            }
        }
    }

    pub async fn send(
        &self,
        sequencer: &(PublicKey, Option<RpcAddress>),
    ) -> Result<<Self as RpcMethod>::Response, Error> {
        if let Some(rpc_address) = &sequencer.1 {
            let rpc_client = RpcClient::new(rpc_address.clone(), 5)?;
            rpc_client
                .request::<Self>(self.clone())
                .await
                .map_err(|error| error.into())
        } else {
            Err(ErrorKind::SequencerNotAvailable.into())
        }
    }
}
