use ark_ec::{AffineCurve, PairingEngine, ProjectiveCurve};
use ark_std::{rand::Rng, UniformRand, Zero};

/// A StructuredReferenceString contains three components:
/// - g = \[ alpha * G, alpha^2 * G,     alpha^3 G,      \dots,   alpha^{n} G,
///           _,        alpha^{n+2} * G, alpha^{n+3} G, \dots,   alpha^{2n} G \]
/// - h = \[ alpha * H, alpha^2 * H,     alpha^3 H,      \dots,   alpha^{n} H, \]
/// - t = e(alpha^{n+1} * G, H)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StructuredReferenceString<E: PairingEngine, const N: usize> {
    pub(crate) g: Vec<E::G1Affine>,
    pub(crate) h: Vec<E::G2Affine>,
    pub(crate) t: E::Fqk,
}

/// The prover parameter is a reference to the G1 coordinates of SRS:
/// - g = \[ alpha * G, alpha^2 * G,     alpha^3 G,      \dots,   alpha^{n} G,
///           _,        alpha^{n+2} * G, alpha^{n+3} G, \dots,   alpha^{2n} G \]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProverParam<E: PairingEngine, const N: usize> {
    pub(crate) g: Vec<E::G1Affine>,
}

/// The verifier parameter is a reference to the G2 and GT coordinates of SRS:
/// - h = \[ alpha * H, alpha^2 * H,     alpha^3 H,      \dots,   alpha^{n} H, \]
/// - t = e(alpha^{n+1} * G, H)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VerifierParam<E: PairingEngine, const N: usize> {
    pub(crate) h: Vec<E::G2Affine>,
    pub(crate) t: E::Fqk,
}

impl<E: PairingEngine, const N: usize> StructuredReferenceString<E, N> {
    /// NOTE: If we can define a single CPU architecture on which we run the library,
    /// we can use SSE 4.2 to improve performance instead of using Rayon's parallel iterator which consumes all cores.
    pub fn new_srs_for_testing<R: Rng>(rng: &mut R) -> Self {
        // compute the alpha base as 1, alpha, alpha^2... alpha^{2n-1}
        // with alpha^n empty
        let alpha = E::Fr::rand(rng);
        let mut alpha_base = Vec::<E::Fr>::with_capacity(N << 1);
        alpha_base.push(alpha);
        for _ in 1..N << 1 {
            alpha_base.push(alpha * alpha_base.last().unwrap())
        }
        // - t  = e(alpha^{n+1} * G, H)
        let t = E::pairing(
            E::G1Affine::prime_subgroup_generator().mul(alpha_base[N]),
            E::G2Affine::prime_subgroup_generator(),
        );

        alpha_base[N] = E::Fr::zero();

        #[cfg(not(feature = "parallel"))]
        let (g, h) = {
            // - g = \[ G, alpha * G,       alpha^2 G,      \dots,   alpha^{n-1} G,
            //          _, alpha^{n+1} * G, alpha^^{n+2} G, \dots,   alpha^{2n-1} G \]
            let g: Vec<E::G1Projective> = alpha_base
                .iter()
                .map(|&alpha_power| E::G1Affine::prime_subgroup_generator().mul(alpha_power))
                .collect();

            // - h = \[ H, alpha * H,       alpha^2 H,      \dots,   alpha^{n-1} H \]
            let h: Vec<E::G2Projective> = alpha_base
                .iter()
                .take(N)
                .map(|&alpha_power| E::G2Affine::prime_subgroup_generator().mul(alpha_power))
                .collect();
            (g, h)
        };

        let g = E::G1Projective::batch_normalization_into_affine(&g);
        let h = E::G2Projective::batch_normalization_into_affine(&h);

        Self { g, h, t }
    }
}

impl<'a, E: PairingEngine, const N: usize> From<&'a StructuredReferenceString<E, N>>
    for ProverParam<E, N>
{
    fn from(srs: &'a StructuredReferenceString<E, N>) -> Self {
        Self { g: srs.g.to_vec() }
    }
}

impl<'a, E: PairingEngine, const N: usize> From<&'a StructuredReferenceString<E, N>>
    for VerifierParam<E, N>
{
    fn from(srs: &'a StructuredReferenceString<E, N>) -> Self {
        Self {
            h: srs.h.to_vec(),
            t: srs.t.clone(),
        }
    }
}
