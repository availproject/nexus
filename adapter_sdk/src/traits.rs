use crate::types::RollupPublicInputs;
use anyhow::Error;
use nexus_core::types::H256;

// #[cfg(any(feature = "native"))]
// pub trait RollupAdapter<PI: RollupPublicInputs, P: Proof<PI>> {
//     fn new(app_id: AppId, avail_signer: PairSigner) -> Self;
//     fn start() -> impl Future<Output = Result<(), anyhow::Error>>;
//     fn submit_blob(blob: Vec<u8>) -> impl Future<Output = Result<(), anyhow::Error>>;
//     fn submit_proof_for_blob() -> impl Future<Output = Result<(), anyhow::Error>>;
// }

pub trait Proof {
    fn verify(&self, vk: &[u8; 32], public_inputs: &RollupPublicInputs) -> Result<(), Error>;
}

// pub trait RollupPublicInputs {
//     fn prev_state_root(&self) -> H256;
//     fn post_state_root(&self) -> H256;
//     fn blob_hash(&self) -> H256;
// }
