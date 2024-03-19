use nexus_core::types::{AppAccountId, AvailHeader, H256};
use risc0_zkvm::sha::Digest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AdapterPublicInputs {
    pub header_hash: H256,
    pub state_root: H256,
    pub avail_start_hash: H256,
    pub app_id: AppAccountId,
    pub img_id: Digest,
}

impl AdapterPublicInputs {
    pub fn check_consistency(&self, img_id: &Digest) -> Result<(), anyhow::Error> {
        if img_id != &self.img_id {
            Err(anyhow::anyhow!("The same img_id not used for recursion"))
        } else {
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AdapterPrivateInputs {
    pub header: AvailHeader,
    pub avail_start_hash: H256,
    pub app_id: AppAccountId,
}
