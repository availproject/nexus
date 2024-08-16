use adapter_sdk::types::AdapterPublicInputs;
use nexus_core::types::RollupPublicInputsV2;
#[cfg(any(feature = "native"))]
use nexus_core::zkvm::risczero::RiscZeroProof;
use serde::{Deserialize, Serialize};
pub use zksync_types::commitment::L1BatchWithMetadata;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MockProof(pub ());

pub struct STF();

//Add generics for risczero types, so SP1 could be used as well.
impl STF {
    pub fn verify_continuity_and_proof(
        previous_adapter_pi: AdapterPublicInputs,
        new_rollup_proof: MockProof,
        new_rollup_pi: L1BatchWithMetadata,
    ) -> Result<AdapterPublicInputs, anyhow::Error> {
        unimplemented!();
    }

    #[cfg(any(feature = "native"))]
    pub fn create_recursive_proof() -> Result<(RiscZeroProof, AdapterPublicInputs), anyhow::Error> {
        unimplemented!()
    }
}
