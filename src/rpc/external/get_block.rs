use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlock {
    pub rollup_id: String,
    pub rollup_block_height: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlockResponse {
    pub block_height: u64,

    pub encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
    pub raw_transaction_list: Vec<RawTransaction>,

    #[serde(serialize_with = "serialize_address")]
    pub block_creator_address: Address,
    pub signature: String,

    pub block_commitment: BlockCommitment,
}

impl RpcParameter<AppState> for GetBlock {
    type Response = GetBlockResponse;

    fn method() -> &'static str {
        "get_block"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let block = Block::get(&self.rollup_id, self.rollup_block_height)?;

        Ok(GetBlockResponse {
            block_height: block.block_height,
            encrypted_transaction_list: block.encrypted_transaction_list,
            raw_transaction_list: block.raw_transaction_list,
            block_creator_address: block.block_creator_address,
            signature: block.signature.as_hex_string(),
            block_commitment: block.block_commitment,
        })
    }
}
