use adapter_sdk::traits::{Proof, RollupPublicInputs};
use nexus_core::types::H256;
use serde::{Deserialize, Serialize, Deserializer};

use ark_bn254::{g1, g1::Parameters, Bn254, FqParameters, Fr, FrParameters, G1Projective};
use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ec::*;
use ark_ff::{Field, Fp256, Fp256Parameters, One, PrimeField, UniformRand, Zero};

pub type G1Point = <Bn254 as PairingEngine>::G1Affine;
pub type G2Point = <Bn254 as PairingEngine>::G2Affine;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ZkEvmProof{
    // pub c1: G1Point,
    // pub c2: G1Point,
    // pub w1: G1Point,
    // pub w2: G1Point,

    // pub eval_ql: Fp256<FrParameters>,
    // pub eval_qr: Fp256<FrParameters>,
    // pub eval_qm: Fp256<FrParameters>,
    // pub eval_qo: Fp256<FrParameters>,
    // pub eval_qc: Fp256<FrParameters>,
    // pub eval_s1: Fp256<FrParameters>,
    // pub eval_s2: Fp256<FrParameters>,
    // pub eval_s3: Fp256<FrParameters>,
    // pub eval_a: Fp256<FrParameters>,
    // pub eval_b: Fp256<FrParameters>,
    // pub eval_c: Fp256<FrParameters>,
    // pub eval_z: Fp256<FrParameters>,
    // pub eval_zw: Fp256<FrParameters>,
    // pub eval_t1w: Fp256<FrParameters>,
    // pub eval_t2w: Fp256<FrParameters>,
    // pub eval_inv: Fp256<FrParameters>,

    pub c1: [u64; 32],
    pub c2: [u64; 32],
    pub w1: [u64; 32],
    pub w2: [u64; 32],

}


impl Proof<ZkEvmRollupPublicInputs> for ZkEvmProof {
    fn verify(
        &self,
        vk: &[u8; 32],
        public_inputs: &ZkEvmRollupPublicInputs,
    ) -> Result<(), anyhow::Error> {
        eprintln!("ZkEvmProof::verify");
        eprintln!("vk: {:?}", vk);
        eprintln!("public_inputs: {:?}", public_inputs);
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Copy)]
pub struct ZkEvmRollupPublicInputs {
    pub prev_state_root: H256,
    pub post_state_root: H256,
    pub blob_hash: H256,
}

impl RollupPublicInputs for ZkEvmRollupPublicInputs {
    fn prev_state_root(&self) -> H256 {
        self.prev_state_root.clone()
    }
    fn post_state_root(&self) -> H256 {
        self.post_state_root.clone()
    }
    fn blob_hash(&self) -> H256 {
        self.blob_hash.clone()
    }
}
