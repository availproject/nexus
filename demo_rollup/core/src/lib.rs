use adapter_sdk::traits::VerificationKey;
use adapter_sdk::{traits::Proof, types::RollupVerificationKey};
use adapter_sdk::types::RollupPublicInputs;
use nexus_core::types::H256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DemoProof(pub ());

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DemoVerificationKey {
    pub vk: [u8; 32],

}

impl VerificationKey for DemoVerificationKey {
    fn temp(&self) -> Self {
       Self {
           vk: [0u8; 32],
       }
    }
}

impl Proof<DemoVerificationKey> for DemoProof {
    fn verify(
        &self,
        vk: &RollupVerificationKey<DemoVerificationKey>,
        public_inputs: &RollupPublicInputs,
    ) -> Result<(), anyhow::Error> {
        let _ = vk;
        Ok(())
    }
}
