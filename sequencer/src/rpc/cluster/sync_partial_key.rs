use std::str::FromStr;

use skde::{
    key_generation::{verify_partial_key_validity, PartialKey, PartialKeyProof},
    setup, BigUint,
};
use tracing::info;

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncPartialKey {
    pub rollup_id: RollupId,

    pub node_address: Address,
    pub cluster_id: ClusterId,

    pub partial_key: PartialKey,
    pub partial_key_proof: PartialKeyProof,
}

pub const PRIME_P: &str = "8155133734070055735139271277173718200941522166153710213522626777763679009805792017274916613411023848268056376687809186180768200590914945958831360737612803";
pub const PRIME_Q: &str = "13379153270147861840625872456862185586039997603014979833900847304743997773803109864546170215161716700184487787472783869920830925415022501258643369350348243";
pub const GENERATOR: &str = "4";
pub const TIME_PARAM_T: u32 = 2;
pub const MAX_SEQUENCER_NUMBER: u32 = 2;

impl SyncPartialKey {
    pub const METHOD_NAME: &'static str = "sync_partial_key";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        info!(
            "sync_partial_key - rollup_id: {:?} / node_address: {:?} / cluster_id: {:?}",
            parameter.rollup_id, parameter.node_address, parameter.cluster_id
        );

        let time = 2_u32.pow(TIME_PARAM_T);
        let p = BigUint::from_str(PRIME_P).expect("Invalid PRIME_P");
        let q = BigUint::from_str(PRIME_Q).expect("Invalid PRIME_Q");
        let g = BigUint::from_str(GENERATOR).expect("Invalid GENERATOR");
        let max_sequencer_number = BigUint::from(MAX_SEQUENCER_NUMBER);

        // TODO:
        let skde_params = setup(time, p, q, g, max_sequencer_number);

        let is_valid = verify_partial_key_validity(
            &skde_params,
            parameter.partial_key.clone(),
            parameter.partial_key_proof,
        );

        // TODO:
        if !is_valid {
            return Ok(());
        }

        let cluster = context.get_cluster(&parameter.cluster_id)?;

        cluster
            .add_partial_key(parameter.node_address, parameter.partial_key)
            .await;

        Ok(())
    }
}
