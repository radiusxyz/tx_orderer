use std::str::FromStr;

use pvde::{
    halo2_proofs::{
        halo2curves::bn256::{Bn256, G1Affine},
        plonk::{ProvingKey, VerifyingKey},
        poly::kzg::commitment::ParamsKZG,
    },
    num_bigint::BigUint,
    poseidon::hash::types::PoseidonHashValue,
    time_lock_puzzle::{
        key_validation_zkp::KeyValidationPublicInput, sigma_protocol::SigmaProtocolPublicInput,
        TimeLockPuzzleParam,
    },
};
use serde::{Deserialize, Serialize};
use skde::SkdeParams;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptionProof(Vec<u8>);

impl EncryptionProof {
    pub fn new(value: Vec<u8>) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimeLockPuzzleProof(Vec<u8>);

impl TimeLockPuzzleProof {
    pub fn new(value: Vec<u8>) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

// Todo(jaemin): Add Skde and handling
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Zkp {
    Pvde(PvdeZkp),
    // Skde(SkdeZkp),
    None,
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

    pub fn k_hash_value(&self) -> &PoseidonHashValue {
        &self.k_hash_value
    }

    pub fn to_sigma_protocol_public_input(&self) -> SigmaProtocolPublicInput {
        SigmaProtocolPublicInput {
            r1: self.r1.clone(),
            r2: self.r2.clone(),
            z: self.z.clone(),
            o: self.o.clone(),
            k_two: self.k_two.clone(),
        }
    }

    pub fn to_key_validation_public_input(&self) -> KeyValidationPublicInput {
        KeyValidationPublicInput {
            k_two: self.k_two.clone(),
            k_hash_value: self.k_hash_value.clone(),
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

    pub fn public_input(&self) -> &PvdePublicInput {
        &self.public_input
    }

    pub fn time_lock_puzzle_proof(&self) -> &TimeLockPuzzleProof {
        &self.time_lock_puzzle_proof
    }

    pub fn encryption_proof(&self) -> &EncryptionProof {
        &self.encryption_proof
    }
}

#[derive(Debug, Clone)]
pub enum ZkpParams {
    Pvde(PvdeParams),
    Skde(SkdeParams),
}

#[derive(Clone, Debug, Default)]
pub struct PvdeParams {
    time_lock_puzzle_param: Option<TimeLockPuzzleParam>,
    key_validation_zkp_param: Option<ParamsKZG<Bn256>>,
    key_validation_proving_key: Option<ProvingKey<G1Affine>>,
    key_validation_verifying_key: Option<VerifyingKey<G1Affine>>,
    poseidon_encryption_zkp_param: Option<ParamsKZG<Bn256>>,
    poseidon_encryption_proving_key: Option<ProvingKey<G1Affine>>,
    poseidon_encryption_verifying_key: Option<VerifyingKey<G1Affine>>,
}

impl PvdeParams {
    pub fn new(
        time_lock_puzzle_param: Option<TimeLockPuzzleParam>,
        key_validation_zkp_param: Option<ParamsKZG<Bn256>>,
        key_validation_proving_key: Option<ProvingKey<G1Affine>>,
        key_validation_verifying_key: Option<VerifyingKey<G1Affine>>,
        poseidon_encryption_zkp_param: Option<ParamsKZG<Bn256>>,
        poseidon_encryption_proving_key: Option<ProvingKey<G1Affine>>,
        poseidon_encryption_verifying_key: Option<VerifyingKey<G1Affine>>,
    ) -> Self {
        Self {
            time_lock_puzzle_param,
            key_validation_zkp_param,
            key_validation_proving_key,
            key_validation_verifying_key,
            poseidon_encryption_zkp_param,
            poseidon_encryption_proving_key,
            poseidon_encryption_verifying_key,
        }
    }

    pub fn time_lock_puzzle_param(&self) -> &Option<TimeLockPuzzleParam> {
        &self.time_lock_puzzle_param
    }

    pub fn key_validation_zkp_param(&self) -> &Option<ParamsKZG<Bn256>> {
        &self.key_validation_zkp_param
    }

    pub fn key_validation_proving_key(&self) -> &Option<ProvingKey<G1Affine>> {
        &self.key_validation_proving_key
    }

    pub fn key_validation_verifying_key(&self) -> &Option<VerifyingKey<G1Affine>> {
        &self.key_validation_verifying_key
    }

    pub fn poseidon_encryption_zkp_param(&self) -> &Option<ParamsKZG<Bn256>> {
        &self.poseidon_encryption_zkp_param
    }

    pub fn poseidon_encryption_proving_key(&self) -> &Option<ProvingKey<G1Affine>> {
        &self.poseidon_encryption_proving_key
    }

    pub fn poseidon_encryption_verifying_key(&self) -> &Option<VerifyingKey<G1Affine>> {
        &self.poseidon_encryption_verifying_key
    }

    pub fn update_time_lock_puzzle_param(&mut self, time_lock_puzzle_param: TimeLockPuzzleParam) {
        self.time_lock_puzzle_param = Some(time_lock_puzzle_param);
    }
    pub fn update_key_validation_zkp_param(&mut self, key_validation_zkp_param: ParamsKZG<Bn256>) {
        self.key_validation_zkp_param = Some(key_validation_zkp_param);
    }

    pub fn update_key_validation_proving_key(
        &mut self,
        key_validation_proving_key: ProvingKey<G1Affine>,
    ) {
        self.key_validation_proving_key = Some(key_validation_proving_key);
    }

    pub fn update_key_validation_verifying_key(
        &mut self,
        key_validation_verifying_key: VerifyingKey<G1Affine>,
    ) {
        self.key_validation_verifying_key = Some(key_validation_verifying_key);
    }

    pub fn update_poseidon_encryption_zkp_param(
        &mut self,
        poseidon_encryption_zkp_param: ParamsKZG<Bn256>,
    ) {
        self.poseidon_encryption_zkp_param = Some(poseidon_encryption_zkp_param);
    }
    pub fn update_poseidon_encryption_proving_key(
        &mut self,
        poseidon_encryption_proving_key: ProvingKey<G1Affine>,
    ) {
        self.poseidon_encryption_proving_key = Some(poseidon_encryption_proving_key);
    }

    pub fn update_poseidon_encryption_verifying_key(
        &mut self,
        poseidon_encryption_verifying_key: VerifyingKey<G1Affine>,
    ) {
        self.poseidon_encryption_verifying_key = Some(poseidon_encryption_verifying_key);
    }
}

impl ZkpParams {
    pub fn skde_params(&self) -> Option<&SkdeParams> {
        match self {
            ZkpParams::Skde(skde_params) => Some(skde_params),
            _ => None,
        }
    }
}
