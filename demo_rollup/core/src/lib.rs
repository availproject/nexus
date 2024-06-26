use adapter_sdk::traits::ValidityProof;
use adapter_sdk::types::RollupPublicInputs;
use nexus_core::types::H256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DemoProof(pub ());

impl ValidityProof for DemoProof {
    fn verify(
        &self,
        vk: &[u8; 32],
        public_inputs: &RollupPublicInputs,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
