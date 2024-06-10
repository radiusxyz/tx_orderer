pub mod kzg;
pub mod param;
pub mod vc;

pub use ark_bn254::Bn254;
use ark_ec::PairingEngine;
use ark_ff::FromBytes;
use ark_std::rand::{rngs::StdRng, SeedableRng};
use param::{ProverParam, StructuredReferenceString};
use sha2::{Digest, Sha224};

pub struct Commitment<E: PairingEngine, const N: usize> {
    commitment: E::G1Projective,
}

pub trait CommitmentScheme {
    type ProverParam;
    type VerifierParam;
    type MessageUnit;
    type Commitment;
    type Witness;

    /// Commit to a list of inputs with prover parameters
    fn commit(pp: &Self::ProverParam, inputs: &[Self::MessageUnit]) -> Self;

    /// Open an input at a given position
    fn open(pp: &Self::ProverParam, inputs: &[Self::MessageUnit], pos: usize) -> Self::Witness;

    /// Verify the input/witness pair is correct
    fn verify(
        &self,
        vp: &Self::VerifierParam,
        input: &Self::MessageUnit,
        pos: usize,
        witness: &Self::Witness,
    ) -> bool;

    fn to_string(&self) -> String;
}

pub fn get_block_commitment<T>(block: &Vec<T>, seed: [u8; 32]) -> Vec<u8>
where
    T: AsRef<[u8]>,
{
    let mut rng = StdRng::from_seed(seed);
    let srs = StructuredReferenceString::<Bn254, 1024>::new_srs_for_testing(&mut rng);
    let prover_param: ProverParam<Bn254, 1024> = (&srs).into();
    let message: Vec<<Bn254 as PairingEngine>::Fr> = block
        .iter()
        .map(|element| {
            let mut hasher = Sha224::new();
            hasher.update(element.as_ref());
            let mut hashed_raw_tx = hasher.finalize().to_vec();
            hashed_raw_tx.extend_from_slice(&[0; 24]);
            <Bn254 as PairingEngine>::Fr::read(hashed_raw_tx.as_slice()).unwrap()
        })
        .collect();

    let commitment = Commitment::<Bn254, 1024>::commit(&prover_param, &message);
    commitment.to_string().into_bytes()
}
