use pvde::{num_bigint::BigUint, poseidon::hash::types::PoseidonHashValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptionProof(Vec<u8>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimeLockPuzzleProof(Vec<u8>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PvdePublicInput {
    r1: BigUint,
    r2: BigUint,
    z: BigUint,
    o: BigUint,
    k_two: BigUint,
    k_hash_value: PoseidonHashValue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PvdeZkp {
    public_input: PvdePublicInput,
    time_lock_puzzle_proof: TimeLockPuzzleProof,
    encryption_proof: EncryptionProof,
}
