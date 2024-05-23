use adapter_sdk::traits::Proof;
use adapter_sdk::types::RollupPublicInputs;
use nexus_core::types::H256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CdkProof {
    last_verified_batch: u64,
	new_verified_batch: u64,
	new_state_root: Vec<u8>,
	new_local_exit_root: Vec<u8>,
	proof: Vec<u8>
}

impl Proof for CdkProof {
    fn verify(
        &self,
        vk: &[u8; 32],
        public_inputs: &RollupPublicInputs,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
