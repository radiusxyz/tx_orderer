use pvde::{num_bigint::BigUint, poseidon::hash::types::PoseidonHashValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptionProof(Vec<u8>);

impl EncryptionProof {
    pub fn new(value: Vec<u8>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimeLockPuzzleProof(Vec<u8>);

impl TimeLockPuzzleProof {
    pub fn new(value: Vec<u8>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PvdePublicInput {
    r1: BigUint,
    r2: BigUint,
    z: BigUint,
    o: BigUint,
    k_two: BigUint,
    k_hash_value: PoseidonHashValue,
}

impl PvdePublicInput {
    pub fn new(
        r1: BigUint,
        r2: BigUint,
        z: BigUint,
        o: BigUint,
        k_two: BigUint,
        k_hash_value: PoseidonHashValue,
    ) -> Self {
        Self {
            r1,
            r2,
            z,
            o,
            k_two,
            k_hash_value,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PvdeZkp {
    public_input: PvdePublicInput,
    time_lock_puzzle_proof: TimeLockPuzzleProof,
    encryption_proof: EncryptionProof,
}

impl PvdeZkp {
    pub fn new(
        public_input: PvdePublicInput,
        time_lock_puzzle_proof: TimeLockPuzzleProof,
        encryption_proof: EncryptionProof,
    ) -> Self {
        Self {
            public_input,
            time_lock_puzzle_proof,
            encryption_proof,
        }
    }
}
