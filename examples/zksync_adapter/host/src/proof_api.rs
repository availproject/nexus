use zksync_core::{L1BatchWithMetadata, MockProof};

pub struct ProofAPI {
    url: String,
}

pub enum ProofAPIResponse {
    Pruned,
    Pending,
    Found((L1BatchWithMetadata, MockProof)),
}

impl ProofAPI {
    pub fn get_proof_for_l1_batch(
        &self,
        l1_batch_number: u32,
    ) -> Result<ProofAPIResponse, anyhow::Error> {
        unimplemented!("Yet to implement API")
    }

    pub fn new(url: &str) -> Self {
        Self {
            url: String::from(url),
        }
    }
}
