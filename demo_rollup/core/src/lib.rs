use nexus_core::traits::{Proof, RollupPublicInputs};
use nexus_core::types::H256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DemoProof(pub ());

impl Proof<DemoRollupPublicInputs> for DemoProof {
    fn verify(
        &self,
        vk: &[u8; 32],
        public_inputs: &DemoRollupPublicInputs,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DemoRollupPublicInputs {
    pub prev_state_root: H256,
    pub post_state_root: H256,
    pub blob_hash: H256,
}

impl RollupPublicInputs for DemoRollupPublicInputs {
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
