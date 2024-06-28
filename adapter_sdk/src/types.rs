use crate::traits::RollupProof;
pub use nexus_core::types::RollupPublicInputsV2 as AdapterPublicInputs;
use nexus_core::types::{AppId, AvailHeader, NexusHeader, StatementDigest, H256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterPrivateInputs {
    pub nexus_header: NexusHeader,
    pub avail_header: AvailHeader,
    pub app_id: AppId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupPublicInputs {
    pub prev_state_root: H256,
    pub post_state_root: H256,
    pub height: u32,
    pub blob_hash: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupProofWithPublicInputs<P: RollupProof> {
    pub proof: P,
    pub public_inputs: RollupPublicInputs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub app_id: AppId,
    pub elf: Vec<u8>,
    pub adapter_elf_id: StatementDigest,
    pub vk: [u8; 32],
    pub rollup_start_height: u32,
}
