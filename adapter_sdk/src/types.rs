use crate::traits::{Proof, RollupPublicInputs};
use nexus_core::types::AppId;
pub use nexus_core::types::RollupPublicInputsV2 as AdapterPublicInputs;
use nexus_core::types::{AppAccountId, AvailHeader, H256};
use relayer::types::Header;
use serde::{Deserialize, Deserializer, Serialize};
use std::marker::PhantomData;
use std::marker::{Send, Sync};

// #[derive(Serialize, Deserialize, Debug)]
// pub struct AdapterPublicInputs {
//     pub header_hash: H256,
//     pub state_root: H256,
//     pub avail_start_hash: H256,
//     pub app_id: AppAccountId,
//     pub img_id: Digest,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterPrivateInputs {
    pub header: AvailHeader,
    pub app_id: AppId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupProof<PI, P>
where
    PI: RollupPublicInputs,
    P: Proof<PI>,
{
    pub proof: P,
    pub public_inputs: PI,
}
