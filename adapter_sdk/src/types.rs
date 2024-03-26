pub use nexus_core::types::RollupPublicInputsV2 as AdapterPublicInputs;
use nexus_core::types::{AppAccountId, AvailHeader, H256};
use nexus_core::{
    traits::{Proof, RollupPublicInputs},
    types::AppId,
};
use risc0_zkvm::sha::Digest;
use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, Debug)]
// pub struct AdapterPublicInputs {
//     pub header_hash: H256,
//     pub state_root: H256,
//     pub avail_start_hash: H256,
//     pub app_id: AppAccountId,
//     pub img_id: Digest,
// }

#[derive(Serialize, Deserialize)]
pub struct AdapterPrivateInputs {
    pub header: AvailHeader,
    pub app_id: AppId,
}

pub struct RollupProof<PI: RollupPublicInputs, P: Proof<PI>> {
    pub proof: P,
    pub public_inputs: PI,
}
