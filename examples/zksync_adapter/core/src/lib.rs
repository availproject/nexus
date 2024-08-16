use adapter_sdk::types::AdapterPublicInputs;
use nexus_core::types::RollupPublicInputsV2;
#[cfg(any(feature = "native"))]
use nexus_core::zkvm::risczero::RiscZeroProof;
use serde::{Deserialize, Serialize};
pub mod types;
//pub use zksync_types::commitment::L1BatchWithMetadata;
pub use crate::types::L1BatchWithMetadata;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MockProof(pub ());

pub struct STF {
    img_id: [u32; 8],
}

//TODO: Add generics for risczero types, so SP1 could be used as well.
impl STF {
    pub fn new(img_id: [u32; 8]) -> Self {
        Self { img_id }
    }

    pub fn verify_continuity_and_proof(
        previous_adapter_pi: AdapterPublicInputs,
        new_rollup_proof: MockProof,
        new_rollup_pi: L1BatchWithMetadata,
    ) -> Result<AdapterPublicInputs, anyhow::Error> {
        unimplemented!()
    }

    #[cfg(any(feature = "native"))]
    pub fn create_recursive_proof(
        &self,
        //previous_adapter_pi: AdapterPublicInputs,
        prev_adapter_proof: RiscZeroProof,
        new_rollup_proof: MockProof,
        new_rollup_pi: L1BatchWithMetadata,
    ) -> Result<RiscZeroProof, anyhow::Error> {
        use nexus_core::zkvm::traits::ZKVMProof;
        let prev_adapter_pi: AdapterPublicInputs = prev_adapter_proof.public_inputs()?;
        //prev_adapter_proof.verify(self.img_id);
        let check =
            Self::verify_continuity_and_proof(prev_adapter_pi, new_rollup_proof, new_rollup_pi)?;

        //Run elf and generate proof.
        unimplemented!()
    }
}
