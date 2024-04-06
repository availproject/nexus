use adapter_sdk::traits::Proof;
use adapter_sdk::types::RollupPublicInputs;
use nexus_core::types::H256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DemoProof{
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

impl Proof for DemoProof {
    fn verify(
        &self,
        vk: &[u8; 32],
        public_inputs: &RollupPublicInputs,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
