use nexus_core::types::{AppAccountId, AvailHeader, H256};
use risc0_zkvm::sha::Digest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AdapterPublicInputs {
    pub header_hash: H256,
    pub state_root: H256,
    pub avail_start_hash: H256,
    pub app_id: AppAccountId,
}

pub struct AdapterPrivateInputs {
    pub header: AvailHeader,
    pub state_root: H256,
    pub avail_start_hash: H256,
    pub app_id: AppAccountId,
}
