use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlock {
    rollup_block_number: RollupBlockNumber,
}

impl RpcMethod for GetBlock {
    type Response = Vec<u64>;

    fn method_name() -> &'static str {
        stringify!(GetBlock)
    }
}

impl GetBlock {
    pub async fn request(rollup_block_number: RollupBlockNumber) -> Result<Vec<u64>, Error> {
        let cluster = Cluster::get(rollup_block_number)?;
        let rpc_method = Self {
            rollup_block_number,
        };

        loop {
            match rpc_method.send(cluster.leader()).await {
                Ok(block) => return Ok(block),
                Err(_) => {
                    for follower in cluster.followers() {
                        match rpc_method.send(follower).await {
                            Ok(block) => return Ok(block),
                            Err(_) => continue,
                        }
                    }

                    continue;
                }
            }
        }
    }

    pub async fn send(
        &self,
        sequencer: &(PublicKey, Option<RpcAddress>),
    ) -> Result<<Self as RpcMethod>::Response, Error> {
        if let Some(rpc_address) = &sequencer.1 {
            let rpc_client = RpcClient::new(rpc_address.clone(), 5)?;
            let block = rpc_client.request::<Self>(self.clone()).await?;
            Ok(block)
        } else {
            Err(ErrorKind::SequencerNotAvailable.into())
        }
    }
}
