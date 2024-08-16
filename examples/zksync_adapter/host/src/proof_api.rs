use zksync_core::{L1BatchWithMetadata, MockProof};

pub struct ProofApi {
    url: String,
}

impl ProofApi {
    pub fn get_proof_for_l1_batch(
        &self,
        l1_batch_number: u32,
    ) -> Result<(L1BatchWithMetadata, MockProof), anyhow::Error> {
        unimplemented!("Yet to implement API")
    }
}
