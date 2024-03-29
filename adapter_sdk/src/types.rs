use crate::traits::{Proof, RollupPublicInputs};
pub use nexus_core::types::RollupPublicInputsV2 as AdapterPublicInputs;
use nexus_core::types::{AppId, AvailHeader, StatementDigest};
use serde::{Deserialize, Serialize};

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

pub struct AdapterConfig {
    pub app_id: AppId,
    pub elf: Vec<u8>,
    pub adapter_elf_id: StatementDigest,
    pub vk: [u8; 32],
    pub rollup_start_height: u32,
}
